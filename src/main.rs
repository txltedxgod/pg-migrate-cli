mod checksum;
mod cli;
mod migrator;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use migrator::discover_migrations;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, Row};
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Create { name } => {
            fs::create_dir_all(&args.dir)?;
            let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
            let up_file = args.dir.join(format!("{}_{}.up.sql", timestamp, name));
            let down_file = args.dir.join(format!("{}_{}.down.sql", timestamp, name));

            fs::write(&up_file, "-- Write your UP migration SQL here\n")?;
            fs::write(&down_file, "-- Write your DOWN rollback SQL here\n")?;

            println!("{} Created migration pair:", "✔".green());
            println!("  {}", up_file.display());
            println!("  {}", down_file.display());
            return Ok(());
        }
        _ => {}
    }

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&args.url)
        .await
        .context("Failed to connect to database")?;

    // Create schema_migrations table if not exists
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS _schema_migrations (
            version BIGINT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            checksum VARCHAR(64) NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .await?;

    let migrations = discover_migrations(&args.dir)?;

    match args.command {
        Commands::Up => {
            println!("{}", "🚀 Running pending forward migrations...".cyan().bold());
            let mut applied_count = 0;

            for m in migrations {
                let row = sqlx::query("SELECT checksum FROM _schema_migrations WHERE version = $1")
                    .bind(m.version)
                    .fetch_optional(&pool)
                    .await?;

                if let Some(r) = row {
                    let db_checksum: String = r.get(0);
                    if db_checksum != m.checksum {
                        eprintln!(
                            "{} Checksum mismatch for migration {} (DB: {}, File: {})",
                            "✖ ERROR:".red().bold(),
                            m.version,
                            &db_checksum[..8],
                            &m.checksum[..8]
                        );
                        std::process::exit(1);
                    }
                    continue;
                }

                print!("  Applying [{}] {} ... ", m.version, m.name);
                let mut tx = pool.begin().await?;
                tx.execute(m.up_sql.as_str()).await?;
                sqlx::query("INSERT INTO _schema_migrations (version, name, checksum) VALUES ($1, $2, $3)")
                    .bind(m.version)
                    .bind(&m.name)
                    .bind(&m.checksum)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;

                println!("{}", "OK".green());
                applied_count += 1;
            }

            if applied_count == 0 {
                println!("{}", "Database schema is already up to date.".dimmed());
            } else {
                println!("{} Successfully applied {} migrations.", "✔".green().bold(), applied_count);
            }
        }
        Commands::Down => {
            let row = sqlx::query("SELECT version, name FROM _schema_migrations ORDER BY version DESC LIMIT 1")
                .fetch_optional(&pool)
                .await?;

            if let Some(r) = row {
                let version: i64 = r.get(0);
                let name: String = r.get(1);

                if let Some(m) = migrations.iter().find(|m| m.version == version) {
                    if let Some(down_path) = &m.down_path {
                        let down_sql = fs::read_to_string(down_path)?;
                        print!("  Rolling back [{}] {} ... ", version, name);
                        let mut tx = pool.begin().await?;
                        tx.execute(down_sql.as_str()).await?;
                        sqlx::query("DELETE FROM _schema_migrations WHERE version = $1")
                            .bind(version)
                            .execute(&mut *tx)
                            .await?;
                        tx.commit().await?;
                        println!("{}", "ROLLED BACK".yellow());
                    } else {
                        eprintln!("{} No down migration file found for {}", "✖".red(), version);
                    }
                }
            } else {
                println!("No applied migrations to roll back.");
            }
        }
        Commands::Status => {
            println!("\n{}", "── Migration Status ──".bold());
            for m in migrations {
                let row = sqlx::query("SELECT applied_at FROM _schema_migrations WHERE version = $1")
                    .bind(m.version)
                    .fetch_optional(&pool)
                    .await?;

                let status = match row {
                    Some(_) => "APPLIED".green(),
                    None => "PENDING".yellow(),
                };
                println!("  [{:>14}] {:<30} [{}]", m.version, m.name, status);
            }
            println!();
        }
        _ => {}
    }

    Ok(())
}
