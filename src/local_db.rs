#![allow(dead_code)]

use std::path::PathBuf;

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

/// Local SQLite database client
#[derive(Debug)]
pub struct LocalD1Client {
    path: PathBuf,
}

impl LocalD1Client {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Execute a SQL query on the local database
    /// This is a simplified implementation - in production you'd use rusqlite
    pub fn execute(&self, _sql: &str) -> Result<LocalQueryResult, String> {
        // For now, we'll just check if the file exists
        if !self.path.exists() {
            return Err(format!("Database file not found: {}", self.path.display()));
        }

        // In a full implementation, we would:
        // 1. Open SQLite connection with rusqlite
        // 2. Execute the query
        // 3. Return results

        // For now, return a placeholder
        Ok(LocalQueryResult {
            columns: vec![],
            rows: vec![],
            changes: 0,
        })
    }

    /// Get list of tables from local database
    pub fn get_tables(&self) -> Result<Vec<String>, String> {
        // Placeholder - would use rusqlite to query sqlite_master
        if !self.path.exists() {
            return Err(format!("Database file not found: {}", self.path.display()));
        }

        Ok(vec![])
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
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
}
