# pg-migrate-cli

> Fast, zero-dependency transactional database schema migration CLI for **PostgreSQL** and **SQLite** written in **Rust**.

[![Rust](https://img.shields.io/badge/Rust-2021-DEA584?style=flat-square&logo=rust)](https://rust-lang.org)
[![SQLx](https://img.shields.io/badge/Database-SQLx-336791?style=flat-square&logo=postgresql)](https://github.com/launchbadge/sqlx)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?style=flat-square&logo=docker)](https://docker.com)
[![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)

`#database-migrations` `#postgresql` `#sqlite` `#rust` `#sqlx` `#devops` `#cli` `#schema-management`

---

## Features

- **Transactional Execution:** Each migration is executed inside a safe database transaction (`BEGIN ... COMMIT`).
- **Checksum Verification:** Computes and verifies SHA-256 hashes of SQL scripts to detect modified past migrations.
- **Rollback Support:** Pairs `.up.sql` and `.down.sql` scripts for forward upgrades and backwards rollbacks.
- **CI/CD Friendly:** Environment variable support (`DATABASE_URL`) with non-zero exit codes on checksum errors.

## Quick Start

```bash
# 1. Create a new migration pair
cargo run -- create add_organization_table

# 2. Run pending migrations
export DATABASE_URL="postgres://postgres:secret@localhost:5432/appdb"
cargo run -- up

# 3. Check migration status
cargo run -- status

# 4. Rollback last migration
cargo run -- down
```
