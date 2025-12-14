# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

D1 Manager is a native desktop GUI client for managing Cloudflare D1 databases. Built with Rust using the egui/eframe framework, it provides a visual interface for browsing tables, editing data, running SQL queries, and importing/exporting data.

## Build Commands

```bash
# Development build
cargo build

# Release build (optimized for size with LTO)
cargo build --release

# Run the application
cargo run

# Run tests
cargo test

# Run a specific test
cargo test <test_name>
```

## Architecture

### Module Structure

- **main.rs** - Application entry point, configures eframe window
- **app.rs** - Main application state and UI logic (`D1ManagerApp` struct), handles all egui rendering
- **api.rs** - Cloudflare D1 API client (`D1Client`), handles HTTP requests to D1 REST API
- **export.rs** - Data export (CSV, JSON, SQL) and import functionality with format parsing
- **i18n.rs** - Internationalization with English and Japanese language support
- **secure_storage.rs** - API token storage using macOS Keychain via `keyring` crate
- **local_db.rs** - Local wrangler D1 database discovery (`.wrangler/state/v3/d1/`)
- **theme.rs** - egui theme configuration, color palette (`AppColors`), and custom styled widgets

### Key Patterns

**Async Communication**: The app uses `std::sync::mpsc` channels to communicate between async Tokio tasks (API calls) and the synchronous egui render loop. API responses are sent back via channels and processed in `update()`.

**Connection Profiles**: User connections are stored as `ConnectionProfile` with metadata in JSON files and API tokens stored separately in the macOS Keychain.

**Environment Safety**: Connections have environment types (Development/Staging/Production) that control confirmation dialogs for write operations.

### Dependencies

- **egui/eframe** - Immediate mode GUI framework
- **reqwest** - HTTP client for Cloudflare API
- **tokio** - Async runtime
- **keyring** - Secure credential storage (macOS Keychain)
- **rfd** - Native file dialogs
- **serde/serde_json** - JSON serialization
- **zeroize** - Secure memory wiping for sensitive data

## Cloudflare D1 API

The app communicates with D1 via the REST API at:
```
https://api.cloudflare.com/client/v4/accounts/{account_id}/d1/database/{database_id}/query
```

Queries are sent as POST requests with Bearer token authentication. The API has rate limits (1,200 requests per 5 minutes) which the app tracks and displays.
