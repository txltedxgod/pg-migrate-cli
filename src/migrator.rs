use crate::checksum::compute_sha256;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MigrationFile {
    pub version: i64,
    pub name: String,
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

        let Some(name_os) = path.file_name() else {
            continue;
        };
        let filename = name_os.to_string_lossy().to_string();
        if filename.ends_with(".up.sql")
            || (!filename.contains(".down.") && filename.ends_with(".sql"))
        {
            let parts: Vec<&str> = filename.split('_').collect();
            if parts.is_empty() {
                continue;
            }

            if let Ok(version) = parts[0].parse::<i64>() {
                let up_sql = fs::read_to_string(&path)?;
                let checksum = compute_sha256(&up_sql);
                let name = parts[1..]
                    .join("_")
                    .replace(".up.sql", "")
                    .replace(".sql", "");

                let down_name = format!("{}_{}.down.sql", parts[0], name);
                let down_path = dir.join(&down_name);
                let down_exists = if down_path.exists() {
                    Some(down_path)
                } else {
                    None
                };

                migrations.push(MigrationFile {
                    version,
                    name,
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

#[cfg(test)]
mod tests {
    use super::discover_migrations;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_discover_migrations_missing_dir_returns_empty() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let missing = std::env::temp_dir().join(format!("pg_migrate_missing_{nanos}"));
        let found = discover_migrations(&missing).expect("missing dir should return empty vec");
        assert!(found.is_empty());
    }

    #[test]
    fn test_discover_migrations_sorts_and_pairs_down_files() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pg_migrate_test_{nanos}"));
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("002_posts.up.sql"), "CREATE TABLE posts (id INT);").unwrap();
        fs::write(dir.join("001_users.up.sql"), "CREATE TABLE users (id INT);").unwrap();
        fs::write(dir.join("001_users.down.sql"), "DROP TABLE users;").unwrap();
        fs::write(dir.join("notes.txt"), "ignore non-sql").unwrap();

        let migrations = discover_migrations(&dir).expect("should parse migrations");
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].name, "users");
        assert!(migrations[0].down_path.is_some());
        assert_eq!(migrations[1].version, 2);
        assert_eq!(migrations[1].name, "posts");
        assert!(migrations[1].down_path.is_none());
    }
}
