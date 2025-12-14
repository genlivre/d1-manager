//! Schema Diff Module
//!
//! This module provides schema comparison between two databases:
//! - Detect added/removed/modified tables
//! - Detect added/removed/modified columns
//! - Generate migration SQL to sync schemas

use std::collections::{HashMap, HashSet};

/// Column definition for comparison
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSchema {
    pub name: String,
    pub col_type: String,
    pub notnull: bool,
    pub pk: bool,
    pub default_value: Option<String>,
}

/// Table definition for comparison
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

impl TableSchema {
    pub fn column_map(&self) -> HashMap<String, &ColumnSchema> {
        self.columns.iter().map(|c| (c.name.to_lowercase(), c)).collect()
    }
}

/// Database schema snapshot
#[derive(Debug, Clone, Default)]
pub struct DatabaseSchema {
    pub tables: HashMap<String, TableSchema>,
}

impl DatabaseSchema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_table(&mut self, table: TableSchema) {
        self.tables.insert(table.name.to_lowercase(), table);
    }

    pub fn table_names(&self) -> HashSet<String> {
        self.tables.keys().cloned().collect()
    }
}

/// Type of schema difference
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
}

impl DiffType {
    pub fn label(&self) -> &'static str {
        match self {
            DiffType::Added => "Added",
            DiffType::Removed => "Removed",
            DiffType::Modified => "Modified",
        }
    }

    pub fn label_ja(&self) -> &'static str {
        match self {
            DiffType::Added => "追加",
            DiffType::Removed => "削除",
            DiffType::Modified => "変更",
        }
    }
}

/// Column difference
#[derive(Debug, Clone)]
pub struct ColumnDiff {
    pub column_name: String,
    pub diff_type: DiffType,
    pub old_def: Option<ColumnSchema>,
    pub new_def: Option<ColumnSchema>,
}

/// Table difference
#[derive(Debug, Clone)]
pub struct TableDiff {
    pub table_name: String,
    pub diff_type: DiffType,
    pub column_diffs: Vec<ColumnDiff>,
}

/// Complete schema difference
#[derive(Debug, Clone, Default)]
pub struct SchemaDiff {
    pub table_diffs: Vec<TableDiff>,
}

impl SchemaDiff {
    pub fn is_empty(&self) -> bool {
        self.table_diffs.is_empty()
    }

    pub fn added_tables(&self) -> Vec<&TableDiff> {
        self.table_diffs.iter().filter(|t| t.diff_type == DiffType::Added).collect()
    }

    pub fn removed_tables(&self) -> Vec<&TableDiff> {
        self.table_diffs.iter().filter(|t| t.diff_type == DiffType::Removed).collect()
    }

    pub fn modified_tables(&self) -> Vec<&TableDiff> {
        self.table_diffs.iter().filter(|t| t.diff_type == DiffType::Modified).collect()
    }

    pub fn summary(&self) -> String {
        let added = self.added_tables().len();
        let removed = self.removed_tables().len();
        let modified = self.modified_tables().len();

        if added == 0 && removed == 0 && modified == 0 {
            "No differences found".to_string()
        } else {
            format!(
                "{} added, {} removed, {} modified",
                added, removed, modified
            )
        }
    }
}

/// Compare two database schemas
pub fn compare_schemas(source: &DatabaseSchema, target: &DatabaseSchema) -> SchemaDiff {
    let mut diffs = Vec::new();

    let source_tables = source.table_names();
    let target_tables = target.table_names();

    // Find added tables (in target but not in source)
    for table_name in target_tables.difference(&source_tables) {
        if let Some(table) = target.tables.get(table_name) {
            diffs.push(TableDiff {
                table_name: table.name.clone(),
                diff_type: DiffType::Added,
                column_diffs: table.columns.iter().map(|c| ColumnDiff {
                    column_name: c.name.clone(),
                    diff_type: DiffType::Added,
                    old_def: None,
                    new_def: Some(c.clone()),
                }).collect(),
            });
        }
    }

    // Find removed tables (in source but not in target)
    for table_name in source_tables.difference(&target_tables) {
        if let Some(table) = source.tables.get(table_name) {
            diffs.push(TableDiff {
                table_name: table.name.clone(),
                diff_type: DiffType::Removed,
                column_diffs: table.columns.iter().map(|c| ColumnDiff {
                    column_name: c.name.clone(),
                    diff_type: DiffType::Removed,
                    old_def: Some(c.clone()),
                    new_def: None,
                }).collect(),
            });
        }
    }

    // Find modified tables (in both, but with differences)
    for table_name in source_tables.intersection(&target_tables) {
        if let (Some(source_table), Some(target_table)) =
            (source.tables.get(table_name), target.tables.get(table_name))
        {
            let column_diffs = compare_columns(source_table, target_table);
            if !column_diffs.is_empty() {
                diffs.push(TableDiff {
                    table_name: target_table.name.clone(),
                    diff_type: DiffType::Modified,
                    column_diffs,
                });
            }
        }
    }

    // Sort by table name
    diffs.sort_by(|a, b| a.table_name.cmp(&b.table_name));

    SchemaDiff { table_diffs: diffs }
}

/// Compare columns between two tables
fn compare_columns(source: &TableSchema, target: &TableSchema) -> Vec<ColumnDiff> {
    let mut diffs = Vec::new();

    let source_cols = source.column_map();
    let target_cols = target.column_map();

    let source_names: HashSet<String> = source_cols.keys().cloned().collect();
    let target_names: HashSet<String> = target_cols.keys().cloned().collect();

    // Added columns
    for col_name in target_names.difference(&source_names) {
        if let Some(col) = target_cols.get(col_name) {
            diffs.push(ColumnDiff {
                column_name: col.name.clone(),
                diff_type: DiffType::Added,
                old_def: None,
                new_def: Some((*col).clone()),
            });
        }
    }

    // Removed columns
    for col_name in source_names.difference(&target_names) {
        if let Some(col) = source_cols.get(col_name) {
            diffs.push(ColumnDiff {
                column_name: col.name.clone(),
                diff_type: DiffType::Removed,
                old_def: Some((*col).clone()),
                new_def: None,
            });
        }
    }

    // Modified columns
    for col_name in source_names.intersection(&target_names) {
        if let (Some(source_col), Some(target_col)) =
            (source_cols.get(col_name), target_cols.get(col_name))
        {
            if *source_col != *target_col {
                diffs.push(ColumnDiff {
                    column_name: target_col.name.clone(),
                    diff_type: DiffType::Modified,
                    old_def: Some((*source_col).clone()),
                    new_def: Some((*target_col).clone()),
                });
            }
        }
    }

    // Sort by column name
    diffs.sort_by(|a, b| a.column_name.cmp(&b.column_name));

    diffs
}

/// Generate migration SQL from source schema to target schema
pub fn generate_migration_sql(diff: &SchemaDiff, source_schema: &DatabaseSchema) -> Vec<String> {
    let mut statements = Vec::new();

    for table_diff in &diff.table_diffs {
        match table_diff.diff_type {
            DiffType::Added => {
                // CREATE TABLE
                let create_sql = generate_create_table(&table_diff.table_name, &table_diff.column_diffs);
                statements.push(create_sql);
            }
            DiffType::Removed => {
                // DROP TABLE
                statements.push(format!("DROP TABLE IF EXISTS {};", table_diff.table_name));
            }
            DiffType::Modified => {
                // ALTER TABLE for each column change
                // Note: SQLite has limited ALTER TABLE support
                let alter_statements = generate_alter_table(
                    &table_diff.table_name,
                    &table_diff.column_diffs,
                    source_schema.tables.get(&table_diff.table_name.to_lowercase()),
                );
                statements.extend(alter_statements);
            }
        }
    }

    statements
}

/// Generate CREATE TABLE statement
fn generate_create_table(table_name: &str, column_diffs: &[ColumnDiff]) -> String {
    let columns: Vec<String> = column_diffs
        .iter()
        .filter_map(|cd| cd.new_def.as_ref())
        .map(|col| {
            let mut def = format!("{} {}", col.name, col.col_type);
            if col.pk {
                def.push_str(" PRIMARY KEY");
            }
            if col.notnull && !col.pk {
                def.push_str(" NOT NULL");
            }
            if let Some(ref default) = col.default_value {
                def.push_str(&format!(" DEFAULT {}", default));
            }
            def
        })
        .collect();

    format!(
        "CREATE TABLE {} (\n  {}\n);",
        table_name,
        columns.join(",\n  ")
    )
}

/// Generate ALTER TABLE statements
/// Note: SQLite has limited ALTER TABLE support, so for complex changes
/// we may need to recreate the table
fn generate_alter_table(
    table_name: &str,
    column_diffs: &[ColumnDiff],
    source_table: Option<&TableSchema>,
) -> Vec<String> {
    let mut statements = Vec::new();

    // Check if we need full table recreation
    let needs_recreation = column_diffs.iter().any(|cd| {
        matches!(cd.diff_type, DiffType::Removed | DiffType::Modified)
    });

    if needs_recreation {
        // For SQLite, complex changes require table recreation
        if let Some(source) = source_table {
            statements.push(format!("-- Table {} requires recreation due to column changes", table_name));
            statements.push(format!("-- Backup data: CREATE TABLE {}_backup AS SELECT * FROM {};", table_name, table_name));

            // Build new column list
            let mut new_columns: Vec<&ColumnSchema> = Vec::new();
            let source_col_map = source.column_map();

            // Keep existing columns that aren't removed, apply modifications
            for col in &source.columns {
                let col_name_lower = col.name.to_lowercase();

                // Check if column is removed
                let is_removed = column_diffs.iter().any(|cd|
                    cd.column_name.to_lowercase() == col_name_lower &&
                    cd.diff_type == DiffType::Removed
                );

                if !is_removed {
                    // Check if column is modified
                    if let Some(modified) = column_diffs.iter().find(|cd|
                        cd.column_name.to_lowercase() == col_name_lower &&
                        cd.diff_type == DiffType::Modified
                    ) {
                        if let Some(ref new_def) = modified.new_def {
                            // Use modified definition (we'll add it separately)
                            let _ = new_def; // Mark as used
                        }
                    }
                    new_columns.push(col);
                }
            }

            // Add new columns
            for cd in column_diffs {
                if cd.diff_type == DiffType::Added {
                    if let Some(ref new_def) = cd.new_def {
                        let _ = new_def; // Will be handled in CREATE TABLE
                    }
                }
            }

            statements.push(format!("-- DROP TABLE {};", table_name));
            statements.push(format!("-- Recreate with new schema and restore data"));
        }
    } else {
        // Simple ADD COLUMN operations
        for cd in column_diffs {
            if cd.diff_type == DiffType::Added {
                if let Some(ref col) = cd.new_def {
                    let mut def = format!("ALTER TABLE {} ADD COLUMN {} {}",
                        table_name, col.name, col.col_type);
                    if col.notnull {
                        // SQLite requires default for NOT NULL in ADD COLUMN
                        if let Some(ref default) = col.default_value {
                            def.push_str(&format!(" NOT NULL DEFAULT {}", default));
                        } else {
                            def.push_str(" DEFAULT ''");
                        }
                    }
                    if let Some(ref default) = col.default_value {
                        if !col.notnull {
                            def.push_str(&format!(" DEFAULT {}", default));
                        }
                    }
                    def.push(';');
                    statements.push(def);
                }
            }
        }
    }

    statements
}

/// Format column definition for display
pub fn format_column_def(col: &ColumnSchema) -> String {
    let mut parts = vec![col.col_type.clone()];

    if col.pk {
        parts.push("PK".to_string());
    }
    if col.notnull {
        parts.push("NOT NULL".to_string());
    }
    if let Some(ref default) = col.default_value {
        parts.push(format!("DEFAULT {}", default));
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_schema() -> DatabaseSchema {
        let mut schema = DatabaseSchema::new();

        schema.add_table(TableSchema {
            name: "users".to_string(),
            columns: vec![
                ColumnSchema {
                    name: "id".to_string(),
                    col_type: "INTEGER".to_string(),
                    notnull: true,
                    pk: true,
                    default_value: None,
                },
                ColumnSchema {
                    name: "name".to_string(),
                    col_type: "TEXT".to_string(),
                    notnull: true,
                    pk: false,
                    default_value: None,
                },
            ],
        });

        schema
    }

    #[test]
    fn test_no_diff() {
        let schema1 = create_test_schema();
        let schema2 = create_test_schema();

        let diff = compare_schemas(&schema1, &schema2);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_added_table() {
        let schema1 = DatabaseSchema::new();
        let schema2 = create_test_schema();

        let diff = compare_schemas(&schema1, &schema2);
        assert_eq!(diff.added_tables().len(), 1);
        assert_eq!(diff.added_tables()[0].table_name, "users");
    }

    #[test]
    fn test_removed_table() {
        let schema1 = create_test_schema();
        let schema2 = DatabaseSchema::new();

        let diff = compare_schemas(&schema1, &schema2);
        assert_eq!(diff.removed_tables().len(), 1);
    }

    #[test]
    fn test_added_column() {
        let schema1 = create_test_schema();
        let mut schema2 = create_test_schema();

        if let Some(table) = schema2.tables.get_mut("users") {
            table.columns.push(ColumnSchema {
                name: "email".to_string(),
                col_type: "TEXT".to_string(),
                notnull: false,
                pk: false,
                default_value: None,
            });
        }

        let diff = compare_schemas(&schema1, &schema2);
        assert_eq!(diff.modified_tables().len(), 1);

        let table_diff = &diff.modified_tables()[0];
        assert_eq!(table_diff.column_diffs.len(), 1);
        assert_eq!(table_diff.column_diffs[0].diff_type, DiffType::Added);
        assert_eq!(table_diff.column_diffs[0].column_name, "email");
    }

    #[test]
    fn test_generate_create_table() {
        let column_diffs = vec![
            ColumnDiff {
                column_name: "id".to_string(),
                diff_type: DiffType::Added,
                old_def: None,
                new_def: Some(ColumnSchema {
                    name: "id".to_string(),
                    col_type: "INTEGER".to_string(),
                    notnull: true,
                    pk: true,
                    default_value: None,
                }),
            },
            ColumnDiff {
                column_name: "name".to_string(),
                diff_type: DiffType::Added,
                old_def: None,
                new_def: Some(ColumnSchema {
                    name: "name".to_string(),
                    col_type: "TEXT".to_string(),
                    notnull: true,
                    pk: false,
                    default_value: None,
                }),
            },
        ];

        let sql = generate_create_table("users", &column_diffs);
        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("id INTEGER PRIMARY KEY"));
        assert!(sql.contains("name TEXT NOT NULL"));
    }
}
