//! SQL Safety Analysis
//! Detects dangerous queries and provides warnings/blocks

/// Risk level of a SQL query
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Safe read-only query
    Safe,
    /// Low risk write operation with proper WHERE clause
    Low,
    /// Medium risk - large potential impact but recoverable
    Medium,
    /// High risk - potentially destructive, hard to recover
    High,
    /// Critical - schema changes, data loss possible
    Critical,
}

impl RiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "Safe",
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        }
    }

    pub fn label_ja(&self) -> &'static str {
        match self {
            RiskLevel::Safe => "安全",
            RiskLevel::Low => "低リスク",
            RiskLevel::Medium => "中リスク",
            RiskLevel::High => "高リスク",
            RiskLevel::Critical => "危険",
        }
    }

    pub fn requires_confirmation(&self) -> bool {
        matches!(self, RiskLevel::High | RiskLevel::Critical)
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            RiskLevel::Safe => (34, 197, 94),      // Green
            RiskLevel::Low => (34, 197, 94),       // Green
            RiskLevel::Medium => (245, 158, 11),   // Amber
            RiskLevel::High => (239, 68, 68),      // Red
            RiskLevel::Critical => (220, 38, 38),  // Dark Red
        }
    }
}

/// Result of SQL safety analysis
#[derive(Debug, Clone)]
pub struct SafetyAnalysis {
    pub risk_level: RiskLevel,
    pub warnings: Vec<SafetyWarning>,
    pub query_type: QueryType,
    pub affected_tables: Vec<String>,
    pub has_where_clause: bool,
    pub has_limit: bool,
    pub is_multi_statement: bool,
}

/// Type of SQL query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Drop,
    Alter,
    Create,
    Truncate,
    Pragma,
    Other,
    Multiple,
}

impl QueryType {
    pub fn is_write(&self) -> bool {
        matches!(
            self,
            QueryType::Insert
                | QueryType::Update
                | QueryType::Delete
                | QueryType::Drop
                | QueryType::Alter
                | QueryType::Create
                | QueryType::Truncate
        )
    }

    pub fn is_destructive(&self) -> bool {
        matches!(
            self,
            QueryType::Update | QueryType::Delete | QueryType::Drop | QueryType::Truncate | QueryType::Alter
        )
    }

    pub fn label(&self) -> &'static str {
        match self {
            QueryType::Select => "SELECT",
            QueryType::Insert => "INSERT",
            QueryType::Update => "UPDATE",
            QueryType::Delete => "DELETE",
            QueryType::Drop => "DROP",
            QueryType::Alter => "ALTER",
            QueryType::Create => "CREATE",
            QueryType::Truncate => "TRUNCATE",
            QueryType::Pragma => "PRAGMA",
            QueryType::Other => "OTHER",
            QueryType::Multiple => "MULTIPLE",
        }
    }
}

/// Safety warning with details
#[derive(Debug, Clone)]
pub struct SafetyWarning {
    pub code: WarningCode,
    pub message: String,
    pub message_ja: String,
}

/// Warning codes for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningCode {
    /// UPDATE/DELETE without WHERE clause
    NoWhereClause,
    /// DROP TABLE/DATABASE
    DropStatement,
    /// TRUNCATE TABLE
    TruncateStatement,
    /// ALTER TABLE (schema change)
    AlterStatement,
    /// Multiple statements in one query
    MultipleStatements,
    /// UPDATE/DELETE with no LIMIT
    NoLimit,
    /// Potential to affect all rows
    AffectsAllRows,
    /// Using wildcards in destructive query
    WildcardInDestructive,
}

impl WarningCode {
    pub fn severity(&self) -> RiskLevel {
        match self {
            WarningCode::DropStatement => RiskLevel::Critical,
            WarningCode::TruncateStatement => RiskLevel::Critical,
            WarningCode::AlterStatement => RiskLevel::High,
            WarningCode::NoWhereClause => RiskLevel::High,
            WarningCode::AffectsAllRows => RiskLevel::High,
            WarningCode::MultipleStatements => RiskLevel::Medium,
            WarningCode::NoLimit => RiskLevel::Medium,
            WarningCode::WildcardInDestructive => RiskLevel::Medium,
        }
    }
}

/// Analyze SQL query for safety
pub fn analyze_query(sql: &str) -> SafetyAnalysis {
    let sql_upper = sql.to_uppercase();
    let sql_normalized = normalize_sql(&sql_upper);

    let mut warnings = Vec::new();
    let mut affected_tables = Vec::new();

    // Check for multiple statements
    let statements: Vec<&str> = sql.split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let is_multi_statement = statements.len() > 1;

    if is_multi_statement {
        warnings.push(SafetyWarning {
            code: WarningCode::MultipleStatements,
            message: format!("Multiple statements ({}) will be executed", statements.len()),
            message_ja: format!("複数のステートメント（{}件）が実行されます", statements.len()),
        });
    }

    // Determine query type
    let query_type = if is_multi_statement {
        QueryType::Multiple
    } else {
        detect_query_type(&sql_normalized)
    };

    // Check for WHERE clause
    let has_where_clause = sql_normalized.contains(" WHERE ");
    let has_limit = sql_normalized.contains(" LIMIT ");

    // Extract affected tables
    affected_tables.extend(extract_tables(&sql_normalized));

    // Analyze based on query type
    match query_type {
        QueryType::Drop => {
            warnings.push(SafetyWarning {
                code: WarningCode::DropStatement,
                message: "DROP will permanently delete table/database structure".to_string(),
                message_ja: "DROP はテーブル/データベース構造を永久に削除します".to_string(),
            });
        }
        QueryType::Truncate => {
            warnings.push(SafetyWarning {
                code: WarningCode::TruncateStatement,
                message: "TRUNCATE will delete ALL rows from the table".to_string(),
                message_ja: "TRUNCATE はテーブルの全行を削除します".to_string(),
            });
        }
        QueryType::Alter => {
            warnings.push(SafetyWarning {
                code: WarningCode::AlterStatement,
                message: "ALTER will modify table schema".to_string(),
                message_ja: "ALTER はテーブルスキーマを変更します".to_string(),
            });
        }
        QueryType::Delete => {
            if !has_where_clause {
                warnings.push(SafetyWarning {
                    code: WarningCode::NoWhereClause,
                    message: "DELETE without WHERE will delete ALL rows".to_string(),
                    message_ja: "WHERE なしの DELETE は全行を削除します".to_string(),
                });
            } else if !has_limit {
                // Check for potentially dangerous WHERE clauses
                if contains_broad_condition(&sql_normalized) {
                    warnings.push(SafetyWarning {
                        code: WarningCode::AffectsAllRows,
                        message: "WHERE condition may affect many rows".to_string(),
                        message_ja: "WHERE 条件が多くの行に影響する可能性があります".to_string(),
                    });
                }
            }
        }
        QueryType::Update => {
            if !has_where_clause {
                warnings.push(SafetyWarning {
                    code: WarningCode::NoWhereClause,
                    message: "UPDATE without WHERE will update ALL rows".to_string(),
                    message_ja: "WHERE なしの UPDATE は全行を更新します".to_string(),
                });
            } else if !has_limit {
                if contains_broad_condition(&sql_normalized) {
                    warnings.push(SafetyWarning {
                        code: WarningCode::AffectsAllRows,
                        message: "WHERE condition may affect many rows".to_string(),
                        message_ja: "WHERE 条件が多くの行に影響する可能性があります".to_string(),
                    });
                }
            }
        }
        _ => {}
    }

    // Calculate overall risk level
    let risk_level = calculate_risk_level(&query_type, &warnings, has_where_clause);

    SafetyAnalysis {
        risk_level,
        warnings,
        query_type,
        affected_tables,
        has_where_clause,
        has_limit,
        is_multi_statement,
    }
}

/// Normalize SQL for analysis (remove extra whitespace, etc.)
fn normalize_sql(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Detect the primary query type
fn detect_query_type(sql: &str) -> QueryType {
    let sql_trimmed = sql.trim();

    if sql_trimmed.starts_with("SELECT") || sql_trimmed.starts_with("WITH") {
        QueryType::Select
    } else if sql_trimmed.starts_with("INSERT") {
        QueryType::Insert
    } else if sql_trimmed.starts_with("UPDATE") {
        QueryType::Update
    } else if sql_trimmed.starts_with("DELETE") {
        QueryType::Delete
    } else if sql_trimmed.starts_with("DROP") {
        QueryType::Drop
    } else if sql_trimmed.starts_with("ALTER") {
        QueryType::Alter
    } else if sql_trimmed.starts_with("CREATE") {
        QueryType::Create
    } else if sql_trimmed.starts_with("TRUNCATE") {
        QueryType::Truncate
    } else if sql_trimmed.starts_with("PRAGMA") {
        QueryType::Pragma
    } else {
        QueryType::Other
    }
}

/// Extract table names from SQL
fn extract_tables(sql: &str) -> Vec<String> {
    let mut tables = Vec::new();

    // Common patterns: FROM table, UPDATE table, INTO table, TABLE table
    let patterns = [
        " FROM ", " UPDATE ", " INTO ", " TABLE ", " JOIN ",
    ];

    for pattern in patterns {
        if let Some(pos) = sql.find(pattern) {
            let after = &sql[pos + pattern.len()..];
            if let Some(table) = after.split_whitespace().next() {
                let table_name = table.trim_matches(|c| c == '"' || c == '`' || c == '[' || c == ']');
                if !table_name.is_empty() && !tables.contains(&table_name.to_string()) {
                    tables.push(table_name.to_string());
                }
            }
        }
    }

    tables
}

/// Check if WHERE condition is potentially broad
fn contains_broad_condition(sql: &str) -> bool {
    // Check for conditions that always evaluate to true
    let broad_patterns = [
        "WHERE 1",
        "WHERE TRUE",
        "WHERE 1=1",
        "WHERE '1'='1'",
        "WHERE NOT FALSE",
    ];

    for pattern in broad_patterns {
        if sql.contains(pattern) {
            return true;
        }
    }

    // Check for LIKE with leading wildcard on all conditions
    if sql.contains("LIKE '%") || sql.contains("LIKE \"%") {
        return true;
    }

    false
}

/// Calculate overall risk level
fn calculate_risk_level(query_type: &QueryType, warnings: &[SafetyWarning], has_where: bool) -> RiskLevel {
    // Get max severity from warnings
    let max_warning_severity = warnings.iter()
        .map(|w| w.code.severity())
        .max()
        .unwrap_or(RiskLevel::Safe);

    // Base risk from query type
    let base_risk = match query_type {
        QueryType::Select | QueryType::Pragma => RiskLevel::Safe,
        QueryType::Insert | QueryType::Create => RiskLevel::Low,
        QueryType::Update | QueryType::Delete => {
            if has_where { RiskLevel::Medium } else { RiskLevel::High }
        }
        QueryType::Alter => RiskLevel::High,
        QueryType::Drop | QueryType::Truncate => RiskLevel::Critical,
        QueryType::Multiple => RiskLevel::Medium,
        QueryType::Other => RiskLevel::Low,
    };

    // Return the higher risk
    std::cmp::max(base_risk, max_warning_severity)
}

/// Check if a query should be blocked in read-only mode
pub fn is_write_query(sql: &str) -> bool {
    let analysis = analyze_query(sql);
    analysis.query_type.is_write()
}

/// Format analysis result for display
pub fn format_analysis(analysis: &SafetyAnalysis, japanese: bool) -> String {
    let mut result = String::new();

    let level_str = if japanese {
        analysis.risk_level.label_ja()
    } else {
        analysis.risk_level.label()
    };

    result.push_str(&format!("Risk: {} | Type: {}", level_str, analysis.query_type.label()));

    if !analysis.affected_tables.is_empty() {
        result.push_str(&format!(" | Tables: {}", analysis.affected_tables.join(", ")));
    }

    if !analysis.warnings.is_empty() {
        result.push_str("\n");
        for warning in &analysis.warnings {
            let msg = if japanese { &warning.message_ja } else { &warning.message };
            result.push_str(&format!("⚠ {}\n", msg));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_select() {
        let analysis = analyze_query("SELECT * FROM users WHERE id = 1");
        assert_eq!(analysis.risk_level, RiskLevel::Safe);
        assert_eq!(analysis.query_type, QueryType::Select);
        assert!(analysis.warnings.is_empty());
    }

    #[test]
    fn test_dangerous_delete() {
        let analysis = analyze_query("DELETE FROM users");
        assert_eq!(analysis.risk_level, RiskLevel::High);
        assert_eq!(analysis.query_type, QueryType::Delete);
        assert!(!analysis.has_where_clause);
        assert!(analysis.warnings.iter().any(|w| w.code == WarningCode::NoWhereClause));
    }

    #[test]
    fn test_safe_delete() {
        let analysis = analyze_query("DELETE FROM users WHERE id = 1");
        assert_eq!(analysis.risk_level, RiskLevel::Medium);
        assert!(analysis.has_where_clause);
    }

    #[test]
    fn test_drop_critical() {
        let analysis = analyze_query("DROP TABLE users");
        assert_eq!(analysis.risk_level, RiskLevel::Critical);
        assert!(analysis.warnings.iter().any(|w| w.code == WarningCode::DropStatement));
    }

    #[test]
    fn test_update_without_where() {
        let analysis = analyze_query("UPDATE users SET active = 0");
        assert_eq!(analysis.risk_level, RiskLevel::High);
        assert!(analysis.warnings.iter().any(|w| w.code == WarningCode::NoWhereClause));
    }

    #[test]
    fn test_multiple_statements() {
        let analysis = analyze_query("DELETE FROM a; DELETE FROM b;");
        assert!(analysis.is_multi_statement);
        assert!(analysis.warnings.iter().any(|w| w.code == WarningCode::MultipleStatements));
    }
}
