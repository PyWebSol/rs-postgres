# Rs-Postgres 🐘🦀

[![Rust](https://img.shields.io/badge/Rust-1.72+-blue?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Modern PostgreSQL client with native GUI, built in Rust using [egui](https://www.egui.rs/). Designed for developers who want a lightweight and fast database management tool.

## Features ✨
- Multi-server connections management
- Interactive SQL query editor
- Query results in tabular format
- Database tree navigation
- Cross-platform (Windows, Linux, macOS)

## Installation 📦

### Prerequisites
- Rust ([install guide](https://www.rust-lang.org/tools/install))

### Build from Source
```bash
git clone https://github.com/pywebsol/rs-postgres.git
cd rs-postgres
cargo run --release
```

## Usage 🖥️
1. **Add Server Connection**
   - Click "Add Server" in left panel
   - Enter connection details:
     ```yaml
     Name: My Production DB
     Host: db.example.com
     Port: 5432
     User: admin
     Password: ********
     Service DB: postgres
     ```

2. **Execute Queries**
   - Select database in connection tree
   - Write SQL in editor panel
   - Click "Run" (F5) to execute

3. **Result Handling**
   - View results in table
   - Click on cells to copy values

## Development 🛠️

### Build Commands
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run with debug logging
cargo run -- --debug
```

## Roadmap 🗺️
- [ ] SQL scripts opening and saving
- [ ] Creating new databases and tables in the connection tree
- [ ] Connection health monitoring
- [x] Query execution time tracking
- [ ] Query results pagination
- [ ] Query results export to CSV and JSON
- [ ] Editing server connection details
- [x] Syntax highlighting
- [ ] Translations

# Support 🤗
If you have any questions, do not hesitate to write to me: https://t.me/bot_token

---

> "Simplicity is the ultimate sophistication" - Leonardo da Vinci