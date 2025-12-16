use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

// ============================================================================
// API Error Types for better error handling and user guidance
// ============================================================================

/// Structured API error for user-friendly error handling
#[derive(Debug, Clone)]
pub enum ApiError {
    /// Invalid API token (401)
    InvalidToken,
    /// Token lacks required permission (403)
    InsufficientScope { required: String },
    /// Rate limited (429)
    RateLimited { retry_after: u64 },
    /// Network/connection error
    NetworkError(String),
    /// Server error (5xx)
    ServerError { status: u16, message: String },
    /// Unknown error
    Unknown(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::InvalidToken => write!(f, "Invalid API token"),
            ApiError::InsufficientScope { required } => {
                write!(f, "Token lacks permission. Required: {}", required)
            }
            ApiError::RateLimited { retry_after } => {
                write!(f, "Rate limited. Retry after {} seconds", retry_after)
            }
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::ServerError { status, message } => {
                write!(f, "Server error ({}): {}", status, message)
            }
            ApiError::Unknown(msg) => write!(f, "{}", msg),
        }
    }
}

impl ApiError {
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            ApiError::InvalidToken => {
                "Invalid API token. Please check your token and try again.".to_string()
            }
            ApiError::InsufficientScope { required } => {
                format!(
                    "Your API token doesn't have the required permissions.\n\
                     Required scope: {}",
                    required
                )
            }
            ApiError::RateLimited { retry_after } => {
                format!(
                    "API rate limit exceeded. Please wait {} seconds before trying again.",
                    retry_after
                )
            }
            ApiError::NetworkError(_) => {
                "Network connection failed. Please check your internet connection.".to_string()
            }
            ApiError::ServerError { status, .. } => {
                format!(
                    "Cloudflare server error ({}). This may be temporary, please try again later.",
                    status
                )
            }
            ApiError::Unknown(msg) => msg.clone(),
        }
    }

    /// Get an action hint for the user
    pub fn action_hint(&self) -> Option<String> {
        match self {
            ApiError::InvalidToken => Some(
                "1. Go to Cloudflare Dashboard → My Profile → API Tokens\n\
                 2. Verify your token is correct and not expired\n\
                 3. Create a new token if needed"
                    .to_string(),
            ),
            ApiError::InsufficientScope { required } => Some(format!(
                "1. Go to Cloudflare Dashboard → My Profile → API Tokens\n\
                 2. Edit your token or create a new one\n\
                 3. Add the '{}' permission\n\
                 4. Save and copy the new token",
                required
            )),
            ApiError::RateLimited { retry_after } => {
                Some(format!("Wait {} seconds and try again.", retry_after))
            }
            ApiError::NetworkError(_) => Some(
                "1. Check your internet connection\n\
                 2. Try disabling VPN if enabled\n\
                 3. Check if api.cloudflare.com is accessible"
                    .to_string(),
            ),
            ApiError::ServerError { .. } => {
                Some("This is a temporary server issue. Try again in a few minutes.".to_string())
            }
            ApiError::Unknown(_) => None,
        }
    }

    /// Get the error kind for i18n lookup
    pub fn kind(&self) -> &'static str {
        match self {
            ApiError::InvalidToken => "invalid_token",
            ApiError::InsufficientScope { .. } => "insufficient_scope",
            ApiError::RateLimited { .. } => "rate_limited",
            ApiError::NetworkError(_) => "network_error",
            ApiError::ServerError { .. } => "server_error",
            ApiError::Unknown(_) => "unknown",
        }
    }
}

#[derive(Clone, Debug)]
pub struct D1Client {
    account_id: String,
    database_id: String,
    api_token: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct QueryRequest {
    sql: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct D1Response {
    pub result: Vec<QueryResult>,
    pub success: bool,
    pub errors: Vec<D1Error>,
}

/// Rate limit information from Cloudflare API headers
#[derive(Debug, Clone, Default)]
pub struct ApiRateLimitInfo {
    pub remaining: Option<u32>,
    pub limit: Option<u32>,
    pub reset_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub results: Option<Vec<serde_json::Value>>,
    pub success: bool,
    pub meta: Option<QueryMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryMeta {
    pub changed_db: Option<bool>,
    pub changes: Option<i64>,
    pub duration: Option<f64>,
    pub rows_read: Option<i64>,
    pub rows_written: Option<i64>,
}

impl QueryMeta {
    pub fn rows_read_or_default(&self) -> i64 {
        self.rows_read.unwrap_or(0)
    }

    pub fn rows_written_or_default(&self) -> i64 {
        self.rows_written.unwrap_or(0)
    }

    pub fn duration_ms_or_default(&self) -> f64 {
        self.duration.unwrap_or(0.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct D1Error {
    pub code: Option<i64>,
    pub message: String,
}

/// Foreign key reference information
#[derive(Debug, Clone)]
pub struct ForeignKeyRef {
    pub table: String,
    pub column: String,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,
    pub pk: bool,
    pub notnull: bool,
    pub foreign_key: Option<ForeignKeyRef>,
}

impl D1Client {
    pub fn new(account_id: String, database_id: String, api_token: String) -> Self {
        Self {
            account_id,
            database_id,
            api_token,
            client: reqwest::Client::new(),
        }
    }

    fn api_url(&self) -> String {
        format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/d1/database/{}/query",
            self.account_id, self.database_id
        )
    }

    pub async fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<D1Response, String> {
        let (response, _, _) = self.execute_with_metrics(sql, params).await?;
        Ok(response)
    }

    /// Execute a query and return response, rate limit info, and query metadata
    pub async fn execute_with_metrics(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<(D1Response, ApiRateLimitInfo, Option<QueryMeta>), String> {
        let request = QueryRequest {
            sql: sql.to_string(),
            params,
        };

        let response = self
            .client
            .post(&self.api_url())
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        // Extract rate limit info from headers
        let rate_limit = ApiRateLimitInfo {
            remaining: response.headers()
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            limit: response.headers()
                .get("x-ratelimit-limit")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            reset_at: response.headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
        };

        let status = response.status();
        let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("API error ({}): {}", status, text));
        }

        let d1_response: D1Response = serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {} - {}", e, text))?;

        // Extract query meta from the first result
        let query_meta = d1_response.result.first().and_then(|r| r.meta.clone());

        Ok((d1_response, rate_limit, query_meta))
    }

    /// Get tables with metrics
    pub async fn get_tables_with_metrics(&self) -> Result<(Vec<String>, ApiRateLimitInfo, Option<QueryMeta>), String> {
        let (response, rate_limit, meta) = self
            .execute_with_metrics(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
                vec![],
            )
            .await?;

        if !response.success {
            let error_msg = response
                .errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(error_msg);
        }

        let tables = response
            .result
            .first()
            .and_then(|r| r.results.as_ref())
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.get("name").and_then(|v| v.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok((tables, rate_limit, meta))
    }

    /// Get row count with multiple filter conditions
    pub async fn get_row_count_with_filters(&self, table: &str, filters: &[(&str, &str, &str)]) -> Result<i64, String> {
        let (where_clause, params) = Self::build_where_clause(filters);
        let sql = if where_clause.is_empty() {
            format!("SELECT COUNT(*) as count FROM {}", table)
        } else {
            format!("SELECT COUNT(*) as count FROM {} WHERE {}", table, where_clause)
        };

        let response = self.execute(&sql, params).await?;

        if !response.success {
            return Err("Failed to get row count".to_string());
        }

        let count = response
            .result
            .first()
            .and_then(|r| r.results.as_ref())
            .and_then(|rows| rows.first())
            .and_then(|row| row.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(count)
    }

    /// Get table data with multiple filter conditions
    pub async fn get_table_data_with_filters(
        &self,
        table: &str,
        limit: i64,
        offset: i64,
        filters: &[(&str, &str, &str)],
    ) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>), String> {
        let (where_clause, params) = Self::build_where_clause(filters);
        let sql = if where_clause.is_empty() {
            format!("SELECT * FROM {} LIMIT {} OFFSET {}", table, limit, offset)
        } else {
            format!("SELECT * FROM {} WHERE {} LIMIT {} OFFSET {}", table, where_clause, limit, offset)
        };

        let response = self.execute(&sql, params).await?;

        if !response.success {
            let error_msg = response
                .errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(error_msg);
        }

        let result = response.result.first().ok_or("No result")?;
        let rows = result.results.as_ref().ok_or("No rows")?;

        // Get column order from schema (PRAGMA table_info returns columns in definition order)
        let schema = self.get_table_schema(table).await?;
        let columns: Vec<String> = schema.into_iter().map(|c| c.name).collect();

        if rows.is_empty() {
            return Ok((columns, vec![]));
        }

        let data: Vec<Vec<serde_json::Value>> = rows
            .iter()
            .map(|row| {
                columns
                    .iter()
                    .map(|col| row.get(col).cloned().unwrap_or(serde_json::Value::Null))
                    .collect()
            })
            .collect();

        Ok((columns, data))
    }

    /// Build WHERE clause from filter conditions
    fn build_where_clause(filters: &[(&str, &str, &str)]) -> (String, Vec<serde_json::Value>) {
        if filters.is_empty() {
            return (String::new(), vec![]);
        }

        let mut clauses = Vec::new();
        let mut params = Vec::new();

        for (column, operator, value) in filters {
            let op_upper = operator.to_uppercase();
            match op_upper.as_str() {
                "IS NULL" => {
                    clauses.push(format!("{} IS NULL", column));
                }
                "IS NOT NULL" => {
                    clauses.push(format!("{} IS NOT NULL", column));
                }
                "LIKE" => {
                    clauses.push(format!("{} LIKE ?", column));
                    params.push(serde_json::json!(value));
                }
                "IN" => {
                    // Parse comma-separated values
                    let values: Vec<&str> = value.split(',').map(|s| s.trim()).collect();
                    let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
                    clauses.push(format!("{} IN ({})", column, placeholders.join(", ")));
                    for v in values {
                        params.push(serde_json::json!(v));
                    }
                }
                "!=" | "<>" => {
                    clauses.push(format!("{} != ?", column));
                    params.push(serde_json::json!(value));
                }
                ">" | "<" | ">=" | "<=" | "=" => {
                    clauses.push(format!("{} {} ?", column, operator));
                    params.push(serde_json::json!(value));
                }
                _ => {
                    // Default to equals
                    clauses.push(format!("{} = ?", column));
                    params.push(serde_json::json!(value));
                }
            }
        }

        (clauses.join(" AND "), params)
    }

    pub async fn get_table_schema(&self, table: &str) -> Result<Vec<ColumnInfo>, String> {
        // Get basic column info
        let sql = format!("PRAGMA table_info({})", table);
        let response = self.execute(&sql, vec![]).await?;

        if !response.success {
            let error_msg = response
                .errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(error_msg);
        }

        let mut columns: Vec<ColumnInfo> = response
            .result
            .first()
            .and_then(|r| r.results.as_ref())
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| {
                        let name = row.get("name").and_then(|v| v.as_str()).map(String::from)?;
                        let col_type = row.get("type").and_then(|v| v.as_str()).map(String::from).unwrap_or_default();
                        let pk = row.get("pk").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
                        let notnull = row.get("notnull").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
                        Some(ColumnInfo { name, col_type, pk, notnull, foreign_key: None })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Get foreign key info
        let fk_sql = format!("PRAGMA foreign_key_list({})", table);
        if let Ok(fk_response) = self.execute(&fk_sql, vec![]).await {
            if fk_response.success {
                if let Some(fk_rows) = fk_response.result.first().and_then(|r| r.results.as_ref()) {
                    for fk_row in fk_rows {
                        let from_col = fk_row.get("from").and_then(|v| v.as_str());
                        let to_table = fk_row.get("table").and_then(|v| v.as_str());
                        let to_col = fk_row.get("to").and_then(|v| v.as_str());

                        if let (Some(from), Some(table), Some(to)) = (from_col, to_table, to_col) {
                            if let Some(col) = columns.iter_mut().find(|c| c.name == from) {
                                col.foreign_key = Some(ForeignKeyRef {
                                    table: table.to_string(),
                                    column: to.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(columns)
    }
}

pub type SharedClient = Arc<Mutex<Option<D1Client>>>;

pub fn create_shared_client() -> SharedClient {
    Arc::new(Mutex::new(None))
}

// ============================================================================
// Cloudflare Account and D1 Database listing APIs
// ============================================================================

/// Account info from Cloudflare API
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    pub id: String,
    pub name: String,
}

/// Response structure for accounts list
#[derive(Debug, Deserialize)]
struct AccountsResponse {
    result: Vec<AccountInfo>,
    success: bool,
    errors: Vec<D1Error>,
}

/// D1 Database info from Cloudflare API
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseInfo {
    pub uuid: String,
    pub name: String,
    pub created_at: String,
}

/// Response structure for D1 databases list
#[derive(Debug, Deserialize)]
struct DatabasesResponse {
    result: Vec<DatabaseInfo>,
    success: bool,
    errors: Vec<D1Error>,
}

/// List all Cloudflare accounts accessible with the given API token
pub async fn list_accounts(api_token: &str) -> Result<Vec<AccountInfo>, ApiError> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.cloudflare.com/client/v4/accounts")
        .header("Authorization", format!("Bearer {}", api_token))
        .query(&[("per_page", "50")])
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    let status = response.status();

    // Extract rate limit info for 429 errors
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    let text = response
        .text()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    match status.as_u16() {
        401 => return Err(ApiError::InvalidToken),
        403 => {
            return Err(ApiError::InsufficientScope {
                required: "Account:Read".to_string(),
            })
        }
        429 => {
            return Err(ApiError::RateLimited {
                retry_after,
            })
        }
        500..=599 => {
            return Err(ApiError::ServerError {
                status: status.as_u16(),
                message: text,
            })
        }
        _ if !status.is_success() => {
            // Try to parse error message from response
            if let Ok(err_response) = serde_json::from_str::<AccountsResponse>(&text) {
                if let Some(err) = err_response.errors.first() {
                    return Err(ApiError::Unknown(err.message.clone()));
                }
            }
            return Err(ApiError::Unknown(format!("API error ({}): {}", status, text)));
        }
        _ => {}
    }

    let accounts_response: AccountsResponse = serde_json::from_str(&text)
        .map_err(|e| ApiError::Unknown(format!("Failed to parse response: {}", e)))?;

    if !accounts_response.success {
        let error_msg = accounts_response
            .errors
            .first()
            .map(|e| e.message.clone())
            .unwrap_or_else(|| "Unknown error".to_string());
        return Err(ApiError::Unknown(error_msg));
    }

    Ok(accounts_response.result)
}

/// List all D1 databases in an account
pub async fn list_databases(api_token: &str, account_id: &str) -> Result<Vec<DatabaseInfo>, ApiError> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/d1/database",
        account_id
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .query(&[("per_page", "50")])
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    let status = response.status();

    // Extract rate limit info for 429 errors
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    let text = response
        .text()
        .await
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    match status.as_u16() {
        401 => return Err(ApiError::InvalidToken),
        403 => {
            return Err(ApiError::InsufficientScope {
                required: "Account:D1:Read".to_string(),
            })
        }
        429 => {
            return Err(ApiError::RateLimited {
                retry_after,
            })
        }
        500..=599 => {
            return Err(ApiError::ServerError {
                status: status.as_u16(),
                message: text,
            })
        }
        _ if !status.is_success() => {
            // Try to parse error message from response
            if let Ok(err_response) = serde_json::from_str::<DatabasesResponse>(&text) {
                if let Some(err) = err_response.errors.first() {
                    return Err(ApiError::Unknown(err.message.clone()));
                }
            }
            return Err(ApiError::Unknown(format!("API error ({}): {}", status, text)));
        }
        _ => {}
    }

    let db_response: DatabasesResponse = serde_json::from_str(&text)
        .map_err(|e| ApiError::Unknown(format!("Failed to parse response: {}", e)))?;

    if !db_response.success {
        let error_msg = db_response
            .errors
            .first()
            .map(|e| e.message.clone())
            .unwrap_or_else(|| "Unknown error".to_string());
        return Err(ApiError::Unknown(error_msg));
    }

    Ok(db_response.result)
}
