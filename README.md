# D1 Manager

A production-safe, native GUI client for Cloudflare D1 databases.

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

## Features

### Core Features

- **Multi-connection support** - Manage multiple D1 databases with tabs
- **Table browser** - Browse tables, view schema, and explore data
- **SQL editor** - Write and execute SQL queries with syntax highlighting
- **Data export** - Export to CSV, JSON, or SQL formats
- **Data import** - Import from CSV, JSON, SQL, or MySQL dump files

### Safety Features

- **Environment badges** - Visual DEV/STG/PROD indicators
- **Production lock** - Block dangerous operations in production
- **Dangerous query detection** - Warn before DELETE without WHERE, DROP TABLE, etc.
- **Execution log** - Track all queries with timestamps and affected rows
- **Audit journal** - Record all data changes with rollback SQL generation

### Advanced Features

- **Schema comparison** - Compare schemas between two databases
- **Migration SQL generation** - Auto-generate migration scripts
- **Smart SQL suggestions** - Context-aware query suggestions (local processing only)
- **MySQL dump import** - Convert MySQL dumps to SQLite-compatible SQL

### Security

- **Secure credential storage** - Uses OS-native keychain (macOS/Windows/Linux)
- **Memory protection** - API tokens are zeroed on drop
- **Encrypted exports** - AES-256-GCM encryption for settings backup

## Installation

### Prerequisites

- Rust 1.70 or later
- Platform-specific dependencies:
  - **Linux**: `libsecret-1-dev` (for keyring support)

### Build from source

```bash
git clone https://github.com/yourusername/d1-manager.git
cd d1-manager
cargo build --release
```

The binary will be at `target/release/d1-manager`.

### Platform-specific notes

#### Linux

Install libsecret for credential storage:

```bash
# Ubuntu/Debian
sudo apt install libsecret-1-dev

# Fedora
sudo dnf install libsecret-devel

# Arch
sudo pacman -S libsecret
```

## Usage

### Creating a connection

1. Launch D1 Manager
2. Click "+ New Connection"
3. Enter your connection details:
   - **Name**: A friendly name for this connection
   - **Account ID**: Your Cloudflare account ID
   - **Database ID**: Your D1 database ID
   - **API Token**: Your Cloudflare API token with D1 permissions
   - **Environment**: Development, Staging, or Production
   - **Read-only**: Enable to prevent write operations

### Getting your Cloudflare credentials

1. **Account ID**: Found in the Cloudflare dashboard URL or Workers & Pages overview
2. **Database ID**: Found in Workers & Pages > D1 > Your Database
3. **API Token**: Create at [Cloudflare API Tokens](https://dash.cloudflare.com/profile/api-tokens)
   - Use template "Edit Cloudflare Workers" or create custom with D1 permissions

### SQL Editor

- Write SQL queries in the editor
- Click "Execute" or press the run button
- Use the format button to auto-format SQL
- Access query history and snippets from the toolbar
- Use AI suggestions (lightbulb icon) for context-aware query templates

### Safety features

#### Production Lock

When a connection is marked as "Production":

- DELETE, DROP, and TRUNCATE require confirmation
- Visual warning badges are displayed
- Execution log tracks all operations

#### Audit Journal

- Automatically records INSERT, UPDATE, DELETE operations
- View change history with before/after values
- Generate rollback SQL to undo changes

## Configuration

Settings are stored in:

- **macOS**: `~/Library/Application Support/d1-manager/`
- **Windows**: `%APPDATA%\d1-manager\`
- **Linux**: `~/.config/d1-manager/`

### Files

- `profiles.json` - Connection profiles (without API tokens)
- `query_history.json` - Query execution history
- `metadata_cache.json` - Cached table schemas
- `audit_journal.json` - Change audit log
- `execution_log.json` - Query execution log

## Development

### Running tests

```bash
cargo test
```

### Building for release

```bash
cargo build --release
```

### Project structure

```text
src/
├── main.rs            # Entry point
├── app.rs             # Main application (egui UI)
├── api.rs             # Cloudflare D1 API client
├── i18n.rs            # Internationalization (EN/JA)
├── theme.rs           # UI theme and colors
├── secure_storage.rs  # Cross-platform credential storage
├── settings_io.rs     # Settings export/import
├── export.rs          # Data export (CSV/JSON/SQL/MySQL)
├── sql_safety.rs      # SQL safety checks
├── sql_highlight.rs   # SQL syntax highlighting
├── audit.rs           # Audit journal
├── schema_diff.rs     # Schema comparison
├── schema_explorer.rs # Schema exploration
├── ai_suggest.rs      # AI SQL suggestions
└── local_db.rs        # Local cache
```

## Localization

D1 Manager supports:

- English (default)
- Japanese (日本語)

Language is automatically detected from system settings.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built with [egui](https://github.com/emilk/egui) - Immediate mode GUI library for Rust
- Uses [keyring](https://github.com/hwchen/keyring-rs) for secure credential storage
