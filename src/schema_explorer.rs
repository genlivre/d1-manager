//! Schema Explorer Module
//!
//! This module provides enhanced schema exploration features:
//! - Table relationship detection via foreign keys
//! - Simple ER diagram generation (ASCII-based)
//! - Quick navigation between related tables

use std::collections::{HashMap, HashSet};

/// Represents a foreign key relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForeignKeyRelation {
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
}

/// Represents a table in the schema
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub primary_key: Vec<String>,
    pub foreign_keys: Vec<ForeignKeyRelation>,
}

/// Column definition
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
}

/// Schema with all tables and relationships
#[derive(Debug, Clone, Default)]
pub struct SchemaGraph {
    pub tables: HashMap<String, TableInfo>,
    pub relations: Vec<ForeignKeyRelation>,
}

impl SchemaGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a table to the schema
    pub fn add_table(&mut self, table: TableInfo) {
        for fk in &table.foreign_keys {
            if !self.relations.contains(fk) {
                self.relations.push(fk.clone());
            }
        }
        self.tables.insert(table.name.clone(), table);
    }

    /// Get tables that reference the given table (incoming foreign keys)
    pub fn get_referencing_tables(&self, table_name: &str) -> Vec<&ForeignKeyRelation> {
        self.relations
            .iter()
            .filter(|r| r.to_table.eq_ignore_ascii_case(table_name))
            .collect()
    }

    /// Get tables that the given table references (outgoing foreign keys)
    pub fn get_referenced_tables(&self, table_name: &str) -> Vec<&ForeignKeyRelation> {
        self.relations
            .iter()
            .filter(|r| r.from_table.eq_ignore_ascii_case(table_name))
            .collect()
    }

    /// Get all related tables (both incoming and outgoing)
    pub fn get_related_tables(&self, table_name: &str) -> HashSet<String> {
        let mut related = HashSet::new();

        for rel in &self.relations {
            if rel.from_table.eq_ignore_ascii_case(table_name) {
                related.insert(rel.to_table.clone());
            }
            if rel.to_table.eq_ignore_ascii_case(table_name) {
                related.insert(rel.from_table.clone());
            }
        }

        related
    }

    /// Generate a simple ASCII ER diagram for a table and its relations
    pub fn generate_er_diagram(&self, center_table: &str) -> String {
        let mut lines = Vec::new();

        // Get the center table
        let center = match self.tables.get(center_table) {
            Some(t) => t,
            None => return format!("Table '{}' not found", center_table),
        };

        // Get related tables
        let incoming = self.get_referencing_tables(center_table);
        let outgoing = self.get_referenced_tables(center_table);

        // Build the diagram
        lines.push(String::new());

        // Incoming relationships (tables that reference this table)
        for rel in &incoming {
            lines.push(format!("  +---------------+"));
            lines.push(format!("  | {} |", truncate_name(&rel.from_table, 13)));
            lines.push(format!("  +---------------+"));
            lines.push(format!("         |"));
            lines.push(format!("         | {} -> {}", rel.from_column, rel.to_column));
            lines.push(format!("         v"));
        }

        // Center table (main)
        let box_width = center.name.len().max(20) + 4;
        let border = "=".repeat(box_width);

        lines.push(format!("  +{}+", border));
        lines.push(format!("  | {:^width$} |", center.name.to_uppercase(), width = box_width - 2));
        lines.push(format!("  +{}+", border));

        // Show columns
        for col in &center.columns {
            let pk_marker = if col.is_primary_key { "*" } else { " " };
            let nullable = if col.is_nullable { "?" } else { "" };
            lines.push(format!(
                "  |{} {}: {}{}",
                pk_marker,
                col.name,
                col.data_type,
                nullable
            ));
        }
        lines.push(format!("  +{}+", "-".repeat(box_width)));

        // Outgoing relationships (tables this table references)
        for rel in &outgoing {
            lines.push(format!("         |"));
            lines.push(format!("         | {} -> {}", rel.from_column, rel.to_column));
            lines.push(format!("         v"));
            lines.push(format!("  +---------------+"));
            lines.push(format!("  | {} |", truncate_name(&rel.to_table, 13)));
            lines.push(format!("  +---------------+"));
        }

        lines.push(String::new());
        lines.join("\n")
    }

    /// Generate a compact relationship summary
    pub fn generate_relationship_summary(&self, table_name: &str) -> String {
        let mut lines = Vec::new();

        let incoming = self.get_referencing_tables(table_name);
        let outgoing = self.get_referenced_tables(table_name);

        if incoming.is_empty() && outgoing.is_empty() {
            return format!("No relationships found for table '{}'", table_name);
        }

        lines.push(format!("=== {} ===", table_name.to_uppercase()));
        lines.push(String::new());

        if !outgoing.is_empty() {
            lines.push("References (this table -> other tables):".to_string());
            for rel in &outgoing {
                lines.push(format!(
                    "  {} ({}) -> {} ({})",
                    table_name, rel.from_column, rel.to_table, rel.to_column
                ));
            }
            lines.push(String::new());
        }

        if !incoming.is_empty() {
            lines.push("Referenced by (other tables -> this table):".to_string());
            for rel in &incoming {
                lines.push(format!(
                    "  {} ({}) -> {} ({})",
                    rel.from_table, rel.from_column, table_name, rel.to_column
                ));
            }
        }

        lines.join("\n")
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        format!("{:^width$}", name, width = max_len)
    } else {
        format!("{:.width$}", name, width = max_len - 2) + ".."
    }
}

/// Parse foreign key info from SQLite pragma result
pub fn parse_foreign_key_info(
    table_name: &str,
    fk_data: &[(String, String, String)], // (from_col, to_table, to_col)
) -> Vec<ForeignKeyRelation> {
    fk_data
        .iter()
        .map(|(from_col, to_table, to_col)| ForeignKeyRelation {
            from_table: table_name.to_string(),
            from_column: from_col.clone(),
            to_table: to_table.clone(),
            to_column: to_col.clone(),
        })
        .collect()
}

/// Generate SQL to query for related data
pub fn generate_join_sql(
    base_table: &str,
    relation: &ForeignKeyRelation,
    limit: i64,
) -> String {
    if relation.from_table.eq_ignore_ascii_case(base_table) {
        // This table references another
        format!(
            "SELECT t1.*, t2.*\nFROM {} t1\nJOIN {} t2 ON t1.{} = t2.{}\nLIMIT {};",
            base_table, relation.to_table, relation.from_column, relation.to_column, limit
        )
    } else {
        // Another table references this one
        format!(
            "SELECT t1.*, t2.*\nFROM {} t1\nJOIN {} t2 ON t1.{} = t2.{}\nLIMIT {};",
            base_table, relation.from_table, relation.to_column, relation.from_column, limit
        )
    }
}

/// Relationship type for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// This table has a foreign key to another table (many-to-one)
    References,
    /// Another table has a foreign key to this table (one-to-many)
    ReferencedBy,
}

impl RelationType {
    pub fn label(&self) -> &'static str {
        match self {
            RelationType::References => "References",
            RelationType::ReferencedBy => "Referenced by",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            RelationType::References => "->",
            RelationType::ReferencedBy => "<-",
        }
    }
}

/// A simplified view of a relationship for UI display
#[derive(Debug, Clone)]
pub struct RelationshipView {
    pub relation_type: RelationType,
    pub this_column: String,
    pub other_table: String,
    pub other_column: String,
}

impl RelationshipView {
    pub fn from_relation(table_name: &str, rel: &ForeignKeyRelation) -> Self {
        if rel.from_table.eq_ignore_ascii_case(table_name) {
            RelationshipView {
                relation_type: RelationType::References,
                this_column: rel.from_column.clone(),
                other_table: rel.to_table.clone(),
                other_column: rel.to_column.clone(),
            }
        } else {
            RelationshipView {
                relation_type: RelationType::ReferencedBy,
                this_column: rel.to_column.clone(),
                other_table: rel.from_table.clone(),
                other_column: rel.from_column.clone(),
            }
        }
    }

    pub fn display_string(&self) -> String {
        match self.relation_type {
            RelationType::References => {
                format!(
                    "{} {} -> {}.{}",
                    self.this_column, self.relation_type.icon(), self.other_table, self.other_column
                )
            }
            RelationType::ReferencedBy => {
                format!(
                    "{}.{} -> {} {}",
                    self.other_table, self.other_column, self.this_column, self.relation_type.icon()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_graph() {
        let mut graph = SchemaGraph::new();

        // Add users table
        graph.add_table(TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    is_nullable: false,
                    is_primary_key: true,
                    default_value: None,
                },
                ColumnDef {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    is_nullable: false,
                    is_primary_key: false,
                    default_value: None,
                },
            ],
            primary_key: vec!["id".to_string()],
            foreign_keys: vec![],
        });

        // Add posts table with FK to users
        graph.add_table(TableInfo {
            name: "posts".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    is_nullable: false,
                    is_primary_key: true,
                    default_value: None,
                },
                ColumnDef {
                    name: "user_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    is_nullable: false,
                    is_primary_key: false,
                    default_value: None,
                },
            ],
            primary_key: vec!["id".to_string()],
            foreign_keys: vec![ForeignKeyRelation {
                from_table: "posts".to_string(),
                from_column: "user_id".to_string(),
                to_table: "users".to_string(),
                to_column: "id".to_string(),
            }],
        });

        // Test relationships
        let related = graph.get_related_tables("users");
        assert!(related.contains("posts"));

        let incoming = graph.get_referencing_tables("users");
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].from_table, "posts");

        let outgoing = graph.get_referenced_tables("posts");
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].to_table, "users");
    }

    #[test]
    fn test_relationship_view() {
        let rel = ForeignKeyRelation {
            from_table: "posts".to_string(),
            from_column: "user_id".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
        };

        // From posts perspective (references)
        let view = RelationshipView::from_relation("posts", &rel);
        assert_eq!(view.relation_type, RelationType::References);
        assert_eq!(view.other_table, "users");

        // From users perspective (referenced by)
        let view = RelationshipView::from_relation("users", &rel);
        assert_eq!(view.relation_type, RelationType::ReferencedBy);
        assert_eq!(view.other_table, "posts");
    }

    #[test]
    fn test_generate_join_sql() {
        let rel = ForeignKeyRelation {
            from_table: "posts".to_string(),
            from_column: "user_id".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
        };

        let sql = generate_join_sql("posts", &rel, 50);
        assert!(sql.contains("JOIN users"));
        assert!(sql.contains("ON t1.user_id = t2.id"));
    }
}
