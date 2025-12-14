//! Audit Journal System (Pseudo Time-travel)
//!
//! This module provides change tracking for D1 database operations:
//! - Records INSERT/UPDATE/DELETE operations with before/after values
//! - Stores change history in local SQLite database
//! - Allows viewing historical states of rows
//! - Generates rollback SQL for reverting changes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type of data modification operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Insert,
    Update,
    Delete,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Insert => write!(f, "INSERT"),
            OperationType::Update => write!(f, "UPDATE"),
            OperationType::Delete => write!(f, "DELETE"),
        }
    }
}

/// A single row change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    /// Unique identifier for this change
    pub id: u64,
    /// Timestamp of the change (Unix epoch seconds)
    pub timestamp: u64,
    /// Database profile name
    pub profile_name: String,
    /// Database ID
    pub database_id: String,
    /// Table name
    pub table_name: String,
    /// Type of operation
    pub operation: OperationType,
    /// Primary key columns and values (for identifying the row)
    pub primary_key: HashMap<String, serde_json::Value>,
    /// Values before the change (None for INSERT)
    pub before: Option<HashMap<String, serde_json::Value>>,
    /// Values after the change (None for DELETE)
    pub after: Option<HashMap<String, serde_json::Value>>,
    /// The original SQL that caused this change
    pub original_sql: String,
    /// User-provided note (optional)
    pub note: Option<String>,
}

impl ChangeRecord {
    /// Generate SQL to rollback this change
    pub fn generate_rollback_sql(&self) -> Option<String> {
        match self.operation {
            OperationType::Insert => {
                // Rollback INSERT = DELETE
                if self.primary_key.is_empty() {
                    return None;
                }
                let where_clause = self.primary_key
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, value_to_sql(v)))
                    .collect::<Vec<_>>()
                    .join(" AND ");
                Some(format!("DELETE FROM {} WHERE {};", self.table_name, where_clause))
            }
            OperationType::Update => {
                // Rollback UPDATE = UPDATE with old values
                let before = self.before.as_ref()?;
                if self.primary_key.is_empty() || before.is_empty() {
                    return None;
                }
                let set_clause = before
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, value_to_sql(v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let where_clause = self.primary_key
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, value_to_sql(v)))
                    .collect::<Vec<_>>()
                    .join(" AND ");
                Some(format!("UPDATE {} SET {} WHERE {};", self.table_name, set_clause, where_clause))
            }
            OperationType::Delete => {
                // Rollback DELETE = INSERT
                let before = self.before.as_ref()?;
                if before.is_empty() {
                    return None;
                }
                let columns = before.keys().cloned().collect::<Vec<_>>().join(", ");
                let values = before
                    .values()
                    .map(value_to_sql)
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!("INSERT INTO {} ({}) VALUES ({});", self.table_name, columns, values))
            }
        }
    }

    /// Get a summary of what changed
    pub fn get_change_summary(&self) -> String {
        match self.operation {
            OperationType::Insert => {
                let count = self.after.as_ref().map(|a| a.len()).unwrap_or(0);
                format!("Inserted row with {} columns", count)
            }
            OperationType::Update => {
                if let (Some(before), Some(after)) = (&self.before, &self.after) {
                    let changed: Vec<String> = after
                        .iter()
                        .filter(|(k, v)| before.get(*k) != Some(v))
                        .map(|(k, _)| k.clone())
                        .collect();
                    if changed.is_empty() {
                        "No changes detected".to_string()
                    } else {
                        format!("Changed columns: {}", changed.join(", "))
                    }
                } else {
                    "Update performed".to_string()
                }
            }
            OperationType::Delete => {
                let count = self.before.as_ref().map(|b| b.len()).unwrap_or(0);
                format!("Deleted row with {} columns", count)
            }
        }
    }
}

/// Convert JSON value to SQL literal
fn value_to_sql(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        serde_json::Value::Array(arr) => {
            format!("'{}'", serde_json::to_string(arr).unwrap_or_default().replace('\'', "''"))
        }
        serde_json::Value::Object(obj) => {
            format!("'{}'", serde_json::to_string(obj).unwrap_or_default().replace('\'', "''"))
        }
    }
}

/// Audit Journal - manages change history
#[derive(Debug, Default)]
pub struct AuditJournal {
    /// All recorded changes (in-memory, most recent first)
    records: Vec<ChangeRecord>,
    /// Next ID to assign
    next_id: u64,
    /// Maximum number of records to keep in memory
    max_records: usize,
    /// Whether audit is enabled
    pub enabled: bool,
}

impl AuditJournal {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_id: 1,
            max_records: 10000,
            enabled: true,
        }
    }

    /// Record a new change
    pub fn record_change(
        &mut self,
        profile_name: String,
        database_id: String,
        table_name: String,
        operation: OperationType,
        primary_key: HashMap<String, serde_json::Value>,
        before: Option<HashMap<String, serde_json::Value>>,
        after: Option<HashMap<String, serde_json::Value>>,
        original_sql: String,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let record = ChangeRecord {
            id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            profile_name,
            database_id,
            table_name,
            operation,
            primary_key,
            before,
            after,
            original_sql,
            note: None,
        };

        self.records.insert(0, record);

        // Trim if exceeds max
        if self.records.len() > self.max_records {
            self.records.truncate(self.max_records);
        }

        id
    }

    /// Get all records for a specific table
    pub fn get_table_history(&self, database_id: &str, table_name: &str) -> Vec<&ChangeRecord> {
        self.records
            .iter()
            .filter(|r| r.database_id == database_id && r.table_name == table_name)
            .collect()
    }

    /// Get all records for a specific database
    pub fn get_database_history(&self, database_id: &str) -> Vec<&ChangeRecord> {
        self.records
            .iter()
            .filter(|r| r.database_id == database_id)
            .collect()
    }

    /// Get records within a time range
    pub fn get_history_in_range(&self, start: u64, end: u64) -> Vec<&ChangeRecord> {
        self.records
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect()
    }

    /// Get a specific record by ID
    pub fn get_record(&self, id: u64) -> Option<&ChangeRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    /// Get all records (most recent first)
    pub fn get_all_records(&self) -> &[ChangeRecord] {
        &self.records
    }

    /// Get record count
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Clear all records
    pub fn clear(&mut self) {
        self.records.clear();
    }

    /// Clear records older than given timestamp
    pub fn clear_before(&mut self, timestamp: u64) {
        self.records.retain(|r| r.timestamp >= timestamp);
    }

    /// Add a note to a record
    pub fn add_note(&mut self, id: u64, note: String) {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == id) {
            record.note = Some(note);
        }
    }

    /// Export records to JSON
    pub fn export_to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.records)
            .map_err(|e| format!("Failed to export audit log: {}", e))
    }

    /// Import records from JSON
    pub fn import_from_json(&mut self, json: &str) -> Result<usize, String> {
        let imported: Vec<ChangeRecord> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to import audit log: {}", e))?;

        let count = imported.len();

        // Update next_id to avoid conflicts
        if let Some(max_id) = imported.iter().map(|r| r.id).max() {
            self.next_id = self.next_id.max(max_id + 1);
        }

        // Merge and sort by timestamp (most recent first)
        self.records.extend(imported);
        self.records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Trim if needed
        if self.records.len() > self.max_records {
            self.records.truncate(self.max_records);
        }

        Ok(count)
    }

    /// Get path for storing audit log
    pub fn get_storage_path() -> Option<PathBuf> {
        dirs::data_local_dir().map(|p| p.join("d1-manager").join("audit_journal.json"))
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_storage_path()
            .ok_or_else(|| "Could not determine storage path".to_string())?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let json = self.export_to_json()?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write audit log: {}", e))
    }

    /// Load from disk
    pub fn load() -> Result<Self, String> {
        let path = Self::get_storage_path()
            .ok_or_else(|| "Could not determine storage path".to_string())?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read audit log: {}", e))?;

        let records: Vec<ChangeRecord> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse audit log: {}", e))?;

        let next_id = records.iter().map(|r| r.id).max().unwrap_or(0) + 1;

        Ok(Self {
            records,
            next_id,
            max_records: 10000,
            enabled: true,
        })
    }
}

/// Parse an SQL statement to extract table name and operation type
pub fn parse_mutation_sql(sql: &str) -> Option<(OperationType, String)> {
    let sql_upper = sql.trim().to_uppercase();

    if sql_upper.starts_with("INSERT") {
        // INSERT INTO table_name ...
        let re_insert = regex_lite::Regex::new(r"(?i)INSERT\s+INTO\s+[`\x22]?(\w+)").ok()?;
        let caps = re_insert.captures(sql)?;
        Some((OperationType::Insert, caps.get(1)?.as_str().to_string()))
    } else if sql_upper.starts_with("UPDATE") {
        // UPDATE table_name SET ...
        let re_update = regex_lite::Regex::new(r"(?i)UPDATE\s+[`\x22]?(\w+)").ok()?;
        let caps = re_update.captures(sql)?;
        Some((OperationType::Update, caps.get(1)?.as_str().to_string()))
    } else if sql_upper.starts_with("DELETE") {
        // DELETE FROM table_name ...
        let re_delete = regex_lite::Regex::new(r"(?i)DELETE\s+FROM\s+[`\x22]?(\w+)").ok()?;
        let caps = re_delete.captures(sql)?;
        Some((OperationType::Delete, caps.get(1)?.as_str().to_string()))
    } else {
        None
    }
}

/// Format timestamp as human-readable string
pub fn format_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);
    let now = std::time::SystemTime::now();

    let secs_ago = now.duration_since(datetime).unwrap_or_default().as_secs();

    if secs_ago < 60 {
        format!("{}s ago", secs_ago)
    } else if secs_ago < 3600 {
        format!("{}m ago", secs_ago / 60)
    } else if secs_ago < 86400 {
        format!("{}h ago", secs_ago / 3600)
    } else {
        format!("{}d ago", secs_ago / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mutation_sql() {
        assert_eq!(
            parse_mutation_sql("INSERT INTO users (name) VALUES ('test')"),
            Some((OperationType::Insert, "users".to_string()))
        );
        assert_eq!(
            parse_mutation_sql("UPDATE users SET name = 'test' WHERE id = 1"),
            Some((OperationType::Update, "users".to_string()))
        );
        assert_eq!(
            parse_mutation_sql("DELETE FROM users WHERE id = 1"),
            Some((OperationType::Delete, "users".to_string()))
        );
        assert_eq!(parse_mutation_sql("SELECT * FROM users"), None);
    }

    #[test]
    fn test_change_record_rollback() {
        let mut pk = HashMap::new();
        pk.insert("id".to_string(), serde_json::json!(1));

        let mut before = HashMap::new();
        before.insert("id".to_string(), serde_json::json!(1));
        before.insert("name".to_string(), serde_json::json!("old"));

        let mut after = HashMap::new();
        after.insert("id".to_string(), serde_json::json!(1));
        after.insert("name".to_string(), serde_json::json!("new"));

        // Test UPDATE rollback
        let record = ChangeRecord {
            id: 1,
            timestamp: 0,
            profile_name: "test".to_string(),
            database_id: "db1".to_string(),
            table_name: "users".to_string(),
            operation: OperationType::Update,
            primary_key: pk.clone(),
            before: Some(before.clone()),
            after: Some(after),
            original_sql: "UPDATE users SET name = 'new' WHERE id = 1".to_string(),
            note: None,
        };

        let rollback = record.generate_rollback_sql().unwrap();
        assert!(rollback.contains("UPDATE users SET"));
        assert!(rollback.contains("WHERE id = 1"));
    }

    #[test]
    fn test_audit_journal() {
        let mut journal = AuditJournal::new();

        let mut pk = HashMap::new();
        pk.insert("id".to_string(), serde_json::json!(1));

        let mut after = HashMap::new();
        after.insert("id".to_string(), serde_json::json!(1));
        after.insert("name".to_string(), serde_json::json!("test"));

        let id = journal.record_change(
            "profile1".to_string(),
            "db1".to_string(),
            "users".to_string(),
            OperationType::Insert,
            pk,
            None,
            Some(after),
            "INSERT INTO users (name) VALUES ('test')".to_string(),
        );

        assert_eq!(id, 1);
        assert_eq!(journal.record_count(), 1);

        let record = journal.get_record(1).unwrap();
        assert_eq!(record.table_name, "users");
    }

    #[test]
    fn test_value_to_sql() {
        assert_eq!(value_to_sql(&serde_json::json!(null)), "NULL");
        assert_eq!(value_to_sql(&serde_json::json!(true)), "1");
        assert_eq!(value_to_sql(&serde_json::json!(42)), "42");
        assert_eq!(value_to_sql(&serde_json::json!("hello")), "'hello'");
        assert_eq!(value_to_sql(&serde_json::json!("it's")), "'it''s'");
    }
}
