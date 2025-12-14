//! AI SQL Suggestion Module
//!
//! Provides intelligent SQL query suggestions based on schema context.
//! This module operates entirely locally without external API calls for privacy and safety.
//!
//! Safety Design:
//! - All processing is local (no data sent to external services)
//! - Only suggests safe, read-only queries by default
//! - Write operations require explicit user opt-in
//! - Suggestions are context-aware based on actual table schema

use crate::api::ColumnInfo;

/// Category of SQL suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionCategory {
    /// Basic SELECT queries
    BasicQuery,
    /// Aggregate functions (COUNT, SUM, AVG, etc.)
    Aggregation,
    /// JOIN queries
    Join,
    /// Filtering and sorting
    FilterSort,
    /// Data modification (INSERT, UPDATE, DELETE)
    Modification,
    /// Schema exploration
    Schema,
}

impl SuggestionCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BasicQuery => "Basic Query",
            Self::Aggregation => "Aggregation",
            Self::Join => "Join",
            Self::FilterSort => "Filter & Sort",
            Self::Modification => "Modification",
            Self::Schema => "Schema",
        }
    }

    pub fn label_ja(&self) -> &'static str {
        match self {
            Self::BasicQuery => "基本クエリ",
            Self::Aggregation => "集計",
            Self::Join => "結合",
            Self::FilterSort => "フィルタ/ソート",
            Self::Modification => "変更",
            Self::Schema => "スキーマ",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::BasicQuery => "📋",
            Self::Aggregation => "📊",
            Self::Join => "🔗",
            Self::FilterSort => "🔍",
            Self::Modification => "✏️",
            Self::Schema => "📐",
        }
    }

    pub fn is_safe(&self) -> bool {
        !matches!(self, Self::Modification)
    }
}

/// A single SQL suggestion
#[derive(Debug, Clone)]
pub struct SqlSuggestion {
    /// Display title for the suggestion
    pub title: String,
    /// Title in Japanese
    pub title_ja: String,
    /// Brief description of what this query does
    pub description: String,
    /// Description in Japanese
    pub description_ja: String,
    /// The actual SQL query
    pub sql: String,
    /// Category of the suggestion
    pub category: SuggestionCategory,
    /// Whether this is a safe (read-only) query
    pub is_safe: bool,
}

impl SqlSuggestion {
    pub fn new(
        title: impl Into<String>,
        title_ja: impl Into<String>,
        description: impl Into<String>,
        description_ja: impl Into<String>,
        sql: impl Into<String>,
        category: SuggestionCategory,
    ) -> Self {
        let is_safe = category.is_safe();
        Self {
            title: title.into(),
            title_ja: title_ja.into(),
            description: description.into(),
            description_ja: description_ja.into(),
            sql: sql.into(),
            category,
            is_safe,
        }
    }
}

/// Context for generating suggestions
#[derive(Debug, Clone)]
pub struct SuggestionContext {
    /// Current table name
    pub table_name: String,
    /// Column information for the current table
    pub columns: Vec<ColumnInfo>,
    /// All available tables
    pub all_tables: Vec<String>,
    /// Foreign key relationships (table, column, referenced_table, referenced_column)
    pub foreign_keys: Vec<(String, String, String, String)>,
}

/// Generate SQL suggestions based on the current context
pub fn generate_suggestions(ctx: &SuggestionContext) -> Vec<SqlSuggestion> {
    let mut suggestions = Vec::new();

    // Basic query suggestions
    suggestions.extend(generate_basic_queries(ctx));

    // Aggregation suggestions
    suggestions.extend(generate_aggregation_queries(ctx));

    // Filter and sort suggestions
    suggestions.extend(generate_filter_sort_queries(ctx));

    // Join suggestions (if foreign keys exist)
    suggestions.extend(generate_join_queries(ctx));

    // Modification suggestions (marked as unsafe)
    suggestions.extend(generate_modification_queries(ctx));

    // Schema suggestions
    suggestions.extend(generate_schema_queries(ctx));

    suggestions
}

fn generate_basic_queries(ctx: &SuggestionContext) -> Vec<SqlSuggestion> {
    let table = &ctx.table_name;
    let mut suggestions = vec![];

    // Select all
    suggestions.push(SqlSuggestion::new(
        "Select All",
        "全件取得",
        "Retrieve all rows from the table",
        "テーブルの全行を取得",
        format!("SELECT * FROM {} LIMIT 100;", table),
        SuggestionCategory::BasicQuery,
    ));

    // Select specific columns
    if ctx.columns.len() > 1 {
        let cols: Vec<_> = ctx.columns.iter().take(3).map(|c| c.name.as_str()).collect();
        suggestions.push(SqlSuggestion::new(
            "Select Specific Columns",
            "特定カラム取得",
            "Retrieve only selected columns",
            "選択したカラムのみを取得",
            format!("SELECT {} FROM {} LIMIT 100;", cols.join(", "), table),
            SuggestionCategory::BasicQuery,
        ));
    }

    // Count rows
    suggestions.push(SqlSuggestion::new(
        "Count Rows",
        "行数カウント",
        "Count total number of rows",
        "総行数を取得",
        format!("SELECT COUNT(*) AS total FROM {};", table),
        SuggestionCategory::BasicQuery,
    ));

    // Distinct values
    if let Some(col) = find_likely_category_column(&ctx.columns) {
        suggestions.push(SqlSuggestion::new(
            "Distinct Values",
            "ユニーク値一覧",
            format!("Get unique values of {}", col),
            format!("{}のユニーク値を取得", col),
            format!("SELECT DISTINCT {} FROM {};", col, table),
            SuggestionCategory::BasicQuery,
        ));
    }

    suggestions
}

fn generate_aggregation_queries(ctx: &SuggestionContext) -> Vec<SqlSuggestion> {
    let table = &ctx.table_name;
    let mut suggestions = vec![];

    // Find numeric columns for aggregation
    let numeric_cols: Vec<_> = ctx
        .columns
        .iter()
        .filter(|c| is_numeric_type(&c.col_type))
        .collect();

    if let Some(num_col) = numeric_cols.first() {
        suggestions.push(SqlSuggestion::new(
            "Sum Values",
            "合計値",
            format!("Calculate sum of {}", num_col.name),
            format!("{}の合計を計算", num_col.name),
            format!("SELECT SUM({}) AS total FROM {};", num_col.name, table),
            SuggestionCategory::Aggregation,
        ));

        suggestions.push(SqlSuggestion::new(
            "Average Value",
            "平均値",
            format!("Calculate average of {}", num_col.name),
            format!("{}の平均を計算", num_col.name),
            format!("SELECT AVG({}) AS average FROM {};", num_col.name, table),
            SuggestionCategory::Aggregation,
        ));

        suggestions.push(SqlSuggestion::new(
            "Min/Max Values",
            "最小/最大値",
            format!("Get min and max of {}", num_col.name),
            format!("{}の最小・最大を取得", num_col.name),
            format!(
                "SELECT MIN({col}) AS min_val, MAX({col}) AS max_val FROM {table};",
                col = num_col.name,
                table = table
            ),
            SuggestionCategory::Aggregation,
        ));
    }

    // Group by with category column
    if let Some(cat_col) = find_likely_category_column(&ctx.columns) {
        suggestions.push(SqlSuggestion::new(
            "Group By Count",
            "グループ別カウント",
            format!("Count rows grouped by {}", cat_col),
            format!("{}別の件数を取得", cat_col),
            format!(
                "SELECT {}, COUNT(*) AS count FROM {} GROUP BY {} ORDER BY count DESC;",
                cat_col, table, cat_col
            ),
            SuggestionCategory::Aggregation,
        ));

        if let Some(num_col) = numeric_cols.first() {
            suggestions.push(SqlSuggestion::new(
                "Group By Sum",
                "グループ別合計",
                format!("Sum {} grouped by {}", num_col.name, cat_col),
                format!("{}別の{}合計を取得", cat_col, num_col.name),
                format!(
                    "SELECT {cat}, SUM({num}) AS total FROM {table} GROUP BY {cat} ORDER BY total DESC;",
                    cat = cat_col,
                    num = num_col.name,
                    table = table
                ),
                SuggestionCategory::Aggregation,
            ));
        }
    }

    suggestions
}

fn generate_filter_sort_queries(ctx: &SuggestionContext) -> Vec<SqlSuggestion> {
    let table = &ctx.table_name;
    let mut suggestions = vec![];

    // Find primary key or ID column
    if let Some(pk) = find_primary_key(&ctx.columns) {
        suggestions.push(SqlSuggestion::new(
            "Find by ID",
            "ID検索",
            format!("Find row by {}", pk),
            format!("{}で行を検索", pk),
            format!("SELECT * FROM {} WHERE {} = ?;", table, pk),
            SuggestionCategory::FilterSort,
        ));
    }

    // Text search
    if let Some(text_col) = find_text_column(&ctx.columns) {
        suggestions.push(SqlSuggestion::new(
            "Text Search (LIKE)",
            "テキスト検索 (LIKE)",
            format!("Search {} containing text", text_col),
            format!("{}にテキストを含む行を検索", text_col),
            format!(
                "SELECT * FROM {} WHERE {} LIKE '%search_term%';",
                table, text_col
            ),
            SuggestionCategory::FilterSort,
        ));
    }

    // Date filtering
    if let Some(date_col) = find_date_column(&ctx.columns) {
        suggestions.push(SqlSuggestion::new(
            "Filter by Date",
            "日付フィルタ",
            format!("Filter by {} date range", date_col),
            format!("{}の日付範囲でフィルタ", date_col),
            format!(
                "SELECT * FROM {} WHERE {} >= '2024-01-01' AND {} < '2025-01-01';",
                table, date_col, date_col
            ),
            SuggestionCategory::FilterSort,
        ));

        suggestions.push(SqlSuggestion::new(
            "Recent Records",
            "最新レコード",
            format!("Get most recent records by {}", date_col),
            format!("{}で最新のレコードを取得", date_col),
            format!(
                "SELECT * FROM {} ORDER BY {} DESC LIMIT 10;",
                table, date_col
            ),
            SuggestionCategory::FilterSort,
        ));
    }

    // NULL check
    if ctx.columns.iter().any(|c| !c.notnull) {
        if let Some(nullable_col) = ctx.columns.iter().find(|c| !c.notnull) {
            suggestions.push(SqlSuggestion::new(
                "Find NULL Values",
                "NULL値検索",
                format!("Find rows where {} is NULL", nullable_col.name),
                format!("{}がNULLの行を検索", nullable_col.name),
                format!(
                    "SELECT * FROM {} WHERE {} IS NULL;",
                    table, nullable_col.name
                ),
                SuggestionCategory::FilterSort,
            ));
        }
    }

    suggestions
}

fn generate_join_queries(ctx: &SuggestionContext) -> Vec<SqlSuggestion> {
    let table = &ctx.table_name;
    let mut suggestions = vec![];

    for (from_table, from_col, to_table, to_col) in &ctx.foreign_keys {
        if from_table == table {
            suggestions.push(SqlSuggestion::new(
                format!("Join with {}", to_table),
                format!("{}と結合", to_table),
                format!("Join {} with {} via {}", table, to_table, from_col),
                format!("{}と{}を{}で結合", table, to_table, from_col),
                format!(
                    "SELECT t1.*, t2.*\nFROM {} t1\nINNER JOIN {} t2 ON t1.{} = t2.{}\nLIMIT 100;",
                    table, to_table, from_col, to_col
                ),
                SuggestionCategory::Join,
            ));
        }
    }

    // If no FK but multiple tables, suggest cross-table query
    if ctx.foreign_keys.is_empty() && ctx.all_tables.len() > 1 {
        if let Some(other_table) = ctx.all_tables.iter().find(|t| *t != table) {
            suggestions.push(SqlSuggestion::new(
                format!("Join with {} (manual)", other_table),
                format!("{}と結合（手動）", other_table),
                "Template for joining tables - specify join condition",
                "テーブル結合のテンプレート - 結合条件を指定してください",
                format!(
                    "SELECT t1.*, t2.*\nFROM {} t1\nINNER JOIN {} t2 ON t1.column_name = t2.column_name\nLIMIT 100;",
                    table, other_table
                ),
                SuggestionCategory::Join,
            ));
        }
    }

    suggestions
}

fn generate_modification_queries(ctx: &SuggestionContext) -> Vec<SqlSuggestion> {
    let table = &ctx.table_name;
    let mut suggestions = vec![];

    // INSERT template
    let col_names: Vec<_> = ctx.columns.iter().map(|c| c.name.as_str()).collect();
    let placeholders: Vec<_> = ctx.columns.iter().map(|_| "?").collect();

    suggestions.push(SqlSuggestion::new(
        "Insert Row",
        "行挿入",
        "Insert a new row into the table",
        "テーブルに新しい行を挿入",
        format!(
            "INSERT INTO {} ({})\nVALUES ({});",
            table,
            col_names.join(", "),
            placeholders.join(", ")
        ),
        SuggestionCategory::Modification,
    ));

    // UPDATE template
    if let Some(pk) = find_primary_key(&ctx.columns) {
        let set_clauses: Vec<_> = ctx
            .columns
            .iter()
            .filter(|c| c.name != pk)
            .take(2)
            .map(|c| format!("{} = ?", c.name))
            .collect();

        if !set_clauses.is_empty() {
            suggestions.push(SqlSuggestion::new(
                "Update Row",
                "行更新",
                "Update an existing row by primary key",
                "主キーで既存の行を更新",
                format!(
                    "UPDATE {}\nSET {}\nWHERE {} = ?;",
                    table,
                    set_clauses.join(", "),
                    pk
                ),
                SuggestionCategory::Modification,
            ));
        }

        // DELETE template
        suggestions.push(SqlSuggestion::new(
            "Delete Row",
            "行削除",
            "Delete a row by primary key",
            "主キーで行を削除",
            format!("DELETE FROM {} WHERE {} = ?;", table, pk),
            SuggestionCategory::Modification,
        ));
    }

    suggestions
}

fn generate_schema_queries(ctx: &SuggestionContext) -> Vec<SqlSuggestion> {
    let table = &ctx.table_name;
    vec![
        SqlSuggestion::new(
            "Table Info",
            "テーブル情報",
            "Get table schema information",
            "テーブルのスキーマ情報を取得",
            format!("PRAGMA table_info({});", table),
            SuggestionCategory::Schema,
        ),
        SqlSuggestion::new(
            "Foreign Keys",
            "外部キー一覧",
            "List foreign key constraints",
            "外部キー制約を一覧表示",
            format!("PRAGMA foreign_key_list({});", table),
            SuggestionCategory::Schema,
        ),
        SqlSuggestion::new(
            "Index List",
            "インデックス一覧",
            "List all indexes on this table",
            "このテーブルのインデックス一覧",
            format!("PRAGMA index_list({});", table),
            SuggestionCategory::Schema,
        ),
        SqlSuggestion::new(
            "All Tables",
            "全テーブル一覧",
            "List all tables in the database",
            "データベースの全テーブルを一覧表示",
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;".to_string(),
            SuggestionCategory::Schema,
        ),
    ]
}

// Helper functions

fn is_numeric_type(col_type: &str) -> bool {
    let t = col_type.to_uppercase();
    t.contains("INT")
        || t.contains("REAL")
        || t.contains("FLOAT")
        || t.contains("DOUBLE")
        || t.contains("NUMERIC")
        || t.contains("DECIMAL")
}

fn find_primary_key(columns: &[ColumnInfo]) -> Option<&str> {
    columns
        .iter()
        .find(|c| c.pk)
        .map(|c| c.name.as_str())
        .or_else(|| {
            columns
                .iter()
                .find(|c| c.name.to_lowercase() == "id" || c.name.to_lowercase().ends_with("_id"))
                .map(|c| c.name.as_str())
        })
}

fn find_likely_category_column(columns: &[ColumnInfo]) -> Option<&str> {
    // Look for columns that are likely categories (status, type, category, etc.)
    let category_patterns = ["status", "type", "category", "state", "kind", "level", "role"];

    columns
        .iter()
        .find(|c| {
            let name = c.name.to_lowercase();
            category_patterns.iter().any(|p| name.contains(p))
        })
        .map(|c| c.name.as_str())
        .or_else(|| {
            // Fall back to first text column that's not an ID
            columns
                .iter()
                .find(|c| {
                    let name = c.name.to_lowercase();
                    !name.ends_with("_id")
                        && !name.eq("id")
                        && (c.col_type.to_uppercase().contains("TEXT")
                            || c.col_type.to_uppercase().contains("VARCHAR"))
                })
                .map(|c| c.name.as_str())
        })
}

fn find_text_column(columns: &[ColumnInfo]) -> Option<&str> {
    let priority_patterns = ["name", "title", "description", "content", "body", "text", "message"];

    columns
        .iter()
        .find(|c| {
            let name = c.name.to_lowercase();
            priority_patterns.iter().any(|p| name.contains(p))
        })
        .or_else(|| {
            columns.iter().find(|c| {
                c.col_type.to_uppercase().contains("TEXT")
                    || c.col_type.to_uppercase().contains("VARCHAR")
            })
        })
        .map(|c| c.name.as_str())
}

fn find_date_column(columns: &[ColumnInfo]) -> Option<&str> {
    let date_patterns = [
        "created",
        "updated",
        "modified",
        "date",
        "time",
        "timestamp",
        "_at",
    ];

    columns
        .iter()
        .find(|c| {
            let name = c.name.to_lowercase();
            let col_type = c.col_type.to_uppercase();
            date_patterns.iter().any(|p| name.contains(p))
                || col_type.contains("DATE")
                || col_type.contains("TIME")
        })
        .map(|c| c.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_columns() -> Vec<ColumnInfo> {
        vec![
            ColumnInfo {
                name: "id".to_string(),
                col_type: "INTEGER".to_string(),
                notnull: true,
                pk: true,
                foreign_key: None,
            },
            ColumnInfo {
                name: "name".to_string(),
                col_type: "TEXT".to_string(),
                notnull: true,
                pk: false,
                foreign_key: None,
            },
            ColumnInfo {
                name: "status".to_string(),
                col_type: "TEXT".to_string(),
                notnull: false,
                pk: false,
                foreign_key: None,
            },
            ColumnInfo {
                name: "amount".to_string(),
                col_type: "REAL".to_string(),
                notnull: false,
                pk: false,
                foreign_key: None,
            },
            ColumnInfo {
                name: "created_at".to_string(),
                col_type: "TEXT".to_string(),
                notnull: false,
                pk: false,
                foreign_key: None,
            },
        ]
    }

    #[test]
    fn test_generate_suggestions() {
        let ctx = SuggestionContext {
            table_name: "orders".to_string(),
            columns: sample_columns(),
            all_tables: vec!["orders".to_string(), "users".to_string()],
            foreign_keys: vec![],
        };

        let suggestions = generate_suggestions(&ctx);
        assert!(!suggestions.is_empty());

        // Should have basic queries
        assert!(suggestions
            .iter()
            .any(|s| s.category == SuggestionCategory::BasicQuery));

        // Should have aggregation queries (we have numeric column)
        assert!(suggestions
            .iter()
            .any(|s| s.category == SuggestionCategory::Aggregation));

        // Should have filter/sort queries
        assert!(suggestions
            .iter()
            .any(|s| s.category == SuggestionCategory::FilterSort));

        // Modification queries should be marked as unsafe
        let mod_suggestions: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == SuggestionCategory::Modification)
            .collect();
        assert!(mod_suggestions.iter().all(|s| !s.is_safe));
    }

    #[test]
    fn test_find_primary_key() {
        let columns = sample_columns();
        let pk = find_primary_key(&columns);
        assert_eq!(pk, Some("id"));
    }

    #[test]
    fn test_find_category_column() {
        let columns = sample_columns();
        let cat = find_likely_category_column(&columns);
        assert_eq!(cat, Some("status"));
    }

    #[test]
    fn test_find_text_column() {
        let columns = sample_columns();
        let text = find_text_column(&columns);
        assert_eq!(text, Some("name"));
    }

    #[test]
    fn test_find_date_column() {
        let columns = sample_columns();
        let date = find_date_column(&columns);
        assert_eq!(date, Some("created_at"));
    }

    #[test]
    fn test_foreign_key_join_suggestion() {
        let ctx = SuggestionContext {
            table_name: "orders".to_string(),
            columns: sample_columns(),
            all_tables: vec!["orders".to_string(), "users".to_string()],
            foreign_keys: vec![(
                "orders".to_string(),
                "user_id".to_string(),
                "users".to_string(),
                "id".to_string(),
            )],
        };

        let suggestions = generate_suggestions(&ctx);
        let join_suggestions: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == SuggestionCategory::Join)
            .collect();

        assert!(!join_suggestions.is_empty());
        assert!(join_suggestions
            .iter()
            .any(|s| s.sql.contains("INNER JOIN users")));
    }
}
