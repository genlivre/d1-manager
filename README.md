# D1 Manager

A production-safe, native GUI client for Cloudflare D1 databases.

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

## Features

### Core Features

- **Multi-connection support** - Manage multiple D1 databases with tabs
- **Table browser** - Browse tables, view schema, and explore data with pagination
- **SQL editor** - Write and execute SQL queries with syntax highlighting
- **Data export** - Export to CSV, JSON, or SQL formats
- **Data import** - Import from CSV, JSON, SQL, or MySQL dump files
- **Batch execution** - Execute multiple SQL statements with progress tracking
- **Saved queries** - Save and manage frequently used queries
- **Local D1 support** - Connect to Wrangler local D1 databases

### Safety Features

- **Environment badges** - Visual DEV/STG/PROD indicators with color coding
- **Production lock** - Block dangerous operations with timeout-based unlock
- **Dangerous query detection** - 5-level risk analysis (Safe/Low/Medium/High/Critical)
- **Execution log** - Track all queries with timestamps, duration, and affected rows
- **Audit journal** - Record all data changes with before/after values and rollback SQL generation
- **Query history** - Search and filter past query executions (up to 500 entries)

### Advanced Features

- **Schema comparison** - Compare schemas between two databases with diff visualization
- **Schema explorer** - Visualize foreign key relationships and generate ER diagrams
- **Migration SQL generation** - Auto-generate migration scripts from schema diffs
- **Smart SQL suggestions** - Context-aware query suggestions (local processing only, no external API)
- **MySQL dump import** - Convert MySQL dumps to SQLite-compatible SQL
- **Data filtering** - Advanced filtering with operators (=, !=, >, <, LIKE, IN, IS NULL, etc.)
- **Inline cell editing** - Edit data directly in the table with undo/redo support
- **Row operations** - Insert, delete, and duplicate rows

### Security

- **Secure credential storage** - Uses OS-native keychain (macOS Keychain/Windows Credential Manager/Linux Secret Service)
- **Memory protection** - API tokens are zeroed on drop using `zeroize`
- **Settings backup** - Export/import profiles with optional password protection

### Monitoring

- **Rate limit tracking** - Monitor Cloudflare API rate limits in real-time
- **Usage metrics** - Track session statistics (queries, rows read/written, execution time)
- **Update checker** - Check for new versions from GitHub Releases

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
- `saved_queries.json` - User-saved query templates
- `metadata_cache.json` - Cached table schemas
- `audit_journal.json` - Change audit log with rollback SQL
- `execution_log.json` - Detailed query execution log

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
├── main.rs            # Entry point, eframe window configuration
├── app.rs             # Main application state and UI logic
├── api.rs             # Cloudflare D1 API client
├── i18n.rs            # Internationalization (EN/JA)
├── theme.rs           # UI theme, colors, and styled widgets
├── secure_storage.rs  # Cross-platform credential storage (Keychain/Credential Manager/Secret Service)
├── settings_io.rs     # Settings export/import with encryption
├── export.rs          # Data export/import (CSV/JSON/SQL/MySQL dump)
├── sql_safety.rs      # SQL risk analysis and safety checks
├── sql_highlight.rs   # SQL syntax highlighting
├── audit.rs           # Audit journal with rollback SQL generation
├── schema_diff.rs     # Schema comparison and migration generation
├── schema_explorer.rs # Schema exploration and ER diagram generation
├── ai_suggest.rs      # Local SQL suggestions (no external API)
├── local_db.rs        # Wrangler local D1 database discovery
└── version.rs         # Version info and update checker
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
