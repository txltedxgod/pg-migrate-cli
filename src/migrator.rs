use crate::checksum::compute_sha256;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MigrationFile {
    pub version: i64,
    pub name: String,
    pub up_path: PathBuf,
    pub down_path: Option<PathBuf>,
    pub up_sql: String,
    pub checksum: String,
}

pub fn discover_migrations(dir: &Path) -> Result<Vec<MigrationFile>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut migrations = Vec::new();
    for entry in fs::read_dir(dir).context("Reading migration directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("sql") {
            continue;
        }

        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        if filename.ends_with(".up.sql") || (!filename.contains(".down.") && filename.ends_with(".sql")) {
            let parts: Vec<&str> = filename.split('_').collect();
            if parts.is_empty() {
                continue;
            }

            if let Ok(version) = parts[0].parse::<i64>() {
                let up_sql = fs::read_to_string(&path)?;
                let checksum = compute_sha256(&up_sql);
                let name = parts[1..].join("_").replace(".up.sql", "").replace(".sql", "");

                let down_name = format!("{}_{}.down.sql", parts[0], name);
                let down_path = dir.join(&down_name);
                let down_exists = if down_path.exists() { Some(down_path) } else { None };

                migrations.push(MigrationFile {
                    version,
                    name,
                    up_path: path,
                    down_path: down_exists,
                    up_sql,
                    checksum,
                });
            }
        }
    }

    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}
