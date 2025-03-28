# Rs-Postgres 🐘🦀

[![Rust](https://img.shields.io/badge/Rust-1.72+-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![Header](assets/other/git-header.png)

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
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

### Automatic installation
```bash
bash install.sh
```
This script will install Rs-Postgres to your system and create a desktop entry.

### Build from Source
```bash
git clone https://github.com/pywebsol/rs-postgres.git
cd rs-postgres
cargo run --release
```

## Usage 🖥️
1. **Login**
   - Enter your encryption password or create new one
   - Click "Login" (Enter)

2. **Add Server Connection**
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

3. **Execute Queries**
   - Select database in connection tree
   - Write SQL in editor panel
   - Click "Run" (F5) to execute

4. **Result Handling**
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
- [x] SQL scripts opening and saving
- [ ] Creating new databases and tables in the connection tree
- [ ] Connection health monitoring
- [x] Query execution time tracking
- [x] Query results pagination
- [x] Query results export to CSV
- [x] Editing server connection details
- [x] Syntax highlighting
- [x] Translations
- [x] SQL query operations presets for tables

# Support 🤗
If you have any questions, do not hesitate to write to me: https://t.me/bot_token

# Screenshots 📸
![Login page](assets/screenshots/login.png)
![Welcome page](assets/screenshots/welcome.png)
![Add server](assets/screenshots/add_server.png)
![SQL query tool](assets/screenshots/sql_query_tool.png)
![Settings](assets/screenshots/settings.png)

---

> "Simplicity is the ultimate sophistication" - Leonardo da Vinci