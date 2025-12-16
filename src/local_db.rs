#![allow(dead_code)]

use std::path::{Path, PathBuf};
use rusqlite::{Connection, params_from_iter};

/// Structured error type for local database operations
#[derive(Debug, Clone)]
pub enum LocalDbError {
    NotFound(String),
    OpenFailed(String),
    CreateFailed(String),
    PrepareFailed(String),
    QueryFailed(String),
    ExecuteFailed(String),
}

impl LocalDbError {
    /// Get the error kind for translation lookup
    pub fn kind(&self) -> &'static str {
        match self {
            LocalDbError::NotFound(_) => "not_found",
            LocalDbError::OpenFailed(_) => "open_failed",
            LocalDbError::CreateFailed(_) => "create_failed",
            LocalDbError::PrepareFailed(_) => "prepare_failed",
            LocalDbError::QueryFailed(_) => "query_failed",
            LocalDbError::ExecuteFailed(_) => "execute_failed",
        }
    }

    /// Get the technical detail (preserved in original language for searchability)
    pub fn detail(&self) -> &str {
        match self {
            LocalDbError::NotFound(s) => s,
            LocalDbError::OpenFailed(s) => s,
            LocalDbError::CreateFailed(s) => s,
            LocalDbError::PrepareFailed(s) => s,
            LocalDbError::QueryFailed(s) => s,
            LocalDbError::ExecuteFailed(s) => s,
        }
    }
}

impl std::fmt::Display for LocalDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalDbError::NotFound(path) => write!(f, "Database file not found: {}", path),
            LocalDbError::OpenFailed(e) => write!(f, "Failed to open database: {}", e),
            LocalDbError::CreateFailed(e) => write!(f, "Failed to create database: {}", e),
            LocalDbError::PrepareFailed(e) => write!(f, "Prepare error: {}", e),
            LocalDbError::QueryFailed(e) => write!(f, "Query error: {}", e),
            LocalDbError::ExecuteFailed(e) => write!(f, "Execute error: {}", e),
        }
    }
}

/// Find wrangler local D1 database files
/// These are stored in .wrangler/state/v3/d1/<binding>/<db-id>.sqlite
pub fn find_local_d1_databases() -> Vec<LocalD1Database> {
    let mut databases = Vec::new();

    // Check common project locations
    let search_paths = vec![
        std::env::current_dir().ok(),
        dirs::home_dir(),
    ];

    for base_path in search_paths.into_iter().flatten() {
        // Look for .wrangler directories
        find_wrangler_dbs(&base_path, &mut databases);
    }

    databases
}

/// Search for wrangler state directories and extract D1 databases
fn find_wrangler_dbs(base: &PathBuf, databases: &mut Vec<LocalD1Database>) {
    let wrangler_state = base.join(".wrangler").join("state").join("v3").join("d1");

    if wrangler_state.exists() && wrangler_state.is_dir() {
        if let Ok(bindings) = std::fs::read_dir(&wrangler_state) {
            for binding_entry in bindings.flatten() {
                let binding_path = binding_entry.path();
                if binding_path.is_dir() {
                    let binding_name = binding_entry.file_name().to_string_lossy().to_string();

                    if let Ok(db_files) = std::fs::read_dir(&binding_path) {
                        for db_entry in db_files.flatten() {
                            let db_path = db_entry.path();
                            if db_path.extension().map(|e| e == "sqlite").unwrap_or(false) {
                                let db_name = db_path.file_stem()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_default();

                                databases.push(LocalD1Database {
                                    name: format!("{} ({})", binding_name, &db_name[..8.min(db_name.len())]),
                                    binding: binding_name.clone(),
                                    path: db_path,
                                    project_path: base.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Represents a local D1 database (SQLite file)
#[derive(Debug, Clone)]
pub struct LocalD1Database {
    pub name: String,
    pub binding: String,
    pub path: PathBuf,
    pub project_path: PathBuf,
}

impl LocalD1Database {
    /// Get the SQLite connection string
    pub fn connection_string(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

/// Column info from local database
#[derive(Debug, Clone)]
pub struct LocalColumnInfo {
    pub name: String,
    pub col_type: String,
    pub pk: bool,
    pub notnull: bool,
}

/// Local SQLite database client using rusqlite
#[derive(Debug)]
pub struct LocalD1Client {
    path: PathBuf,
}

impl LocalD1Client {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Create a new empty SQLite database
    pub fn create_new(path: &Path) -> Result<Self, LocalDbError> {
        let conn = Connection::open(path)
            .map_err(|e| LocalDbError::CreateFailed(e.to_string()))?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| LocalDbError::CreateFailed(format!("journal_mode: {}", e)))?;

        drop(conn);
        Ok(Self { path: path.to_path_buf() })
    }

    /// Execute multiple SQL statements in a batch (for template application)
    pub fn execute_batch(&self, sql: &str) -> Result<(), LocalDbError> {
        if !self.path.exists() {
            return Err(LocalDbError::NotFound(self.path.display().to_string()));
        }

        let conn = Connection::open(&self.path)
            .map_err(|e| LocalDbError::OpenFailed(e.to_string()))?;

        conn.execute_batch(sql)
            .map_err(|e| LocalDbError::ExecuteFailed(e.to_string()))?;

        Ok(())
    }

    /// Execute a SQL query on the local database
    pub fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<LocalQueryResult, LocalDbError> {
        if !self.path.exists() {
            return Err(LocalDbError::NotFound(self.path.display().to_string()));
        }

        let conn = Connection::open(&self.path)
            .map_err(|e| LocalDbError::OpenFailed(e.to_string()))?;

        // Convert JSON params to rusqlite values
        let param_values: Vec<rusqlite::types::Value> = params
            .iter()
            .map(json_to_sqlite_value)
            .collect();

        // Check if SELECT or write operation
        let sql_upper = sql.trim().to_uppercase();
        if sql_upper.starts_with("SELECT") || sql_upper.starts_with("PRAGMA") {
            let mut stmt = conn.prepare(sql)
                .map_err(|e| LocalDbError::PrepareFailed(e.to_string()))?;

            let column_count = stmt.column_count();
            let columns: Vec<String> = (0..column_count)
                .map(|i| stmt.column_name(i).unwrap_or("").to_string())
                .collect();

            let rows_result: Result<Vec<Vec<serde_json::Value>>, rusqlite::Error> = stmt
                .query_map(params_from_iter(param_values), |row| {
                    let values: Vec<serde_json::Value> = (0..column_count)
                        .map(|i| sqlite_value_to_json(row.get_ref(i).ok()))
                        .collect();
                    Ok(values)
                })
                .map_err(|e| LocalDbError::QueryFailed(e.to_string()))?
                .collect();

            let rows = rows_result.map_err(|e| LocalDbError::QueryFailed(e.to_string()))?;

            Ok(LocalQueryResult { columns, rows, changes: 0 })
        } else {
            let changes = conn.execute(sql, params_from_iter(param_values))
                .map_err(|e| LocalDbError::ExecuteFailed(e.to_string()))?;

            Ok(LocalQueryResult {
                columns: vec![],
                rows: vec![],
                changes: changes as i64,
            })
        }
    }

    /// Get list of tables from local database
    pub fn get_tables(&self) -> Result<Vec<String>, LocalDbError> {
        if !self.path.exists() {
            return Err(LocalDbError::NotFound(self.path.display().to_string()));
        }

        let result = self.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            vec![]
        )?;

        Ok(result.rows.iter()
            .filter_map(|row| row.first()?.as_str().map(String::from))
            .collect())
    }

    /// Get table schema
    pub fn get_table_schema(&self, table: &str) -> Result<Vec<LocalColumnInfo>, LocalDbError> {
        let result = self.execute(&format!("PRAGMA table_info({})", table), vec![])?;

        Ok(result.rows.iter()
            .filter_map(|row| {
                Some(LocalColumnInfo {
                    name: row.get(1)?.as_str()?.to_string(),
                    col_type: row.get(2)?.as_str().unwrap_or("").to_string(),
                    pk: row.get(5)?.as_i64().unwrap_or(0) == 1,
                    notnull: row.get(3)?.as_i64().unwrap_or(0) == 1,
                })
            })
            .collect())
    }

    /// Get row count for a table
    pub fn get_row_count(&self, table: &str) -> Result<i64, LocalDbError> {
        let result = self.execute(&format!("SELECT COUNT(*) as count FROM {}", table), vec![])?;

        Ok(result.rows.first()
            .and_then(|row| row.first())
            .and_then(|v| v.as_i64())
            .unwrap_or(0))
    }

    /// Get table data with pagination
    pub fn get_table_data(&self, table: &str, limit: i64, offset: i64) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>), LocalDbError> {
        let result = self.execute(
            &format!("SELECT * FROM {} LIMIT {} OFFSET {}", table, limit, offset),
            vec![]
        )?;

        Ok((result.columns, result.rows))
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Convert serde_json::Value to rusqlite Value
fn json_to_sqlite_value(v: &serde_json::Value) -> rusqlite::types::Value {
    match v {
        serde_json::Value::Null => rusqlite::types::Value::Null,
        serde_json::Value::Bool(b) => rusqlite::types::Value::Integer(*b as i64),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                rusqlite::types::Value::Real(f)
            } else {
                rusqlite::types::Value::Null
            }
        }
        serde_json::Value::String(s) => rusqlite::types::Value::Text(s.clone()),
        _ => rusqlite::types::Value::Text(v.to_string()),
    }
}

/// Convert rusqlite ValueRef to serde_json::Value
fn sqlite_value_to_json(v: Option<rusqlite::types::ValueRef>) -> serde_json::Value {
    match v {
        None => serde_json::Value::Null,
        Some(val) => match val {
            rusqlite::types::ValueRef::Null => serde_json::Value::Null,
            rusqlite::types::ValueRef::Integer(i) => serde_json::json!(i),
            rusqlite::types::ValueRef::Real(f) => serde_json::json!(f),
            rusqlite::types::ValueRef::Text(s) => {
                serde_json::Value::String(String::from_utf8_lossy(s).to_string())
            }
            rusqlite::types::ValueRef::Blob(b) => {
                // Encode blob as base64 string for JSON representation
                serde_json::Value::String(format!("[BLOB {} bytes]", b.len()))
            }
        }
    }
}

/// Result from local query execution
#[derive(Debug)]
pub struct LocalQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub changes: i64,
}

/// Schema diff between two databases
#[derive(Debug, Clone)]
pub struct SchemaDiff {
    pub table_name: String,
    pub diff_type: SchemaDiffType,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum SchemaDiffType {
    TableAdded,
    TableRemoved,
    ColumnAdded,
    ColumnRemoved,
    ColumnModified,
}

impl SchemaDiffType {
    pub fn label(&self) -> &'static str {
        match self {
            SchemaDiffType::TableAdded => "Table Added",
            SchemaDiffType::TableRemoved => "Table Removed",
            SchemaDiffType::ColumnAdded => "Column Added",
            SchemaDiffType::ColumnRemoved => "Column Removed",
            SchemaDiffType::ColumnModified => "Column Modified",
        }
    }

    pub fn is_addition(&self) -> bool {
        matches!(self, SchemaDiffType::TableAdded | SchemaDiffType::ColumnAdded)
    }

    pub fn is_removal(&self) -> bool {
        matches!(self, SchemaDiffType::TableRemoved | SchemaDiffType::ColumnRemoved)
    }
}

/// Compare schemas between two databases and return differences
pub fn compare_schemas(
    local_tables: &[String],
    remote_tables: &[String],
) -> Vec<SchemaDiff> {
    let mut diffs = Vec::new();

    // Tables in local but not in remote (new tables)
    for table in local_tables {
        if !remote_tables.contains(table) {
            diffs.push(SchemaDiff {
                table_name: table.clone(),
                diff_type: SchemaDiffType::TableAdded,
                details: format!("Table '{}' exists locally but not in remote", table),
            });
        }
    }

    // Tables in remote but not in local (removed tables)
    for table in remote_tables {
        if !local_tables.contains(table) {
            diffs.push(SchemaDiff {
                table_name: table.clone(),
                diff_type: SchemaDiffType::TableRemoved,
                details: format!("Table '{}' exists in remote but not locally", table),
            });
        }
    }

    diffs
}

// ============================================================================
// Database Templates
// ============================================================================

/// A database template with schema and sample data
#[derive(Debug, Clone)]
pub struct DatabaseTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub schema_sql: &'static str,
    pub seed_sql: &'static str,
}

/// Get all built-in database templates
pub fn builtin_templates() -> &'static [DatabaseTemplate] {
    &[
        TEMPLATE_EMPTY,
        TEMPLATE_ECOMMERCE,
        TEMPLATE_BLOG,
        TEMPLATE_TASKS,
    ]
}

/// Empty database template
pub const TEMPLATE_EMPTY: DatabaseTemplate = DatabaseTemplate {
    id: "empty",
    name: "Empty Database",
    description: "Start with a blank SQLite database",
    schema_sql: "",
    seed_sql: "",
};

/// E-Commerce database template
pub const TEMPLATE_ECOMMERCE: DatabaseTemplate = DatabaseTemplate {
    id: "ecommerce",
    name: "E-Commerce",
    description: "Users, products, orders with sample data",
    schema_sql: r#"
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    price REAL NOT NULL,
    stock INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    total REAL NOT NULL,
    status TEXT DEFAULT 'pending',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    price REAL NOT NULL,
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (product_id) REFERENCES products(id)
);
"#,
    seed_sql: r#"
INSERT INTO users (email, name) VALUES
    ('alice@example.com', 'Alice Johnson'),
    ('bob@example.com', 'Bob Smith'),
    ('carol@example.com', 'Carol Williams');

INSERT INTO products (title, description, price, stock) VALUES
    ('Laptop Pro', 'High-performance laptop for professionals', 1299.99, 50),
    ('Wireless Mouse', 'Ergonomic wireless mouse', 29.99, 200),
    ('USB-C Hub', '7-in-1 USB-C hub with HDMI', 49.99, 150),
    ('Mechanical Keyboard', 'RGB mechanical keyboard', 89.99, 75),
    ('Monitor Stand', 'Adjustable monitor stand', 39.99, 100);

INSERT INTO orders (user_id, total, status) VALUES
    (1, 1379.97, 'completed'),
    (2, 119.98, 'shipped'),
    (3, 49.99, 'pending');

INSERT INTO order_items (order_id, product_id, quantity, price) VALUES
    (1, 1, 1, 1299.99),
    (1, 2, 1, 29.99),
    (1, 3, 1, 49.99),
    (2, 4, 1, 89.99),
    (2, 2, 1, 29.99),
    (3, 3, 1, 49.99);
"#,
};

/// Blog platform database template
pub const TEMPLATE_BLOG: DatabaseTemplate = DatabaseTemplate {
    id: "blog",
    name: "Blog Platform",
    description: "Posts, comments, tags with sample content",
    schema_sql: r#"
CREATE TABLE authors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    bio TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    author_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    published INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES authors(id)
);

CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE
);

CREATE TABLE post_tags (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TABLE comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER NOT NULL,
    author_name TEXT NOT NULL,
    email TEXT NOT NULL,
    content TEXT NOT NULL,
    approved INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (post_id) REFERENCES posts(id)
);
"#,
    seed_sql: r#"
INSERT INTO authors (username, email, bio) VALUES
    ('johndoe', 'john@example.com', 'Tech writer and developer'),
    ('janedoe', 'jane@example.com', 'Full-stack engineer');

INSERT INTO posts (author_id, title, slug, content, published) VALUES
    (1, 'Getting Started with D1', 'getting-started-d1', 'Cloudflare D1 is a serverless SQL database...', 1),
    (1, 'Building APIs with Workers', 'building-apis-workers', 'Learn how to build REST APIs using Cloudflare Workers...', 1),
    (2, 'Database Best Practices', 'database-best-practices', 'Here are some tips for optimizing your database...', 0);

INSERT INTO tags (name, slug) VALUES
    ('cloudflare', 'cloudflare'),
    ('database', 'database'),
    ('tutorial', 'tutorial'),
    ('api', 'api');

INSERT INTO post_tags (post_id, tag_id) VALUES
    (1, 1), (1, 2), (1, 3),
    (2, 1), (2, 4), (2, 3),
    (3, 2);

INSERT INTO comments (post_id, author_name, email, content, approved) VALUES
    (1, 'Reader One', 'reader1@example.com', 'Great introduction!', 1),
    (1, 'Reader Two', 'reader2@example.com', 'Very helpful, thanks!', 1),
    (2, 'API Fan', 'apifan@example.com', 'Exactly what I needed!', 1);
"#,
};

/// Task management database template
pub const TEMPLATE_TASKS: DatabaseTemplate = DatabaseTemplate {
    id: "tasks",
    name: "Task Management",
    description: "Projects, tasks, users for project management",
    schema_sql: r#"
CREATE TABLE team_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    role TEXT DEFAULT 'member',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    owner_id INTEGER NOT NULL,
    status TEXT DEFAULT 'active',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (owner_id) REFERENCES team_members(id)
);

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    assignee_id INTEGER,
    priority TEXT DEFAULT 'medium',
    status TEXT DEFAULT 'todo',
    due_date TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (assignee_id) REFERENCES team_members(id)
);

CREATE TABLE task_comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES tasks(id),
    FOREIGN KEY (author_id) REFERENCES team_members(id)
);
"#,
    seed_sql: r#"
INSERT INTO team_members (email, name, role) VALUES
    ('lead@example.com', 'Team Lead', 'admin'),
    ('dev1@example.com', 'Developer One', 'member'),
    ('dev2@example.com', 'Developer Two', 'member');

INSERT INTO projects (name, description, owner_id, status) VALUES
    ('Website Redesign', 'Complete redesign of company website', 1, 'active'),
    ('Mobile App', 'New mobile application for customers', 1, 'active'),
    ('API v2', 'Next version of our public API', 2, 'planning');

INSERT INTO tasks (project_id, title, description, assignee_id, priority, status, due_date) VALUES
    (1, 'Design mockups', 'Create initial design mockups', 2, 'high', 'in_progress', '2025-02-01'),
    (1, 'Implement homepage', 'Build the new homepage', 2, 'high', 'todo', '2025-02-15'),
    (1, 'Setup CI/CD', 'Configure deployment pipeline', 3, 'medium', 'done', '2025-01-15'),
    (2, 'User authentication', 'Implement login/signup', 3, 'high', 'in_progress', '2025-02-10'),
    (2, 'Push notifications', 'Add push notification support', NULL, 'low', 'todo', NULL),
    (3, 'API specification', 'Write OpenAPI spec', 2, 'medium', 'todo', '2025-03-01');

INSERT INTO task_comments (task_id, author_id, content) VALUES
    (1, 1, 'Looking good so far!'),
    (1, 2, 'Thanks! Will have more ready by Friday.'),
    (4, 1, 'Consider using OAuth2 for third-party auth.');
"#,
};

/// Create a demo database with sample data
pub fn create_demo_database() -> Result<PathBuf, LocalDbError> {
    // Get data directory
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("d1-manager");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| LocalDbError::CreateFailed(format!("Failed to create data directory: {}", e)))?;

    let demo_path = data_dir.join("demo.sqlite");

    // Create the database
    let client = LocalD1Client::create_new(&demo_path)?;

    // Apply the e-commerce template
    apply_template(&client, &TEMPLATE_ECOMMERCE)?;

    Ok(demo_path)
}

/// Apply a database template to an existing database
pub fn apply_template(client: &LocalD1Client, template: &DatabaseTemplate) -> Result<(), LocalDbError> {
    // Apply schema
    if !template.schema_sql.is_empty() {
        client.execute_batch(template.schema_sql)?;
    }

    // Apply seed data
    if !template.seed_sql.is_empty() {
        client.execute_batch(template.seed_sql)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_schemas() {
        let local = vec!["users".to_string(), "posts".to_string(), "comments".to_string()];
        let remote = vec!["users".to_string(), "posts".to_string(), "likes".to_string()];

        let diffs = compare_schemas(&local, &remote);

        assert_eq!(diffs.len(), 2);
        assert!(diffs.iter().any(|d| d.table_name == "comments" && matches!(d.diff_type, SchemaDiffType::TableAdded)));
        assert!(diffs.iter().any(|d| d.table_name == "likes" && matches!(d.diff_type, SchemaDiffType::TableRemoved)));
    }

    #[test]
    fn test_builtin_templates() {
        let templates = builtin_templates();
        assert_eq!(templates.len(), 4);
        assert_eq!(templates[0].id, "empty");
        assert_eq!(templates[1].id, "ecommerce");
        assert_eq!(templates[2].id, "blog");
        assert_eq!(templates[3].id, "tasks");
    }
}
