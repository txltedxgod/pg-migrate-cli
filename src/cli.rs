use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "pg-migrate",
    author = "txltedxgod",
    version = "0.1.0",
    about = "Fast transactional schema migration tool for PostgreSQL & SQLite"
)]
pub struct Cli {
    /// Database connection URL (e.g. postgres://user:pass@localhost:5432/dbname)
    #[arg(short, long, env = "DATABASE_URL")]
    pub url: String,

    /// Path to directory containing migration files (.sql)
    #[arg(short, long, default_value = "./migrations")]
    pub dir: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run all pending forward migrations (up)
    Up,
    /// Rollback the most recently applied migration (down)
    Down,
    /// Display current migration status table
    Status,
    /// Create a new numbered migration pair file
    Create {
        /// Migration descriptive name (e.g. add_users_table)
        name: String,
    },
}
