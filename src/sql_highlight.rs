//! SQL Syntax Highlighting for the editor
//! Provides colorized SQL display using egui LayoutJob

use eframe::egui::{self, text::LayoutJob, Color32, FontId, TextFormat};

/// SQL syntax highlighting colors
pub struct SqlColors {
    pub keyword: Color32,
    pub function: Color32,
    pub string: Color32,
    pub number: Color32,
    pub operator: Color32,
    pub comment: Color32,
    pub identifier: Color32,
    pub default: Color32,
}

impl Default for SqlColors {
    fn default() -> Self {
        Self {
            keyword: Color32::from_rgb(86, 156, 214),    // Blue
            function: Color32::from_rgb(220, 220, 170),  // Yellow-ish
            string: Color32::from_rgb(206, 145, 120),    // Orange/Brown
            number: Color32::from_rgb(181, 206, 168),    // Light green
            operator: Color32::from_rgb(212, 212, 212),  // Light gray
            comment: Color32::from_rgb(106, 153, 85),    // Green
            identifier: Color32::from_rgb(156, 220, 254), // Light blue
            default: Color32::from_rgb(212, 212, 212),   // Light gray
        }
    }
}

/// SQL Keywords (uppercase for matching)
const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "AND", "OR", "NOT", "IN", "IS", "NULL",
    "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE",
    "CREATE", "TABLE", "INDEX", "VIEW", "DROP", "ALTER", "ADD", "COLUMN",
    "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "UNIQUE", "DEFAULT",
    "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "CROSS", "ON",
    "ORDER", "BY", "ASC", "DESC", "LIMIT", "OFFSET", "GROUP", "HAVING",
    "UNION", "ALL", "DISTINCT", "AS", "CASE", "WHEN", "THEN", "ELSE", "END",
    "EXISTS", "BETWEEN", "LIKE", "GLOB", "ESCAPE",
    "BEGIN", "COMMIT", "ROLLBACK", "TRANSACTION",
    "PRAGMA", "EXPLAIN", "ANALYZE", "VACUUM", "REINDEX",
    "WITH", "RECURSIVE", "REPLACE", "IGNORE", "CONFLICT",
    "CAST", "COLLATE", "NATURAL", "USING", "EXCEPT", "INTERSECT",
    "AUTOINCREMENT", "ROWID", "INTEGER", "TEXT", "REAL", "BLOB", "NUMERIC",
    "TRUE", "FALSE", "IF", "TRUNCATE",
];

/// SQL Functions
const SQL_FUNCTIONS: &[&str] = &[
    "COUNT", "SUM", "AVG", "MIN", "MAX", "TOTAL",
    "ABS", "ROUND", "RANDOM", "LENGTH", "LOWER", "UPPER",
    "SUBSTR", "REPLACE", "TRIM", "LTRIM", "RTRIM", "INSTR",
    "COALESCE", "NULLIF", "IFNULL", "IIF", "TYPEOF",
    "DATE", "TIME", "DATETIME", "JULIANDAY", "STRFTIME",
    "HEX", "UNHEX", "ZEROBLOB", "QUOTE", "PRINTF",
    "JSON", "JSON_ARRAY", "JSON_OBJECT", "JSON_EXTRACT", "JSON_TYPE",
    "GROUP_CONCAT", "JSON_GROUP_ARRAY", "JSON_GROUP_OBJECT",
];

/// Token type for SQL highlighting
#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    Keyword,
    Function,
    String,
    Number,
    Operator,
    Comment,
    Identifier,
    Whitespace,
    Other,
}

/// A token with its text and type
struct Token {
    text: String,
    token_type: TokenType,
}

/// Tokenize SQL for syntax highlighting
fn tokenize_sql(sql: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Whitespace
        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Whitespace,
            });
            continue;
        }

        // Single-line comment (--)
        if c == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
            let start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Comment,
            });
            continue;
        }

        // Multi-line comment (/* */)
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Comment,
            });
            continue;
        }

        // String (single quotes)
        if c == '\'' {
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\'' {
                    if i + 1 < chars.len() && chars[i + 1] == '\'' {
                        // Escaped quote
                        i += 2;
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::String,
            });
            continue;
        }

        // String (double quotes - identifiers in SQL standard)
        if c == '"' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Identifier,
            });
            continue;
        }

        // Backtick quoted identifier
        if c == '`' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '`' {
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Identifier,
            });
            continue;
        }

        // Number
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            let mut has_dot = c == '.';
            i += 1;
            while i < chars.len() {
                let ch = chars[i];
                if ch.is_ascii_digit() {
                    i += 1;
                } else if ch == '.' && !has_dot {
                    has_dot = true;
                    i += 1;
                } else if ch == 'e' || ch == 'E' {
                    i += 1;
                    if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Number,
            });
            continue;
        }

        // Identifier or Keyword
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let upper = word.to_uppercase();

            let token_type = if SQL_KEYWORDS.contains(&upper.as_str()) {
                TokenType::Keyword
            } else if SQL_FUNCTIONS.contains(&upper.as_str()) {
                TokenType::Function
            } else {
                TokenType::Identifier
            };

            tokens.push(Token {
                text: word,
                token_type,
            });
            continue;
        }

        // Operators and punctuation
        let op_chars = ['=', '<', '>', '!', '+', '-', '*', '/', '%', '|', '&', '^', '~'];
        if op_chars.contains(&c) {
            let start = i;
            // Handle multi-character operators
            i += 1;
            if i < chars.len() {
                let next = chars[i];
                if (c == '<' && next == '=') || (c == '>' && next == '=') ||
                   (c == '!' && next == '=') || (c == '<' && next == '>') ||
                   (c == '|' && next == '|') || (c == '&' && next == '&') {
                    i += 1;
                }
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                token_type: TokenType::Operator,
            });
            continue;
        }

        // Other single characters (parentheses, comma, semicolon, etc.)
        tokens.push(Token {
            text: c.to_string(),
            token_type: TokenType::Other,
        });
        i += 1;
    }

    tokens
}

/// Create a LayoutJob for SQL syntax highlighting
pub fn highlight_sql(sql: &str, font_id: FontId, colors: &SqlColors) -> LayoutJob {
    let mut job = LayoutJob::default();
    let tokens = tokenize_sql(sql);

    for token in tokens {
        let color = match token.token_type {
            TokenType::Keyword => colors.keyword,
            TokenType::Function => colors.function,
            TokenType::String => colors.string,
            TokenType::Number => colors.number,
            TokenType::Operator => colors.operator,
            TokenType::Comment => colors.comment,
            TokenType::Identifier => colors.identifier,
            TokenType::Whitespace | TokenType::Other => colors.default,
        };

        job.append(
            &token.text,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            },
        );
    }

    job
}

/// Create a LayoutJob for SQL highlighting (to be used with TextEdit layouter)
pub fn create_layout_job(ui: &egui::Ui, text: &str, wrap_width: f32) -> LayoutJob {
    let colors = SqlColors::default();
    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
    let mut job = highlight_sql(text, font_id, &colors);
    job.wrap.max_width = wrap_width;
    job
}

/// Simple SQL formatter
/// Formats SQL with basic indentation and line breaks
pub fn format_sql(sql: &str) -> String {
    let sql = sql.trim();
    if sql.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut indent_level = 0;
    let indent = "    ";

    // Keywords that should start on a new line
    let newline_before = ["SELECT", "FROM", "WHERE", "AND", "OR", "JOIN", "LEFT", "RIGHT",
                          "INNER", "OUTER", "ORDER", "GROUP", "HAVING", "LIMIT", "UNION",
                          "INSERT", "UPDATE", "DELETE", "SET", "VALUES", "ON"];

    // Keywords that increase indent
    let indent_increase = ["("];
    let indent_decrease = [")"];

    let tokens = tokenize_sql(sql);
    let mut prev_was_newline = true;
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];
        let upper = token.text.to_uppercase();

        // Skip whitespace at beginning or after we added our own
        if token.token_type == TokenType::Whitespace {
            if !prev_was_newline && !result.is_empty() {
                // Add single space for regular whitespace
                if !result.ends_with(' ') && !result.ends_with('\n') {
                    result.push(' ');
                }
            }
            i += 1;
            continue;
        }

        // Handle indent changes
        if indent_increase.contains(&token.text.as_str()) {
            indent_level += 1;
        }
        if indent_decrease.contains(&token.text.as_str()) && indent_level > 0 {
            indent_level -= 1;
        }

        // Add newline before certain keywords
        if newline_before.contains(&upper.as_str()) && !prev_was_newline && !result.is_empty() {
            result.push('\n');
            for _ in 0..indent_level {
                result.push_str(indent);
            }
            prev_was_newline = true;
        } else if prev_was_newline && !result.is_empty() {
            // Add indent at start of line
            for _ in 0..indent_level {
                result.push_str(indent);
            }
        }

        // Add the token
        result.push_str(&token.text);
        prev_was_newline = false;

        // Handle comma - add space after
        if token.text == "," {
            result.push(' ');
        }

        i += 1;
    }

    result.trim().to_string()
}

/// SQL Snippets - commonly used query templates
#[derive(Debug, Clone)]
pub struct SqlSnippet {
    pub name: &'static str,
    pub name_ja: &'static str,
    pub description: &'static str,
    pub description_ja: &'static str,
    pub template: &'static str,
}

/// Get all available SQL snippets
pub fn get_snippets() -> Vec<SqlSnippet> {
    vec![
        SqlSnippet {
            name: "Select All",
            name_ja: "全件取得",
            description: "Select all rows from a table",
            description_ja: "テーブルから全行を取得",
            template: "SELECT * FROM table_name LIMIT 100;",
        },
        SqlSnippet {
            name: "Select Count",
            name_ja: "件数カウント",
            description: "Count rows in a table",
            description_ja: "テーブルの行数をカウント",
            template: "SELECT COUNT(*) FROM table_name;",
        },
        SqlSnippet {
            name: "Select with Condition",
            name_ja: "条件付き検索",
            description: "Select with WHERE clause",
            description_ja: "WHERE句で条件検索",
            template: "SELECT * FROM table_name WHERE column = 'value' LIMIT 100;",
        },
        SqlSnippet {
            name: "Insert Row",
            name_ja: "行挿入",
            description: "Insert a new row",
            description_ja: "新しい行を挿入",
            template: "INSERT INTO table_name (column1, column2) VALUES ('value1', 'value2');",
        },
        SqlSnippet {
            name: "Update Row",
            name_ja: "行更新",
            description: "Update existing row",
            description_ja: "既存の行を更新",
            template: "UPDATE table_name SET column = 'new_value' WHERE id = 1;",
        },
        SqlSnippet {
            name: "Delete Row",
            name_ja: "行削除",
            description: "Delete a row",
            description_ja: "行を削除",
            template: "DELETE FROM table_name WHERE id = 1;",
        },
        SqlSnippet {
            name: "Create Table",
            name_ja: "テーブル作成",
            description: "Create a new table",
            description_ja: "新しいテーブルを作成",
            template: r#"CREATE TABLE table_name (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);"#,
        },
        SqlSnippet {
            name: "Add Column",
            name_ja: "カラム追加",
            description: "Add a column to existing table",
            description_ja: "既存テーブルにカラムを追加",
            template: "ALTER TABLE table_name ADD COLUMN new_column TEXT;",
        },
        SqlSnippet {
            name: "Create Index",
            name_ja: "インデックス作成",
            description: "Create an index",
            description_ja: "インデックスを作成",
            template: "CREATE INDEX idx_name ON table_name (column_name);",
        },
        SqlSnippet {
            name: "Join Tables",
            name_ja: "テーブル結合",
            description: "Join two tables",
            description_ja: "2つのテーブルを結合",
            template: r#"SELECT a.*, b.*
FROM table_a a
LEFT JOIN table_b b ON a.id = b.a_id
LIMIT 100;"#,
        },
        SqlSnippet {
            name: "Group By",
            name_ja: "グループ集計",
            description: "Group and aggregate",
            description_ja: "グループ化して集計",
            template: r#"SELECT column, COUNT(*) as count
FROM table_name
GROUP BY column
ORDER BY count DESC;"#,
        },
        SqlSnippet {
            name: "Schema Info",
            name_ja: "スキーマ情報",
            description: "Get table schema",
            description_ja: "テーブルスキーマを取得",
            template: "PRAGMA table_info(table_name);",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_select() {
        let tokens = tokenize_sql("SELECT * FROM users");
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].token_type, TokenType::Keyword); // SELECT
        assert_eq!(tokens[4].token_type, TokenType::Keyword); // FROM
    }

    #[test]
    fn test_tokenize_string() {
        let tokens = tokenize_sql("WHERE name = 'test'");
        let string_token = tokens.iter().find(|t| t.token_type == TokenType::String);
        assert!(string_token.is_some());
        assert_eq!(string_token.unwrap().text, "'test'");
    }

    #[test]
    fn test_tokenize_comment() {
        let tokens = tokenize_sql("SELECT * -- comment\nFROM users");
        let comment_token = tokens.iter().find(|t| t.token_type == TokenType::Comment);
        assert!(comment_token.is_some());
    }

    #[test]
    fn test_format_simple() {
        let sql = "SELECT * FROM users WHERE id = 1";
        let formatted = format_sql(sql);
        assert!(formatted.contains("SELECT"));
        assert!(formatted.contains("FROM"));
        assert!(formatted.contains("WHERE"));
    }
}
