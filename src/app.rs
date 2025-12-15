use crate::api::{create_shared_client, ApiRateLimitInfo, ColumnInfo, D1Client, ForeignKeyRef, SharedClient};
use crate::audit::{self, AuditJournal, ChangeRecord, OperationType};
use crate::export::{self, ConflictStrategy, ExportFormat, ImportData, ImportFormat};
use crate::i18n::{I18n, Language};
use crate::schema_diff::{self, ColumnSchema, DatabaseSchema, SchemaDiff, TableSchema, DiffType};
use crate::schema_explorer::{self, ForeignKeyRelation, RelationshipView, SchemaGraph};
use crate::secure_storage::{self, SecureString};
use crate::settings_io;
use crate::sql_highlight;
use crate::sql_safety::{self, SafetyAnalysis};
use crate::theme::{self, AppColors, Radius, Spacing};
use eframe::egui::{self, Color32, RichText, Stroke};
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};
use zeroize::Zeroize;

/// Safely truncate a string to a maximum number of characters (UTF-8 safe)
fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

/// Environment type for safety classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EnvironmentType {
    #[default]
    Development,
    Staging,
    Production,
}

impl EnvironmentType {
    pub fn label(&self) -> &'static str {
        match self {
            EnvironmentType::Development => "Development",
            EnvironmentType::Staging => "Staging",
            EnvironmentType::Production => "Production",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            EnvironmentType::Development => "DEV",
            EnvironmentType::Staging => "STG",
            EnvironmentType::Production => "PROD",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            EnvironmentType::Development => AppColors::SUCCESS,
            EnvironmentType::Staging => AppColors::WARNING,
            EnvironmentType::Production => AppColors::ERROR,
        }
    }

    pub fn is_dangerous(&self) -> bool {
        matches!(self, EnvironmentType::Production | EnvironmentType::Staging)
    }
}

/// Connection type distinguishing remote vs local
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConnectionType {
    #[default]
    Remote,
    Local,
}

impl ConnectionType {
    pub fn label(&self) -> &'static str {
        match self {
            ConnectionType::Remote => "Remote (Cloudflare D1)",
            ConnectionType::Local => "Local (SQLite)",
        }
    }
}

/// Onboarding wizard step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnboardingStep {
    #[default]
    Welcome,
    ApiToken,
    SelectAccount,
    SelectDatabase,
    SelectLocalDatabase,  // For local mode
    ConfigureEnv,
    TestConnection,
}

impl OnboardingStep {
    /// Get step index for remote mode
    pub fn index(&self) -> usize {
        match self {
            OnboardingStep::Welcome => 0,
            OnboardingStep::ApiToken => 1,
            OnboardingStep::SelectAccount => 2,
            OnboardingStep::SelectDatabase => 3,
            OnboardingStep::SelectLocalDatabase => 1, // Local mode: step 2
            OnboardingStep::ConfigureEnv => 4,
            OnboardingStep::TestConnection => 5,
        }
    }

    /// Get step index for local mode (fewer steps)
    pub fn local_index(&self) -> usize {
        match self {
            OnboardingStep::Welcome => 0,
            OnboardingStep::SelectLocalDatabase => 1,
            OnboardingStep::ConfigureEnv => 2,
            OnboardingStep::TestConnection => 3,
            _ => 0,
        }
    }

    pub fn total_steps(local_mode: bool) -> usize {
        if local_mode { 4 } else { 6 }
    }

    pub fn can_go_back(&self) -> bool {
        !matches!(self, OnboardingStep::Welcome)
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Option<OnboardingStep> {
        match self {
            OnboardingStep::Welcome => Some(OnboardingStep::ApiToken),
            OnboardingStep::ApiToken => Some(OnboardingStep::SelectAccount),
            OnboardingStep::SelectAccount => Some(OnboardingStep::SelectDatabase),
            OnboardingStep::SelectDatabase => Some(OnboardingStep::ConfigureEnv),
            OnboardingStep::SelectLocalDatabase => Some(OnboardingStep::ConfigureEnv),
            OnboardingStep::ConfigureEnv => Some(OnboardingStep::TestConnection),
            OnboardingStep::TestConnection => None,
        }
    }

    pub fn prev(&self, local_mode: bool) -> Option<OnboardingStep> {
        match self {
            OnboardingStep::Welcome => None,
            OnboardingStep::ApiToken => Some(OnboardingStep::Welcome),
            OnboardingStep::SelectAccount => Some(OnboardingStep::ApiToken),
            OnboardingStep::SelectDatabase => Some(OnboardingStep::SelectAccount),
            OnboardingStep::SelectLocalDatabase => Some(OnboardingStep::Welcome),
            OnboardingStep::ConfigureEnv => {
                if local_mode {
                    Some(OnboardingStep::SelectLocalDatabase)
                } else {
                    Some(OnboardingStep::SelectDatabase)
                }
            }
            OnboardingStep::TestConnection => Some(OnboardingStep::ConfigureEnv),
        }
    }
}

/// Cloudflare account info from API
#[derive(Debug, Clone)]
pub struct CloudflareAccount {
    pub id: String,
    pub name: String,
}

/// Cloudflare D1 database info from API
#[derive(Debug, Clone)]
pub struct CloudflareDatabase {
    pub uuid: String,
    pub name: String,
    pub created_at: String,
}

/// Wizard mode: Remote or Local setup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WizardMode {
    #[default]
    Remote,
    Local,
}

/// Onboarding wizard state
#[derive(Debug, Default)]
pub struct OnboardingWizardState {
    pub current_step: OnboardingStep,
    pub wizard_mode: WizardMode,

    // Step 1: API Token
    pub api_token: String,

    // Step 2: Account selection
    pub accounts: Vec<CloudflareAccount>,
    pub accounts_loading: bool,
    pub accounts_error: Option<String>,
    pub selected_account_idx: Option<usize>,

    // Step 3: Database selection
    pub databases: Vec<CloudflareDatabase>,
    pub databases_loading: bool,
    pub databases_error: Option<String>,
    pub selected_database_idx: Option<usize>,

    // Step 4: Environment config
    pub connection_name: String,
    pub environment: EnvironmentType,
    pub read_only: bool,

    // Step 5: Test results
    pub test_in_progress: bool,
    pub test_result: Option<(bool, String)>,

    // Local mode
    pub local_databases: Vec<crate::local_db::LocalD1Database>,
    pub local_db_scanning: bool,
    pub selected_local_db_idx: Option<usize>,
}

/// Profile metadata stored in JSON file (no sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub id: String,
    pub name: String,
    pub account_id: String,
    pub database_id: String,
    #[serde(default)]
    pub environment: EnvironmentType,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub connection_type: ConnectionType,
    #[serde(default)]
    pub local_path: Option<String>,
}

/// Full profile with API token
#[derive(Debug)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub account_id: String,
    pub database_id: String,
    pub api_token: SecureString,
    pub environment: EnvironmentType,
    pub read_only: bool,
    pub connection_type: ConnectionType,
    pub local_path: Option<String>,
}

impl Clone for ConnectionProfile {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            account_id: self.account_id.clone(),
            database_id: self.database_id.clone(),
            api_token: SecureString::new(self.api_token.as_str().to_string()),
            environment: self.environment,
            read_only: self.read_only,
            connection_type: self.connection_type,
            local_path: self.local_path.clone(),
        }
    }
}

impl ConnectionProfile {
    fn new() -> Self {
        Self {
            id: uuid_simple(),
            name: "New Connection".to_string(),
            account_id: String::new(),
            database_id: String::new(),
            api_token: SecureString::default(),
            environment: EnvironmentType::Development,
            read_only: false,
            connection_type: ConnectionType::Remote,
            local_path: None,
        }
    }

    fn from_metadata(meta: &ProfileMetadata) -> Self {
        let token = secure_storage::get_token(&meta.id)
            .unwrap_or_else(|_| SecureString::default());

        Self {
            id: meta.id.clone(),
            name: meta.name.clone(),
            account_id: meta.account_id.clone(),
            database_id: meta.database_id.clone(),
            api_token: token,
            environment: meta.environment,
            read_only: meta.read_only,
            connection_type: meta.connection_type,
            local_path: meta.local_path.clone(),
        }
    }

    fn to_metadata(&self) -> ProfileMetadata {
        ProfileMetadata {
            id: self.id.clone(),
            name: self.name.clone(),
            account_id: self.account_id.clone(),
            database_id: self.database_id.clone(),
            environment: self.environment,
            read_only: self.read_only,
            connection_type: self.connection_type,
            local_path: self.local_path.clone(),
        }
    }

    /// Check if write operations are allowed
    fn can_write(&self) -> bool {
        !self.read_only
    }

    /// Check if this is a dangerous environment requiring extra confirmation
    fn requires_confirmation(&self) -> bool {
        self.environment.is_dangerous()
    }
}

/// Mode for local database selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum LocalDatabaseMode {
    #[default]
    CreateNew,
    OpenExisting,
}

struct EditingProfile {
    id: String,
    name: String,
    account_id: String,
    database_id: String,
    api_token: String,
    environment: EnvironmentType,
    read_only: bool,
    connection_type: ConnectionType,
    local_path: String,
    local_mode: LocalDatabaseMode,
}

impl EditingProfile {
    fn from_profile(profile: &ConnectionProfile) -> Self {
        let local_path = profile.local_path.clone().unwrap_or_default();
        Self {
            id: profile.id.clone(),
            name: profile.name.clone(),
            account_id: profile.account_id.clone(),
            database_id: profile.database_id.clone(),
            api_token: profile.api_token.as_str().to_string(),
            environment: profile.environment,
            read_only: profile.read_only,
            connection_type: profile.connection_type,
            local_path,
            // If editing existing profile with path, default to OpenExisting
            local_mode: LocalDatabaseMode::OpenExisting,
        }
    }

    fn new() -> Self {
        Self {
            id: uuid_simple(),
            name: String::new(),
            account_id: String::new(),
            database_id: String::new(),
            api_token: String::new(),
            environment: EnvironmentType::Development,
            read_only: false,
            connection_type: ConnectionType::default(),
            local_path: String::new(),
            local_mode: LocalDatabaseMode::default(),
        }
    }

    fn to_profile(&self) -> ConnectionProfile {
        ConnectionProfile {
            id: self.id.clone(),
            name: self.name.clone(),
            account_id: self.account_id.clone(),
            database_id: self.database_id.clone(),
            api_token: SecureString::new(self.api_token.clone()),
            environment: self.environment,
            read_only: self.read_only,
            connection_type: self.connection_type,
            local_path: if self.local_path.is_empty() { None } else { Some(self.local_path.clone()) },
        }
    }
}

impl Drop for EditingProfile {
    fn drop(&mut self) {
        self.api_token.zeroize();
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", now)
}

fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", now)
}

/// Filter operator for table view
#[derive(Debug, Clone, PartialEq)]
enum FilterOperator {
    Equals,         // =
    NotEquals,      // !=
    GreaterThan,    // >
    LessThan,       // <
    GreaterOrEqual, // >=
    LessOrEqual,    // <=
    Like,           // LIKE
    In,             // IN
    IsNull,         // IS NULL
    IsNotNull,      // IS NOT NULL
}

impl FilterOperator {
    fn label(&self) -> &'static str {
        match self {
            FilterOperator::Equals => "=",
            FilterOperator::NotEquals => "≠",
            FilterOperator::GreaterThan => ">",
            FilterOperator::LessThan => "<",
            FilterOperator::GreaterOrEqual => "≥",
            FilterOperator::LessOrEqual => "≤",
            FilterOperator::Like => "LIKE",
            FilterOperator::In => "IN",
            FilterOperator::IsNull => "IS NULL",
            FilterOperator::IsNotNull => "IS NOT NULL",
        }
    }

    fn all() -> &'static [FilterOperator] {
        &[
            FilterOperator::Equals,
            FilterOperator::NotEquals,
            FilterOperator::GreaterThan,
            FilterOperator::LessThan,
            FilterOperator::GreaterOrEqual,
            FilterOperator::LessOrEqual,
            FilterOperator::Like,
            FilterOperator::In,
            FilterOperator::IsNull,
            FilterOperator::IsNotNull,
        ]
    }

    fn needs_value(&self) -> bool {
        !matches!(self, FilterOperator::IsNull | FilterOperator::IsNotNull)
    }
}

/// Filter condition for table view
#[derive(Debug, Clone)]
struct FilterCondition {
    column: String,
    operator: FilterOperator,
    value: String,
}

/// Multiple filters for table view
#[derive(Debug, Clone, Default)]
struct TableFilter {
    conditions: Vec<FilterCondition>,
}

#[derive(Debug)]
struct ConnectionTab {
    profile: ConnectionProfile,
    connected: bool,
    tables: Vec<String>,
    selected_table: Option<String>,
    columns: Vec<String>,
    column_info: Vec<ColumnInfo>,
    rows: Vec<Vec<serde_json::Value>>,
    row_count: i64,
    current_page: i64,
    sql_query: String,
    query_result: String,
    status_message: String,
    loading: bool,
    client: SharedClient,
    selected_row: Option<usize>,
    highlighted_column: Option<String>,
    filter: Option<TableFilter>,
}

impl ConnectionTab {
    fn new(profile: ConnectionProfile) -> Self {
        Self {
            profile,
            connected: false,
            tables: vec![],
            selected_table: None,
            columns: vec![],
            column_info: vec![],
            rows: vec![],
            row_count: 0,
            current_page: 0,
            sql_query: String::new(),
            query_result: String::new(),
            status_message: "Not connected".to_string(),
            loading: false,
            client: create_shared_client(),
            selected_row: None,
            highlighted_column: None,
            filter: None,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct EditingCell {
    row_idx: usize,
    col_idx: usize,
    column_name: String,
    value: String,
    pk_value: serde_json::Value,
    cursor_position: Option<usize>,
    selection_range: Option<(usize, usize)>, // (start, end) character indices
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    last_snapshot: String,
}

#[derive(Debug, Clone)]
struct RowEditorState {
    values: Vec<String>,
    columns: Vec<ColumnInfo>,
    is_duplicate: bool,
    source_row: Option<Vec<serde_json::Value>>,
}

/// Pending write operation that needs confirmation
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum PendingOperation {
    DeleteRow { table: String, id: i64 },
    UpdateCell { table: String, pk_column: String, pk_value: serde_json::Value, column: String, old_value: String, new_value: serde_json::Value },
    InsertRow { table: String, columns: Vec<String>, values: Vec<serde_json::Value> },
    ExecuteQuery { sql: String },
    ImportData { statements: Vec<String> },
}

impl PendingOperation {
    fn description(&self) -> String {
        match self {
            PendingOperation::DeleteRow { table, id } => {
                format!("DELETE FROM {} WHERE id = {}", table, id)
            }
            PendingOperation::UpdateCell { table, pk_column, pk_value, column, old_value, new_value } => {
                format!("UPDATE {} SET {} = {} WHERE {} = {}\n\nOld value: {}",
                    table, column, new_value, pk_column, pk_value, old_value)
            }
            PendingOperation::InsertRow { table, columns, values } => {
                let cols = columns.join(", ");
                let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                format!("INSERT INTO {} ({}) VALUES ({})", table, cols, vals.join(", "))
            }
            PendingOperation::ExecuteQuery { sql } => sql.clone(),
            PendingOperation::ImportData { statements } => {
                format!("{} SQL statements to execute:\n\n{}",
                    statements.len(),
                    statements.iter().take(5).cloned().collect::<Vec<_>>().join("\n")
                    + if statements.len() > 5 { "\n..." } else { "" }
                )
            }
        }
    }

    fn operation_type(&self) -> &'static str {
        match self {
            PendingOperation::DeleteRow { .. } => "Delete Row",
            PendingOperation::UpdateCell { .. } => "Update Cell",
            PendingOperation::InsertRow { .. } => "Insert Row",
            PendingOperation::ExecuteQuery { .. } => "Execute Query",
            PendingOperation::ImportData { .. } => "Import Data",
        }
    }
}

/// Query history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistoryEntry {
    pub timestamp: u64,
    pub database_id: String,
    pub database_name: String,
    pub sql: String,
    pub success: bool,
    pub rows_affected: Option<i64>,
    pub duration_ms: Option<f64>,
}

/// Execution log entry for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogEntry {
    pub timestamp: u64,
    pub database_id: String,
    pub database_name: String,
    pub environment: EnvironmentType,
    pub sql: String,
    pub query_type: String,
    pub risk_level: String,
    pub success: bool,
    pub rows_affected: Option<i64>,
    pub duration_ms: Option<f64>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

/// Production lock state per database
#[derive(Debug, Clone, Default)]
pub struct ProductionLockState {
    /// Timestamp when unlock expires (0 = locked)
    pub unlock_expires: u64,
}

impl ProductionLockState {
    pub fn is_unlocked(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.unlock_expires > now
    }

    pub fn remaining_seconds(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.unlock_expires.saturating_sub(now)
    }

    pub fn unlock_for_duration(&mut self, seconds: u64) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.unlock_expires = now + seconds;
    }

    pub fn lock(&mut self) {
        self.unlock_expires = 0;
    }
}

/// Cached metadata for a database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMetadata {
    pub tables: Vec<String>,
    pub schemas: std::collections::HashMap<String, Vec<CachedColumnInfo>>,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedForeignKeyRef {
    pub table: String,
    pub column: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedColumnInfo {
    pub name: String,
    pub col_type: String,
    pub pk: bool,
    pub notnull: bool,
    #[serde(default)]
    pub foreign_key: Option<CachedForeignKeyRef>,
}

impl From<&ColumnInfo> for CachedColumnInfo {
    fn from(col: &ColumnInfo) -> Self {
        Self {
            name: col.name.clone(),
            col_type: col.col_type.clone(),
            pk: col.pk,
            notnull: col.notnull,
            foreign_key: col.foreign_key.as_ref().map(|fk| CachedForeignKeyRef {
                table: fk.table.clone(),
                column: fk.column.clone(),
            }),
        }
    }
}

impl From<&CachedColumnInfo> for ColumnInfo {
    fn from(col: &CachedColumnInfo) -> Self {
        Self {
            name: col.name.clone(),
            col_type: col.col_type.clone(),
            pk: col.pk,
            notnull: col.notnull,
            foreign_key: col.foreign_key.as_ref().map(|fk| ForeignKeyRef {
                table: fk.table.clone(),
                column: fk.column.clone(),
            }),
        }
    }
}

/// Rate limit tracking (wraps API info with timestamp)
#[derive(Debug, Clone, Default)]
pub struct RateLimitInfo {
    pub api_info: ApiRateLimitInfo,
    pub last_updated: u64,
}

impl RateLimitInfo {
    fn update(&mut self, info: ApiRateLimitInfo) {
        use std::time::{SystemTime, UNIX_EPOCH};
        self.api_info = info;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }
}

/// Usage metrics for tracking API and query usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageMetrics {
    /// Total API requests made in current session
    pub session_api_requests: u64,
    /// Total rows read in current session
    pub session_rows_read: u64,
    /// Total rows written in current session
    pub session_rows_written: u64,
    /// Total query execution time in ms
    pub session_total_duration_ms: f64,
    /// Number of queries executed in current session
    pub session_query_count: u64,
    /// Session start timestamp
    pub session_start: u64,
    /// Historical usage (last 5 minutes, for rate limit context)
    pub requests_last_5min: Vec<u64>,
}

impl UsageMetrics {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        Self {
            session_start: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            ..Default::default()
        }
    }

    pub fn record_request(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.session_api_requests += 1;
        self.requests_last_5min.push(now);

        // Clean up old entries (older than 5 minutes)
        let cutoff = now.saturating_sub(300);
        self.requests_last_5min.retain(|&t| t > cutoff);
    }

    pub fn record_query_result(&mut self, rows_read: i64, rows_written: i64, duration_ms: f64) {
        self.session_rows_read += rows_read.max(0) as u64;
        self.session_rows_written += rows_written.max(0) as u64;
        self.session_total_duration_ms += duration_ms;
        self.session_query_count += 1;
    }

    pub fn requests_in_last_5min(&self) -> usize {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let cutoff = now.saturating_sub(300);
        self.requests_last_5min.iter().filter(|&&t| t > cutoff).count()
    }

    pub fn session_duration_secs(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.session_start)
    }

    pub fn average_query_duration_ms(&self) -> f64 {
        if self.session_query_count == 0 {
            0.0
        } else {
            self.session_total_duration_ms / self.session_query_count as f64
        }
    }
}

/// Query metrics collected from API responses
#[derive(Debug, Clone, Default)]
pub struct QueryMetrics {
    pub rows_read: i64,
    pub rows_written: i64,
    pub duration_ms: f64,
    pub rate_limit: ApiRateLimitInfo,
}

#[derive(Debug)]
enum Message {
    TablesLoaded(usize, Result<Vec<String>, String>, Option<QueryMetrics>),
    DataLoaded(usize, Result<(Vec<String>, Vec<ColumnInfo>, Vec<Vec<serde_json::Value>>, i64), String>, Option<QueryMetrics>),
    QueryExecuted(usize, Result<String, String>, Option<QueryMetrics>),
    RowDeleted(usize, Result<(), String>, Option<QueryMetrics>),
    ConnectionTestResult(Result<String, String>, Option<QueryMetrics>),
    SchemaLoaded(usize, Result<Vec<ColumnInfo>, String>, Option<QueryMetrics>),
    RowInserted(usize, Result<(), String>, Option<QueryMetrics>),
    CellUpdated(usize, Result<(), String>, Option<QueryMetrics>),
    SchemaDiffResult(Option<SchemaDiff>, DatabaseSchema, Vec<String>),
    UpdateCheckResult(Result<crate::version::UpdateInfo, String>),
    // Onboarding wizard messages
    AccountsLoaded(Result<Vec<CloudflareAccount>, String>),
    DatabasesListLoaded(Result<Vec<CloudflareDatabase>, String>),
    OnboardingTestResult(Result<String, String>),
    LocalDatabasesScanned(Vec<crate::local_db::LocalD1Database>),
}

pub struct D1ManagerApp {
    tabs: Vec<ConnectionTab>,
    active_tab: usize,
    profile_metadata: Vec<ProfileMetadata>,
    editing_profile: Option<EditingProfile>,
    editing_index: Option<usize>,
    rows_per_page: i64,
    show_settings: bool,
    show_profile_editor: bool,
    show_export_dialog: bool,
    export_format: ExportFormat,
    export_message: Option<(bool, String)>,
    // Import state
    show_import_dialog: bool,
    import_format: ImportFormat,
    import_conflict_strategy: ConflictStrategy,
    import_preview: Option<ImportData>,
    import_sql_statements: Option<Vec<String>>,
    import_filename: String,
    import_message: Option<(bool, String)>,
    import_loading: bool,
    // MySQL dump conversion result
    import_mysql_result: Option<export::MySqlConversionResult>,
    keychain_error: Option<String>,
    // Connection test state
    connection_test_result: Option<(bool, String)>,
    connection_testing: bool,
    theme_initialized: bool,
    // Cell editing state
    editing_cell: Option<EditingCell>,
    // Row editor state (for new/duplicate rows)
    show_row_editor: bool,
    row_editor: Option<RowEditorState>,
    row_editor_loading: bool,
    // Confirmation dialog state
    show_confirmation_dialog: bool,
    pending_operation: Option<PendingOperation>,
    // Query history
    query_history: Vec<QueryHistoryEntry>,
    show_history_panel: bool,
    history_search_filter: String,
    // SQL Snippets
    show_snippets_panel: bool,
    // Metadata cache
    metadata_cache: std::collections::HashMap<String, CachedMetadata>,
    // Rate limit info
    rate_limit_info: RateLimitInfo,
    // Usage metrics
    usage_metrics: UsageMetrics,
    show_metrics_panel: bool,
    // Filter editing state
    show_filter_section: bool,
    filter_edit_column: String,
    filter_edit_operator: FilterOperator,
    filter_edit_value: String,
    // Internationalization
    i18n: I18n,
    // Tutorial
    show_tutorial: bool,
    // Settings export/import
    show_settings_export_dialog: bool,
    show_settings_import_dialog: bool,
    settings_export_include_tokens: bool,
    settings_export_password: String,
    settings_export_password_confirm: String,
    settings_import_password: String,
    settings_import_content: Option<String>,
    settings_import_merge: bool,
    settings_message: Option<(bool, String)>,
    // SQL Safety features
    execution_log: Vec<ExecutionLogEntry>,
    show_execution_log_panel: bool,
    production_locks: std::collections::HashMap<String, ProductionLockState>,
    show_dangerous_query_dialog: bool,
    dangerous_query_analysis: Option<SafetyAnalysis>,
    dangerous_query_sql: String,
    dangerous_query_confirm_text: String,
    // Audit Journal (pseudo time-travel)
    audit_journal: AuditJournal,
    show_audit_panel: bool,
    audit_filter_table: String,
    audit_selected_record: Option<u64>,
    // Schema Explorer
    show_schema_explorer: bool,
    schema_graph: SchemaGraph,
    // Schema Diff (DB Comparison)
    show_schema_diff_panel: bool,
    schema_diff_source_idx: Option<usize>,
    schema_diff_target_idx: Option<usize>,
    schema_diff_result: Option<SchemaDiff>,
    schema_diff_source_schema: Option<DatabaseSchema>,
    schema_diff_migration_sql: Vec<String>,
    schema_diff_loading: bool,
    // AI SQL Suggestions
    show_ai_suggest_panel: bool,
    ai_suggest_show_unsafe: bool,
    ai_suggest_category_filter: Option<crate::ai_suggest::SuggestionCategory>,
    // Version and Update
    show_about_dialog: bool,
    update_check_in_progress: bool,
    update_info: Option<Result<crate::version::UpdateInfo, String>>,
    // Onboarding wizard
    show_onboarding_wizard: bool,
    onboarding_state: OnboardingWizardState,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    runtime: tokio::runtime::Runtime,
}

impl D1ManagerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = channel();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mut app = Self {
            tabs: vec![],
            active_tab: 0,
            profile_metadata: vec![],
            editing_profile: None,
            editing_index: None,
            rows_per_page: 50,
            show_settings: false,
            show_profile_editor: false,
            show_export_dialog: false,
            export_format: ExportFormat::Csv,
            export_message: None,
            show_import_dialog: false,
            import_format: ImportFormat::Csv,
            import_conflict_strategy: ConflictStrategy::Fail,
            import_preview: None,
            import_sql_statements: None,
            import_filename: String::new(),
            import_message: None,
            import_loading: false,
            import_mysql_result: None,
            keychain_error: None,
            connection_test_result: None,
            connection_testing: false,
            theme_initialized: false,
            editing_cell: None,
            show_row_editor: false,
            row_editor: None,
            row_editor_loading: false,
            show_confirmation_dialog: false,
            pending_operation: None,
            query_history: vec![],
            show_history_panel: false,
            history_search_filter: String::new(),
            show_snippets_panel: false,
            metadata_cache: std::collections::HashMap::new(),
            rate_limit_info: RateLimitInfo::default(),
            usage_metrics: UsageMetrics::new(),
            show_metrics_panel: false,
            show_filter_section: false,
            filter_edit_column: String::new(),
            filter_edit_operator: FilterOperator::Equals,
            filter_edit_value: String::new(),
            i18n: I18n::default(),
            show_tutorial: false,
            show_settings_export_dialog: false,
            show_settings_import_dialog: false,
            settings_export_include_tokens: false,
            settings_export_password: String::new(),
            settings_export_password_confirm: String::new(),
            settings_import_password: String::new(),
            settings_import_content: None,
            settings_import_merge: true,
            settings_message: None,
            execution_log: vec![],
            show_execution_log_panel: false,
            production_locks: std::collections::HashMap::new(),
            show_dangerous_query_dialog: false,
            dangerous_query_analysis: None,
            dangerous_query_sql: String::new(),
            dangerous_query_confirm_text: String::new(),
            audit_journal: AuditJournal::load().unwrap_or_else(|_| AuditJournal::new()),
            show_audit_panel: false,
            audit_filter_table: String::new(),
            audit_selected_record: None,
            show_schema_explorer: false,
            schema_graph: SchemaGraph::new(),
            show_schema_diff_panel: false,
            schema_diff_source_idx: None,
            schema_diff_target_idx: None,
            schema_diff_result: None,
            schema_diff_source_schema: None,
            schema_diff_migration_sql: vec![],
            schema_diff_loading: false,
            show_ai_suggest_panel: false,
            ai_suggest_show_unsafe: false,
            ai_suggest_category_filter: None,
            show_about_dialog: false,
            update_check_in_progress: false,
            update_info: None,
            show_onboarding_wizard: false,
            onboarding_state: OnboardingWizardState::default(),
            sender,
            receiver,
            runtime,
        };

        app.load_profile_metadata();
        app.load_query_history();
        app.load_metadata_cache();
        app.load_language_setting();
        app.load_execution_log();

        // Show onboarding wizard for first-time users
        if app.profile_metadata.is_empty() {
            app.show_onboarding_wizard = true;
        }

        app
    }

    fn load_profile_metadata(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("d1-manager").join("profiles.json");
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(metadata) = serde_json::from_str::<Vec<ProfileMetadata>>(&content) {
                    self.profile_metadata = metadata;
                }
            }
        }
    }

    fn save_profile_metadata(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("d1-manager");
            let _ = std::fs::create_dir_all(&config_path);
            if let Ok(json) = serde_json::to_string_pretty(&self.profile_metadata) {
                let _ = std::fs::write(config_path.join("profiles.json"), json);
            }
        }
    }

    fn load_query_history(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let history_path = config_dir.join("d1-manager").join("query_history.json");
            if let Ok(content) = std::fs::read_to_string(&history_path) {
                if let Ok(history) = serde_json::from_str::<Vec<QueryHistoryEntry>>(&content) {
                    self.query_history = history;
                }
            }
        }
    }

    fn save_query_history(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("d1-manager");
            let _ = std::fs::create_dir_all(&config_path);
            // Keep only last 500 entries
            let history: Vec<_> = self.query_history.iter().rev().take(500).cloned().collect();
            if let Ok(json) = serde_json::to_string_pretty(&history) {
                let _ = std::fs::write(config_path.join("query_history.json"), json);
            }
        }
    }

    fn add_query_to_history(&mut self, database_id: &str, database_name: &str, sql: &str, success: bool, rows_affected: Option<i64>, duration_ms: Option<f64>) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.query_history.push(QueryHistoryEntry {
            timestamp,
            database_id: database_id.to_string(),
            database_name: database_name.to_string(),
            sql: sql.to_string(),
            success,
            rows_affected,
            duration_ms,
        });

        self.save_query_history();
    }

    fn load_metadata_cache(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let cache_path = config_dir.join("d1-manager").join("metadata_cache.json");
            if let Ok(content) = std::fs::read_to_string(&cache_path) {
                if let Ok(cache) = serde_json::from_str::<std::collections::HashMap<String, CachedMetadata>>(&content) {
                    self.metadata_cache = cache;
                }
            }
        }
    }

    fn save_metadata_cache(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("d1-manager");
            let _ = std::fs::create_dir_all(&config_path);
            if let Ok(json) = serde_json::to_string_pretty(&self.metadata_cache) {
                let _ = std::fs::write(config_path.join("metadata_cache.json"), json);
            }
        }
    }

    fn update_metadata_cache(&mut self, database_id: &str, tables: &[String]) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = self.metadata_cache.entry(database_id.to_string()).or_insert_with(|| CachedMetadata {
            tables: vec![],
            schemas: std::collections::HashMap::new(),
            last_updated: 0,
        });
        entry.tables = tables.to_vec();
        entry.last_updated = timestamp;
        self.save_metadata_cache();
    }

    #[allow(dead_code)]
    fn update_schema_cache(&mut self, database_id: &str, table: &str, columns: &[ColumnInfo]) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = self.metadata_cache.entry(database_id.to_string()).or_insert_with(|| CachedMetadata {
            tables: vec![],
            schemas: std::collections::HashMap::new(),
            last_updated: 0,
        });
        entry.schemas.insert(table.to_string(), columns.iter().map(CachedColumnInfo::from).collect());
        entry.last_updated = timestamp;
        self.save_metadata_cache();
    }

    #[allow(dead_code)]
    fn get_cached_tables(&self, database_id: &str) -> Option<&Vec<String>> {
        self.metadata_cache.get(database_id).map(|c| &c.tables)
    }

    #[allow(dead_code)]
    fn get_cached_schema(&self, database_id: &str, table: &str) -> Option<Vec<ColumnInfo>> {
        self.metadata_cache
            .get(database_id)
            .and_then(|c| c.schemas.get(table))
            .map(|cols| cols.iter().map(ColumnInfo::from).collect())
    }

    fn get_cache_age(&self, database_id: &str) -> Option<u64> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.metadata_cache.get(database_id).map(|c| now.saturating_sub(c.last_updated))
    }

    fn load_language_setting(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let lang_path = config_dir.join("d1-manager").join("language.json");
            if let Ok(content) = std::fs::read_to_string(&lang_path) {
                if let Ok(lang) = serde_json::from_str::<Language>(&content) {
                    self.i18n = I18n::new(lang);
                }
            }
        }
    }

    fn save_language_setting(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("d1-manager");
            let _ = std::fs::create_dir_all(&config_path);
            if let Ok(json) = serde_json::to_string(&self.i18n.lang) {
                let _ = std::fs::write(config_path.join("language.json"), json);
            }
        }
    }

    fn load_execution_log(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let log_path = config_dir.join("d1-manager").join("execution_log.json");
            if let Ok(content) = std::fs::read_to_string(&log_path) {
                if let Ok(log) = serde_json::from_str::<Vec<ExecutionLogEntry>>(&content) {
                    self.execution_log = log;
                }
            }
        }
    }

    fn save_execution_log(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("d1-manager");
            let _ = std::fs::create_dir_all(&config_path);
            // Keep only last 1000 entries
            let log: Vec<_> = self.execution_log.iter().rev().take(1000).cloned().collect();
            if let Ok(json) = serde_json::to_string_pretty(&log) {
                let _ = std::fs::write(config_path.join("execution_log.json"), json);
            }
        }
    }

    fn add_execution_log(&mut self, entry: ExecutionLogEntry) {
        self.execution_log.push(entry);
        self.save_execution_log();
    }

    /// Check if production environment is currently unlocked
    fn is_production_unlocked(&self, database_id: &str) -> bool {
        self.production_locks
            .get(database_id)
            .map(|lock| lock.is_unlocked())
            .unwrap_or(false)
    }

    /// Unlock production for 5 minutes
    fn unlock_production(&mut self, database_id: &str) {
        let lock = self.production_locks
            .entry(database_id.to_string())
            .or_insert_with(ProductionLockState::default);
        lock.unlock_for_duration(300); // 5 minutes
    }

    /// Lock production immediately
    fn lock_production(&mut self, database_id: &str) {
        if let Some(lock) = self.production_locks.get_mut(database_id) {
            lock.lock();
        }
    }

    /// Get remaining unlock time for production
    fn get_production_unlock_remaining(&self, database_id: &str) -> Option<u64> {
        self.production_locks
            .get(database_id)
            .filter(|lock| lock.is_unlocked())
            .map(|lock| lock.remaining_seconds())
    }

    /// Analyze SQL and determine if dangerous query dialog should be shown
    fn check_query_safety(&mut self, sql: &str) -> bool {
        let analysis = sql_safety::analyze_query(sql);

        // For production environment, require unlock for any write
        if self.active_tab < self.tabs.len() {
            let tab = &self.tabs[self.active_tab];
            let is_production = tab.profile.environment == EnvironmentType::Production;
            let is_write = analysis.query_type.is_write();

            if is_production && is_write && !self.is_production_unlocked(&tab.profile.database_id) {
                // Show dangerous query dialog for production writes
                self.dangerous_query_analysis = Some(analysis);
                self.dangerous_query_sql = sql.to_string();
                self.dangerous_query_confirm_text.clear();
                self.show_dangerous_query_dialog = true;
                return false;
            }
        }

        // For high/critical risk, show confirmation
        if analysis.risk_level.requires_confirmation() {
            self.dangerous_query_analysis = Some(analysis);
            self.dangerous_query_sql = sql.to_string();
            self.dangerous_query_confirm_text.clear();
            self.show_dangerous_query_dialog = true;
            return false;
        }

        true // Safe to execute
    }

    /// Check if write operation should require confirmation
    fn should_confirm_write(&self) -> bool {
        if self.active_tab >= self.tabs.len() {
            return false;
        }
        self.tabs[self.active_tab].profile.requires_confirmation()
    }

    /// Check if current tab is read-only
    fn is_read_only(&self) -> bool {
        if self.active_tab >= self.tabs.len() {
            return false;
        }
        !self.tabs[self.active_tab].profile.can_write()
    }

    fn save_profile_to_keychain(&mut self, profile: &ConnectionProfile) {
        if let Err(e) = secure_storage::store_token(&profile.id, profile.api_token.as_str()) {
            self.keychain_error = Some(self.i18n.keychain_error(&e));
        } else {
            self.keychain_error = None;
        }
    }

    fn delete_profile_from_keychain(&mut self, profile_id: &str) {
        if let Err(e) = secure_storage::delete_token(profile_id) {
            self.keychain_error = Some(self.i18n.keychain_error(&e));
        }
    }

    fn open_connection(&mut self, meta: &ProfileMetadata) {
        for (i, tab) in self.tabs.iter().enumerate() {
            if tab.profile.id == meta.id {
                self.active_tab = i;
                return;
            }
        }

        let profile = ConnectionProfile::from_metadata(meta);

        // Only check API token for remote connections
        if profile.connection_type == ConnectionType::Remote && profile.api_token.is_empty() {
            self.keychain_error = Some(self.i18n.api_token_not_found().to_string());
            return;
        }

        let tab = ConnectionTab::new(profile);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.connect_tab(self.active_tab);
    }

    fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    fn connect_tab(&mut self, tab_index: usize) {
        if tab_index >= self.tabs.len() {
            return;
        }

        let tab = &mut self.tabs[tab_index];
        tab.loading = true;
        tab.status_message = "Connecting...".to_string();

        // Check if this is a local or remote connection
        if tab.profile.connection_type == ConnectionType::Local {
            // Local SQLite connection - synchronous
            if let Some(ref local_path) = tab.profile.local_path {
                let path = std::path::PathBuf::from(local_path);
                let local_client = crate::local_db::LocalD1Client::new(path);

                match local_client.get_tables() {
                    Ok(tables) => {
                        let _ = self.sender.send(Message::TablesLoaded(tab_index, Ok(tables), None));
                    }
                    Err(e) => {
                        let error_msg = self.i18n.translate_local_db_error(&e);
                        let _ = self.sender.send(Message::TablesLoaded(tab_index, Err(error_msg), None));
                    }
                }
            } else {
                let error_msg = self.i18n.local_db_path_not_configured().to_string();
                let _ = self.sender.send(Message::TablesLoaded(
                    tab_index,
                    Err(error_msg),
                    None
                ));
            }
        } else {
            // Remote D1 connection - asynchronous
            let client = D1Client::new(
                tab.profile.account_id.clone(),
                tab.profile.database_id.clone(),
                tab.profile.api_token.as_str().to_string(),
            );

            let shared_client = tab.client.clone();
            let sender = self.sender.clone();

            self.runtime.spawn(async move {
                {
                    let mut lock = shared_client.lock().await;
                    *lock = Some(client.clone());
                }

                match client.get_tables_with_metrics().await {
                    Ok((tables, rate_limit, meta)) => {
                        let metrics = Some(QueryMetrics {
                            rows_read: meta.as_ref().map(|m| m.rows_read_or_default()).unwrap_or(0),
                            rows_written: meta.as_ref().map(|m| m.rows_written_or_default()).unwrap_or(0),
                            duration_ms: meta.as_ref().map(|m| m.duration_ms_or_default()).unwrap_or(0.0),
                            rate_limit,
                        });
                        let _ = sender.send(Message::TablesLoaded(tab_index, Ok(tables), metrics));
                    }
                    Err(e) => {
                        let _ = sender.send(Message::TablesLoaded(tab_index, Err(e), None));
                    }
                }
            });
        }
    }

    fn load_table_data(&mut self, tab_index: usize, table: &str) {
        if tab_index >= self.tabs.len() {
            return;
        }

        let tab = &mut self.tabs[tab_index];
        let table = table.to_string();
        let limit = self.rows_per_page;
        let offset = tab.current_page * self.rows_per_page;

        tab.loading = true;

        // Check if this is a local or remote connection
        if tab.profile.connection_type == ConnectionType::Local {
            // Local SQLite connection - synchronous
            if let Some(ref local_path) = tab.profile.local_path {
                let path = std::path::PathBuf::from(local_path);
                let local_client = crate::local_db::LocalD1Client::new(path);

                let count_result = local_client.get_row_count(&table);
                let data_result = local_client.get_table_data(&table, limit, offset);
                let schema_result = local_client.get_table_schema(&table);

                match (data_result, count_result, schema_result) {
                    (Ok((cols, rows)), Ok(count), Ok(schema)) => {
                        // Convert LocalColumnInfo to ColumnInfo
                        let column_info: Vec<ColumnInfo> = schema.iter().map(|c| ColumnInfo {
                            name: c.name.clone(),
                            col_type: c.col_type.clone(),
                            pk: c.pk,
                            notnull: c.notnull,
                            foreign_key: None,
                        }).collect();
                        let _ = self.sender.send(Message::DataLoaded(tab_index, Ok((cols, column_info, rows, count)), None));
                    }
                    (Ok((cols, rows)), Ok(count), Err(_)) => {
                        let _ = self.sender.send(Message::DataLoaded(tab_index, Ok((cols, vec![], rows, count)), None));
                    }
                    (Err(e), _, _) | (_, Err(e), _) => {
                        let error_msg = self.i18n.translate_local_db_error(&e);
                        let _ = self.sender.send(Message::DataLoaded(tab_index, Err(error_msg), None));
                    }
                }
            }
        } else {
            // Remote D1 connection - asynchronous
            let client = tab.client.clone();
            let sender = self.sender.clone();
            let filter = tab.filter.clone();

            self.runtime.spawn(async move {
                let lock = client.lock().await;
                if let Some(ref client) = *lock {
                    // Build filter tuples from conditions
                    let filter_tuples: Vec<(String, String, String)> = filter
                        .map(|f| {
                            f.conditions.iter().map(|c| {
                                let op = match c.operator {
                                    FilterOperator::Equals => "=".to_string(),
                                    FilterOperator::NotEquals => "!=".to_string(),
                                    FilterOperator::GreaterThan => ">".to_string(),
                                    FilterOperator::LessThan => "<".to_string(),
                                    FilterOperator::GreaterOrEqual => ">=".to_string(),
                                    FilterOperator::LessOrEqual => "<=".to_string(),
                                    FilterOperator::Like => "LIKE".to_string(),
                                    FilterOperator::In => "IN".to_string(),
                                    FilterOperator::IsNull => "IS NULL".to_string(),
                                    FilterOperator::IsNotNull => "IS NOT NULL".to_string(),
                                };
                                (c.column.clone(), op, c.value.clone())
                            }).collect()
                        })
                        .unwrap_or_default();

                    let filters: Vec<(&str, &str, &str)> = filter_tuples
                        .iter()
                        .map(|(c, o, v)| (c.as_str(), o.as_str(), v.as_str()))
                        .collect();

                    let count_result = client.get_row_count_with_filters(&table, &filters).await;
                    let data_result = client.get_table_data_with_filters(&table, limit, offset, &filters).await;
                    let schema_result = client.get_table_schema(&table).await;

                    match (data_result, count_result, schema_result) {
                        (Ok((cols, rows)), Ok(count), Ok(schema)) => {
                            // Note: Metrics from multiple queries are not aggregated here for simplicity
                            let _ = sender.send(Message::DataLoaded(tab_index, Ok((cols, schema, rows, count)), None));
                        }
                        (Ok((cols, rows)), Ok(count), Err(_)) => {
                            // Schema fetch failed, continue with empty column info
                            let _ = sender.send(Message::DataLoaded(tab_index, Ok((cols, vec![], rows, count)), None));
                        }
                        (Err(e), _, _) | (_, Err(e), _) => {
                            let _ = sender.send(Message::DataLoaded(tab_index, Err(e), None));
                        }
                    }
                }
            });
        }
    }

    fn execute_query(&mut self, tab_index: usize) {
        if tab_index >= self.tabs.len() {
            return;
        }

        let tab = &mut self.tabs[tab_index];
        let sql = tab.sql_query.clone();

        tab.loading = true;

        // Check if this is a local or remote connection
        if tab.profile.connection_type == ConnectionType::Local {
            // Local SQLite connection - synchronous
            if let Some(ref local_path) = tab.profile.local_path {
                let path = std::path::PathBuf::from(local_path);
                let local_client = crate::local_db::LocalD1Client::new(path);

                match local_client.execute(&sql, vec![]) {
                    Ok(result) => {
                        // Format result as JSON-like output
                        let output = if !result.columns.is_empty() {
                            let mut json_rows: Vec<serde_json::Value> = Vec::new();
                            for row in &result.rows {
                                let mut obj = serde_json::Map::new();
                                for (i, col) in result.columns.iter().enumerate() {
                                    if let Some(val) = row.get(i) {
                                        obj.insert(col.clone(), val.clone());
                                    }
                                }
                                json_rows.push(serde_json::Value::Object(obj));
                            }
                            serde_json::to_string_pretty(&json_rows).unwrap_or_default()
                        } else {
                            format!("{} {}", self.i18n.query_success(), self.i18n.rows_affected(result.changes))
                        };
                        let _ = self.sender.send(Message::QueryExecuted(tab_index, Ok(output), None));
                    }
                    Err(e) => {
                        let error_msg = self.i18n.translate_local_db_error(&e);
                        let _ = self.sender.send(Message::QueryExecuted(tab_index, Err(error_msg), None));
                    }
                }
            }
        } else {
            // Remote D1 connection - asynchronous
            let client = tab.client.clone();
            let sender = self.sender.clone();

            self.runtime.spawn(async move {
                let lock = client.lock().await;
                if let Some(ref client) = *lock {
                    match client.execute_with_metrics(&sql, vec![]).await {
                        Ok((response, rate_limit, meta)) => {
                            let result = serde_json::to_string_pretty(&response).unwrap_or_default();
                            let metrics = Some(QueryMetrics {
                                rows_read: meta.as_ref().map(|m| m.rows_read_or_default()).unwrap_or(0),
                                rows_written: meta.as_ref().map(|m| m.rows_written_or_default()).unwrap_or(0),
                                duration_ms: meta.as_ref().map(|m| m.duration_ms_or_default()).unwrap_or(0.0),
                                rate_limit,
                            });
                            let _ = sender.send(Message::QueryExecuted(tab_index, Ok(result), metrics));
                        }
                        Err(e) => {
                            let _ = sender.send(Message::QueryExecuted(tab_index, Err(e), None));
                        }
                    }
                }
            });
        }
    }

    fn test_connection(&mut self, account_id: String, database_id: String, api_token: String) {
        self.connection_testing = true;
        self.connection_test_result = None;
        let sender = self.sender.clone();

        self.runtime.spawn(async move {
            let client = D1Client::new(account_id, database_id, api_token);
            match client.execute_with_metrics("SELECT 1 as test", vec![]).await {
                Ok((response, rate_limit, meta)) => {
                    if response.success {
                        let metrics = Some(QueryMetrics {
                            rows_read: meta.as_ref().map(|m| m.rows_read_or_default()).unwrap_or(0),
                            rows_written: meta.as_ref().map(|m| m.rows_written_or_default()).unwrap_or(0),
                            duration_ms: meta.as_ref().map(|m| m.duration_ms_or_default()).unwrap_or(0.0),
                            rate_limit,
                        });
                        let _ = sender.send(Message::ConnectionTestResult(Ok("Connection successful".to_string()), metrics));
                    } else {
                        let error_msg = response
                            .errors
                            .first()
                            .map(|e| e.message.clone())
                            .unwrap_or_else(|| "Unknown error".to_string());
                        let _ = sender.send(Message::ConnectionTestResult(Err(error_msg), None));
                    }
                }
                Err(e) => {
                    let _ = sender.send(Message::ConnectionTestResult(Err(e), None));
                }
            }
        });
    }

    fn delete_row(&mut self, tab_index: usize, id: i64) {
        if tab_index >= self.tabs.len() {
            return;
        }

        let tab = &self.tabs[tab_index];
        if let Some(ref table) = tab.selected_table.clone() {
            let client = tab.client.clone();
            let sender = self.sender.clone();
            let table = table.clone();

            self.runtime.spawn(async move {
                let lock = client.lock().await;
                if let Some(ref client) = *lock {
                    let sql = format!("DELETE FROM {} WHERE id = ?", table);
                    match client.execute_with_metrics(&sql, vec![serde_json::json!(id)]).await {
                        Ok((response, rate_limit, meta)) => {
                            if response.success {
                                let metrics = Some(QueryMetrics {
                                    rows_read: meta.as_ref().map(|m| m.rows_read_or_default()).unwrap_or(0),
                                    rows_written: meta.as_ref().map(|m| m.rows_written_or_default()).unwrap_or(0),
                                    duration_ms: meta.as_ref().map(|m| m.duration_ms_or_default()).unwrap_or(0.0),
                                    rate_limit,
                                });
                                let _ = sender.send(Message::RowDeleted(tab_index, Ok(()), metrics));
                            } else {
                                let error_msg = response
                                    .errors
                                    .first()
                                    .map(|e| e.message.clone())
                                    .unwrap_or_else(|| "Unknown error".to_string());
                                let _ = sender.send(Message::RowDeleted(tab_index, Err(error_msg), None));
                            }
                        }
                        Err(e) => {
                            let _ = sender.send(Message::RowDeleted(tab_index, Err(e), None));
                        }
                    }
                }
            });
        }
    }

    fn load_table_schema(&mut self, tab_index: usize) {
        if tab_index >= self.tabs.len() {
            return;
        }

        let tab = &self.tabs[tab_index];
        if let Some(ref table) = tab.selected_table.clone() {
            let client = tab.client.clone();
            let sender = self.sender.clone();
            let table = table.clone();

            self.runtime.spawn(async move {
                let lock = client.lock().await;
                if let Some(ref client) = *lock {
                    let result = client.get_table_schema(&table).await;
                    // Schema loading doesn't track metrics for simplicity
                    let _ = sender.send(Message::SchemaLoaded(tab_index, result, None));
                }
            });
        }
    }

    fn insert_row(&mut self, tab_index: usize, columns: Vec<String>, values: Vec<serde_json::Value>) {
        if tab_index >= self.tabs.len() {
            return;
        }

        let tab = &self.tabs[tab_index];
        if let Some(ref table) = tab.selected_table.clone() {
            let client = tab.client.clone();
            let sender = self.sender.clone();
            let table = table.clone();

            self.row_editor_loading = true;

            self.runtime.spawn(async move {
                let lock = client.lock().await;
                if let Some(ref client) = *lock {
                    let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
                    let sql = format!(
                        "INSERT INTO {} ({}) VALUES ({})",
                        table,
                        columns.join(", "),
                        placeholders.join(", ")
                    );
                    match client.execute_with_metrics(&sql, values).await {
                        Ok((response, rate_limit, meta)) => {
                            if response.success {
                                let metrics = Some(QueryMetrics {
                                    rows_read: meta.as_ref().map(|m| m.rows_read_or_default()).unwrap_or(0),
                                    rows_written: meta.as_ref().map(|m| m.rows_written_or_default()).unwrap_or(0),
                                    duration_ms: meta.as_ref().map(|m| m.duration_ms_or_default()).unwrap_or(0.0),
                                    rate_limit,
                                });
                                let _ = sender.send(Message::RowInserted(tab_index, Ok(()), metrics));
                            } else {
                                let error_msg = response
                                    .errors
                                    .first()
                                    .map(|e| e.message.clone())
                                    .unwrap_or_else(|| "Unknown error".to_string());
                                let _ = sender.send(Message::RowInserted(tab_index, Err(error_msg), None));
                            }
                        }
                        Err(e) => {
                            let _ = sender.send(Message::RowInserted(tab_index, Err(e), None));
                        }
                    }
                }
            });
        }
    }

    fn update_cell(&mut self, tab_index: usize, pk_column: String, pk_value: serde_json::Value, column: String, value: serde_json::Value) {
        if tab_index >= self.tabs.len() {
            return;
        }

        let tab = &self.tabs[tab_index];
        if let Some(ref table) = tab.selected_table.clone() {
            let client = tab.client.clone();
            let sender = self.sender.clone();
            let table = table.clone();

            self.runtime.spawn(async move {
                let lock = client.lock().await;
                if let Some(ref client) = *lock {
                    let sql = format!("UPDATE {} SET {} = ? WHERE {} = ?", table, column, pk_column);
                    match client.execute_with_metrics(&sql, vec![value.clone(), pk_value.clone()]).await {
                        Ok((response, rate_limit, meta)) => {
                            if response.success {
                                let metrics = Some(QueryMetrics {
                                    rows_read: meta.as_ref().map(|m| m.rows_read_or_default()).unwrap_or(0),
                                    rows_written: meta.as_ref().map(|m| m.rows_written_or_default()).unwrap_or(0),
                                    duration_ms: meta.as_ref().map(|m| m.duration_ms_or_default()).unwrap_or(0.0),
                                    rate_limit,
                                });
                                let _ = sender.send(Message::CellUpdated(tab_index, Ok(()), metrics));
                            } else {
                                let error_msg = response
                                    .errors
                                    .first()
                                    .map(|e| e.message.clone())
                                    .unwrap_or_else(|| "Unknown error".to_string());
                                let _ = sender.send(Message::CellUpdated(tab_index, Err(error_msg), None));
                            }
                        }
                        Err(e) => {
                            let _ = sender.send(Message::CellUpdated(tab_index, Err(e), None));
                        }
                    }
                }
            });
        }
    }

    /// Update usage metrics from query results
    fn update_metrics(&mut self, metrics: Option<QueryMetrics>) {
        if let Some(m) = metrics {
            self.usage_metrics.record_request();
            self.usage_metrics.record_query_result(m.rows_read, m.rows_written, m.duration_ms);
            self.rate_limit_info.update(m.rate_limit);
        }
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                Message::TablesLoaded(tab_index, result, metrics) => {
                    self.update_metrics(metrics);
                    if tab_index < self.tabs.len() {
                        self.tabs[tab_index].loading = false;
                        match result {
                            Ok(tables) => {
                                // Update metadata cache
                                let db_id = self.tabs[tab_index].profile.database_id.clone();
                                self.update_metadata_cache(&db_id, &tables);

                                self.tabs[tab_index].tables = tables;
                                self.tabs[tab_index].connected = true;
                                self.tabs[tab_index].status_message = "Connected".to_string();
                            }
                            Err(e) => {
                                self.tabs[tab_index].status_message = format!("Connection failed: {}", e);
                                self.tabs[tab_index].connected = false;
                            }
                        }
                    }
                }
                Message::DataLoaded(tab_index, result, metrics) => {
                    self.update_metrics(metrics);
                    if tab_index < self.tabs.len() {
                        let tab = &mut self.tabs[tab_index];
                        tab.loading = false;
                        match result {
                            Ok((cols, col_info, rows, count)) => {
                                tab.columns = cols;
                                tab.column_info = col_info;
                                tab.rows = rows;
                                tab.row_count = count;
                                tab.status_message = format!("{} rows total", count);
                            }
                            Err(e) => {
                                tab.status_message = format!("Error: {}", e);
                            }
                        }
                    }
                }
                Message::QueryExecuted(tab_index, result, metrics) => {
                    self.update_metrics(metrics.clone());
                    if tab_index < self.tabs.len() {
                        self.tabs[tab_index].loading = false;

                        // Record query in history
                        let tab = &self.tabs[tab_index];
                        let db_id = tab.profile.database_id.clone();
                        let db_name = tab.profile.name.clone();
                        let sql = tab.sql_query.clone();
                        let environment = tab.profile.environment;

                        // Analyze query for execution log
                        let analysis = sql_safety::analyze_query(&sql);

                        // Handle import completion
                        let should_refresh = if self.import_loading {
                            self.import_loading = false;
                            match &result {
                                Ok(output) => {
                                    self.import_message = Some((true, output.clone()));
                                    self.tabs[tab_index].selected_table.clone()
                                }
                                Err(e) => {
                                    self.import_message = Some((false, e.clone()));
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        match result {
                            Ok(output) => {
                                self.tabs[tab_index].query_result = output.clone();
                                self.tabs[tab_index].status_message = "Query executed successfully".to_string();
                                // Add to history (success) with duration from metrics
                                let duration = metrics.as_ref().map(|m| m.duration_ms);
                                self.add_query_to_history(&db_id, &db_name, &sql, true, None, duration);

                                // Record in audit journal for mutations
                                self.record_audit_for_tab(tab_index, &sql, None, None);

                                // Add to execution log
                                let log_entry = ExecutionLogEntry {
                                    timestamp: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_secs())
                                        .unwrap_or(0),
                                    database_id: db_id.clone(),
                                    database_name: db_name.clone(),
                                    environment,
                                    sql: sql.clone(),
                                    query_type: analysis.query_type.label().to_string(),
                                    risk_level: analysis.risk_level.label().to_string(),
                                    success: true,
                                    rows_affected: metrics.as_ref().map(|m| m.rows_written as i64),
                                    duration_ms: duration,
                                    error: None,
                                    warnings: analysis.warnings.iter().map(|w| w.message.clone()).collect(),
                                };
                                self.add_execution_log(log_entry);
                            }
                            Err(e) => {
                                self.tabs[tab_index].query_result = format!("Error: {}", e);
                                self.tabs[tab_index].status_message = "Query failed".to_string();
                                // Add to history (failure)
                                self.add_query_to_history(&db_id, &db_name, &sql, false, None, None);

                                // Add to execution log (failure)
                                let log_entry = ExecutionLogEntry {
                                    timestamp: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_secs())
                                        .unwrap_or(0),
                                    database_id: db_id.clone(),
                                    database_name: db_name.clone(),
                                    environment,
                                    sql: sql.clone(),
                                    query_type: analysis.query_type.label().to_string(),
                                    risk_level: analysis.risk_level.label().to_string(),
                                    success: false,
                                    rows_affected: None,
                                    duration_ms: None,
                                    error: Some(e.clone()),
                                    warnings: analysis.warnings.iter().map(|w| w.message.clone()).collect(),
                                };
                                self.add_execution_log(log_entry);
                            }
                        }

                        // Refresh table data after import
                        if let Some(table) = should_refresh {
                            self.load_table_data(tab_index, &table);
                        }
                    }
                }
                Message::RowDeleted(tab_index, result, metrics) => {
                    self.update_metrics(metrics);
                    if tab_index < self.tabs.len() {
                        match result {
                            Ok(_) => {
                                let tab = &mut self.tabs[tab_index];
                                tab.status_message = "Row deleted".to_string();
                                if let Some(ref table) = tab.selected_table.clone() {
                                    self.load_table_data(tab_index, table);
                                }
                            }
                            Err(e) => {
                                self.tabs[tab_index].status_message = format!("Delete failed: {}", e);
                            }
                        }
                    }
                }
                Message::ConnectionTestResult(result, metrics) => {
                    self.update_metrics(metrics);
                    self.connection_testing = false;
                    match result {
                        Ok(msg) => {
                            self.connection_test_result = Some((true, msg));
                        }
                        Err(e) => {
                            self.connection_test_result = Some((false, e));
                        }
                    }
                }
                Message::SchemaLoaded(tab_index, result, metrics) => {
                    self.update_metrics(metrics);
                    if tab_index < self.tabs.len() {
                        match result {
                            Ok(columns) => {
                                let is_duplicate = self.row_editor.as_ref().map(|r| r.is_duplicate).unwrap_or(false);
                                let source_row = self.row_editor.as_ref().and_then(|r| r.source_row.clone());

                                let values: Vec<String> = if is_duplicate {
                                    if let Some(ref row) = source_row {
                                        columns.iter().enumerate().map(|(i, col)| {
                                            if col.pk {
                                                String::new() // Clear PK for duplicate
                                            } else {
                                                row.get(i).map(|v| match v {
                                                    serde_json::Value::Null => String::new(),
                                                    serde_json::Value::String(s) => s.clone(),
                                                    _ => v.to_string(),
                                                }).unwrap_or_default()
                                            }
                                        }).collect()
                                    } else {
                                        columns.iter().map(|_| String::new()).collect()
                                    }
                                } else {
                                    columns.iter().map(|_| String::new()).collect()
                                };

                                self.row_editor = Some(RowEditorState {
                                    values,
                                    columns,
                                    is_duplicate,
                                    source_row,
                                });
                                self.show_row_editor = true;
                            }
                            Err(e) => {
                                self.tabs[tab_index].status_message = format!("Failed to load schema: {}", e);
                            }
                        }
                    }
                }
                Message::RowInserted(tab_index, result, metrics) => {
                    self.update_metrics(metrics);
                    self.row_editor_loading = false;
                    if tab_index < self.tabs.len() {
                        match result {
                            Ok(_) => {
                                self.tabs[tab_index].status_message = "Row inserted".to_string();
                                self.show_row_editor = false;
                                self.row_editor = None;
                                if let Some(ref table) = self.tabs[tab_index].selected_table.clone() {
                                    self.load_table_data(tab_index, table);
                                }
                            }
                            Err(e) => {
                                self.tabs[tab_index].status_message = format!("Insert failed: {}", e);
                            }
                        }
                    }
                }
                Message::CellUpdated(tab_index, result, metrics) => {
                    self.update_metrics(metrics);
                    if tab_index < self.tabs.len() {
                        match result {
                            Ok(_) => {
                                self.tabs[tab_index].status_message = "Cell updated".to_string();
                                self.editing_cell = None;
                                if let Some(ref table) = self.tabs[tab_index].selected_table.clone() {
                                    self.load_table_data(tab_index, table);
                                }
                            }
                            Err(e) => {
                                self.tabs[tab_index].status_message = format!("Update failed: {}", e);
                            }
                        }
                    }
                }
                Message::SchemaDiffResult(diff, source_schema, migration_sql) => {
                    self.schema_diff_loading = false;
                    self.schema_diff_result = diff;
                    self.schema_diff_source_schema = Some(source_schema);
                    self.schema_diff_migration_sql = migration_sql;
                }
                Message::UpdateCheckResult(result) => {
                    self.update_check_in_progress = false;
                    self.update_info = Some(result);
                }
                // Onboarding wizard messages
                Message::AccountsLoaded(result) => {
                    self.onboarding_state.accounts_loading = false;
                    match result {
                        Ok(accounts) => {
                            self.onboarding_state.accounts = accounts.into_iter()
                                .map(|a| CloudflareAccount { id: a.id, name: a.name })
                                .collect();
                            self.onboarding_state.accounts_error = None;
                        }
                        Err(e) => {
                            self.onboarding_state.accounts_error = Some(e);
                        }
                    }
                }
                Message::DatabasesListLoaded(result) => {
                    self.onboarding_state.databases_loading = false;
                    match result {
                        Ok(databases) => {
                            self.onboarding_state.databases = databases.into_iter()
                                .map(|d| CloudflareDatabase {
                                    uuid: d.uuid,
                                    name: d.name,
                                    created_at: d.created_at,
                                })
                                .collect();
                            self.onboarding_state.databases_error = None;
                        }
                        Err(e) => {
                            self.onboarding_state.databases_error = Some(e);
                        }
                    }
                }
                Message::OnboardingTestResult(result) => {
                    self.onboarding_state.test_in_progress = false;
                    match result {
                        Ok(msg) => {
                            self.onboarding_state.test_result = Some((true, msg));
                        }
                        Err(e) => {
                            self.onboarding_state.test_result = Some((false, e));
                        }
                    }
                }
                Message::LocalDatabasesScanned(databases) => {
                    self.onboarding_state.local_db_scanning = false;
                    self.onboarding_state.local_databases = databases;
                }
            }
        }
    }

    fn render_profile_list(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new(self.i18n.connections()).size(24.0).strong().color(AppColors::TEXT_PRIMARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::primary_button(ui, self.i18n.new_connection()).clicked() {
                        self.editing_profile = Some(EditingProfile::new());
                        self.editing_index = None;
                        self.show_profile_editor = true;
                    }
                });
            });

            ui.add_space(Spacing::MD);

            // Error message
            if let Some(ref error) = self.keychain_error {
                egui::Frame::new()
                    .fill(AppColors::ERROR.gamma_multiply(0.2))
                    .corner_radius(Radius::MD)
                    .inner_margin(egui::Margin::same(Spacing::SM as i8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠").color(AppColors::ERROR));
                            ui.label(RichText::new(error).color(AppColors::ERROR));
                        });
                    });
                ui.add_space(Spacing::MD);
            }

            // Connection list
            if self.profile_metadata.is_empty() {
                ui.add_space(Spacing::XL);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(self.i18n.no_connections_yet()).size(16.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::SM);
                    ui.label(RichText::new(self.i18n.click_new_connection()).size(14.0).color(AppColors::TEXT_MUTED));
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut connect_index: Option<usize> = None;
                    let mut edit_index: Option<usize> = None;
                    let mut delete_index: Option<usize> = None;

                    for (i, meta) in self.profile_metadata.iter().enumerate() {
                        ui.add_space(Spacing::XS);
                        theme::card(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&meta.name).size(16.0).strong().color(AppColors::TEXT_PRIMARY));
                                        ui.add_space(Spacing::SM);
                                        // Environment badge
                                        let env_color = meta.environment.color();
                                        egui::Frame::new()
                                            .fill(env_color.gamma_multiply(0.2))
                                            .corner_radius(Radius::SM)
                                            .inner_margin(egui::Margin::symmetric(6, 2))
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(meta.environment.short_label()).size(10.0).color(env_color).strong());
                                            });
                                        // Read-only badge
                                        if meta.read_only {
                                            egui::Frame::new()
                                                .fill(AppColors::TEXT_MUTED.gamma_multiply(0.2))
                                                .corner_radius(Radius::SM)
                                                .inner_margin(egui::Margin::symmetric(6, 2))
                                                .show(ui, |ui| {
                                                    ui.label(RichText::new(self.i18n.read_only_badge()).size(10.0).color(AppColors::TEXT_MUTED).strong());
                                                });
                                        }
                                    });
                                    ui.add_space(Spacing::XS);
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(self.i18n.database()).size(12.0).color(AppColors::TEXT_MUTED));
                                        ui.label(RichText::new(&meta.database_id).size(12.0).color(AppColors::TEXT_SECONDARY).monospace());
                                    });
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if theme::secondary_button(ui, self.i18n.delete()).clicked() {
                                        delete_index = Some(i);
                                    }
                                    ui.add_space(Spacing::XS);
                                    if theme::secondary_button(ui, self.i18n.edit()).clicked() {
                                        edit_index = Some(i);
                                    }
                                    ui.add_space(Spacing::XS);
                                    if theme::primary_button(ui, self.i18n.connect()).clicked() {
                                        connect_index = Some(i);
                                    }
                                });
                            });
                        });
                    }

                    if let Some(i) = connect_index {
                        let meta = self.profile_metadata[i].clone();
                        self.open_connection(&meta);
                        if self.keychain_error.is_none() {
                            self.show_settings = false;
                        }
                    }

                    if let Some(i) = edit_index {
                        let meta = &self.profile_metadata[i];
                        let profile = ConnectionProfile::from_metadata(meta);
                        self.editing_profile = Some(EditingProfile::from_profile(&profile));
                        self.editing_index = Some(i);
                        self.show_profile_editor = true;
                    }

                    if let Some(i) = delete_index {
                        let profile_id = self.profile_metadata[i].id.clone();
                        self.delete_profile_from_keychain(&profile_id);
                        self.profile_metadata.remove(i);
                        self.save_profile_metadata();
                    }
                });
            }

            // Backup & Restore section
            ui.add_space(Spacing::LG);
            ui.separator();
            ui.add_space(Spacing::MD);

            ui.label(RichText::new(self.i18n.backup_restore()).size(14.0).color(AppColors::TEXT_SECONDARY));
            ui.add_space(Spacing::SM);

            ui.horizontal(|ui| {
                if theme::secondary_button(ui, self.i18n.export_connections()).clicked() {
                    self.show_settings_export_dialog = true;
                    self.settings_export_password.clear();
                    self.settings_export_password_confirm.clear();
                    self.settings_export_include_tokens = false;
                    self.settings_message = None;
                }
                ui.add_space(Spacing::SM);
                if theme::secondary_button(ui, self.i18n.import_connections()).clicked() {
                    self.show_settings_import_dialog = true;
                    self.settings_import_password.clear();
                    self.settings_import_content = None;
                    self.settings_import_merge = true;
                    self.settings_message = None;
                }
            });

            // Show settings message if any
            if let Some((success, msg)) = &self.settings_message {
                ui.add_space(Spacing::SM);
                let color = if *success { AppColors::SUCCESS } else { AppColors::ERROR };
                ui.label(RichText::new(msg).size(12.0).color(color));
            }
        });
    }

    fn render_profile_editor(&mut self, ui: &mut egui::Ui) {
        let is_editing = self.editing_index.is_some();
        let title = if is_editing { self.i18n.edit_connection() } else { self.i18n.new_connection() };

        // Header (fixed, not scrolled)
        ui.horizontal(|ui| {
            if theme::secondary_button(ui, &format!("← {}", self.i18n.cancel())).clicked() {
                self.editing_profile = None;
                self.editing_index = None;
                self.show_profile_editor = false;
                self.keychain_error = None;
                self.connection_test_result = None;
            }
            ui.add_space(Spacing::MD);
            ui.label(RichText::new(title).size(24.0).strong().color(AppColors::TEXT_PRIMARY));
        });

        ui.add_space(Spacing::LG);

        // Connection type tabs (only for new connections)
        if !is_editing {
            if let Some(ref mut profile) = self.editing_profile {
                ui.horizontal(|ui| {
                    // Remote D1 tab
                    let remote_selected = profile.connection_type == ConnectionType::Remote;
                    let remote_bg = if remote_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
                    let remote_border = if remote_selected { AppColors::PRIMARY } else { AppColors::BORDER };
                    let remote_text = if remote_selected { AppColors::PRIMARY } else { AppColors::TEXT_SECONDARY };

                    let remote_btn = egui::Button::new(
                        RichText::new(format!("☁ {}", self.i18n.remote_d1())).color(remote_text)
                    )
                    .fill(remote_bg)
                    .stroke(Stroke::new(1.0, remote_border))
                    .corner_radius(Radius::MD);

                    if ui.add(remote_btn).clicked() {
                        profile.connection_type = ConnectionType::Remote;
                    }

                    ui.add_space(Spacing::SM);

                    // Local SQLite tab
                    let local_selected = profile.connection_type == ConnectionType::Local;
                    let local_bg = if local_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
                    let local_border = if local_selected { AppColors::PRIMARY } else { AppColors::BORDER };
                    let local_text = if local_selected { AppColors::PRIMARY } else { AppColors::TEXT_SECONDARY };

                    let local_btn = egui::Button::new(
                        RichText::new(format!("💾 {}", self.i18n.local_sqlite())).color(local_text)
                    )
                    .fill(local_bg)
                    .stroke(Stroke::new(1.0, local_border))
                    .corner_radius(Radius::MD);

                    if ui.add(local_btn).clicked() {
                        profile.connection_type = ConnectionType::Local;
                    }
                });
                ui.add_space(Spacing::MD);
            }
        }

        // Scrollable content area
        egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical(|ui| {

            if let Some(ref error) = self.keychain_error {
                egui::Frame::new()
                    .fill(AppColors::ERROR.gamma_multiply(0.2))
                    .corner_radius(Radius::MD)
                    .inner_margin(egui::Margin::same(Spacing::SM as i8))
                    .show(ui, |ui| {
                        ui.label(RichText::new(error).color(AppColors::ERROR));
                    });
                ui.add_space(Spacing::MD);
            }

            // Get connection type before borrowing profile mutably
            let connection_type = self.editing_profile.as_ref().map(|p| p.connection_type);

            if connection_type == Some(ConnectionType::Local) {
                // Local SQLite connection form
                self.render_local_connection_form(ui);
            } else {
                // Remote D1 connection form (original form)
                self.render_remote_connection_form(ui);
            }
        });
        });
    }

    fn render_remote_connection_form(&mut self, ui: &mut egui::Ui) {
        // Extract values needed for test connection (to avoid borrow issues)
        let test_params = self.editing_profile.as_ref().map(|p| {
            (p.account_id.clone(), p.database_id.clone(), p.api_token.clone())
        });
        let can_test = test_params.as_ref().map(|(a, d, t)| {
            !a.is_empty() && !d.is_empty() && !t.is_empty()
        }).unwrap_or(false);

        if let Some(ref mut profile) = self.editing_profile {
            theme::card(ui, |ui| {
                ui.set_width(500.0);
                theme::labeled_input(ui, self.i18n.connection_name(), &mut profile.name, false);
                ui.add_space(Spacing::MD);
                theme::labeled_input(ui, self.i18n.account_id(), &mut profile.account_id, false);
                ui.add_space(Spacing::MD);
                theme::labeled_input(ui, self.i18n.database_id(), &mut profile.database_id, false);
                ui.add_space(Spacing::MD);
                theme::labeled_input(ui, self.i18n.api_token(), &mut profile.api_token, true);
                ui.add_space(Spacing::XS);
                ui.label(RichText::new(self.i18n.stored_in_keychain()).size(11.0).color(AppColors::TEXT_MUTED));

                    ui.add_space(Spacing::LG);
                    ui.separator();
                    ui.add_space(Spacing::MD);

                    // Environment type selector
                    ui.label(RichText::new(self.i18n.environment()).size(13.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::XS);
                    ui.horizontal(|ui| {
                        for env_type in [EnvironmentType::Development, EnvironmentType::Staging, EnvironmentType::Production] {
                            let is_selected = profile.environment == env_type;
                            let (bg, text_color) = if is_selected {
                                (env_type.color().gamma_multiply(0.3), env_type.color())
                            } else {
                                (AppColors::BG_TERTIARY, AppColors::TEXT_SECONDARY)
                            };

                            let btn = egui::Button::new(
                                RichText::new(env_type.label()).color(text_color)
                            )
                            .fill(bg)
                            .stroke(Stroke::new(1.0, if is_selected { env_type.color() } else { AppColors::BORDER }))
                            .corner_radius(Radius::MD);

                            if ui.add(btn).clicked() {
                                profile.environment = env_type;
                                // Auto-enable read-only for production
                                if env_type == EnvironmentType::Production {
                                    profile.read_only = true;
                                }
                            }
                        }
                    });
                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(self.i18n.env_confirmation_note()).size(11.0).color(AppColors::TEXT_MUTED));

                    ui.add_space(Spacing::MD);

                    // Read-only toggle with visible frame
                    ui.horizontal(|ui| {
                        let response = ui.checkbox(&mut profile.read_only, "");

                        // Draw a visible border around checkbox when unchecked
                        if !profile.read_only {
                            let checkbox_size = 18.0;
                            let checkbox_pos = response.rect.min;
                            let border_rect = egui::Rect::from_min_size(
                                checkbox_pos,
                                egui::vec2(checkbox_size, checkbox_size)
                            );
                            ui.painter().rect_stroke(
                                border_rect,
                                Radius::SM,
                                Stroke::new(1.5, AppColors::BORDER_LIGHT),
                                egui::StrokeKind::Outside,
                            );
                        }

                        ui.label(RichText::new(self.i18n.read_only_mode()).color(AppColors::TEXT_PRIMARY));
                    });
                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(self.i18n.read_only_description()).size(11.0).color(AppColors::TEXT_MUTED));

                    ui.add_space(Spacing::LG);
                    ui.separator();
                    ui.add_space(Spacing::MD);

                    // Token permissions guidance
                    ui.label(RichText::new(self.i18n.api_token_permissions()).size(13.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::SM);

                    egui::Frame::new()
                        .fill(AppColors::BG_TERTIARY)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(self.i18n.recommended_permissions()).size(12.0).color(AppColors::TEXT_PRIMARY));
                            ui.add_space(Spacing::XS);

                            let (read_perms, write_perms) = if profile.read_only {
                                (
                                    vec!["Account:D1:Read"],
                                    vec![]
                                )
                            } else {
                                (
                                    vec!["Account:D1:Read"],
                                    vec!["Account:D1:Edit"]
                                )
                            };

                            for perm in &read_perms {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("✓").color(AppColors::SUCCESS).size(12.0));
                                    ui.label(RichText::new(*perm).monospace().size(11.0).color(AppColors::TEXT_SECONDARY));
                                });
                            }

                            if !write_perms.is_empty() {
                                for perm in &write_perms {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("✓").color(AppColors::WARNING).size(12.0));
                                        ui.label(RichText::new(*perm).monospace().size(11.0).color(AppColors::TEXT_SECONDARY));
                                    });
                                }
                            }

                            ui.add_space(Spacing::SM);
                            ui.label(RichText::new(self.i18n.create_tokens_at()).size(10.0).color(AppColors::TEXT_MUTED));
                        });
                });

                ui.add_space(Spacing::MD);

                // Connection test result display
                if let Some((success, ref msg)) = self.connection_test_result {
                    let (color, bg_color) = if success {
                        (AppColors::SUCCESS, AppColors::SUCCESS.gamma_multiply(0.2))
                    } else {
                        (AppColors::ERROR, AppColors::ERROR.gamma_multiply(0.2))
                    };
                    egui::Frame::new()
                        .fill(bg_color)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            ui.set_width(500.0);
                            let icon = if success { "✓" } else { "✗" };
                            ui.label(RichText::new(format!("{} {}", icon, msg)).color(color));
                        });
                    ui.add_space(Spacing::MD);
                }

                ui.add_space(Spacing::SM);
            }

            // Test connection button (outside borrow)
            let mut should_test = false;
            ui.horizontal(|ui| {
                let test_button_text = if self.connection_testing {
                    self.i18n.testing()
                } else {
                    self.i18n.test_connection()
                };
                ui.add_enabled_ui(!self.connection_testing && can_test, |ui| {
                    if theme::secondary_button(ui, test_button_text).clicked() {
                        should_test = true;
                    }
                });
            });

            if should_test {
                if let Some((account_id, database_id, api_token)) = test_params {
                    self.test_connection(account_id, database_id, api_token);
                }
            }

            ui.add_space(Spacing::MD);

            // Save button
            let mut should_save = false;
            if theme::primary_button(ui, self.i18n.save_connection()).clicked() {
                should_save = true;
            }

            if should_save {
                if let Some(profile) = self.editing_profile.take() {
                    let profile = profile.to_profile();
                    self.save_profile_to_keychain(&profile);
                    let meta = profile.to_metadata();
                    if let Some(i) = self.editing_index {
                        self.profile_metadata[i] = meta;
                    } else {
                        self.profile_metadata.push(meta);
                    }
                    self.save_profile_metadata();
                    self.editing_index = None;
                    self.show_profile_editor = false;
                    self.connection_test_result = None;
                }
            }

            // Add bottom padding for scroll area
            ui.add_space(Spacing::LG);
    }

    fn render_local_connection_form(&mut self, ui: &mut egui::Ui) {
        if let Some(ref mut profile) = self.editing_profile {
            theme::card(ui, |ui| {
                ui.set_width(500.0);

                // Mode selection tabs (Create New vs Open Existing)
                ui.horizontal(|ui| {
                    // Create New tab
                    let create_selected = profile.local_mode == LocalDatabaseMode::CreateNew;
                    let create_bg = if create_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
                    let create_border = if create_selected { AppColors::PRIMARY } else { AppColors::BORDER };
                    let create_text = if create_selected { AppColors::PRIMARY } else { AppColors::TEXT_SECONDARY };

                    let create_btn = egui::Button::new(
                        RichText::new(format!("✨ {}", self.i18n.create_new_database())).color(create_text)
                    )
                    .fill(create_bg)
                    .stroke(Stroke::new(1.0, create_border))
                    .corner_radius(Radius::MD);

                    if ui.add(create_btn).clicked() {
                        profile.local_mode = LocalDatabaseMode::CreateNew;
                        profile.local_path.clear();
                    }

                    ui.add_space(Spacing::SM);

                    // Open Existing tab
                    let open_selected = profile.local_mode == LocalDatabaseMode::OpenExisting;
                    let open_bg = if open_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
                    let open_border = if open_selected { AppColors::PRIMARY } else { AppColors::BORDER };
                    let open_text = if open_selected { AppColors::PRIMARY } else { AppColors::TEXT_SECONDARY };

                    let open_btn = egui::Button::new(
                        RichText::new(format!("📂 {}", self.i18n.open_existing_database())).color(open_text)
                    )
                    .fill(open_bg)
                    .stroke(Stroke::new(1.0, open_border))
                    .corner_radius(Radius::MD);

                    if ui.add(open_btn).clicked() {
                        profile.local_mode = LocalDatabaseMode::OpenExisting;
                        profile.local_path.clear();
                    }
                });

                ui.add_space(Spacing::LG);

                // Connection name
                theme::labeled_input(ui, self.i18n.connection_name(), &mut profile.name, false);
                ui.add_space(Spacing::MD);

                // Mode-specific UI
                match profile.local_mode {
                    LocalDatabaseMode::CreateNew => {
                        // Create new database mode
                        ui.label(RichText::new(self.i18n.select_folder_for_new_db()).size(13.0).color(AppColors::TEXT_SECONDARY));
                        ui.add_space(Spacing::XS);

                        ui.horizontal(|ui| {
                            // Path display
                            let path_display = if profile.local_path.is_empty() {
                                self.i18n.no_file_selected().to_string()
                            } else {
                                profile.local_path.clone()
                            };

                            egui::Frame::new()
                                .fill(AppColors::BG_TERTIARY)
                                .corner_radius(Radius::MD)
                                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                                .show(ui, |ui| {
                                    ui.set_width(330.0);
                                    ui.label(RichText::new(&path_display)
                                        .monospace()
                                        .size(11.0)
                                        .color(if profile.local_path.is_empty() {
                                            AppColors::TEXT_MUTED
                                        } else {
                                            AppColors::TEXT_PRIMARY
                                        }));
                                });

                            ui.add_space(Spacing::SM);

                            if theme::secondary_button(ui, self.i18n.select_folder()).clicked() {
                                if let Some(folder) = rfd::FileDialog::new()
                                    .set_title("Select folder for new database")
                                    .pick_folder()
                                {
                                    let db_name = format!("local_d1_{}.sqlite", chrono_timestamp());
                                    let db_path = folder.join(&db_name);
                                    profile.local_path = db_path.to_string_lossy().to_string();
                                    if profile.name.is_empty() {
                                        profile.name = db_path.file_stem()
                                            .map(|s| s.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "New Local DB".to_string());
                                    }
                                }
                            }
                        });

                        if !profile.local_path.is_empty() {
                            ui.add_space(Spacing::SM);
                            ui.label(RichText::new(format!("💡 {}", self.i18n.database_will_be_created()))
                                .size(11.0)
                                .color(AppColors::TEXT_MUTED));
                        }
                    }
                    LocalDatabaseMode::OpenExisting => {
                        // Open existing database mode
                        ui.label(RichText::new(self.i18n.select_sqlite_file()).size(13.0).color(AppColors::TEXT_SECONDARY));
                        ui.add_space(Spacing::XS);

                        ui.horizontal(|ui| {
                            // Path display
                            let path_display = if profile.local_path.is_empty() {
                                self.i18n.no_file_selected().to_string()
                            } else {
                                profile.local_path.clone()
                            };

                            egui::Frame::new()
                                .fill(AppColors::BG_TERTIARY)
                                .corner_radius(Radius::MD)
                                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                                .show(ui, |ui| {
                                    ui.set_width(350.0);
                                    ui.label(RichText::new(&path_display)
                                        .monospace()
                                        .size(11.0)
                                        .color(if profile.local_path.is_empty() {
                                            AppColors::TEXT_MUTED
                                        } else {
                                            AppColors::TEXT_PRIMARY
                                        }));
                                });

                            ui.add_space(Spacing::SM);

                            if theme::secondary_button(ui, self.i18n.browse()).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("SQLite Database", &["sqlite", "db", "sqlite3"])
                                    .pick_file()
                                {
                                    profile.local_path = path.to_string_lossy().to_string();
                                    if profile.name.is_empty() {
                                        profile.name = path.file_stem()
                                            .map(|s| s.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "Local DB".to_string());
                                    }
                                }
                            }
                        });
                    }
                }

                ui.add_space(Spacing::LG);
                ui.separator();
                ui.add_space(Spacing::MD);

                // Environment type selector
                ui.label(RichText::new(self.i18n.environment()).size(13.0).color(AppColors::TEXT_SECONDARY));
                ui.add_space(Spacing::XS);
                ui.horizontal(|ui| {
                    for env_type in [EnvironmentType::Development, EnvironmentType::Staging, EnvironmentType::Production] {
                        let is_selected = profile.environment == env_type;
                        let (bg, text_color) = if is_selected {
                            (env_type.color().gamma_multiply(0.3), env_type.color())
                        } else {
                            (AppColors::BG_TERTIARY, AppColors::TEXT_SECONDARY)
                        };

                        let btn = egui::Button::new(
                            RichText::new(env_type.label()).color(text_color)
                        )
                        .fill(bg)
                        .stroke(Stroke::new(1.0, if is_selected { env_type.color() } else { AppColors::BORDER }))
                        .corner_radius(Radius::MD);

                        if ui.add(btn).clicked() {
                            profile.environment = env_type;
                        }
                    }
                });

                ui.add_space(Spacing::MD);

                // Read-only toggle
                ui.horizontal(|ui| {
                    ui.checkbox(&mut profile.read_only, "");
                    ui.label(RichText::new(self.i18n.read_only_mode()).color(AppColors::TEXT_PRIMARY));
                });
            });

            ui.add_space(Spacing::MD);

            // Test connection button (only for existing databases)
            let local_mode = profile.local_mode;
            let can_test = !profile.local_path.is_empty() && local_mode == LocalDatabaseMode::OpenExisting;
            let local_path = profile.local_path.clone();

            if local_mode == LocalDatabaseMode::OpenExisting {
                ui.horizontal(|ui| {
                    let test_button_text = if self.connection_testing {
                        self.i18n.testing()
                    } else {
                        self.i18n.test_connection()
                    };
                    ui.add_enabled_ui(!self.connection_testing && can_test, |ui| {
                        if theme::secondary_button(ui, test_button_text).clicked() {
                            let path = std::path::PathBuf::from(&local_path);
                            let client = crate::local_db::LocalD1Client::new(path);
                            match client.execute("SELECT 1", vec![]) {
                                Ok(_) => {
                                    self.connection_test_result = Some((true, self.i18n.connection_success().to_string()));
                                }
                                Err(e) => {
                                    self.connection_test_result = Some((false, self.i18n.translate_local_db_error(&e)));
                                }
                            }
                        }
                    });
                });

                // Connection test result display
                if let Some((success, ref msg)) = self.connection_test_result {
                    ui.add_space(Spacing::SM);
                    let (color, bg_color) = if success {
                        (AppColors::SUCCESS, AppColors::SUCCESS.gamma_multiply(0.2))
                    } else {
                        (AppColors::ERROR, AppColors::ERROR.gamma_multiply(0.2))
                    };
                    egui::Frame::new()
                        .fill(bg_color)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            let icon = if success { "✓" } else { "✗" };
                            ui.label(RichText::new(format!("{} {}", icon, msg)).color(color));
                        });
                }

                ui.add_space(Spacing::MD);
            }

            // Save button
            let can_save = !profile.local_path.is_empty();
            let local_mode_for_save = profile.local_mode;
            let local_path_for_save = profile.local_path.clone();

            ui.add_enabled_ui(can_save, |ui| {
                if theme::primary_button(ui, self.i18n.save_connection()).clicked() {
                    // For CreateNew mode, create the database file first
                    if local_mode_for_save == LocalDatabaseMode::CreateNew {
                        let db_path = std::path::PathBuf::from(&local_path_for_save);
                        match crate::local_db::LocalD1Client::create_new(&db_path) {
                            Ok(_) => {
                                // Database created successfully, proceed with save
                            }
                            Err(e) => {
                                self.keychain_error = Some(self.i18n.translate_local_db_error(&e));
                                return;
                            }
                        }
                    }

                    if let Some(profile) = self.editing_profile.take() {
                        let profile = profile.to_profile();
                        let meta = profile.to_metadata();
                        if let Some(i) = self.editing_index {
                            self.profile_metadata[i] = meta;
                        } else {
                            self.profile_metadata.push(meta);
                        }
                        self.save_profile_metadata();
                        self.editing_index = None;
                        self.show_profile_editor = false;
                        self.connection_test_result = None;
                    }
                }
            });

            ui.add_space(Spacing::LG);
        }
    }

    fn render_connection_tabs(&mut self, ui: &mut egui::Ui) {
        let mut close_tab: Option<usize> = None;
        let mut switch_tab: Option<usize> = None;

        egui::Frame::new()
            .fill(AppColors::BG_SECONDARY)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(Spacing::SM);

                    for (i, tab) in self.tabs.iter().enumerate() {
                        let is_active = i == self.active_tab;
                        let bg_color = if is_active { AppColors::BG_PRIMARY } else { Color32::TRANSPARENT };
                        let text_color = if is_active { AppColors::TEXT_PRIMARY } else { AppColors::TEXT_SECONDARY };

                        egui::Frame::new()
                            .fill(bg_color)
                            .corner_radius(egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 })
                            .inner_margin(egui::Margin::symmetric(Spacing::MD as i8, Spacing::SM as i8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let status_color = if tab.connected { AppColors::SUCCESS } else { AppColors::TEXT_MUTED };
                                    ui.label(RichText::new("●").size(8.0).color(status_color));
                                    let response = ui.label(RichText::new(&tab.profile.name).color(text_color));
                                    if response.clicked() { switch_tab = Some(i); }
                                    ui.add_space(Spacing::SM);
                                    if ui.add(egui::Button::new(RichText::new("×").size(14.0).color(AppColors::TEXT_MUTED)).frame(false)).clicked() {
                                        close_tab = Some(i);
                                    }
                                });
                            });
                    }

                    ui.add_space(Spacing::SM);
                    if ui.add(egui::Button::new(RichText::new("+").size(16.0).color(AppColors::TEXT_SECONDARY)).frame(false)).clicked() {
                        self.show_settings = true;
                    }
                });
            });

        if let Some(i) = switch_tab { self.active_tab = i; }
        if let Some(i) = close_tab { self.close_tab(i); }
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        if self.active_tab >= self.tabs.len() { return; }

        let loading = self.tabs[self.active_tab].loading;
        let tables = self.tabs[self.active_tab].tables.clone();
        let selected = self.tabs[self.active_tab].selected_table.clone();
        let active_tab = self.active_tab;

        let mut should_refresh = false;
        let mut select_table: Option<String> = None;

        ui.vertical(|ui| {
            ui.add_space(Spacing::MD);
            ui.horizontal(|ui| {
                ui.label(RichText::new(self.i18n.tables()).size(14.0).strong().color(AppColors::TEXT_SECONDARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !loading && ui.add(egui::Button::new(RichText::new("↻").size(14.0).color(AppColors::TEXT_SECONDARY)).frame(false)).clicked() {
                        should_refresh = true;
                    }
                    if loading { ui.spinner(); }
                });
            });

            ui.add_space(Spacing::SM);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for table in &tables {
                    let is_selected = selected.as_ref() == Some(table);
                    let bg_color = if is_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { Color32::TRANSPARENT };
                    let text_color = if is_selected { AppColors::PRIMARY } else { AppColors::TEXT_PRIMARY };

                    let response = egui::Frame::new()
                        .fill(bg_color)
                        .corner_radius(Radius::SM)
                        .inner_margin(egui::Margin::symmetric(Spacing::SM as i8, Spacing::XS as i8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("📋").size(12.0));
                                ui.label(RichText::new(table).color(text_color));
                            });
                        });

                    if response.response.interact(egui::Sense::click()).clicked() {
                        select_table = Some(table.clone());
                    }
                }
            });
        });

        if should_refresh {
            self.connect_tab(active_tab);
        }
        if let Some(table) = select_table {
            self.tabs[active_tab].selected_table = Some(table.clone());
            self.tabs[active_tab].current_page = 0;
            self.tabs[active_tab].highlighted_column = None;  // Clear highlight when selecting from sidebar
            self.tabs[active_tab].filter = None;  // Clear filter when selecting from sidebar
            self.load_table_data(active_tab, &table);
        }
    }

    fn render_table_view(&mut self, ui: &mut egui::Ui) {
        if self.active_tab >= self.tabs.len() { return; }

        let selected_table = self.tabs[self.active_tab].selected_table.clone();

        if selected_table.is_none() {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(Spacing::XL);
                    ui.label(RichText::new("📊").size(48.0));
                    ui.add_space(Spacing::MD);
                    ui.label(RichText::new(self.i18n.select_table()).size(16.0).color(AppColors::TEXT_SECONDARY));
                });
            });
            return;
        }

        let table_name = selected_table.unwrap();
        let current_page = self.tabs[self.active_tab].current_page;
        let row_count = self.tabs[self.active_tab].row_count;
        let rows_per_page = self.rows_per_page;
        let total_pages = (row_count / rows_per_page) + 1;
        let columns = self.tabs[self.active_tab].columns.clone();
        let column_info = self.tabs[self.active_tab].column_info.clone();
        let rows = self.tabs[self.active_tab].rows.clone();
        let selected_row = self.tabs[self.active_tab].selected_row;
        let active_tab = self.active_tab;

        let mut prev_page = false;
        let mut next_page = false;
        let mut refresh = false;
        let delete_row_action = false;
        let add_new_row = false;
        let duplicate_row_action = false;
        let mut start_edit: Option<EditingCell> = None;
        let mut click_row: Option<usize> = None;
        let mut jump_to_fk: Option<(String, String, String)> = None;  // (table_name, column_name, value)
        let mut apply_filter = false;
        let mut remove_filter_idx: Option<usize> = None;
        let mut clear_all_filters = false;
        let highlighted_column = self.tabs[self.active_tab].highlighted_column.clone();
        let current_filter = self.tabs[self.active_tab].filter.clone();
        let filter_count = current_filter.as_ref().map(|f| f.conditions.len()).unwrap_or(0);

        ui.vertical(|ui| {
            ui.add_space(Spacing::MD);

            // Header with table name and filter toggle
            ui.horizontal(|ui| {
                ui.label(RichText::new(&table_name).size(20.0).strong().color(AppColors::TEXT_PRIMARY));
                ui.add_space(Spacing::MD);
                ui.label(RichText::new(self.i18n.rows(row_count)).size(14.0).color(AppColors::TEXT_MUTED));
                if let Some(idx) = selected_row {
                    ui.add_space(Spacing::MD);
                    ui.label(RichText::new(self.i18n.row_selected(idx)).size(12.0).color(AppColors::PRIMARY));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Filter toggle button
                    let filter_label = if filter_count > 0 {
                        self.i18n.filter_with_count(filter_count)
                    } else {
                        self.i18n.filter().to_string()
                    };
                    let filter_btn_color = if self.show_filter_section || filter_count > 0 {
                        AppColors::PRIMARY
                    } else {
                        AppColors::TEXT_SECONDARY
                    };
                    if ui.add(egui::Button::new(RichText::new(filter_label).color(filter_btn_color))).clicked() {
                        self.show_filter_section = !self.show_filter_section;
                    }

                    // Schema Explorer button (show if table has foreign keys)
                    let has_fk = column_info.iter().any(|c| c.foreign_key.is_some());
                    if has_fk {
                        let schema_btn_color = if self.show_schema_explorer {
                            AppColors::PRIMARY
                        } else {
                            AppColors::TEXT_SECONDARY
                        };
                        if ui.add(egui::Button::new(RichText::new(self.i18n.relationships_btn()).color(schema_btn_color))).clicked() {
                            self.show_schema_explorer = !self.show_schema_explorer;
                        }
                    }
                });
            });

            // Filter section
            if self.show_filter_section {
                ui.add_space(Spacing::SM);
                egui::Frame::new()
                    .fill(AppColors::BG_SECONDARY)
                    .corner_radius(Radius::MD)
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(self.i18n.filter()).size(12.0).strong().color(AppColors::TEXT_SECONDARY));
                        });
                        ui.add_space(4.0);

                        // Show existing filters
                        if let Some(ref filter) = current_filter {
                            for (idx, cond) in filter.conditions.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    egui::Frame::new()
                                        .fill(AppColors::PRIMARY.gamma_multiply(0.15))
                                        .corner_radius(Radius::SM)
                                        .inner_margin(egui::Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                let display = if cond.operator.needs_value() {
                                                    format!("{} {} {}", cond.column, cond.operator.label(), cond.value)
                                                } else {
                                                    format!("{} {}", cond.column, cond.operator.label())
                                                };
                                                ui.label(RichText::new(display).size(12.0).color(AppColors::PRIMARY));
                                                ui.add_space(4.0);
                                                if ui.add(egui::Button::new(RichText::new("x").size(10.0).color(AppColors::TEXT_MUTED))
                                                    .frame(false)
                                                    .small()
                                                ).clicked() {
                                                    remove_filter_idx = Some(idx);
                                                }
                                            });
                                        });
                                });
                            }
                            if !filter.conditions.is_empty() {
                                ui.add_space(4.0);
                            }
                        }

                        // Add new filter row
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;

                            // Column dropdown
                            ui.label(RichText::new(self.i18n.column()).size(11.0).color(AppColors::TEXT_MUTED));
                            egui::ComboBox::from_id_salt("filter_column")
                                .selected_text(if self.filter_edit_column.is_empty() {
                                    self.i18n.select().to_string()
                                } else {
                                    self.filter_edit_column.clone()
                                })
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    for col in &columns {
                                        ui.selectable_value(&mut self.filter_edit_column, col.clone(), col);
                                    }
                                });

                            ui.add_space(4.0);

                            // Operator dropdown
                            ui.label(RichText::new(self.i18n.operator()).size(11.0).color(AppColors::TEXT_MUTED));
                            egui::ComboBox::from_id_salt("filter_operator")
                                .selected_text(self.filter_edit_operator.label())
                                .width(80.0)
                                .show_ui(ui, |ui| {
                                    for op in FilterOperator::all() {
                                        ui.selectable_value(&mut self.filter_edit_operator, op.clone(), op.label());
                                    }
                                });

                            ui.add_space(4.0);

                            // Value input (hidden for IS NULL / IS NOT NULL)
                            if self.filter_edit_operator.needs_value() {
                                ui.label(RichText::new(self.i18n.value()).size(11.0).color(AppColors::TEXT_MUTED));
                                let hint = match self.filter_edit_operator {
                                    FilterOperator::Like => "%pattern%",
                                    FilterOperator::In => "val1, val2, ...",
                                    _ => "value",
                                };
                                ui.add(egui::TextEdit::singleline(&mut self.filter_edit_value)
                                    .desired_width(100.0)
                                    .hint_text(hint));
                            }

                            ui.add_space(4.0);

                            // Add filter button
                            let can_add = !self.filter_edit_column.is_empty()
                                && (self.filter_edit_operator.needs_value() == false || !self.filter_edit_value.is_empty());
                            if ui.add_enabled(can_add, egui::Button::new(format!("+ {}", self.i18n.add()))).clicked() {
                                apply_filter = true;
                            }

                            // Clear all button
                            if filter_count > 0 {
                                ui.add_space(8.0);
                                if ui.add(egui::Button::new(RichText::new(self.i18n.clear()).size(11.0).color(AppColors::ERROR))).clicked() {
                                    clear_all_filters = true;
                                }
                            }
                        });
                    });
            }

            ui.add_space(Spacing::MD);

            // Pagination and utility buttons
            ui.horizontal(|ui| {
                let prev_enabled = current_page > 0;
                let next_enabled = (current_page + 1) * rows_per_page < row_count;

                if ui.add_enabled(prev_enabled, egui::Button::new("←")).clicked() {
                    prev_page = true;
                }

                ui.label(RichText::new(format!("{} {} {} {}", self.i18n.page(), current_page + 1, self.i18n.of(), total_pages)).color(AppColors::TEXT_SECONDARY));

                if ui.add_enabled(next_enabled, egui::Button::new("→")).clicked() {
                    next_page = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::secondary_button(ui, &format!("↻ {}", self.i18n.refresh())).clicked() {
                        refresh = true;
                    }
                    ui.add_space(Spacing::SM);
                    if theme::secondary_button(ui, &format!("⬇ {}", self.i18n.export())).clicked() {
                        self.show_export_dialog = true;
                    }
                    ui.add_space(Spacing::SM);
                    if theme::secondary_button(ui, &format!("⬆ {}", self.i18n.import())).clicked() {
                        self.show_import_dialog = true;
                        self.import_preview = None;
                        self.import_sql_statements = None;
                        self.import_message = None;
                        self.import_filename.clear();
                    }
                });
            });

            ui.add_space(Spacing::MD);

            if columns.is_empty() {
                ui.label(RichText::new("No data").color(AppColors::TEXT_MUTED));
                return;
            }

            let available_height = ui.available_height() - 10.0;

            // Horizontal and vertical scroll for table with many columns
            egui::ScrollArea::horizontal()
                .id_salt("table_horizontal_scroll")
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .sense(egui::Sense::click())
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(Column::auto().at_least(120.0).clip(true), columns.len())
                        .min_scrolled_height(0.0)
                        .max_scroll_height(available_height)
                        .header(28.0, |mut header| {
                            for col in columns.iter() {
                                header.col(|ui| {
                                    // Find column info for this column
                                    let col_detail = column_info.iter().find(|c| &c.name == col);
                                    let is_highlighted = highlighted_column.as_ref() == Some(col);

                                    ui.horizontal(|ui| {
                                        // Column name
                                        let mut name_color = if is_highlighted {
                                            AppColors::PRIMARY
                                        } else {
                                            AppColors::TEXT_SECONDARY
                                        };

                                        if let Some(info) = col_detail {
                                            if info.pk {
                                                name_color = AppColors::WARNING;
                                            }
                                        }

                                        ui.label(RichText::new(col).size(12.0).strong().color(name_color));

                                        // Show FK indicator on header (non-clickable, just info)
                                        if let Some(info) = col_detail {
                                            if let Some(ref fk) = info.foreign_key {
                                                ui.label(RichText::new("→").size(10.0).color(AppColors::TEXT_MUTED))
                                                    .on_hover_text(format!("References {}.{}", fk.table, fk.column));
                                            }
                                        }
                                    });
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(28.0, rows.len(), |mut row| {
                                let row_idx = row.index();
                                let row_data = &rows[row_idx];
                                let pk_value = row_data.first().cloned().unwrap_or(serde_json::Value::Null);
                                let is_selected = selected_row == Some(row_idx);

                                // Set row background color for selection
                                row.set_selected(is_selected);

                                for (col_idx, value) in row_data.iter().enumerate() {
                                    let response = row.col(|ui| {
                                        let text = match value {
                                            serde_json::Value::Null => "NULL".to_string(),
                                            serde_json::Value::String(s) => truncate_string(s, 50),
                                            _ => value.to_string(),
                                        };
                                        let color = if matches!(value, serde_json::Value::Null) {
                                            AppColors::TEXT_MUTED
                                        } else if is_selected {
                                            AppColors::TEXT_PRIMARY
                                        } else {
                                            AppColors::TEXT_PRIMARY
                                        };

                                        // Check if this column has a foreign key
                                        let col_name = columns.get(col_idx);
                                        let fk_info = col_name.and_then(|name| {
                                            column_info.iter().find(|c| &c.name == name).and_then(|c| c.foreign_key.as_ref())
                                        });

                                        let label_response = ui.add(
                                            egui::Label::new(RichText::new(&text).size(13.0).color(color))
                                                .sense(egui::Sense::click())
                                        );

                                        if label_response.double_clicked() {
                                            let edit_value = match value {
                                                serde_json::Value::Null => String::new(),
                                                serde_json::Value::String(s) => s.clone(),
                                                _ => value.to_string(),
                                            };
                                            let column_name = columns.get(col_idx).cloned().unwrap_or_default();
                                            start_edit = Some(EditingCell {
                                                row_idx,
                                                col_idx,
                                                column_name,
                                                value: edit_value.clone(),
                                                pk_value: pk_value.clone(),
                                                cursor_position: None,
                                                selection_range: None,
                                                undo_stack: vec![],
                                                redo_stack: vec![],
                                                last_snapshot: edit_value,
                                            });
                                        }

                                        // Show FK link icon for non-null values (right-aligned)
                                        if let Some(fk) = fk_info {
                                            if !matches!(value, serde_json::Value::Null) {
                                                let value_str = match value {
                                                    serde_json::Value::String(s) => s.clone(),
                                                    _ => value.to_string(),
                                                };
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let fk_icon = ui.add(
                                                        egui::Label::new(RichText::new("→").size(11.0).color(AppColors::PRIMARY))
                                                            .sense(egui::Sense::click())
                                                    );
                                                    if fk_icon.clicked() {
                                                        jump_to_fk = Some((fk.table.clone(), fk.column.clone(), value_str));
                                                    }
                                                    fk_icon.on_hover_text(format!("Jump to {}.{}", fk.table, fk.column));
                                                });
                                            }
                                        }
                                    });

                                    // Handle row click for selection
                                    if response.1.clicked() {
                                        click_row = Some(row_idx);
                                    }
                                }
                            });
                        });
                });

        });

        // Handle row selection
        if let Some(row_idx) = click_row {
            // Toggle selection: if clicking the same row, deselect; otherwise select new row
            if self.tabs[active_tab].selected_row == Some(row_idx) {
                self.tabs[active_tab].selected_row = None;
            } else {
                self.tabs[active_tab].selected_row = Some(row_idx);
            }
        }

        // Handle actions after rendering
        if let Some(cell) = start_edit {
            self.editing_cell = Some(cell);
        }

        if prev_page {
            self.tabs[active_tab].current_page -= 1;
            self.tabs[active_tab].selected_row = None; // Clear selection on page change
            self.load_table_data(active_tab, &table_name);
        }
        if next_page {
            self.tabs[active_tab].current_page += 1;
            self.tabs[active_tab].selected_row = None; // Clear selection on page change
            self.load_table_data(active_tab, &table_name);
        }
        if refresh {
            self.tabs[active_tab].selected_row = None; // Clear selection on refresh
            self.load_table_data(active_tab, &table_name);
        }

        // Handle DELETE action
        if delete_row_action {
            if let Some(row_idx) = selected_row {
                if let Some(row_data) = rows.get(row_idx) {
                    if let Some(id) = row_data.first().and_then(|v| v.as_i64()) {
                        self.request_write_operation(PendingOperation::DeleteRow {
                            table: table_name.clone(),
                            id,
                        });
                        self.tabs[active_tab].selected_row = None;
                    }
                }
            }
        }

        // Handle CREATE action
        if add_new_row {
            self.row_editor = Some(RowEditorState {
                values: vec![],
                columns: vec![],
                is_duplicate: false,
                source_row: None,
            });
            self.load_table_schema(active_tab);
        }

        // Handle DUPLICATE action
        if duplicate_row_action {
            if let Some(row_idx) = selected_row {
                if let Some(row_data) = rows.get(row_idx) {
                    self.row_editor = Some(RowEditorState {
                        values: vec![],
                        columns: vec![],
                        is_duplicate: true,
                        source_row: Some(row_data.clone()),
                    });
                    self.load_table_schema(active_tab);
                }
            }
        }

        // Handle add filter
        if apply_filter {
            let new_condition = FilterCondition {
                column: self.filter_edit_column.clone(),
                operator: self.filter_edit_operator.clone(),
                value: self.filter_edit_value.clone(),
            };

            let mut filter = self.tabs[active_tab].filter.take().unwrap_or_default();
            filter.conditions.push(new_condition);
            self.tabs[active_tab].filter = Some(filter);
            self.tabs[active_tab].current_page = 0;

            // Clear edit fields
            self.filter_edit_column.clear();
            self.filter_edit_operator = FilterOperator::Equals;
            self.filter_edit_value.clear();

            self.load_table_data(active_tab, &table_name);
        }

        // Handle remove single filter
        if let Some(idx) = remove_filter_idx {
            if let Some(ref mut filter) = self.tabs[active_tab].filter {
                if idx < filter.conditions.len() {
                    filter.conditions.remove(idx);
                    if filter.conditions.is_empty() {
                        self.tabs[active_tab].filter = None;
                        self.tabs[active_tab].highlighted_column = None;
                    }
                    self.tabs[active_tab].current_page = 0;
                    self.load_table_data(active_tab, &table_name);
                }
            }
        }

        // Handle clear all filters
        if clear_all_filters {
            self.tabs[active_tab].filter = None;
            self.tabs[active_tab].current_page = 0;
            self.tabs[active_tab].highlighted_column = None;
            self.load_table_data(active_tab, &table_name);
        }

        // Handle foreign key jump with filter
        if let Some((target_table, target_column, target_value)) = jump_to_fk {
            // Check if the target table exists in the table list
            if self.tabs[active_tab].tables.contains(&target_table) {
                self.tabs[active_tab].selected_table = Some(target_table.clone());
                self.tabs[active_tab].current_page = 0;
                self.tabs[active_tab].selected_row = None;
                self.tabs[active_tab].highlighted_column = Some(target_column.clone());
                self.tabs[active_tab].filter = Some(TableFilter {
                    conditions: vec![FilterCondition {
                        column: target_column,
                        operator: FilterOperator::Equals,
                        value: target_value,
                    }],
                });
                self.show_filter_section = true;  // Show filter section after FK jump
                self.load_table_data(active_tab, &target_table);
            }
        }
    }

    fn render_action_bar(&mut self, ui: &mut egui::Ui) {
        if self.active_tab >= self.tabs.len() { return; }

        let has_table = self.tabs[self.active_tab].selected_table.is_some();
        let selected_row = self.tabs[self.active_tab].selected_row;
        let has_selection = selected_row.is_some();
        let rows = self.tabs[self.active_tab].rows.clone();
        let active_tab = self.active_tab;

        let mut add_new_row = false;
        let mut duplicate_row_action = false;
        let mut delete_row_action = false;

        ui.horizontal(|ui| {
            if !has_table {
                ui.label(RichText::new("Select a table to manage rows").size(13.0).color(AppColors::TEXT_MUTED).italics());
                return;
            }

            // CREATE button - always enabled when table is selected
            let create_btn = egui::Button::new(
                RichText::new("+ CREATE").color(Color32::WHITE).strong()
            ).fill(AppColors::SUCCESS).corner_radius(Radius::MD);
            if ui.add(create_btn).on_hover_text("Create a new row").clicked() {
                add_new_row = true;
            }

            ui.add_space(Spacing::MD);

            // DUPLICATE button - enabled only when row selected
            let dup_btn = egui::Button::new(
                RichText::new("++ DUPLICATE").color(if has_selection { Color32::WHITE } else { AppColors::TEXT_MUTED }).strong()
            ).fill(if has_selection { AppColors::PRIMARY } else { AppColors::BG_TERTIARY }).corner_radius(Radius::MD);

            let dup_response = ui.add_enabled(has_selection, dup_btn);
            if has_selection {
                if dup_response.on_hover_text("Duplicate selected row").clicked() {
                    duplicate_row_action = true;
                }
            } else {
                dup_response.on_hover_text("Select a row first to duplicate");
            }

            ui.add_space(Spacing::MD);

            // DELETE button - enabled only when row selected
            let del_btn = egui::Button::new(
                RichText::new("− DELETE").color(if has_selection { Color32::WHITE } else { AppColors::TEXT_MUTED }).strong()
            ).fill(if has_selection { AppColors::ERROR } else { AppColors::BG_TERTIARY }).corner_radius(Radius::MD);

            let del_response = ui.add_enabled(has_selection, del_btn);
            if has_selection {
                if del_response.on_hover_text("Delete selected row").clicked() {
                    delete_row_action = true;
                }
            } else {
                del_response.on_hover_text("Select a row first to delete");
            }

            // Clear selection button
            if has_selection {
                ui.add_space(Spacing::LG);
                if ui.add(egui::Button::new(
                    RichText::new(self.i18n.clear_selection()).size(12.0).color(AppColors::TEXT_SECONDARY)
                ).fill(Color32::TRANSPARENT).frame(false)).clicked() {
                    self.tabs[active_tab].selected_row = None;
                }

                // Show selected row info
                if let Some(idx) = selected_row {
                    ui.add_space(Spacing::MD);
                    ui.label(RichText::new(format!("Row {} selected", idx + 1)).size(12.0).color(AppColors::PRIMARY));
                }
            }

            // Show hint when no selection
            if !has_selection && !rows.is_empty() {
                ui.add_space(Spacing::MD);
                ui.label(RichText::new(self.i18n.click_row_to_select()).size(12.0).color(AppColors::TEXT_MUTED).italics());
            }
        });

        // Handle actions
        let table_name = self.tabs[active_tab].selected_table.clone();

        if add_new_row {
            self.row_editor = Some(RowEditorState {
                values: vec![],
                columns: vec![],
                is_duplicate: false,
                source_row: None,
            });
            self.load_table_schema(active_tab);
        }

        if duplicate_row_action {
            if let Some(row_idx) = selected_row {
                if let Some(row_data) = rows.get(row_idx) {
                    self.row_editor = Some(RowEditorState {
                        values: vec![],
                        columns: vec![],
                        is_duplicate: true,
                        source_row: Some(row_data.clone()),
                    });
                    self.load_table_schema(active_tab);
                }
            }
        }

        if delete_row_action {
            if let (Some(row_idx), Some(ref tbl)) = (selected_row, table_name) {
                if let Some(row_data) = rows.get(row_idx) {
                    if let Some(id) = row_data.first().and_then(|v| v.as_i64()) {
                        self.request_write_operation(PendingOperation::DeleteRow {
                            table: tbl.clone(),
                            id,
                        });
                        self.tabs[active_tab].selected_row = None;
                    }
                }
            }
        }
    }

    fn render_sql_editor(&mut self, ui: &mut egui::Ui) {
        if self.active_tab >= self.tabs.len() { return; }

        let active_tab = self.active_tab;
        let loading = self.tabs[active_tab].loading;
        let query_result = self.tabs[active_tab].query_result.clone();
        let mut should_execute = false;
        let mut format_sql = false;

        ui.vertical(|ui| {
            ui.add_space(Spacing::SM);
            ui.horizontal(|ui| {
                ui.label(RichText::new("SQL Query").size(14.0).strong().color(AppColors::TEXT_SECONDARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // AI Suggest button
                    if ui.add(egui::Button::new(RichText::new("💡").size(14.0))
                        .fill(Color32::TRANSPARENT)
                        .frame(false))
                        .on_hover_text(self.i18n.sql_suggestions())
                        .clicked()
                    {
                        self.show_ai_suggest_panel = true;
                    }
                    // Snippets button
                    if ui.add(egui::Button::new(RichText::new("📝").size(14.0))
                        .fill(Color32::TRANSPARENT)
                        .frame(false))
                        .on_hover_text(self.i18n.snippets())
                        .clicked()
                    {
                        self.show_snippets_panel = true;
                    }
                    // Format button
                    if ui.add(egui::Button::new(RichText::new("⚙").size(14.0))
                        .fill(Color32::TRANSPARENT)
                        .frame(false))
                        .on_hover_text(self.i18n.format_sql())
                        .clicked()
                    {
                        format_sql = true;
                    }
                });
            });
            ui.add_space(Spacing::SM);

            egui::Frame::new()
                .fill(AppColors::BG_TERTIARY)
                .corner_radius(Radius::MD)
                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                .show(ui, |ui| {
                    // SQL editor (using monospace font; syntax highlighting available via format button)
                    ui.add(egui::TextEdit::multiline(&mut self.tabs[active_tab].sql_query)
                        .desired_width(f32::INFINITY)
                        .desired_rows(4)
                        .font(egui::TextStyle::Monospace)
                        .frame(false));
                });

            ui.add_space(Spacing::SM);

            let can_execute = !loading && !self.tabs[active_tab].sql_query.is_empty();
            ui.horizontal(|ui| {
                if ui.add_enabled(can_execute, egui::Button::new(RichText::new(format!("▶ {}", self.i18n.execute())).color(Color32::WHITE)).fill(AppColors::PRIMARY)).clicked() {
                    should_execute = true;
                }
                if loading { ui.spinner(); }
                ui.add_space(Spacing::MD);
                if theme::secondary_button(ui, &format!("📜 {}", self.i18n.history())).clicked() {
                    self.show_history_panel = true;
                }
            });

            ui.add_space(Spacing::SM);

            if !query_result.is_empty() {
                ui.label(RichText::new(self.i18n.result()).size(12.0).color(AppColors::TEXT_MUTED));
                ui.add_space(Spacing::XS);
                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    egui::Frame::new()
                        .fill(AppColors::BG_TERTIARY)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(&query_result).monospace().size(12.0).color(AppColors::TEXT_SECONDARY));
                        });
                });
            }
        });

        // Handle format SQL
        if format_sql && !self.tabs[active_tab].sql_query.is_empty() {
            let formatted = sql_highlight::format_sql(&self.tabs[active_tab].sql_query);
            self.tabs[active_tab].sql_query = formatted;
        }

        if should_execute {
            // Check query safety before executing
            let sql = self.tabs[active_tab].sql_query.clone();
            if self.check_query_safety(&sql) {
                // Safe to execute or no confirmation needed
                self.execute_query(active_tab);
            }
            // If check_query_safety returns false, the dangerous query dialog will be shown
        }
    }

    fn render_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_export_dialog {
            return;
        }

        let mut close_dialog = false;
        let mut do_export = false;

        egui::Window::new("Export Data")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(350.0);

                if self.active_tab >= self.tabs.len() {
                    ui.label("No active connection");
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                    return;
                }

                let tab = &self.tabs[self.active_tab];
                let table_name = tab.selected_table.clone().unwrap_or_default();

                ui.add_space(Spacing::SM);
                ui.label(RichText::new(format!("Export table: {}", table_name)).strong());
                ui.label(RichText::new(format!("{} rows, {} columns", tab.rows.len(), tab.columns.len())).color(AppColors::TEXT_MUTED));

                ui.add_space(Spacing::MD);
                ui.label(RichText::new("Export Format").color(AppColors::TEXT_SECONDARY));
                ui.add_space(Spacing::XS);

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.export_format, ExportFormat::Csv, "CSV");
                    ui.selectable_value(&mut self.export_format, ExportFormat::Json, "JSON");
                    ui.selectable_value(&mut self.export_format, ExportFormat::Sql, "SQL");
                });

                ui.add_space(Spacing::XS);
                ui.label(RichText::new(self.export_format.description()).size(12.0).color(AppColors::TEXT_MUTED));

                // Show export message if any
                if let Some((success, msg)) = &self.export_message {
                    ui.add_space(Spacing::SM);
                    let color = if *success { AppColors::SUCCESS } else { AppColors::ERROR };
                    ui.label(RichText::new(msg).color(color));
                }

                ui.add_space(Spacing::MD);
                ui.separator();
                ui.add_space(Spacing::SM);

                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Cancel").clicked() {
                        close_dialog = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::primary_button(ui, "Export").clicked() {
                            do_export = true;
                        }
                    });
                });
            });

        if close_dialog {
            self.show_export_dialog = false;
            self.export_message = None;
        }

        if do_export {
            self.perform_export();
        }
    }

    fn perform_export(&mut self) {
        if self.active_tab >= self.tabs.len() {
            return;
        }

        let tab = &self.tabs[self.active_tab];
        let table_name = tab.selected_table.clone().unwrap_or_else(|| "export".to_string());
        let columns = &tab.columns;
        let rows = &tab.rows;

        if columns.is_empty() {
            self.export_message = Some((false, "No data to export".to_string()));
            return;
        }

        let content = match self.export_format {
            ExportFormat::Csv => export::export_to_csv(columns, rows),
            ExportFormat::Json => export::export_to_json(columns, rows),
            ExportFormat::Sql => export::export_to_sql(&table_name, columns, rows),
        };

        let extension = self.export_format.extension();
        let default_name = format!("{}_{}", table_name, chrono_simple());

        match export::save_to_file(&content, &default_name, extension) {
            Ok(path) => {
                self.export_message = Some((true, format!("Exported to {}", path.display())));
            }
            Err(e) => {
                if e != "Export cancelled" {
                    self.export_message = Some((false, e));
                }
            }
        }
    }

    fn render_import_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_import_dialog {
            return;
        }

        let mut close_dialog = false;
        let mut pick_file = false;
        let mut do_import = false;

        egui::Window::new("Import Data")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(450.0);

                if self.active_tab >= self.tabs.len() {
                    ui.label("No active connection");
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                    return;
                }

                let table_name = self.tabs[self.active_tab]
                    .selected_table
                    .clone()
                    .unwrap_or_default();

                ui.add_space(Spacing::SM);
                // MySQL dump doesn't need table selection
                if self.import_format == ImportFormat::MySqlDump {
                    ui.label(RichText::new(self.i18n.mysql_dump_description()).strong());
                } else {
                    ui.label(RichText::new(format!("Import into table: {}", table_name)).strong());
                }

                ui.add_space(Spacing::MD);
                ui.label(RichText::new("Import Format").color(AppColors::TEXT_SECONDARY));
                ui.add_space(Spacing::XS);

                ui.horizontal(|ui| {
                    if ui.selectable_value(&mut self.import_format, ImportFormat::Csv, "CSV").changed() {
                        self.clear_import_state();
                    }
                    if ui.selectable_value(&mut self.import_format, ImportFormat::Json, "JSON").changed() {
                        self.clear_import_state();
                    }
                    if ui.selectable_value(&mut self.import_format, ImportFormat::Sql, "SQL").changed() {
                        self.clear_import_state();
                    }
                    if ui.selectable_value(&mut self.import_format, ImportFormat::MySqlDump, "MySQL").changed() {
                        self.clear_import_state();
                    }
                });

                ui.add_space(Spacing::XS);
                ui.label(RichText::new(self.import_format.description()).size(12.0).color(AppColors::TEXT_MUTED));

                // Conflict strategy (only for CSV/JSON)
                if self.import_format != ImportFormat::Sql && self.import_format != ImportFormat::MySqlDump {
                    ui.add_space(Spacing::MD);
                    ui.label(RichText::new("On Conflict").color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::XS);

                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.import_conflict_strategy, ConflictStrategy::Fail, "Fail");
                        ui.selectable_value(&mut self.import_conflict_strategy, ConflictStrategy::Replace, "Replace");
                        ui.selectable_value(&mut self.import_conflict_strategy, ConflictStrategy::Skip, "Skip");
                    });

                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(self.import_conflict_strategy.description()).size(12.0).color(AppColors::TEXT_MUTED));
                }

                ui.add_space(Spacing::MD);
                ui.separator();
                ui.add_space(Spacing::SM);

                // File selection
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "📁 Select File").clicked() {
                        pick_file = true;
                    }
                    if !self.import_filename.is_empty() {
                        ui.add_space(Spacing::SM);
                        ui.label(RichText::new(&self.import_filename).color(AppColors::TEXT_SECONDARY));
                    }
                });

                // Preview
                if let Some(ref preview) = self.import_preview {
                    ui.add_space(Spacing::MD);
                    ui.label(RichText::new(format!("Preview: {} columns, {} rows", preview.columns.len(), preview.rows.len())).color(AppColors::SUCCESS));

                    ui.add_space(Spacing::SM);
                    ui.label(RichText::new("Columns:").size(12.0).color(AppColors::TEXT_MUTED));
                    ui.label(RichText::new(preview.columns.join(", ")).size(12.0).monospace());

                    if !preview.rows.is_empty() {
                        ui.add_space(Spacing::XS);
                        ui.label(RichText::new("First row:").size(12.0).color(AppColors::TEXT_MUTED));
                        let first_row: Vec<String> = preview.rows[0]
                            .iter()
                            .map(|v| match v {
                                serde_json::Value::Null => "NULL".to_string(),
                                serde_json::Value::String(s) => truncate_string(s, 20),
                                _ => v.to_string(),
                            })
                            .collect();
                        ui.label(RichText::new(first_row.join(", ")).size(12.0).monospace());
                    }
                }

                if let Some(ref statements) = self.import_sql_statements {
                    ui.add_space(Spacing::MD);
                    ui.label(RichText::new(format!("SQL: {} statements to execute", statements.len())).color(AppColors::SUCCESS));

                    if !statements.is_empty() {
                        ui.add_space(Spacing::SM);
                        egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                            for (i, stmt) in statements.iter().take(5).enumerate() {
                                let truncated = truncate_string(stmt, 80);
                                ui.label(RichText::new(format!("{}. {}", i + 1, truncated)).size(11.0).monospace().color(AppColors::TEXT_SECONDARY));
                            }
                            if statements.len() > 5 {
                                ui.label(RichText::new(format!("... and {} more", statements.len() - 5)).size(11.0).color(AppColors::TEXT_MUTED));
                            }
                        });
                    }
                }

                // MySQL dump conversion result
                if let Some(ref result) = self.import_mysql_result {
                    ui.add_space(Spacing::MD);
                    ui.label(RichText::new(self.i18n.conversion_result()).strong().color(AppColors::TEXT_PRIMARY));
                    ui.add_space(Spacing::XS);

                    // Summary
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(self.i18n.tables_found(result.tables_found.len())).color(AppColors::SUCCESS));
                        ui.add_space(Spacing::MD);
                        ui.label(RichText::new(self.i18n.insert_statements(result.insert_count)).color(AppColors::SUCCESS));
                    });

                    // Tables list
                    if !result.tables_found.is_empty() {
                        ui.add_space(Spacing::XS);
                        ui.label(RichText::new(result.tables_found.join(", ")).size(11.0).monospace().color(AppColors::TEXT_SECONDARY));
                    }

                    // Total statements
                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(format!("{} total statements", result.statements.len())).size(12.0).color(AppColors::TEXT_MUTED));

                    // Warnings
                    if !result.warnings.is_empty() {
                        ui.add_space(Spacing::SM);
                        ui.label(RichText::new(format!("{} ({})", self.i18n.conversion_warnings(), result.warnings.len())).color(AppColors::WARNING));
                        egui::ScrollArea::vertical().max_height(60.0).show(ui, |ui| {
                            for warning in result.warnings.iter().take(5) {
                                ui.label(RichText::new(format!("• {}", truncate_string(warning, 60))).size(10.0).color(AppColors::WARNING));
                            }
                            if result.warnings.len() > 5 {
                                ui.label(RichText::new(format!("... {} more", result.warnings.len() - 5)).size(10.0).color(AppColors::TEXT_MUTED));
                            }
                        });
                    } else {
                        ui.add_space(Spacing::XS);
                        ui.label(RichText::new(self.i18n.no_warnings()).size(11.0).color(AppColors::TEXT_MUTED));
                    }

                    // Preview first few statements
                    ui.add_space(Spacing::SM);
                    ui.label(RichText::new(self.i18n.preview_sql()).size(11.0).color(AppColors::TEXT_SECONDARY));
                    egui::ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                        for stmt in result.statements.iter().take(3) {
                            ui.label(RichText::new(truncate_string(stmt, 70)).size(10.0).monospace().color(AppColors::TEXT_MUTED));
                        }
                        if result.statements.len() > 3 {
                            ui.label(RichText::new(format!("... {} more", result.statements.len() - 3)).size(10.0).color(AppColors::TEXT_MUTED));
                        }
                    });
                }

                // Import message
                if let Some((success, ref msg)) = self.import_message {
                    ui.add_space(Spacing::SM);
                    let color = if success { AppColors::SUCCESS } else { AppColors::ERROR };
                    ui.label(RichText::new(msg).color(color));
                }

                ui.add_space(Spacing::MD);
                ui.separator();
                ui.add_space(Spacing::SM);

                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Cancel").clicked() {
                        close_dialog = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_import = (self.import_preview.is_some()
                            || self.import_sql_statements.is_some()
                            || self.import_mysql_result.is_some()) && !self.import_loading;
                        let import_label = if self.import_format == ImportFormat::MySqlDump {
                            self.i18n.execute_all()
                        } else {
                            "Import"
                        };
                        if ui.add_enabled(can_import, egui::Button::new(RichText::new(import_label).color(Color32::WHITE)).fill(AppColors::PRIMARY)).clicked() {
                            do_import = true;
                        }
                        if self.import_loading {
                            ui.spinner();
                        }
                    });
                });
            });

        if close_dialog {
            self.show_import_dialog = false;
            self.clear_import_state();
        }

        if pick_file {
            self.pick_import_file();
        }

        if do_import {
            self.perform_import();
        }
    }

    fn clear_import_state(&mut self) {
        self.import_preview = None;
        self.import_sql_statements = None;
        self.import_mysql_result = None;
        self.import_message = None;
        self.import_filename.clear();
    }

    fn pick_import_file(&mut self) {
        match export::pick_and_read_file(self.import_format) {
            Ok((content, filename)) => {
                self.import_filename = filename;
                self.import_message = None;

                match self.import_format {
                    ImportFormat::Csv => match export::parse_csv(&content) {
                        Ok(data) => {
                            self.import_preview = Some(data);
                            self.import_sql_statements = None;
                            self.import_mysql_result = None;
                        }
                        Err(e) => {
                            self.import_message = Some((false, e));
                        }
                    },
                    ImportFormat::Json => match export::parse_json(&content) {
                        Ok(data) => {
                            self.import_preview = Some(data);
                            self.import_sql_statements = None;
                            self.import_mysql_result = None;
                        }
                        Err(e) => {
                            self.import_message = Some((false, e));
                        }
                    },
                    ImportFormat::Sql => match export::parse_sql(&content) {
                        Ok(statements) => {
                            self.import_sql_statements = Some(statements);
                            self.import_preview = None;
                            self.import_mysql_result = None;
                        }
                        Err(e) => {
                            self.import_message = Some((false, e));
                        }
                    },
                    ImportFormat::MySqlDump => match export::parse_mysql_dump(&content) {
                        Ok(result) => {
                            self.import_mysql_result = Some(result);
                            self.import_preview = None;
                            self.import_sql_statements = None;
                        }
                        Err(e) => {
                            self.import_message = Some((false, e));
                        }
                    },
                }
            }
            Err(e) => {
                if e != "Import cancelled" {
                    self.import_message = Some((false, e));
                }
            }
        }
    }

    fn perform_import(&mut self) {
        if self.active_tab >= self.tabs.len() {
            return;
        }

        // For MySQL dump, table selection is not required
        let table_name = if self.import_format == ImportFormat::MySqlDump {
            String::new() // Not used for MySQL dump
        } else {
            match &self.tabs[self.active_tab].selected_table {
                Some(name) => name.clone(),
                None => {
                    self.import_message = Some((false, "No table selected".to_string()));
                    return;
                }
            }
        };

        // Generate SQL statements
        let statements: Vec<String> = if let Some(ref result) = self.import_mysql_result {
            result.statements.clone()
        } else if let Some(ref data) = self.import_preview {
            export::generate_insert_statements(&table_name, data, self.import_conflict_strategy)
        } else if let Some(ref sql_statements) = self.import_sql_statements {
            sql_statements.clone()
        } else {
            self.import_message = Some((false, "No data to import".to_string()));
            return;
        };

        if statements.is_empty() {
            self.import_message = Some((false, "No statements to execute".to_string()));
            return;
        }

        // Execute all statements
        let tab = &self.tabs[self.active_tab];
        let client = tab.client.clone();
        let sender = self.sender.clone();
        let tab_idx = self.active_tab;
        let stmt_count = statements.len();

        self.import_loading = true;

        self.runtime.spawn(async move {
            let client_guard = client.lock().await;
            if let Some(ref c) = *client_guard {
                let mut success_count = 0;
                let mut last_error: Option<String> = None;

                for stmt in &statements {
                    match c.execute(stmt, vec![]).await {
                        Ok(response) => {
                            if response.success {
                                success_count += 1;
                            } else {
                                let err = response.errors.first()
                                    .map(|e| e.message.clone())
                                    .unwrap_or_else(|| "Unknown error".to_string());
                                last_error = Some(err);
                                break;
                            }
                        }
                        Err(e) => {
                            last_error = Some(e);
                            break;
                        }
                    }
                }

                let result = if let Some(err) = last_error {
                    Err(format!("Imported {}/{} rows. Error: {}", success_count, stmt_count, err))
                } else {
                    Ok(format!("Successfully imported {} rows", success_count))
                };

                let _ = sender.send(Message::QueryExecuted(tab_idx, result, None));
            } else {
                let _ = sender.send(Message::QueryExecuted(tab_idx, Err("Not connected".to_string()), None));
            }
        });
    }

    fn render_row_editor_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_row_editor {
            return;
        }

        let mut close_dialog = false;
        let mut do_insert = false;

        let title = if self.row_editor.as_ref().map(|r| r.is_duplicate).unwrap_or(false) {
            "Duplicate Row"
        } else {
            "New Row"
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(450.0);

                if self.active_tab >= self.tabs.len() {
                    ui.label("No active connection");
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                    return;
                }

                let table_name = self.tabs[self.active_tab]
                    .selected_table
                    .clone()
                    .unwrap_or_default();

                ui.add_space(Spacing::SM);
                ui.label(RichText::new(format!("Insert into: {}", table_name)).strong());

                ui.add_space(Spacing::MD);

                if let Some(ref mut state) = self.row_editor {
                    if state.columns.is_empty() {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading schema...");
                        });
                    } else {
                        egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                            for (i, col) in state.columns.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let label = if col.pk {
                                        format!("{} (PK)", col.name)
                                    } else if col.notnull {
                                        format!("{}*", col.name)
                                    } else {
                                        col.name.clone()
                                    };

                                    ui.label(RichText::new(label).size(13.0).color(AppColors::TEXT_SECONDARY));
                                    ui.add_space(Spacing::SM);
                                    ui.label(RichText::new(&col.col_type).size(11.0).color(AppColors::TEXT_MUTED));
                                });

                                if i < state.values.len() {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut state.values[i])
                                            .desired_width(f32::INFINITY)
                                            .hint_text(if col.pk { "Auto-generated if empty" } else { "" })
                                    );
                                }
                                ui.add_space(Spacing::XS);
                            }
                        });
                    }
                }

                ui.add_space(Spacing::MD);
                ui.separator();
                ui.add_space(Spacing::SM);

                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Cancel").clicked() {
                        close_dialog = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_insert = self.row_editor.as_ref().map(|r| !r.columns.is_empty()).unwrap_or(false) && !self.row_editor_loading;
                        if ui.add_enabled(can_insert, egui::Button::new(RichText::new("Insert").color(Color32::WHITE)).fill(AppColors::PRIMARY)).clicked() {
                            do_insert = true;
                        }
                        if self.row_editor_loading {
                            ui.spinner();
                        }
                    });
                });
            });

        if close_dialog {
            self.show_row_editor = false;
            self.row_editor = None;
        }

        if do_insert {
            if let Some(ref state) = self.row_editor {
                let columns: Vec<String> = state.columns.iter()
                    .enumerate()
                    .filter(|(i, _)| {
                        // Skip empty PK values (let DB auto-generate)
                        let value = state.values.get(*i).map(|s| s.as_str()).unwrap_or("");
                        !value.is_empty() || !state.columns[*i].pk
                    })
                    .filter(|(i, _)| {
                        let value = state.values.get(*i).map(|s| s.as_str()).unwrap_or("");
                        !value.is_empty()
                    })
                    .map(|(_, col)| col.name.clone())
                    .collect();

                let values: Vec<serde_json::Value> = state.columns.iter()
                    .enumerate()
                    .filter(|(i, _)| {
                        let value = state.values.get(*i).map(|s| s.as_str()).unwrap_or("");
                        !value.is_empty()
                    })
                    .map(|(i, col)| {
                        let value = state.values.get(i).map(|s| s.as_str()).unwrap_or("");
                        // Try to parse as number if column type suggests it
                        let col_type_lower = col.col_type.to_lowercase();
                        if col_type_lower.contains("int") || col_type_lower.contains("real") || col_type_lower.contains("numeric") {
                            if let Ok(num) = value.parse::<i64>() {
                                return serde_json::Value::Number(num.into());
                            }
                            if let Ok(num) = value.parse::<f64>() {
                                return serde_json::json!(num);
                            }
                        }
                        serde_json::Value::String(value.to_string())
                    })
                    .collect();

                if !columns.is_empty() {
                    self.insert_row(self.active_tab, columns, values);
                }
            }
        }
    }

    fn render_cell_editor_dialog(&mut self, ctx: &egui::Context) {
        if self.editing_cell.is_none() {
            return;
        }

        let mut close_dialog = false;
        let mut save_changes = false;
        let mut apply_markdown: Option<(&str, &str)> = None;
        let mut do_undo = false;
        let mut do_redo = false;
        let mut snapshot_needed = false;
        let mut captured_selection: Option<(usize, usize)> = None;

        let column_name = self.editing_cell.as_ref().map(|c| c.column_name.clone()).unwrap_or_default();

        // OS-specific modifier key text
        #[cfg(target_os = "macos")]
        let mod_key = "Cmd";
        #[cfg(not(target_os = "macos"))]
        let mod_key = "Ctrl";

        egui::Window::new(self.i18n.editing_column(&column_name))
            .collapsible(false)
            .resizable(false)
            .fixed_size([550.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {

                // Toolbar with markdown shortcuts and undo/redo
                ui.horizontal(|ui| {
                    // Undo/Redo buttons
                    let can_undo = self.editing_cell.as_ref().map(|c| !c.undo_stack.is_empty()).unwrap_or(false);
                    let can_redo = self.editing_cell.as_ref().map(|c| !c.redo_stack.is_empty()).unwrap_or(false);

                    if ui.add_enabled(can_undo, egui::Button::new(RichText::new("↩").size(14.0)).min_size(egui::vec2(28.0, 24.0))).on_hover_text(format!("Undo ({}+Z)", mod_key)).clicked() {
                        do_undo = true;
                    }
                    if ui.add_enabled(can_redo, egui::Button::new(RichText::new("↪").size(14.0)).min_size(egui::vec2(28.0, 24.0))).on_hover_text(format!("Redo ({}+Shift+Z)", mod_key)).clicked() {
                        do_redo = true;
                    }

                    ui.add_space(Spacing::SM);
                    ui.separator();
                    ui.add_space(Spacing::SM);

                    ui.label(RichText::new("Format:").size(12.0).color(AppColors::TEXT_MUTED));
                    ui.add_space(Spacing::XS);

                    if ui.add(egui::Button::new(RichText::new("B").size(13.0).strong()).min_size(egui::vec2(28.0, 24.0))).on_hover_text(format!("Bold ({}+B)", mod_key)).clicked() {
                        apply_markdown = Some(("**", "**"));
                    }
                    if ui.add(egui::Button::new(RichText::new("I").size(13.0).italics()).min_size(egui::vec2(28.0, 24.0))).on_hover_text(format!("Italic ({}+I)", mod_key)).clicked() {
                        apply_markdown = Some(("*", "*"));
                    }
                    if ui.add(egui::Button::new(RichText::new("S").size(13.0).strikethrough()).min_size(egui::vec2(28.0, 24.0))).on_hover_text(format!("Strikethrough ({}+Shift+S)", mod_key)).clicked() {
                        apply_markdown = Some(("~~", "~~"));
                    }
                    if ui.add(egui::Button::new(RichText::new("</>").size(11.0).monospace()).min_size(egui::vec2(32.0, 24.0))).on_hover_text(format!("Code ({}+E)", mod_key)).clicked() {
                        apply_markdown = Some(("`", "`"));
                    }
                    if ui.add(egui::Button::new(RichText::new("[ ]").size(11.0)).min_size(egui::vec2(32.0, 24.0))).on_hover_text(format!("Link ({}+K)", mod_key)).clicked() {
                        apply_markdown = Some(("[", "](url)"));
                    }
                });

                ui.add_space(Spacing::SM);

                // Text editor area
                if let Some(ref mut cell) = self.editing_cell {
                    egui::Frame::new()
                        .fill(AppColors::BG_TERTIARY)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(220.0)
                                .show(ui, |ui| {
                                    let output = egui::TextEdit::multiline(&mut cell.value)
                                        .desired_width(500.0)
                                        .desired_rows(10)
                                        .font(egui::TextStyle::Body)
                                        .frame(false)
                                        .show(ui);

                                    // Capture cursor range for selection-aware markdown
                                    if let Some(cursor_range) = output.cursor_range {
                                        let primary = cursor_range.primary.index;
                                        let secondary = cursor_range.secondary.index;
                                        if primary != secondary {
                                            // There is a selection
                                            let start = primary.min(secondary);
                                            let end = primary.max(secondary);
                                            captured_selection = Some((start, end));
                                        }
                                    }

                                    // Check if value changed for undo snapshot
                                    if cell.value != cell.last_snapshot {
                                        snapshot_needed = true;
                                    }

                                    // Handle keyboard shortcuts (works on both Mac and Windows)
                                    // On Mac: command key, On Windows/Linux: ctrl key
                                    let modifiers = ui.input(|i| i.modifiers);
                                    #[cfg(target_os = "macos")]
                                    let mod_pressed = modifiers.command;
                                    #[cfg(not(target_os = "macos"))]
                                    let mod_pressed = modifiers.ctrl;

                                    if mod_pressed {
                                        if ui.input(|i| i.key_pressed(egui::Key::Z)) {
                                            if modifiers.shift {
                                                do_redo = true;
                                            } else {
                                                do_undo = true;
                                            }
                                        } else if ui.input(|i| i.key_pressed(egui::Key::Y)) {
                                            // Ctrl+Y for redo on Windows
                                            do_redo = true;
                                        } else if ui.input(|i| i.key_pressed(egui::Key::B)) {
                                            apply_markdown = Some(("**", "**"));
                                        } else if ui.input(|i| i.key_pressed(egui::Key::I)) {
                                            apply_markdown = Some(("*", "*"));
                                        } else if ui.input(|i| i.key_pressed(egui::Key::E)) {
                                            apply_markdown = Some(("`", "`"));
                                        } else if ui.input(|i| i.key_pressed(egui::Key::K)) {
                                            apply_markdown = Some(("[", "](url)"));
                                        } else if modifiers.shift && ui.input(|i| i.key_pressed(egui::Key::S)) {
                                            apply_markdown = Some(("~~", "~~"));
                                        } else if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                            save_changes = true;
                                        }
                                    }

                                    // Escape to cancel
                                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                        close_dialog = true;
                                    }

                                    // Request focus on first render
                                    if cell.cursor_position.is_none() {
                                        output.response.request_focus();
                                        cell.cursor_position = Some(cell.value.len());
                                    }
                                });
                        });
                }

                ui.add_space(Spacing::SM);

                // Character count and shortcuts hint
                ui.horizontal(|ui| {
                    if let Some(ref cell) = self.editing_cell {
                        let char_count = cell.value.chars().count();
                        let line_count = cell.value.lines().count().max(1);
                        ui.label(RichText::new(format!("{} chars, {} lines", char_count, line_count)).size(11.0).color(AppColors::TEXT_MUTED));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{}+Enter to save, Esc to cancel", mod_key)).size(11.0).color(AppColors::TEXT_MUTED));
                    });
                });

                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                // Buttons
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, self.i18n.cancel()).clicked() {
                        close_dialog = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new(self.i18n.save()).color(Color32::WHITE)).fill(AppColors::PRIMARY)).clicked() {
                            save_changes = true;
                        }
                    });
                });
            });

        // Store captured selection for markdown application
        if let Some(sel) = captured_selection {
            if let Some(ref mut cell) = self.editing_cell {
                cell.selection_range = Some(sel);
            }
        }

        // Take snapshot for undo before applying changes
        if snapshot_needed && !do_undo && !do_redo {
            if let Some(ref mut cell) = self.editing_cell {
                // Only snapshot if enough change occurred (avoid spamming undo stack)
                if cell.last_snapshot != cell.value {
                    cell.undo_stack.push(cell.last_snapshot.clone());
                    cell.redo_stack.clear(); // Clear redo on new changes
                    cell.last_snapshot = cell.value.clone();
                    // Limit undo stack size
                    if cell.undo_stack.len() > 100 {
                        cell.undo_stack.remove(0);
                    }
                }
            }
        }

        // Handle undo
        if do_undo {
            if let Some(ref mut cell) = self.editing_cell {
                if let Some(prev) = cell.undo_stack.pop() {
                    cell.redo_stack.push(cell.value.clone());
                    cell.value = prev.clone();
                    cell.last_snapshot = prev;
                }
            }
        }

        // Handle redo
        if do_redo {
            if let Some(ref mut cell) = self.editing_cell {
                if let Some(next) = cell.redo_stack.pop() {
                    cell.undo_stack.push(cell.value.clone());
                    cell.value = next.clone();
                    cell.last_snapshot = next;
                }
            }
        }

        // Apply markdown formatting (wrap selected text or append at end)
        if let Some((prefix, suffix)) = apply_markdown {
            if let Some(ref mut cell) = self.editing_cell {
                // Snapshot before markdown change
                cell.undo_stack.push(cell.value.clone());
                cell.redo_stack.clear();

                if let Some((start, end)) = cell.selection_range {
                    // Wrap selected text with markdown syntax
                    // Convert byte indices to work properly with the string
                    let text = cell.value.clone();
                    let chars: Vec<char> = text.chars().collect();

                    // Ensure indices are within bounds
                    let start = start.min(chars.len());
                    let end = end.min(chars.len());

                    let before: String = chars[..start].iter().collect();
                    let selected: String = chars[start..end].iter().collect();
                    let after: String = chars[end..].iter().collect();

                    cell.value = format!("{}{}{}{}{}", before, prefix, selected, suffix, after);
                    cell.selection_range = None; // Clear selection after applying
                } else {
                    // No selection, append at the end
                    cell.value.push_str(prefix);
                    cell.value.push_str(suffix);
                }

                cell.last_snapshot = cell.value.clone();
            }
        }

        if close_dialog {
            self.editing_cell = None;
        }

        if save_changes {
            if let Some(cell) = self.editing_cell.take() {
                let columns = self.tabs.get(self.active_tab).map(|t| t.columns.clone()).unwrap_or_default();
                let pk_column = columns.first().cloned().unwrap_or_default();
                let new_value: serde_json::Value = if cell.value.is_empty() {
                    serde_json::Value::Null
                } else if let Ok(num) = cell.value.parse::<i64>() {
                    serde_json::Value::Number(num.into())
                } else if let Ok(num) = cell.value.parse::<f64>() {
                    serde_json::json!(num)
                } else {
                    serde_json::Value::String(cell.value)
                };
                self.update_cell(self.active_tab, pk_column, cell.pk_value, cell.column_name, new_value);
            }
        }
    }

    fn render_confirmation_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_confirmation_dialog {
            return;
        }

        let mut close_dialog = false;
        let mut execute_operation = false;

        let (env_type, env_color, db_name) = if self.active_tab < self.tabs.len() {
            let tab = &self.tabs[self.active_tab];
            (tab.profile.environment, tab.profile.environment.color(), tab.profile.name.clone())
        } else {
            (EnvironmentType::Development, AppColors::SUCCESS, "Unknown".to_string())
        };

        egui::Window::new("⚠ Confirm Write Operation")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(450.0);

                // Environment warning banner
                egui::Frame::new()
                    .fill(env_color.gamma_multiply(0.2))
                    .corner_radius(Radius::MD)
                    .inner_margin(egui::Margin::same(Spacing::MD as i8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠").size(24.0).color(env_color));
                            ui.add_space(Spacing::SM);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(format!("{} Environment", env_type.label())).strong().color(env_color));
                                ui.label(RichText::new(format!("Database: {}", db_name)).color(AppColors::TEXT_SECONDARY));
                            });
                        });
                    });

                ui.add_space(Spacing::MD);

                if let Some(ref operation) = self.pending_operation {
                    ui.label(RichText::new(format!("Operation: {}", operation.operation_type())).strong().color(AppColors::TEXT_PRIMARY));
                    ui.add_space(Spacing::SM);

                    // SQL preview
                    egui::Frame::new()
                        .fill(AppColors::BG_TERTIARY)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                                ui.label(RichText::new(operation.description()).monospace().size(12.0).color(AppColors::TEXT_SECONDARY));
                            });
                        });
                }

                ui.add_space(Spacing::LG);
                ui.label(RichText::new("This action cannot be undone. Are you sure you want to proceed?").color(AppColors::WARNING));

                ui.add_space(Spacing::MD);
                ui.separator();
                ui.add_space(Spacing::SM);

                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "Cancel").clicked() {
                        close_dialog = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::danger_button(ui, "Execute").clicked() {
                            execute_operation = true;
                        }
                    });
                });
            });

        if close_dialog {
            self.show_confirmation_dialog = false;
            self.pending_operation = None;
        }

        if execute_operation {
            self.show_confirmation_dialog = false;
            if let Some(operation) = self.pending_operation.take() {
                self.execute_pending_operation(operation);
            }
        }
    }

    fn execute_pending_operation(&mut self, operation: PendingOperation) {
        match operation {
            PendingOperation::DeleteRow { table: _, id } => {
                self.delete_row(self.active_tab, id);
            }
            PendingOperation::UpdateCell { table: _, pk_column, pk_value, column, old_value: _, new_value } => {
                self.update_cell(self.active_tab, pk_column, pk_value, column, new_value);
            }
            PendingOperation::InsertRow { table: _, columns, values } => {
                self.insert_row(self.active_tab, columns, values);
            }
            PendingOperation::ExecuteQuery { sql } => {
                if self.active_tab < self.tabs.len() {
                    self.tabs[self.active_tab].sql_query = sql.clone();
                    // Check safety before executing
                    if self.check_query_safety(&sql) {
                        self.execute_query(self.active_tab);
                    }
                }
            }
            PendingOperation::ImportData { statements } => {
                if self.active_tab < self.tabs.len() {
                    let combined_sql = statements.join(";\n");
                    self.tabs[self.active_tab].sql_query = combined_sql.clone();
                    // Check safety before executing
                    if self.check_query_safety(&combined_sql) {
                        self.execute_query(self.active_tab);
                    }
                }
            }
        }
    }

    fn render_history_panel(&mut self, ctx: &egui::Context) {
        if !self.show_history_panel {
            return;
        }

        let mut close_panel = false;
        let mut selected_sql: Option<String> = None;

        egui::Window::new(self.i18n.query_history())
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{} queries", self.query_history.len())).color(AppColors::TEXT_MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::secondary_button(ui, self.i18n.close()).clicked() {
                            close_panel = true;
                        }
                        if theme::secondary_button(ui, self.i18n.clear_history()).clicked() {
                            self.query_history.clear();
                            self.save_query_history();
                        }
                    });
                });

                // Search box
                ui.add_space(Spacing::SM);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").size(14.0).color(AppColors::TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(&mut self.history_search_filter)
                        .hint_text(self.i18n.search_history())
                        .desired_width(ui.available_width() - 20.0));
                });

                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                // Filter history by search term
                let search_lower = self.history_search_filter.to_lowercase();
                let filtered_entries: Vec<_> = self.query_history.iter()
                    .filter(|e| search_lower.is_empty() ||
                            e.sql.to_lowercase().contains(&search_lower) ||
                            e.database_name.to_lowercase().contains(&search_lower))
                    .collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if filtered_entries.is_empty() && !self.history_search_filter.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(Spacing::LG);
                            ui.label(RichText::new(self.i18n.no_matching_history())
                                .color(AppColors::TEXT_MUTED));
                        });
                    } else {
                        for entry in filtered_entries.iter().rev().take(100) {
                            let status_color = if entry.success { AppColors::SUCCESS } else { AppColors::ERROR };
                            let time_str = format_timestamp(entry.timestamp);

                            egui::Frame::new()
                                .fill(AppColors::BG_SECONDARY)
                                .corner_radius(Radius::MD)
                                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(if entry.success { "✓" } else { "✗" }).color(status_color));
                                        ui.label(RichText::new(&time_str).size(11.0).color(AppColors::TEXT_MUTED));
                                        ui.label(RichText::new(&entry.database_name).size(11.0).color(AppColors::TEXT_SECONDARY));
                                        if let Some(duration) = entry.duration_ms {
                                            ui.label(RichText::new(format!("{:.1}ms", duration)).size(11.0).color(AppColors::TEXT_MUTED));
                                        }
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.add(egui::Button::new(RichText::new(self.i18n.use_query()).size(11.0))
                                                .fill(AppColors::PRIMARY)
                                                .corner_radius(Radius::SM))
                                                .clicked()
                                            {
                                                selected_sql = Some(entry.sql.clone());
                                            }
                                        });
                                    });

                                    let sql_preview = if entry.sql.len() > 100 {
                                        format!("{}...", &entry.sql[..100])
                                    } else {
                                        entry.sql.clone()
                                    };
                                    ui.label(RichText::new(&sql_preview).monospace().size(11.0).color(AppColors::TEXT_PRIMARY));
                                });
                            ui.add_space(Spacing::XS);
                        }
                    }
                });
            });

        if close_panel {
            self.show_history_panel = false;
            self.history_search_filter.clear();
        }

        if let Some(sql) = selected_sql {
            if self.active_tab < self.tabs.len() {
                self.tabs[self.active_tab].sql_query = sql;
            }
            self.show_history_panel = false;
            self.history_search_filter.clear();
        }
    }

    fn render_snippets_panel(&mut self, ctx: &egui::Context) {
        if !self.show_snippets_panel {
            return;
        }

        let mut close_panel = false;
        let mut selected_snippet: Option<String> = None;

        let is_japanese = self.i18n.lang == crate::i18n::Language::Japanese;
        let snippets = sql_highlight::get_snippets();

        egui::Window::new(self.i18n.snippets())
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .default_height(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{} snippets", snippets.len())).color(AppColors::TEXT_MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::secondary_button(ui, self.i18n.close()).clicked() {
                            close_panel = true;
                        }
                    });
                });

                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for snippet in &snippets {
                        let name = if is_japanese { snippet.name_ja } else { snippet.name };
                        let desc = if is_japanese { snippet.description_ja } else { snippet.description };

                        egui::Frame::new()
                            .fill(AppColors::BG_SECONDARY)
                            .corner_radius(Radius::MD)
                            .inner_margin(egui::Margin::same(Spacing::SM as i8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(name).size(13.0).strong().color(AppColors::TEXT_PRIMARY));
                                        ui.label(RichText::new(desc).size(11.0).color(AppColors::TEXT_MUTED));
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::Button::new(RichText::new(self.i18n.insert_snippet()).size(11.0))
                                            .fill(AppColors::PRIMARY)
                                            .corner_radius(Radius::SM))
                                            .clicked()
                                        {
                                            selected_snippet = Some(snippet.template.to_string());
                                        }
                                    });
                                });
                                ui.add_space(Spacing::XS);
                                egui::Frame::new()
                                    .fill(AppColors::BG_TERTIARY)
                                    .corner_radius(Radius::SM)
                                    .inner_margin(egui::Margin::same(4))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(snippet.template)
                                            .monospace()
                                            .size(10.0)
                                            .color(AppColors::TEXT_SECONDARY));
                                    });
                            });
                        ui.add_space(Spacing::SM);
                    }
                });
            });

        if close_panel {
            self.show_snippets_panel = false;
        }

        if let Some(template) = selected_snippet {
            if self.active_tab < self.tabs.len() {
                // Append snippet to existing SQL or replace if empty
                let current = &self.tabs[self.active_tab].sql_query;
                if current.trim().is_empty() {
                    self.tabs[self.active_tab].sql_query = template;
                } else {
                    self.tabs[self.active_tab].sql_query = format!("{}\n\n{}", current, template);
                }
            }
            self.show_snippets_panel = false;
        }
    }

    fn render_metrics_panel(&mut self, ctx: &egui::Context) {
        if !self.show_metrics_panel {
            return;
        }

        let mut close_panel = false;

        egui::Window::new(self.i18n.usage_metrics())
            .collapsible(false)
            .resizable(true)
            .default_width(450.0)
            .default_height(500.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.i18n.cloudflare_limits_info()).size(11.0).color(AppColors::TEXT_MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::secondary_button(ui, self.i18n.close()).clicked() {
                            close_panel = true;
                        }
                    });
                });

                ui.add_space(Spacing::MD);
                ui.separator();
                ui.add_space(Spacing::SM);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Rate Limit Status Section
                    theme::section_heading(ui, self.i18n.rate_limit_status());
                    ui.add_space(Spacing::SM);

                    egui::Frame::new()
                        .fill(AppColors::BG_SECONDARY)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::MD as i8))
                        .show(ui, |ui| {
                            // Use local tracking for rate limit estimation
                            let requests_5min = self.usage_metrics.requests_in_last_5min();
                            let limit = 1200_usize; // Cloudflare's 5-minute limit
                            let remaining_estimate = limit.saturating_sub(requests_5min);

                            // Use API-provided values if available, otherwise use estimates
                            let (remaining_display, limit_display) = if let Some(api_remaining) = self.rate_limit_info.api_info.remaining {
                                (api_remaining as usize, self.rate_limit_info.api_info.limit.unwrap_or(1200) as usize)
                            } else {
                                (remaining_estimate, limit)
                            };

                            let ratio_display = remaining_display as f32 / limit_display as f32;

                            // Status indicator
                            let (status_color, status_text) = if ratio_display > 0.5 {
                                (AppColors::SUCCESS, "OK")
                            } else if ratio_display > 0.2 {
                                (AppColors::WARNING, self.i18n.rate_limit_warning())
                            } else {
                                (AppColors::ERROR, self.i18n.rate_limit_critical())
                            };

                            // Progress bar style display
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(self.i18n.api_requests_remaining()).color(AppColors::TEXT_SECONDARY));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let display_text = if self.rate_limit_info.api_info.remaining.is_some() {
                                        format!("{} / {}", remaining_display, limit_display)
                                    } else {
                                        format!("~{} / {}", remaining_display, limit_display)
                                    };
                                    ui.label(RichText::new(display_text).strong().color(status_color));
                                });
                            });

                            // Visual progress bar
                            ui.add_space(Spacing::XS);
                            let progress_rect = ui.available_rect_before_wrap();
                            let bar_height = 8.0;
                            let bar_rect = egui::Rect::from_min_size(
                                egui::pos2(progress_rect.min.x, progress_rect.min.y),
                                egui::vec2(ui.available_width(), bar_height)
                            );

                            ui.painter().rect_filled(bar_rect, Radius::XS, AppColors::BG_TERTIARY);
                            let filled_width = bar_rect.width() * ratio_display;
                            let filled_rect = egui::Rect::from_min_size(
                                bar_rect.min,
                                egui::vec2(filled_width, bar_height)
                            );
                            ui.painter().rect_filled(filled_rect, Radius::XS, status_color);
                            ui.allocate_space(egui::vec2(ui.available_width(), bar_height + Spacing::SM));

                            // Warning message
                            if ratio_display <= 0.2 {
                                ui.add_space(Spacing::XS);
                                ui.label(RichText::new(status_text).size(12.0).color(status_color));
                            }

                            // Reset time (if API provides it)
                            if let Some(reset_at) = self.rate_limit_info.api_info.reset_at {
                                ui.add_space(Spacing::SM);
                                let reset_time = format_timestamp(reset_at);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(self.i18n.rate_limit_resets_at()).size(11.0).color(AppColors::TEXT_MUTED));
                                    ui.label(RichText::new(reset_time).size(11.0).color(AppColors::TEXT_SECONDARY));
                                });
                            }

                            ui.add_space(Spacing::SM);
                            ui.separator();
                            ui.add_space(Spacing::SM);

                            // Requests in last 5 minutes (local tracking)
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(self.i18n.requests_last_5min()).color(AppColors::TEXT_SECONDARY));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let color = if requests_5min > 1000 {
                                        AppColors::ERROR
                                    } else if requests_5min > 800 {
                                        AppColors::WARNING
                                    } else {
                                        AppColors::TEXT_PRIMARY
                                    };
                                    ui.label(RichText::new(format!("{} / 1,200", requests_5min)).color(color));
                                });
                            });
                        });

                    ui.add_space(Spacing::LG);

                    // Session Statistics Section
                    theme::section_heading(ui, self.i18n.session_statistics());
                    ui.add_space(Spacing::SM);

                    egui::Frame::new()
                        .fill(AppColors::BG_SECONDARY)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::MD as i8))
                        .show(ui, |ui| {
                            let metrics = &self.usage_metrics;

                            // Session duration
                            let duration_secs = metrics.session_duration_secs();
                            let duration_str = if duration_secs < 60 {
                                format!("{}s", duration_secs)
                            } else if duration_secs < 3600 {
                                format!("{}m {}s", duration_secs / 60, duration_secs % 60)
                            } else {
                                format!("{}h {}m", duration_secs / 3600, (duration_secs % 3600) / 60)
                            };

                            self.render_metric_row(ui, self.i18n.session_duration(), &duration_str);
                            ui.add_space(Spacing::XS);
                            self.render_metric_row(ui, self.i18n.total_api_requests(), &format!("{}", metrics.session_api_requests));
                            ui.add_space(Spacing::XS);
                            self.render_metric_row(ui, self.i18n.total_queries(), &format!("{}", metrics.session_query_count));

                            ui.add_space(Spacing::SM);
                            ui.separator();
                            ui.add_space(Spacing::SM);

                            // Data metrics
                            self.render_metric_row(ui, self.i18n.rows_read(), &format!("{}", metrics.session_rows_read));
                            ui.add_space(Spacing::XS);
                            self.render_metric_row(ui, self.i18n.rows_written(), &format!("{}", metrics.session_rows_written));

                            ui.add_space(Spacing::SM);
                            ui.separator();
                            ui.add_space(Spacing::SM);

                            // Performance metrics
                            let avg_duration = metrics.average_query_duration_ms();
                            self.render_metric_row(ui, self.i18n.avg_query_duration(), &format!("{:.2}ms", avg_duration));
                            ui.add_space(Spacing::XS);

                            let total_time = metrics.session_total_duration_ms;
                            let total_time_str = if total_time < 1000.0 {
                                format!("{:.1}ms", total_time)
                            } else {
                                format!("{:.2}s", total_time / 1000.0)
                            };
                            self.render_metric_row(ui, self.i18n.total_query_time(), &total_time_str);
                        });
                });
            });

        if close_panel {
            self.show_metrics_panel = false;
        }
    }

    fn render_metric_row(&self, ui: &mut egui::Ui, label: &str, value: &str) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).color(AppColors::TEXT_SECONDARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(value).strong().color(AppColors::TEXT_PRIMARY));
            });
        });
    }

    /// Request a write operation with confirmation if needed
    fn request_write_operation(&mut self, operation: PendingOperation) {
        // Check if read-only
        if self.is_read_only() {
            if self.active_tab < self.tabs.len() {
                self.tabs[self.active_tab].status_message = "Read-only mode: write operations are blocked".to_string();
            }
            return;
        }

        // Check if confirmation is required
        if self.should_confirm_write() {
            self.pending_operation = Some(operation);
            self.show_confirmation_dialog = true;
        } else {
            self.execute_pending_operation(operation);
        }
    }

    fn render_tutorial_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_tutorial {
            return;
        }

        let mut close_dialog = false;

        egui::Window::new(self.i18n.tutorial_title())
            .collapsible(false)
            .resizable(false)
            .fixed_size([500.0, 420.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Step 1
                    ui.label(RichText::new(self.i18n.tutorial_step1_title()).size(15.0).strong().color(AppColors::PRIMARY));
                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(self.i18n.tutorial_step1_desc()).size(13.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::MD);

                    // Step 2
                    ui.label(RichText::new(self.i18n.tutorial_step2_title()).size(15.0).strong().color(AppColors::PRIMARY));
                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(self.i18n.tutorial_step2_desc()).size(13.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::MD);

                    // Step 3
                    ui.label(RichText::new(self.i18n.tutorial_step3_title()).size(15.0).strong().color(AppColors::PRIMARY));
                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(self.i18n.tutorial_step3_desc()).size(13.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::MD);

                    // Step 4
                    ui.label(RichText::new(self.i18n.tutorial_step4_title()).size(15.0).strong().color(AppColors::PRIMARY));
                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(self.i18n.tutorial_step4_desc()).size(13.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::LG);

                    // API Token hint
                    egui::Frame::new()
                        .fill(AppColors::BG_TERTIARY)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("💡").size(14.0));
                                ui.label(RichText::new(self.i18n.tutorial_api_token_hint_title()).size(12.0).color(AppColors::TEXT_SECONDARY));
                            });
                            ui.add_space(Spacing::XS);
                            ui.label(RichText::new(self.i18n.tutorial_api_token_hint_path()).size(12.0).color(AppColors::TEXT_MUTED));
                        });
                });

                ui.add_space(Spacing::MD);
                ui.separator();
                ui.add_space(Spacing::SM);

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::primary_button(ui, self.i18n.got_it()).clicked() {
                            close_dialog = true;
                        }
                    });
                });
            });

        if close_dialog {
            self.show_tutorial = false;
        }
    }

    fn render_settings_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings_export_dialog {
            return;
        }

        let mut close_dialog = false;
        let mut do_export = false;

        egui::Window::new(self.i18n.export_connections())
            .collapsible(false)
            .resizable(false)
            .fixed_size([400.0, 320.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(Spacing::SM);

                // Connection count info
                let count = self.profile_metadata.len();
                ui.label(RichText::new(self.i18n.connections_count(count)).color(AppColors::TEXT_SECONDARY));
                ui.add_space(Spacing::MD);

                // Include tokens checkbox
                ui.checkbox(&mut self.settings_export_include_tokens, self.i18n.include_api_tokens());

                if self.settings_export_include_tokens {
                    ui.add_space(Spacing::SM);

                    // Warning
                    egui::Frame::new()
                        .fill(AppColors::WARNING.gamma_multiply(0.15))
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("⚠").color(AppColors::WARNING));
                                ui.label(RichText::new(self.i18n.api_tokens_warning()).size(12.0).color(AppColors::WARNING));
                            });
                        });

                    ui.add_space(Spacing::MD);

                    // Password fields
                    ui.label(RichText::new(self.i18n.encryption_password()).size(13.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::XS);
                    ui.add(egui::TextEdit::singleline(&mut self.settings_export_password)
                        .password(true)
                        .desired_width(f32::INFINITY));

                    ui.add_space(Spacing::SM);
                    ui.label(RichText::new(self.i18n.confirm_password()).size(13.0).color(AppColors::TEXT_SECONDARY));
                    ui.add_space(Spacing::XS);
                    ui.add(egui::TextEdit::singleline(&mut self.settings_export_password_confirm)
                        .password(true)
                        .desired_width(f32::INFINITY));

                    ui.add_space(Spacing::XS);
                    ui.label(RichText::new(self.i18n.password_hint()).size(11.0).color(AppColors::TEXT_MUTED));

                    // Password validation
                    if !self.settings_export_password.is_empty()
                        && self.settings_export_password != self.settings_export_password_confirm
                    {
                        ui.add_space(Spacing::SM);
                        ui.label(RichText::new(self.i18n.passwords_dont_match()).size(12.0).color(AppColors::ERROR));
                    }
                }

                ui.add_space(Spacing::LG);
                ui.separator();
                ui.add_space(Spacing::SM);

                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, self.i18n.cancel()).clicked() {
                        close_dialog = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_export = if self.settings_export_include_tokens {
                            !self.settings_export_password.is_empty()
                                && self.settings_export_password == self.settings_export_password_confirm
                        } else {
                            true
                        };

                        ui.add_enabled_ui(can_export, |ui| {
                            if theme::primary_button(ui, self.i18n.export()).clicked() {
                                do_export = true;
                            }
                        });
                    });
                });
            });

        if do_export {
            match settings_io::export_profiles(
                &self.profile_metadata,
                self.settings_export_include_tokens,
                &self.settings_export_password,
            ) {
                Ok(json) => {
                    match settings_io::save_export_to_file(&json) {
                        Ok(path) => {
                            self.settings_message = Some((true, format!("{}: {}", self.i18n.settings_exported(), path.display())));
                            close_dialog = true;
                        }
                        Err(e) => {
                            if e != "Export cancelled" {
                                self.settings_message = Some((false, e));
                            }
                        }
                    }
                }
                Err(e) => {
                    self.settings_message = Some((false, e));
                }
            }
        }

        if close_dialog {
            self.show_settings_export_dialog = false;
            self.settings_export_password.clear();
            self.settings_export_password_confirm.clear();
        }
    }

    fn render_settings_import_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings_import_dialog {
            return;
        }

        let mut close_dialog = false;
        let mut do_select_file = false;
        let mut do_import = false;

        egui::Window::new(self.i18n.import_connections())
            .collapsible(false)
            .resizable(false)
            .fixed_size([400.0, 350.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(Spacing::SM);

                // File selection
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, self.i18n.select_file()).clicked() {
                        do_select_file = true;
                    }
                    ui.add_space(Spacing::SM);
                    if self.settings_import_content.is_some() {
                        ui.label(RichText::new("✓").color(AppColors::SUCCESS));
                        ui.label(RichText::new("File loaded").size(12.0).color(AppColors::TEXT_SECONDARY));
                    }
                });

                if let Some(ref content) = self.settings_import_content {
                    // Try to parse and show preview
                    if let Ok(export) = serde_json::from_str::<settings_io::SettingsExport>(content) {
                        ui.add_space(Spacing::MD);

                        egui::Frame::new()
                            .fill(AppColors::BG_TERTIARY)
                            .corner_radius(Radius::MD)
                            .inner_margin(egui::Margin::same(Spacing::SM as i8))
                            .show(ui, |ui| {
                                ui.label(RichText::new(self.i18n.connections_count(export.profiles.len()))
                                    .color(AppColors::TEXT_PRIMARY));

                                if export.tokens_included {
                                    ui.label(RichText::new(self.i18n.include_api_tokens())
                                        .size(12.0).color(AppColors::PRIMARY));
                                }

                                // Show profile names
                                ui.add_space(Spacing::XS);
                                for profile in export.profiles.iter().take(5) {
                                    ui.label(RichText::new(format!("• {}", profile.name))
                                        .size(12.0).color(AppColors::TEXT_SECONDARY));
                                }
                                if export.profiles.len() > 5 {
                                    ui.label(RichText::new(format!("... and {} more", export.profiles.len() - 5))
                                        .size(11.0).color(AppColors::TEXT_MUTED));
                                }
                            });

                        // Password field if tokens are included
                        if export.tokens_included {
                            ui.add_space(Spacing::MD);
                            ui.label(RichText::new(self.i18n.encryption_password()).size(13.0).color(AppColors::TEXT_SECONDARY));
                            ui.add_space(Spacing::XS);
                            ui.add(egui::TextEdit::singleline(&mut self.settings_import_password)
                                .password(true)
                                .desired_width(f32::INFINITY));
                        }

                        // Import mode
                        ui.add_space(Spacing::MD);
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.settings_import_merge, true, self.i18n.import_merge());
                            ui.radio_value(&mut self.settings_import_merge, false, self.i18n.import_replace());
                        });
                    } else {
                        ui.add_space(Spacing::MD);
                        ui.label(RichText::new(self.i18n.invalid_settings_file()).color(AppColors::ERROR));
                    }
                }

                ui.add_space(Spacing::LG);
                ui.separator();
                ui.add_space(Spacing::SM);

                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, self.i18n.cancel()).clicked() {
                        close_dialog = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_import = self.settings_import_content.is_some();

                        ui.add_enabled_ui(can_import, |ui| {
                            if theme::primary_button(ui, self.i18n.import()).clicked() {
                                do_import = true;
                            }
                        });
                    });
                });
            });

        if do_select_file {
            match settings_io::load_import_from_file() {
                Ok(content) => {
                    self.settings_import_content = Some(content);
                    self.settings_import_password.clear();
                }
                Err(e) => {
                    if e != "Import cancelled" {
                        self.settings_message = Some((false, e));
                    }
                }
            }
        }

        if do_import {
            if let Some(ref content) = self.settings_import_content {
                match settings_io::import_profiles(content, &self.settings_import_password) {
                    Ok(result) => {
                        let imported_count = result.profiles.len();

                        if self.settings_import_merge {
                            // Merge: add profiles that don't exist
                            let existing_ids: std::collections::HashSet<_> =
                                self.profile_metadata.iter().map(|p| p.id.clone()).collect();

                            for profile in result.profiles {
                                if !existing_ids.contains(&profile.id) {
                                    self.profile_metadata.push(profile);
                                }
                            }
                        } else {
                            // Replace all
                            // First, delete old tokens from keychain
                            for meta in &self.profile_metadata {
                                let _ = secure_storage::delete_token(&meta.id);
                            }
                            self.profile_metadata = result.profiles;
                        }

                        // Store tokens in keychain
                        for (profile_id, token) in result.tokens {
                            if let Err(e) = secure_storage::store_token(&profile_id, &token) {
                                self.settings_message = Some((false, format!("Token storage error: {}", e)));
                                close_dialog = true;
                                break;
                            }
                        }

                        self.save_profile_metadata();
                        self.settings_message = Some((true, format!("{} ({})", self.i18n.settings_imported(), imported_count)));
                        close_dialog = true;
                    }
                    Err(e) => {
                        self.settings_message = Some((false, e));
                    }
                }
            }
        }

        if close_dialog {
            self.show_settings_import_dialog = false;
            self.settings_import_password.clear();
            self.settings_import_content = None;
        }
    }

    fn render_dangerous_query_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_dangerous_query_dialog {
            return;
        }

        let mut close_dialog = false;
        let mut execute_query = false;

        let is_japanese = self.i18n.lang == Language::Japanese;

        egui::Window::new(self.i18n.dangerous_query_detected())
            .collapsible(false)
            .resizable(false)
            .fixed_size([500.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if let Some(ref analysis) = self.dangerous_query_analysis {
                    // Risk level badge
                    let risk_label = if is_japanese {
                        analysis.risk_level.label_ja()
                    } else {
                        analysis.risk_level.label()
                    };
                    let (r, g, b) = analysis.risk_level.color();
                    let risk_color = Color32::from_rgb(r, g, b);

                    ui.horizontal(|ui| {
                        egui::Frame::new()
                            .fill(risk_color.gamma_multiply(0.2))
                            .corner_radius(Radius::SM)
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(RichText::new(risk_label).color(risk_color).strong());
                            });
                        ui.label(RichText::new(format!("| {}", analysis.query_type.label()))
                            .color(AppColors::TEXT_SECONDARY));
                    });

                    ui.add_space(Spacing::MD);

                    // Show SQL preview
                    egui::Frame::new()
                        .fill(AppColors::BG_TERTIARY)
                        .corner_radius(Radius::MD)
                        .inner_margin(egui::Margin::same(Spacing::SM as i8))
                        .show(ui, |ui| {
                            let sql_preview = if self.dangerous_query_sql.len() > 200 {
                                format!("{}...", &self.dangerous_query_sql[..200])
                            } else {
                                self.dangerous_query_sql.clone()
                            };
                            ui.label(RichText::new(sql_preview).monospace().size(12.0).color(AppColors::TEXT_PRIMARY));
                        });

                    ui.add_space(Spacing::MD);

                    // Warnings
                    if !analysis.warnings.is_empty() {
                        ui.label(RichText::new(self.i18n.warnings()).strong().color(AppColors::TEXT_PRIMARY));
                        ui.add_space(Spacing::XS);

                        for warning in &analysis.warnings {
                            let msg = if is_japanese { &warning.message_ja } else { &warning.message };
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("⚠").color(AppColors::WARNING));
                                ui.label(RichText::new(msg).size(13.0).color(AppColors::WARNING));
                            });
                        }
                        ui.add_space(Spacing::MD);
                    }

                    // Affected tables
                    if !analysis.affected_tables.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(self.i18n.affected_tables()).size(12.0).color(AppColors::TEXT_MUTED));
                            ui.label(RichText::new(analysis.affected_tables.join(", ")).size(12.0).color(AppColors::TEXT_SECONDARY));
                        });
                        ui.add_space(Spacing::MD);
                    }

                    // Production environment warning
                    if self.active_tab < self.tabs.len() {
                        let tab = &self.tabs[self.active_tab];
                        if tab.profile.environment == EnvironmentType::Production {
                            egui::Frame::new()
                                .fill(AppColors::ERROR.gamma_multiply(0.15))
                                .corner_radius(Radius::MD)
                                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("🔴").size(14.0));
                                        ui.label(RichText::new(self.i18n.production_environment_warning())
                                            .color(AppColors::ERROR).strong());
                                    });
                                });
                            ui.add_space(Spacing::MD);

                            // Type to confirm for production
                            ui.label(RichText::new(self.i18n.type_to_confirm("EXECUTE"))
                                .size(13.0).color(AppColors::TEXT_SECONDARY));
                            ui.add_space(Spacing::XS);
                            ui.add(egui::TextEdit::singleline(&mut self.dangerous_query_confirm_text)
                                .desired_width(f32::INFINITY));
                        }
                    }
                }

                ui.add_space(Spacing::LG);
                ui.separator();
                ui.add_space(Spacing::SM);

                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, self.i18n.cancel()).clicked() {
                        close_dialog = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Check if can execute
                        let can_execute = if self.active_tab < self.tabs.len() {
                            let tab = &self.tabs[self.active_tab];
                            if tab.profile.environment == EnvironmentType::Production {
                                self.dangerous_query_confirm_text.trim().to_uppercase() == "EXECUTE"
                            } else {
                                true
                            }
                        } else {
                            true
                        };

                        ui.add_enabled_ui(can_execute, |ui| {
                            if theme::danger_button(ui, self.i18n.execute_anyway()).clicked() {
                                execute_query = true;
                            }
                        });
                    });
                });
            });

        if execute_query {
            close_dialog = true;
            // Execute the query - SQL is already in the tab's sql_query field
            if self.active_tab < self.tabs.len() {
                self.execute_query(self.active_tab);
            }
        }

        if close_dialog {
            self.show_dangerous_query_dialog = false;
            self.dangerous_query_analysis = None;
            self.dangerous_query_sql.clear();
            self.dangerous_query_confirm_text.clear();
        }
    }

    fn render_execution_log_panel(&mut self, ctx: &egui::Context) {
        if !self.show_execution_log_panel {
            return;
        }

        let mut close_panel = false;
        let mut clear_logs = false;

        egui::Window::new(self.i18n.execution_log())
            .collapsible(true)
            .resizable(true)
            .default_size([700.0, 500.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // Toolbar
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{} entries", self.execution_log.len()))
                        .size(12.0).color(AppColors::TEXT_MUTED));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::secondary_button(ui, self.i18n.close()).clicked() {
                            close_panel = true;
                        }
                        if theme::secondary_button(ui, self.i18n.clear_logs()).clicked() {
                            clear_logs = true;
                        }
                        if theme::secondary_button(ui, self.i18n.export_logs()).clicked() {
                            self.export_execution_log();
                        }
                    });
                });

                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                if self.execution_log.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(Spacing::XL);
                        ui.label(RichText::new(self.i18n.no_logs_yet())
                            .size(14.0).color(AppColors::TEXT_MUTED));
                    });
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Show logs in reverse order (newest first)
                        for entry in self.execution_log.iter().rev().take(100) {
                            let env_color = entry.environment.color();
                            let success_color = if entry.success { AppColors::SUCCESS } else { AppColors::ERROR };

                            egui::Frame::new()
                                .fill(AppColors::BG_SECONDARY)
                                .corner_radius(Radius::MD)
                                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Timestamp
                                        ui.label(RichText::new(format_timestamp(entry.timestamp))
                                            .size(11.0).color(AppColors::TEXT_MUTED));

                                        // Environment badge
                                        egui::Frame::new()
                                            .fill(env_color.gamma_multiply(0.2))
                                            .corner_radius(Radius::XS)
                                            .inner_margin(egui::Margin::symmetric(4, 1))
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(entry.environment.short_label())
                                                    .size(9.0).color(env_color));
                                            });

                                        // Risk level
                                        ui.label(RichText::new(&entry.risk_level)
                                            .size(11.0).color(AppColors::TEXT_SECONDARY));

                                        // Success indicator
                                        ui.label(RichText::new(if entry.success { "✓" } else { "✗" })
                                            .color(success_color));

                                        // Duration
                                        if let Some(ms) = entry.duration_ms {
                                            ui.label(RichText::new(format!("{:.1}ms", ms))
                                                .size(11.0).color(AppColors::TEXT_MUTED));
                                        }
                                    });

                                    // SQL (truncated)
                                    let sql_preview = if entry.sql.len() > 100 {
                                        format!("{}...", &entry.sql[..100])
                                    } else {
                                        entry.sql.clone()
                                    };
                                    ui.label(RichText::new(sql_preview)
                                        .monospace().size(11.0).color(AppColors::TEXT_SECONDARY));

                                    // Error if any
                                    if let Some(ref error) = entry.error {
                                        ui.label(RichText::new(error)
                                            .size(11.0).color(AppColors::ERROR));
                                    }
                                });
                            ui.add_space(Spacing::XS);
                        }
                    });
                }
            });

        if clear_logs {
            self.execution_log.clear();
            self.save_execution_log();
        }

        if close_panel {
            self.show_execution_log_panel = false;
        }
    }

    fn export_execution_log(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.execution_log) {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("d1-manager-execution-log.json")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                let _ = std::fs::write(path, json);
            }
        }
    }

    fn render_audit_panel(&mut self, ctx: &egui::Context) {
        if !self.show_audit_panel {
            return;
        }

        let mut close_panel = false;
        let mut clear_journal = false;
        let mut copy_rollback_sql: Option<String> = None;

        egui::Window::new(self.i18n.audit_journal())
            .default_width(700.0)
            .default_height(500.0)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // Header with controls
                ui.horizontal(|ui| {
                    // Toggle enabled
                    let enabled_text = if self.audit_journal.enabled {
                        self.i18n.audit_enabled()
                    } else {
                        self.i18n.audit_disabled()
                    };
                    if ui.checkbox(&mut self.audit_journal.enabled, enabled_text).changed() {
                        let _ = self.audit_journal.save();
                    }

                    ui.add_space(Spacing::MD);

                    // Filter by table
                    ui.label(self.i18n.filter_table());
                    ui.add(egui::TextEdit::singleline(&mut self.audit_filter_table)
                        .hint_text(self.i18n.all_tables())
                        .desired_width(120.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.i18n.close()).clicked() {
                            close_panel = true;
                        }
                        if ui.button(self.i18n.export()).clicked() {
                            self.export_audit_journal();
                        }
                        if ui.button(self.i18n.clear()).clicked() {
                            clear_journal = true;
                        }
                    });
                });

                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                // Get current database ID for filtering
                let current_db_id = if !self.tabs.is_empty() && self.active_tab < self.tabs.len() {
                    self.tabs[self.active_tab].profile.database_id.clone()
                } else {
                    String::new()
                };

                // Get filtered records
                let records: Vec<&ChangeRecord> = if self.audit_filter_table.is_empty() {
                    if current_db_id.is_empty() {
                        self.audit_journal.get_all_records().iter().collect()
                    } else {
                        self.audit_journal.get_database_history(&current_db_id)
                    }
                } else {
                    self.audit_journal.get_table_history(&current_db_id, &self.audit_filter_table)
                };

                if records.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new(self.i18n.no_audit_records()).color(AppColors::TEXT_MUTED));
                    });
                } else {
                    // Split view: list on left, details on right
                    ui.columns(2, |cols| {
                        // Left: record list
                        egui::ScrollArea::vertical()
                            .id_salt("audit_list")
                            .show(&mut cols[0], |ui| {
                                for record in &records {
                                    let is_selected = self.audit_selected_record == Some(record.id);
                                    let bg_color = if is_selected {
                                        AppColors::BG_TERTIARY
                                    } else {
                                        Color32::TRANSPARENT
                                    };

                                    let op_color = match record.operation {
                                        OperationType::Insert => AppColors::SUCCESS,
                                        OperationType::Update => AppColors::WARNING,
                                        OperationType::Delete => AppColors::ERROR,
                                    };

                                    ui.horizontal(|ui| {
                                        let response = ui.add(
                                            egui::Button::new(
                                                RichText::new(format!(
                                                    "{} {} · {}",
                                                    record.operation,
                                                    record.table_name,
                                                    audit::format_timestamp(record.timestamp)
                                                ))
                                                .color(op_color)
                                                .size(12.0)
                                            )
                                            .fill(bg_color)
                                            .frame(true)
                                        );

                                        if response.clicked() {
                                            self.audit_selected_record = Some(record.id);
                                        }
                                    });
                                    ui.add_space(2.0);
                                }
                            });

                        // Right: details panel
                        if let Some(selected_id) = self.audit_selected_record {
                            if let Some(record) = records.iter().find(|r| r.id == selected_id) {
                                egui::ScrollArea::vertical()
                                    .id_salt("audit_details")
                                    .show(&mut cols[1], |ui| {
                                        // Operation badge
                                        let op_color = match record.operation {
                                            OperationType::Insert => AppColors::SUCCESS,
                                            OperationType::Update => AppColors::WARNING,
                                            OperationType::Delete => AppColors::ERROR,
                                        };
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("{}", record.operation))
                                                .color(op_color)
                                                .strong());
                                            ui.label(RichText::new(&record.table_name).strong());
                                        });

                                        ui.add_space(Spacing::XS);
                                        ui.label(RichText::new(format!(
                                            "{} · {}",
                                            record.profile_name,
                                            audit::format_timestamp(record.timestamp)
                                        )).size(11.0).color(AppColors::TEXT_MUTED));

                                        ui.add_space(Spacing::SM);
                                        ui.separator();
                                        ui.add_space(Spacing::SM);

                                        // Original SQL
                                        ui.label(RichText::new(self.i18n.original_sql()).strong());
                                        ui.add(egui::TextEdit::multiline(&mut record.original_sql.as_str())
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(2));

                                        ui.add_space(Spacing::SM);

                                        // Before/After values
                                        if let Some(before) = &record.before {
                                            ui.label(RichText::new(self.i18n.before_values()).strong());
                                            Self::render_json_values(ui, before);
                                            ui.add_space(Spacing::XS);
                                        }

                                        if let Some(after) = &record.after {
                                            ui.label(RichText::new(self.i18n.after_values()).strong());
                                            Self::render_json_values(ui, after);
                                            ui.add_space(Spacing::XS);
                                        }

                                        // Change summary
                                        ui.add_space(Spacing::SM);
                                        ui.label(RichText::new(record.get_change_summary())
                                            .size(11.0)
                                            .color(AppColors::TEXT_SECONDARY));

                                        // Rollback SQL
                                        if let Some(rollback) = record.generate_rollback_sql() {
                                            ui.add_space(Spacing::SM);
                                            ui.separator();
                                            ui.add_space(Spacing::SM);

                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(self.i18n.rollback_sql()).strong());
                                                if ui.small_button(self.i18n.copy_to_editor()).clicked() {
                                                    copy_rollback_sql = Some(rollback.clone());
                                                }
                                            });
                                            ui.add(egui::TextEdit::multiline(&mut rollback.as_str())
                                                .code_editor()
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(2));
                                        }
                                    });
                            }
                        } else {
                            cols[1].centered_and_justified(|ui| {
                                ui.label(RichText::new(self.i18n.select_record())
                                    .color(AppColors::TEXT_MUTED));
                            });
                        }
                    });
                }
            });

        if close_panel {
            self.show_audit_panel = false;
        }

        if clear_journal {
            self.audit_journal.clear();
            let _ = self.audit_journal.save();
            self.audit_selected_record = None;
        }

        if let Some(sql) = copy_rollback_sql {
            if !self.tabs.is_empty() && self.active_tab < self.tabs.len() {
                self.tabs[self.active_tab].sql_query = sql;
            }
        }
    }

    fn render_json_values(ui: &mut egui::Ui, values: &std::collections::HashMap<String, serde_json::Value>) {
        egui::Frame::new()
            .fill(AppColors::BG_TERTIARY)
            .corner_radius(Radius::SM)
            .inner_margin(egui::Margin::same(Spacing::XS as i8))
            .show(ui, |ui| {
                for (key, value) in values {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}:", key)).color(AppColors::TEXT_SECONDARY).size(11.0));
                        let value_str = match value {
                            serde_json::Value::String(s) => format!("\"{}\"", s),
                            serde_json::Value::Null => "NULL".to_string(),
                            other => other.to_string(),
                        };
                        ui.label(RichText::new(value_str).size(11.0));
                    });
                }
            });
    }

    fn export_audit_journal(&self) {
        if let Ok(json) = self.audit_journal.export_to_json() {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("d1-manager-audit-journal.json")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Record a mutation in the audit journal
    fn record_audit_for_tab(
        &mut self,
        tab_index: usize,
        sql: &str,
        before: Option<std::collections::HashMap<String, serde_json::Value>>,
        after: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) {
        if !self.audit_journal.enabled {
            return;
        }

        if tab_index >= self.tabs.len() {
            return;
        }

        let tab = &self.tabs[tab_index];

        if let Some((op_type, table_name)) = audit::parse_mutation_sql(sql) {
            // Try to extract primary key from the result
            let primary_key = if let Some(ref after_vals) = after {
                // Use 'id' or 'rowid' as primary key if available
                let mut pk = std::collections::HashMap::new();
                if let Some(id) = after_vals.get("id").or(after_vals.get("rowid")) {
                    pk.insert("id".to_string(), id.clone());
                }
                pk
            } else if let Some(ref before_vals) = before {
                let mut pk = std::collections::HashMap::new();
                if let Some(id) = before_vals.get("id").or(before_vals.get("rowid")) {
                    pk.insert("id".to_string(), id.clone());
                }
                pk
            } else {
                std::collections::HashMap::new()
            };

            self.audit_journal.record_change(
                tab.profile.name.clone(),
                tab.profile.database_id.clone(),
                table_name,
                op_type,
                primary_key,
                before,
                after,
                sql.to_string(),
            );

            // Auto-save
            let _ = self.audit_journal.save();
        }
    }

    fn render_schema_explorer(&mut self, ctx: &egui::Context) {
        if !self.show_schema_explorer {
            return;
        }

        if self.tabs.is_empty() || self.active_tab >= self.tabs.len() {
            return;
        }

        let mut close_panel = false;
        let mut jump_to_table: Option<String> = None;
        let mut generate_join_sql: Option<ForeignKeyRelation> = None;

        let tab = &self.tabs[self.active_tab];
        let selected_table = tab.selected_table.clone();
        let column_info = tab.column_info.clone();

        egui::Window::new(self.i18n.schema_explorer())
            .default_width(500.0)
            .default_height(400.0)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.i18n.relationships()).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.i18n.close()).clicked() {
                            close_panel = true;
                        }
                    });
                });

                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                if let Some(table_name) = &selected_table {
                    // Get relationships for the current table
                    let mut outgoing: Vec<RelationshipView> = Vec::new();
                    let mut incoming: Vec<RelationshipView> = Vec::new();

                    // Build relationships from column_info
                    for col in &column_info {
                        if let Some(ref fk) = col.foreign_key {
                            outgoing.push(RelationshipView {
                                relation_type: schema_explorer::RelationType::References,
                                this_column: col.name.clone(),
                                other_table: fk.table.clone(),
                                other_column: fk.column.clone(),
                            });
                        }
                    }

                    // Check schema_graph for incoming references
                    for rel in &self.schema_graph.relations {
                        if rel.to_table.eq_ignore_ascii_case(table_name) {
                            incoming.push(RelationshipView::from_relation(table_name, rel));
                        }
                    }

                    // Display current table
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📋").size(16.0));
                        ui.label(RichText::new(table_name).size(16.0).strong().color(AppColors::PRIMARY));
                    });

                    ui.add_space(Spacing::MD);

                    if outgoing.is_empty() && incoming.is_empty() {
                        ui.label(RichText::new(self.i18n.no_relationships()).color(AppColors::TEXT_MUTED));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // Outgoing relationships (References)
                            if !outgoing.is_empty() {
                                ui.label(RichText::new(self.i18n.references_label()).size(12.0).color(AppColors::TEXT_SECONDARY));
                                ui.add_space(Spacing::XS);

                                for rel in &outgoing {
                                    ui.horizontal(|ui| {
                                        egui::Frame::new()
                                            .fill(AppColors::BG_TERTIARY)
                                            .corner_radius(Radius::SM)
                                            .inner_margin(egui::Margin::same(Spacing::XS as i8))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(RichText::new(&rel.this_column).color(AppColors::TEXT_PRIMARY));
                                                    ui.label(RichText::new("->").color(AppColors::TEXT_MUTED));
                                                    if ui.link(RichText::new(&rel.other_table).color(AppColors::PRIMARY)).clicked() {
                                                        jump_to_table = Some(rel.other_table.clone());
                                                    }
                                                    ui.label(RichText::new(format!(".{}", rel.other_column)).color(AppColors::TEXT_MUTED).size(11.0));

                                                    ui.add_space(Spacing::SM);
                                                    if ui.small_button(self.i18n.generate_join()).clicked() {
                                                        generate_join_sql = Some(ForeignKeyRelation {
                                                            from_table: table_name.clone(),
                                                            from_column: rel.this_column.clone(),
                                                            to_table: rel.other_table.clone(),
                                                            to_column: rel.other_column.clone(),
                                                        });
                                                    }
                                                });
                                            });
                                    });
                                    ui.add_space(2.0);
                                }
                                ui.add_space(Spacing::SM);
                            }

                            // Incoming relationships (Referenced By)
                            if !incoming.is_empty() {
                                ui.label(RichText::new(self.i18n.referenced_by_label()).size(12.0).color(AppColors::TEXT_SECONDARY));
                                ui.add_space(Spacing::XS);

                                for rel in &incoming {
                                    ui.horizontal(|ui| {
                                        egui::Frame::new()
                                            .fill(AppColors::BG_TERTIARY)
                                            .corner_radius(Radius::SM)
                                            .inner_margin(egui::Margin::same(Spacing::XS as i8))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    if ui.link(RichText::new(&rel.other_table).color(AppColors::SUCCESS)).clicked() {
                                                        jump_to_table = Some(rel.other_table.clone());
                                                    }
                                                    ui.label(RichText::new(format!(".{}", rel.other_column)).color(AppColors::TEXT_MUTED).size(11.0));
                                                    ui.label(RichText::new("->").color(AppColors::TEXT_MUTED));
                                                    ui.label(RichText::new(&rel.this_column).color(AppColors::TEXT_PRIMARY));
                                                });
                                            });
                                    });
                                    ui.add_space(2.0);
                                }
                            }
                        });
                    }

                    ui.add_space(Spacing::MD);
                    ui.separator();
                    ui.add_space(Spacing::SM);

                    // Simple ER diagram (ASCII style)
                    ui.collapsing(self.i18n.er_diagram(), |ui| {
                        let diagram = self.generate_simple_er(table_name, &column_info, &outgoing);
                        ui.add(egui::TextEdit::multiline(&mut diagram.as_str())
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(10));
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new(self.i18n.select_table_first()).color(AppColors::TEXT_MUTED));
                    });
                }
            });

        if close_panel {
            self.show_schema_explorer = false;
        }

        if let Some(table) = jump_to_table {
            // Jump to the related table
            let active_tab = self.active_tab;
            if self.tabs[active_tab].tables.contains(&table) {
                self.tabs[active_tab].selected_table = Some(table.clone());
                self.tabs[active_tab].current_page = 0;
                self.tabs[active_tab].filter = None;
                self.load_table_data(active_tab, &table);
            }
        }

        if let Some(rel) = generate_join_sql {
            let sql = schema_explorer::generate_join_sql(
                &rel.from_table,
                &rel,
                self.rows_per_page,
            );
            if self.active_tab < self.tabs.len() {
                self.tabs[self.active_tab].sql_query = sql;
            }
        }
    }

    fn generate_simple_er(&self, table_name: &str, columns: &[ColumnInfo], relations: &[RelationshipView]) -> String {
        let mut lines = Vec::new();

        // Build table box
        let max_col_len = columns.iter().map(|c| c.name.len() + c.col_type.len() + 5).max().unwrap_or(20);
        let box_width = table_name.len().max(max_col_len).max(20);
        let border = "═".repeat(box_width + 2);

        lines.push(format!("╔{}╗", border));
        lines.push(format!("║ {:^width$} ║", table_name.to_uppercase(), width = box_width));
        lines.push(format!("╠{}╣", "═".repeat(box_width + 2)));

        for col in columns {
            let pk = if col.pk { "🔑" } else { "  " };
            let fk = if col.foreign_key.is_some() { "→" } else { " " };
            let nullable = if col.notnull { "" } else { "?" };
            let col_line = format!("{}{} {}: {}{}", pk, fk, col.name, col.col_type, nullable);
            lines.push(format!("║ {:<width$} ║", col_line, width = box_width));
        }

        lines.push(format!("╚{}╝", border));

        // Add relationship arrows
        if !relations.is_empty() {
            lines.push(String::new());
            lines.push("Relationships:".to_string());
            for rel in relations {
                lines.push(format!("  {} → {}.{}", rel.this_column, rel.other_table, rel.other_column));
            }
        }

        lines.join("\n")
    }

    fn render_schema_diff_panel(&mut self, ctx: &egui::Context) {
        if !self.show_schema_diff_panel {
            return;
        }

        let mut close_panel = false;
        let mut start_compare = false;
        let mut copy_migration_sql = false;

        egui::Window::new(self.i18n.compare_databases())
            .id(egui::Id::new("schema_diff_panel"))
            .default_width(700.0)
            .default_height(500.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Header with close button
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(self.i18n.schema_compare()).color(AppColors::TEXT_PRIMARY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.i18n.close()).clicked() {
                            close_panel = true;
                        }
                    });
                });
                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                // Database selection
                ui.horizontal(|ui| {
                    // Source database selection
                    ui.vertical(|ui| {
                        ui.label(RichText::new(self.i18n.source_database()).strong().color(AppColors::TEXT_SECONDARY));
                        egui::ComboBox::from_id_salt("source_db_select")
                            .width(250.0)
                            .selected_text(
                                self.schema_diff_source_idx
                                    .and_then(|idx| self.profile_metadata.get(idx))
                                    .map(|p| p.name.as_str())
                                    .unwrap_or(self.i18n.select_database())
                            )
                            .show_ui(ui, |ui| {
                                for (idx, profile) in self.profile_metadata.iter().enumerate() {
                                    let is_selected = self.schema_diff_source_idx == Some(idx);
                                    if ui.selectable_label(is_selected, &profile.name).clicked() {
                                        self.schema_diff_source_idx = Some(idx);
                                        self.schema_diff_result = None;
                                        self.schema_diff_migration_sql.clear();
                                    }
                                }
                            });
                    });

                    ui.add_space(Spacing::LG);
                    ui.label(RichText::new("→").size(20.0).color(AppColors::TEXT_MUTED));
                    ui.add_space(Spacing::LG);

                    // Target database selection
                    ui.vertical(|ui| {
                        ui.label(RichText::new(self.i18n.target_database()).strong().color(AppColors::TEXT_SECONDARY));
                        egui::ComboBox::from_id_salt("target_db_select")
                            .width(250.0)
                            .selected_text(
                                self.schema_diff_target_idx
                                    .and_then(|idx| self.profile_metadata.get(idx))
                                    .map(|p| p.name.as_str())
                                    .unwrap_or(self.i18n.select_database())
                            )
                            .show_ui(ui, |ui| {
                                for (idx, profile) in self.profile_metadata.iter().enumerate() {
                                    let is_selected = self.schema_diff_target_idx == Some(idx);
                                    if ui.selectable_label(is_selected, &profile.name).clicked() {
                                        self.schema_diff_target_idx = Some(idx);
                                        self.schema_diff_result = None;
                                        self.schema_diff_migration_sql.clear();
                                    }
                                }
                            });
                    });
                });

                ui.add_space(Spacing::MD);

                // Compare button
                let can_compare = self.schema_diff_source_idx.is_some()
                    && self.schema_diff_target_idx.is_some()
                    && self.schema_diff_source_idx != self.schema_diff_target_idx
                    && !self.schema_diff_loading;

                ui.horizontal(|ui| {
                    if self.schema_diff_loading {
                        ui.spinner();
                        ui.label(self.i18n.comparing());
                    } else {
                        ui.add_enabled_ui(can_compare, |ui| {
                            if ui.button(RichText::new(self.i18n.compare()).color(AppColors::TEXT_PRIMARY)).clicked() {
                                start_compare = true;
                            }
                        });
                    }
                });

                ui.add_space(Spacing::MD);
                ui.separator();
                ui.add_space(Spacing::SM);

                // Results section
                if let Some(ref diff) = self.schema_diff_result {
                    ui.label(RichText::new(self.i18n.schema_diff_result()).strong().size(14.0).color(AppColors::TEXT_PRIMARY));

                    // Add explanation text
                    ui.label(RichText::new(self.i18n.schema_diff_explanation()).size(11.0).color(AppColors::TEXT_MUTED));
                    ui.add_space(Spacing::XS);

                    if diff.is_empty() {
                        ui.label(RichText::new(self.i18n.no_differences()).color(AppColors::SUCCESS));
                    } else {
                        // Summary
                        let summary = self.i18n.tables_count(
                            diff.added_tables().len(),
                            diff.removed_tables().len(),
                            diff.modified_tables().len(),
                        );
                        ui.label(RichText::new(summary).color(AppColors::TEXT_SECONDARY));
                        ui.add_space(Spacing::SM);

                        // Scrollable diff list
                        egui::ScrollArea::vertical()
                            .id_salt("schema_diff_tables")
                            .max_height(250.0)
                            .show(ui, |ui| {
                                for table_diff in &diff.table_diffs {
                                    // Use clearer badge text that explains what the diff means
                                    let (badge_text, badge_color, description) = match table_diff.diff_type {
                                        // Added = exists in source but not in target (needs CREATE TABLE)
                                        DiffType::Added => (
                                            self.i18n.table_only_in_source(),
                                            AppColors::SUCCESS,
                                            "→ CREATE TABLE"
                                        ),
                                        // Removed = exists in target but not in source (needs DROP TABLE)
                                        DiffType::Removed => (
                                            self.i18n.table_only_in_target(),
                                            AppColors::ERROR,
                                            "→ DROP TABLE"
                                        ),
                                        DiffType::Modified => (
                                            self.i18n.table_modified(),
                                            AppColors::WARNING,
                                            "→ ALTER TABLE"
                                        ),
                                    };

                                    egui::Frame::new()
                                        .fill(AppColors::BG_SECONDARY)
                                        .corner_radius(Radius::SM)
                                        .inner_margin(egui::Margin::same(8))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Badge showing where the table exists
                                                egui::Frame::new()
                                                    .fill(badge_color.gamma_multiply(0.2))
                                                    .corner_radius(Radius::XS)
                                                    .inner_margin(egui::Margin::symmetric(6, 2))
                                                    .show(ui, |ui| {
                                                        ui.label(RichText::new(badge_text).size(10.0).color(badge_color));
                                                    });

                                                // Table name
                                                ui.label(RichText::new(&table_diff.table_name).strong().color(AppColors::TEXT_PRIMARY));

                                                // Action description (what migration SQL will do)
                                                ui.label(RichText::new(description).size(10.0).color(AppColors::TEXT_MUTED));
                                            });

                                            // Column changes for modified tables
                                            if !table_diff.column_diffs.is_empty() && table_diff.diff_type == DiffType::Modified {
                                                ui.add_space(4.0);
                                                ui.label(RichText::new(self.i18n.column_changes()).size(11.0).color(AppColors::TEXT_SECONDARY));

                                                for col_diff in &table_diff.column_diffs {
                                                    ui.horizontal(|ui| {
                                                        let (col_badge, col_color) = match col_diff.diff_type {
                                                            DiffType::Added => ("+", AppColors::SUCCESS),
                                                            DiffType::Removed => ("-", AppColors::ERROR),
                                                            DiffType::Modified => ("~", AppColors::WARNING),
                                                        };

                                                        ui.label(RichText::new(col_badge).monospace().color(col_color));
                                                        ui.label(RichText::new(&col_diff.column_name).monospace().size(11.0).color(AppColors::TEXT_PRIMARY));

                                                        // Show old/new definitions for modified columns
                                                        if col_diff.diff_type == DiffType::Modified {
                                                            if let (Some(old), Some(new)) = (&col_diff.old_def, &col_diff.new_def) {
                                                                ui.label(RichText::new(format!(
                                                                    "({}: {} → {}: {})",
                                                                    self.i18n.old_definition(),
                                                                    schema_diff::format_column_def(old),
                                                                    self.i18n.new_definition(),
                                                                    schema_diff::format_column_def(new)
                                                                )).size(10.0).color(AppColors::TEXT_MUTED));
                                                            }
                                                        }
                                                    });
                                                }
                                            }
                                        });
                                    ui.add_space(4.0);
                                }
                            });

                        ui.add_space(Spacing::MD);

                        // Migration SQL section
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(self.i18n.migration_sql()).strong().size(14.0).color(AppColors::TEXT_PRIMARY));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(self.i18n.copy_sql()).clicked() {
                                    copy_migration_sql = true;
                                }
                            });
                        });
                        ui.add_space(Spacing::XS);

                        if !self.schema_diff_migration_sql.is_empty() {
                            egui::ScrollArea::vertical()
                                .id_salt("schema_diff_migration_sql")
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    egui::Frame::new()
                                        .fill(AppColors::BG_PRIMARY)
                                        .corner_radius(Radius::SM)
                                        .inner_margin(egui::Margin::same(8))
                                        .show(ui, |ui| {
                                            for stmt in &self.schema_diff_migration_sql {
                                                ui.label(RichText::new(stmt).monospace().size(11.0).color(AppColors::TEXT_PRIMARY));
                                            }
                                        });
                                });
                        }
                    }
                } else if !self.schema_diff_loading {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new(self.i18n.select_both_databases()).color(AppColors::TEXT_MUTED));
                    });
                }
            });

        if close_panel {
            self.show_schema_diff_panel = false;
        }

        if copy_migration_sql && !self.schema_diff_migration_sql.is_empty() {
            let sql = self.schema_diff_migration_sql.join("\n\n");
            ctx.copy_text(sql);
        }

        if start_compare {
            self.start_schema_compare();
        }
    }

    fn start_schema_compare(&mut self) {
        let source_idx = match self.schema_diff_source_idx {
            Some(idx) => idx,
            None => return,
        };
        let target_idx = match self.schema_diff_target_idx {
            Some(idx) => idx,
            None => return,
        };

        if source_idx == target_idx {
            return;
        }

        // Get profile metadata for both databases
        let source_meta = match self.profile_metadata.get(source_idx) {
            Some(m) => m.clone(),
            None => return,
        };
        let target_meta = match self.profile_metadata.get(target_idx) {
            Some(m) => m.clone(),
            None => return,
        };

        // For remote connections, load API tokens
        let source_token = if source_meta.connection_type == ConnectionType::Remote {
            match secure_storage::get_token(&source_meta.id).ok() {
                Some(t) => Some(t),
                None => return, // Remote DB requires token
            }
        } else {
            None
        };
        let target_token = if target_meta.connection_type == ConnectionType::Remote {
            match secure_storage::get_token(&target_meta.id).ok() {
                Some(t) => Some(t),
                None => return, // Remote DB requires token
            }
        } else {
            None
        };

        self.schema_diff_loading = true;
        self.schema_diff_result = None;
        self.schema_diff_migration_sql.clear();

        let sender = self.sender.clone();

        // Check if either database is local - if so, fetch local schema synchronously first
        let source_is_local = source_meta.connection_type == ConnectionType::Local;
        let target_is_local = target_meta.connection_type == ConnectionType::Local;

        // Fetch local schemas synchronously if needed
        let source_local_schema = if source_is_local {
            source_meta.local_path.as_ref().and_then(|path| {
                fetch_local_database_schema(path).ok()
            })
        } else {
            None
        };

        let target_local_schema = if target_is_local {
            target_meta.local_path.as_ref().and_then(|path| {
                fetch_local_database_schema(path).ok()
            })
        } else {
            None
        };

        // If local schemas failed to load, abort
        if source_is_local && source_local_schema.is_none() {
            self.schema_diff_loading = false;
            return;
        }
        if target_is_local && target_local_schema.is_none() {
            self.schema_diff_loading = false;
            return;
        }

        self.runtime.spawn(async move {
            // Fetch remote schemas if needed
            let source_schema = if source_is_local {
                Ok(source_local_schema.unwrap())
            } else {
                let client = D1Client::new(
                    source_meta.account_id.clone(),
                    source_meta.database_id.clone(),
                    source_token.unwrap().as_str().to_string(),
                );
                fetch_database_schema(&client).await
            };

            let target_schema = if target_is_local {
                Ok(target_local_schema.unwrap())
            } else {
                let client = D1Client::new(
                    target_meta.account_id.clone(),
                    target_meta.database_id.clone(),
                    target_token.unwrap().as_str().to_string(),
                );
                fetch_database_schema(&client).await
            };

            match (source_schema, target_schema) {
                (Ok(source), Ok(target)) => {
                    let diff = schema_diff::compare_schemas(&source, &target);
                    let migration_sql = schema_diff::generate_migration_sql(&diff, &source);
                    let _ = sender.send(Message::SchemaDiffResult(Some(diff), source, migration_sql));
                }
                _ => {
                    let _ = sender.send(Message::SchemaDiffResult(None, DatabaseSchema::new(), vec![]));
                }
            }
        });
    }

    fn render_ai_suggest_panel(&mut self, ctx: &egui::Context) {
        use crate::ai_suggest::{generate_suggestions, SqlSuggestion, SuggestionCategory, SuggestionContext};
        use crate::i18n::Language;

        if !self.show_ai_suggest_panel {
            return;
        }

        let mut close_panel = false;
        let mut use_sql: Option<String> = None;

        // Collect context from current tab
        let (table_name, columns, all_tables, foreign_keys) = if self.active_tab < self.tabs.len() {
            let tab = &self.tabs[self.active_tab];
            let table = tab.selected_table.clone().unwrap_or_default();
            let cols = tab.column_info.clone();
            let tables = tab.tables.clone();

            // Get foreign keys from column_info
            let fks: Vec<(String, String, String, String)> = cols
                .iter()
                .filter_map(|col| {
                    col.foreign_key.as_ref().map(|fk| {
                        (table.clone(), col.name.clone(), fk.table.clone(), fk.column.clone())
                    })
                })
                .collect();

            (table, cols, tables, fks)
        } else {
            (String::new(), vec![], vec![], vec![])
        };

        // Generate suggestions if we have a table selected
        let suggestions = if !table_name.is_empty() && !columns.is_empty() {
            let context = SuggestionContext {
                table_name: table_name.clone(),
                columns,
                all_tables,
                foreign_keys,
            };
            generate_suggestions(&context)
        } else {
            vec![]
        };

        // Filter suggestions based on category and safety settings
        let filtered_suggestions: Vec<&SqlSuggestion> = suggestions
            .iter()
            .filter(|s| {
                // Filter by safety (hide modification queries unless explicitly shown)
                if !self.ai_suggest_show_unsafe && !s.is_safe {
                    return false;
                }
                // Filter by category if set
                if let Some(ref cat_filter) = self.ai_suggest_category_filter {
                    return s.category == *cat_filter;
                }
                true
            })
            .collect();

        let is_japanese = self.i18n.lang == Language::Japanese;

        egui::Window::new(self.i18n.ai_suggest_title())
            .id(egui::Id::new("ai_suggest_panel"))
            .default_width(500.0)
            .default_height(450.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Header with close button
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(self.i18n.ai_suggest_title()).color(AppColors::TEXT_PRIMARY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.i18n.close()).clicked() {
                            close_panel = true;
                        }
                    });
                });

                // Privacy notice
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔒").size(12.0));
                    ui.label(RichText::new(self.i18n.generated_locally()).size(11.0).color(AppColors::SUCCESS));
                });
                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                // Filters
                ui.horizontal(|ui| {
                    // Category filter
                    ui.label(RichText::new("Category:").color(AppColors::TEXT_SECONDARY));

                    let all_label = if is_japanese { "すべて" } else { "All" };
                    if ui.selectable_label(self.ai_suggest_category_filter.is_none(), all_label).clicked() {
                        self.ai_suggest_category_filter = None;
                    }

                    let categories = [
                        SuggestionCategory::BasicQuery,
                        SuggestionCategory::Aggregation,
                        SuggestionCategory::FilterSort,
                        SuggestionCategory::Join,
                        SuggestionCategory::Schema,
                    ];

                    for cat in categories {
                        let label = if is_japanese { cat.label_ja() } else { cat.label() };
                        let is_selected = self.ai_suggest_category_filter == Some(cat);
                        if ui.selectable_label(is_selected, format!("{} {}", cat.icon(), label)).clicked() {
                            self.ai_suggest_category_filter = if is_selected { None } else { Some(cat) };
                        }
                    }
                });

                ui.add_space(Spacing::XS);

                // Show unsafe toggle
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.ai_suggest_show_unsafe, self.i18n.show_unsafe_queries());
                    if self.ai_suggest_show_unsafe {
                        ui.label(RichText::new("⚠").color(AppColors::WARNING));
                    }
                });

                ui.add_space(Spacing::SM);
                ui.separator();
                ui.add_space(Spacing::SM);

                // Current table info
                if !table_name.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📋").size(14.0));
                        ui.label(RichText::new(format!("Table: {}", table_name)).strong().color(AppColors::TEXT_PRIMARY));
                    });
                    ui.add_space(Spacing::SM);
                }

                // Suggestions list
                if filtered_suggestions.is_empty() {
                    ui.centered_and_justified(|ui| {
                        if table_name.is_empty() {
                            ui.label(RichText::new(self.i18n.no_suggestions_available()).color(AppColors::TEXT_MUTED));
                        } else {
                            let msg = if is_japanese { "この条件に一致する提案はありません" } else { "No suggestions match the current filters" };
                            ui.label(RichText::new(msg).color(AppColors::TEXT_MUTED));
                        }
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for suggestion in filtered_suggestions {
                                let title = if is_japanese { &suggestion.title_ja } else { &suggestion.title };
                                let description = if is_japanese { &suggestion.description_ja } else { &suggestion.description };

                                egui::Frame::new()
                                    .fill(AppColors::BG_SECONDARY)
                                    .corner_radius(Radius::SM)
                                    .inner_margin(egui::Margin::same(10))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Category icon and title
                                            ui.label(RichText::new(suggestion.category.icon()).size(14.0));
                                            ui.label(RichText::new(title).strong().color(AppColors::TEXT_PRIMARY));

                                            // Safety indicator
                                            if !suggestion.is_safe {
                                                ui.label(RichText::new("⚠").color(AppColors::WARNING));
                                            }

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button(self.i18n.use_this_query()).clicked() {
                                                    use_sql = Some(suggestion.sql.clone());
                                                }
                                            });
                                        });

                                        ui.label(RichText::new(description).size(11.0).color(AppColors::TEXT_SECONDARY));
                                        ui.add_space(Spacing::XS);

                                        // SQL preview
                                        egui::Frame::new()
                                            .fill(AppColors::BG_PRIMARY)
                                            .corner_radius(Radius::XS)
                                            .inner_margin(egui::Margin::same(6))
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(&suggestion.sql).monospace().size(11.0).color(AppColors::TEXT_PRIMARY));
                                            });

                                        // Warning for modification queries
                                        if !suggestion.is_safe {
                                            ui.add_space(Spacing::XS);
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("⚠").size(11.0).color(AppColors::WARNING));
                                                ui.label(RichText::new(self.i18n.unsafe_query_warning()).size(10.0).color(AppColors::WARNING));
                                            });
                                        }
                                    });
                                ui.add_space(Spacing::XS);
                            }
                        });
                }
            });

        if close_panel {
            self.show_ai_suggest_panel = false;
        }

        // Apply the selected SQL to the query editor
        if let Some(sql) = use_sql {
            if self.active_tab < self.tabs.len() {
                self.tabs[self.active_tab].sql_query = sql;
                self.show_ai_suggest_panel = false;
            }
        }
    }

    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        use crate::version;

        if !self.show_about_dialog {
            return;
        }

        let mut close_dialog = false;
        let mut start_update_check = false;
        let mut open_release_url: Option<String> = None;
        let mut open_download_url: Option<String> = None;

        egui::Window::new(self.i18n.about())
            .id(egui::Id::new("about_dialog"))
            .default_width(400.0)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(Spacing::MD);

                    // App icon placeholder
                    ui.label(RichText::new("D1").size(48.0).strong().color(AppColors::PRIMARY));
                    ui.label(RichText::new("Manager").size(24.0).color(AppColors::TEXT_PRIMARY));

                    ui.add_space(Spacing::MD);

                    // Version info
                    ui.label(RichText::new(version::build_info()).size(14.0).color(AppColors::TEXT_SECONDARY));

                    ui.add_space(Spacing::SM);

                    ui.label(
                        RichText::new("Production-safe Cloudflare D1 GUI Client")
                            .size(12.0)
                            .color(AppColors::TEXT_MUTED)
                    );

                    ui.add_space(Spacing::LG);
                });

                ui.separator();
                ui.add_space(Spacing::SM);

                // Update check section
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.i18n.current_version()).color(AppColors::TEXT_SECONDARY));
                    ui.label(RichText::new(version::VERSION).strong().color(AppColors::TEXT_PRIMARY));
                });

                ui.add_space(Spacing::SM);

                if self.update_check_in_progress {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(self.i18n.checking_updates());
                    });
                } else if let Some(ref result) = self.update_info {
                    match result {
                        Ok(info) => {
                            if info.is_update_available {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("🎉").size(16.0));
                                    ui.label(
                                        RichText::new(self.i18n.update_available(&info.latest_version))
                                            .color(AppColors::SUCCESS)
                                            .strong()
                                    );
                                });

                                ui.add_space(Spacing::SM);

                                // Release notes preview
                                if let Some(ref notes) = info.release_notes {
                                    ui.collapsing(self.i18n.release_notes(), |ui| {
                                        egui::ScrollArea::vertical()
                                            .max_height(150.0)
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(notes).size(11.0).color(AppColors::TEXT_SECONDARY));
                                            });
                                    });
                                    ui.add_space(Spacing::SM);
                                }

                                ui.horizontal(|ui| {
                                    if let Some(ref url) = info.download_url {
                                        if ui.button(RichText::new(format!("⬇ {}", self.i18n.download_update())).color(AppColors::PRIMARY)).clicked() {
                                            open_download_url = Some(url.clone());
                                        }
                                    }

                                    if !info.release_url.is_empty() {
                                        if ui.button(self.i18n.view_release()).clicked() {
                                            open_release_url = Some(info.release_url.clone());
                                        }
                                    }
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("✓").size(16.0).color(AppColors::SUCCESS));
                                    ui.label(RichText::new(self.i18n.up_to_date()).color(AppColors::SUCCESS));
                                });
                            }
                        }
                        Err(err) => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("⚠").color(AppColors::WARNING));
                                ui.label(RichText::new(self.i18n.update_check_failed()).color(AppColors::WARNING));
                            });
                            ui.label(RichText::new(err).size(10.0).color(AppColors::TEXT_MUTED));
                        }
                    }
                } else {
                    if ui.button(self.i18n.check_for_updates()).clicked() {
                        start_update_check = true;
                    }
                }

                ui.add_space(Spacing::LG);

                // Close button
                ui.vertical_centered(|ui| {
                    if ui.button(self.i18n.close()).clicked() {
                        close_dialog = true;
                    }
                });

                ui.add_space(Spacing::SM);
            });

        if close_dialog {
            self.show_about_dialog = false;
        }

        if start_update_check {
            self.start_update_check();
        }

        // Open URLs in browser
        if let Some(url) = open_release_url {
            let _ = open::that(&url);
        }
        if let Some(url) = open_download_url {
            let _ = open::that(&url);
        }
    }

    fn start_update_check(&mut self) {
        use crate::version;

        self.update_check_in_progress = true;
        self.update_info = None;

        let sender = self.sender.clone();

        self.runtime.spawn(async move {
            let result = version::check_for_updates().await;
            let _ = sender.send(Message::UpdateCheckResult(result));
        });
    }
}

async fn fetch_database_schema(client: &D1Client) -> Result<DatabaseSchema, String> {
    // Get list of tables using sqlite_master
    let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_cf_%' ORDER BY name";
    let response = client.execute(sql, vec![]).await.map_err(|e| e.to_string())?;

    let mut schema = DatabaseSchema::new();

    let tables: Vec<String> = response
        .result
        .first()
        .and_then(|r| r.results.as_ref())
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    for table_name in tables {
        // Get column info using PRAGMA table_info
        let pragma_sql = format!("PRAGMA table_info('{}')", table_name);
        let col_response = client.execute(&pragma_sql, vec![]).await.map_err(|e| e.to_string())?;

        let columns: Vec<ColumnSchema> = col_response
            .result
            .first()
            .and_then(|r| r.results.as_ref())
            .map(|rows| {
                rows.iter()
                    .map(|row| {
                        ColumnSchema {
                            name: row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            col_type: row.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            notnull: row.get("notnull").and_then(|v| v.as_i64()).unwrap_or(0) == 1,
                            pk: row.get("pk").and_then(|v| v.as_i64()).unwrap_or(0) == 1,
                            default_value: row.get("dflt_value").and_then(|v| {
                                if v.is_null() { None } else { v.as_str().map(|s| s.to_string()) }
                            }),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        schema.add_table(TableSchema {
            name: table_name,
            columns,
        });
    }

    Ok(schema)
}

/// Fetch database schema from a local SQLite database
fn fetch_local_database_schema(path: &str) -> Result<DatabaseSchema, String> {
    use crate::local_db::LocalD1Client;

    let client = LocalD1Client::new(std::path::PathBuf::from(path));

    // Get list of tables
    let tables = client.get_tables().map_err(|e| e.to_string())?;

    let mut schema = DatabaseSchema::new();

    for table_name in tables {
        // Get column info
        let col_info = client.get_table_schema(&table_name).map_err(|e| e.to_string())?;

        let columns: Vec<ColumnSchema> = col_info
            .iter()
            .map(|col| {
                ColumnSchema {
                    name: col.name.clone(),
                    col_type: col.col_type.clone(),
                    notnull: col.notnull,
                    pk: col.pk,
                    default_value: None, // LocalColumnInfo doesn't have default_value currently
                }
            })
            .collect();

        schema.add_table(TableSchema {
            name: table_name,
            columns,
        });
    }

    Ok(schema)
}

fn format_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let time = UNIX_EPOCH + Duration::from_secs(timestamp);
    let now = SystemTime::now();

    if let Ok(elapsed) = now.duration_since(time) {
        let secs = elapsed.as_secs();
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    } else {
        "just now".to_string()
    }
}

fn chrono_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

// ============================================================================
// Onboarding Wizard Implementation
// ============================================================================

impl D1ManagerApp {
    fn render_onboarding_wizard(&mut self, ctx: &egui::Context) {
        if !self.show_onboarding_wizard {
            return;
        }

        let mut close_wizard = false;
        let mut complete_setup = false;

        egui::Window::new(self.i18n.wizard_title())
            .collapsible(false)
            .resizable(false)
            .fixed_size([600.0, 500.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // Progress indicator
                self.render_wizard_progress(ui);
                ui.add_space(Spacing::LG);
                ui.separator();
                ui.add_space(Spacing::MD);

                // Step content
                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.onboarding_state.current_step {
                        OnboardingStep::Welcome => self.render_wizard_welcome(ui),
                        OnboardingStep::ApiToken => self.render_wizard_api_token(ui),
                        OnboardingStep::SelectAccount => self.render_wizard_select_account(ui),
                        OnboardingStep::SelectDatabase => self.render_wizard_select_database(ui),
                        OnboardingStep::SelectLocalDatabase => self.render_wizard_select_local_database(ui),
                        OnboardingStep::ConfigureEnv => self.render_wizard_configure_env(ui),
                        OnboardingStep::TestConnection => {
                            complete_setup = self.render_wizard_test_connection(ui);
                        }
                    }
                    ui.add_space(Spacing::LG);
                });

                ui.separator();
                ui.add_space(Spacing::SM);

                // Navigation buttons
                ui.horizontal(|ui| {
                    // Skip wizard button (only on welcome screen)
                    if self.onboarding_state.current_step == OnboardingStep::Welcome {
                        if theme::secondary_button(ui, self.i18n.wizard_skip()).clicked() {
                            close_wizard = true;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Next/Complete button
                        let (next_text, can_proceed) = match self.onboarding_state.current_step {
                            OnboardingStep::Welcome => (self.i18n.wizard_next(), true),
                            OnboardingStep::ApiToken => (self.i18n.wizard_next(), !self.onboarding_state.api_token.is_empty()),
                            OnboardingStep::SelectAccount => (self.i18n.wizard_next(), self.onboarding_state.selected_account_idx.is_some()),
                            OnboardingStep::SelectDatabase => (self.i18n.wizard_next(), self.onboarding_state.selected_database_idx.is_some()),
                            OnboardingStep::SelectLocalDatabase => (self.i18n.wizard_next(), self.onboarding_state.selected_local_db_idx.is_some()),
                            OnboardingStep::ConfigureEnv => (self.i18n.wizard_next(), !self.onboarding_state.connection_name.is_empty()),
                            OnboardingStep::TestConnection => {
                                if self.onboarding_state.test_result.as_ref().map(|(s, _)| *s).unwrap_or(false) {
                                    (self.i18n.wizard_complete_setup(), true)
                                } else {
                                    (self.i18n.wizard_test_connection(), !self.onboarding_state.test_in_progress)
                                }
                            }
                        };

                        ui.add_enabled_ui(can_proceed, |ui| {
                            if theme::primary_button(ui, next_text).clicked() {
                                self.wizard_handle_next();
                            }
                        });

                        // Back button
                        if self.onboarding_state.current_step.can_go_back() {
                            ui.add_space(Spacing::SM);
                            if theme::secondary_button(ui, self.i18n.wizard_back()).clicked() {
                                let local_mode = self.onboarding_state.wizard_mode == WizardMode::Local;
                                if let Some(prev) = self.onboarding_state.current_step.prev(local_mode) {
                                    self.onboarding_state.current_step = prev;
                                }
                            }
                        }
                    });
                });
            });

        if close_wizard {
            self.show_onboarding_wizard = false;
            self.onboarding_state = OnboardingWizardState::default();
            self.show_settings = true;
        }

        if complete_setup {
            self.wizard_complete_setup();
            self.show_onboarding_wizard = false;
            self.onboarding_state = OnboardingWizardState::default();
        }
    }

    fn render_wizard_progress(&self, ui: &mut egui::Ui) {
        let local_mode = self.onboarding_state.wizard_mode == WizardMode::Local;
        let current = if local_mode {
            self.onboarding_state.current_step.local_index()
        } else {
            self.onboarding_state.current_step.index()
        };
        let total = OnboardingStep::total_steps(local_mode);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - (total as f32 * 28.0 + (total - 1) as f32 * 38.0)) / 2.0);

            for i in 0..total {
                let (bg_color, text_color) = if i < current {
                    (AppColors::SUCCESS, AppColors::TEXT_PRIMARY)
                } else if i == current {
                    (AppColors::PRIMARY, AppColors::TEXT_PRIMARY)
                } else {
                    (AppColors::BG_TERTIARY, AppColors::TEXT_MUTED)
                };

                let size = 28.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(size, size),
                    egui::Sense::hover()
                );

                ui.painter().circle_filled(rect.center(), size / 2.0, bg_color);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", i + 1),
                    egui::FontId::proportional(12.0),
                    text_color,
                );

                if i < total - 1 {
                    ui.add_space(4.0);
                    let line_color = if i < current { AppColors::SUCCESS } else { AppColors::BG_TERTIARY };
                    let (line_rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 2.0), egui::Sense::hover());
                    ui.painter().rect_filled(line_rect, 0.0, line_color);
                    ui.add_space(4.0);
                }
            }
        });
    }

    fn render_wizard_welcome(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(Spacing::LG);
            ui.label(RichText::new(self.i18n.wizard_welcome_title())
                .size(24.0).strong().color(AppColors::TEXT_PRIMARY));
            ui.add_space(Spacing::SM);
            ui.label(RichText::new(self.i18n.wizard_welcome_desc())
                .size(14.0).color(AppColors::TEXT_SECONDARY));
        });

        ui.add_space(Spacing::XL);

        ui.label(RichText::new(self.i18n.wizard_choose_mode())
            .size(14.0).color(AppColors::TEXT_PRIMARY));
        ui.add_space(Spacing::MD);

        // Remote mode option
        let remote_selected = self.onboarding_state.wizard_mode == WizardMode::Remote;
        let remote_bg = if remote_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
        let remote_border = if remote_selected { AppColors::PRIMARY } else { AppColors::BORDER };

        egui::Frame::new()
            .fill(remote_bg)
            .stroke(Stroke::new(2.0, remote_border))
            .corner_radius(Radius::MD)
            .inner_margin(egui::Margin::same(Spacing::MD as i8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if ui.add(egui::Button::new(
                    RichText::new(format!("☁ {}", self.i18n.wizard_remote_mode()))
                        .size(16.0).color(if remote_selected { AppColors::PRIMARY } else { AppColors::TEXT_PRIMARY })
                ).frame(false)).clicked() {
                    self.onboarding_state.wizard_mode = WizardMode::Remote;
                }
                ui.label(RichText::new(self.i18n.wizard_remote_desc())
                    .size(12.0).color(AppColors::TEXT_SECONDARY));
            });

        ui.add_space(Spacing::SM);

        // Local mode option
        let local_selected = self.onboarding_state.wizard_mode == WizardMode::Local;
        let local_bg = if local_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
        let local_border = if local_selected { AppColors::PRIMARY } else { AppColors::BORDER };

        egui::Frame::new()
            .fill(local_bg)
            .stroke(Stroke::new(2.0, local_border))
            .corner_radius(Radius::MD)
            .inner_margin(egui::Margin::same(Spacing::MD as i8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if ui.add(egui::Button::new(
                    RichText::new(format!("💾 {}", self.i18n.wizard_local_mode()))
                        .size(16.0).color(if local_selected { AppColors::PRIMARY } else { AppColors::TEXT_PRIMARY })
                ).frame(false)).clicked() {
                    self.onboarding_state.wizard_mode = WizardMode::Local;
                }
                ui.label(RichText::new(self.i18n.wizard_local_desc())
                    .size(12.0).color(AppColors::TEXT_SECONDARY));
            });
    }

    fn render_wizard_api_token(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(self.i18n.wizard_step1_title())
            .size(18.0).strong().color(AppColors::TEXT_PRIMARY));
        ui.add_space(Spacing::SM);
        ui.label(RichText::new(self.i18n.wizard_step1_desc())
            .color(AppColors::TEXT_SECONDARY));
        ui.add_space(Spacing::LG);

        // Required scopes info box
        egui::Frame::new()
            .fill(AppColors::BG_TERTIARY)
            .corner_radius(Radius::MD)
            .inner_margin(egui::Margin::same(Spacing::SM as i8))
            .show(ui, |ui| {
                ui.label(RichText::new(self.i18n.wizard_required_scopes())
                    .size(13.0).color(AppColors::TEXT_PRIMARY));
                ui.add_space(Spacing::XS);
                for scope in ["Account:Read", "Account:D1:Read", "Account:D1:Edit (optional for write access)"] {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("•").color(AppColors::PRIMARY));
                        ui.label(RichText::new(scope).monospace().size(12.0).color(AppColors::TEXT_SECONDARY));
                    });
                }
            });

        ui.add_space(Spacing::MD);

        // Token input
        ui.label(RichText::new(self.i18n.api_token()).color(AppColors::TEXT_SECONDARY));
        ui.add(
            egui::TextEdit::singleline(&mut self.onboarding_state.api_token)
                .password(true)
                .hint_text("Enter your Cloudflare API token...")
                .desired_width(ui.available_width())
        );

        ui.add_space(Spacing::SM);

        // Link to create token
        ui.horizontal(|ui| {
            ui.label(RichText::new(self.i18n.create_tokens_at())
                .size(11.0).color(AppColors::TEXT_MUTED));
            if ui.link(self.i18n.wizard_open_dashboard()).clicked() {
                let _ = open::that("https://dash.cloudflare.com/profile/api-tokens");
            }
        });

        // Show error if any
        if let Some(ref error) = self.onboarding_state.accounts_error {
            ui.add_space(Spacing::MD);
            egui::Frame::new()
                .fill(AppColors::ERROR.gamma_multiply(0.2))
                .corner_radius(Radius::MD)
                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                .show(ui, |ui| {
                    ui.label(RichText::new(error).color(AppColors::ERROR));
                });
        }
    }

    fn render_wizard_select_account(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(self.i18n.wizard_step2_title())
            .size(18.0).strong().color(AppColors::TEXT_PRIMARY));
        ui.add_space(Spacing::SM);
        ui.label(RichText::new(self.i18n.wizard_step2_desc())
            .color(AppColors::TEXT_SECONDARY));
        ui.add_space(Spacing::LG);

        if self.onboarding_state.accounts_loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(self.i18n.wizard_loading_accounts())
                    .color(AppColors::TEXT_MUTED));
            });
        } else if let Some(ref error) = self.onboarding_state.accounts_error {
            egui::Frame::new()
                .fill(AppColors::ERROR.gamma_multiply(0.2))
                .corner_radius(Radius::MD)
                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                .show(ui, |ui| {
                    ui.label(RichText::new(error).color(AppColors::ERROR));
                });
            ui.add_space(Spacing::SM);
            if theme::secondary_button(ui, self.i18n.wizard_retry()).clicked() {
                self.wizard_fetch_accounts();
            }
        } else if self.onboarding_state.accounts.is_empty() {
            ui.label(RichText::new(self.i18n.wizard_no_accounts())
                .color(AppColors::TEXT_MUTED));
        } else {
            for (i, account) in self.onboarding_state.accounts.iter().enumerate() {
                let is_selected = self.onboarding_state.selected_account_idx == Some(i);
                let bg = if is_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
                let border = if is_selected { AppColors::PRIMARY } else { AppColors::BORDER };

                egui::Frame::new()
                    .fill(bg)
                    .stroke(Stroke::new(1.0, border))
                    .corner_radius(Radius::MD)
                    .inner_margin(egui::Margin::same(Spacing::SM as i8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if ui.add(egui::Button::new(
                            RichText::new(&account.name)
                                .color(if is_selected { AppColors::PRIMARY } else { AppColors::TEXT_PRIMARY })
                        ).frame(false)).clicked() {
                            self.onboarding_state.selected_account_idx = Some(i);
                            // Reset downstream selections
                            self.onboarding_state.databases.clear();
                            self.onboarding_state.selected_database_idx = None;
                        }
                        ui.label(RichText::new(&account.id)
                            .size(11.0).monospace().color(AppColors::TEXT_MUTED));
                    });
                ui.add_space(Spacing::XS);
            }
        }
    }

    fn render_wizard_select_database(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(self.i18n.wizard_step3_title())
            .size(18.0).strong().color(AppColors::TEXT_PRIMARY));
        ui.add_space(Spacing::SM);
        ui.label(RichText::new(self.i18n.wizard_step3_desc())
            .color(AppColors::TEXT_SECONDARY));
        ui.add_space(Spacing::LG);

        if self.onboarding_state.databases_loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(self.i18n.wizard_loading_databases())
                    .color(AppColors::TEXT_MUTED));
            });
        } else if let Some(ref error) = self.onboarding_state.databases_error {
            egui::Frame::new()
                .fill(AppColors::ERROR.gamma_multiply(0.2))
                .corner_radius(Radius::MD)
                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                .show(ui, |ui| {
                    ui.label(RichText::new(error).color(AppColors::ERROR));
                });
            ui.add_space(Spacing::SM);
            if theme::secondary_button(ui, self.i18n.wizard_retry()).clicked() {
                self.wizard_fetch_databases();
            }
        } else if self.onboarding_state.databases.is_empty() {
            ui.label(RichText::new(self.i18n.wizard_no_databases())
                .color(AppColors::TEXT_MUTED));
        } else {
            for (i, db) in self.onboarding_state.databases.iter().enumerate() {
                let is_selected = self.onboarding_state.selected_database_idx == Some(i);
                let bg = if is_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
                let border = if is_selected { AppColors::PRIMARY } else { AppColors::BORDER };

                egui::Frame::new()
                    .fill(bg)
                    .stroke(Stroke::new(1.0, border))
                    .corner_radius(Radius::MD)
                    .inner_margin(egui::Margin::same(Spacing::SM as i8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if ui.add(egui::Button::new(
                            RichText::new(&db.name)
                                .color(if is_selected { AppColors::PRIMARY } else { AppColors::TEXT_PRIMARY })
                        ).frame(false)).clicked() {
                            self.onboarding_state.selected_database_idx = Some(i);
                            // Auto-fill connection name
                            if self.onboarding_state.connection_name.is_empty() {
                                self.onboarding_state.connection_name = db.name.clone();
                            }
                        }
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&db.uuid)
                                .size(11.0).monospace().color(AppColors::TEXT_MUTED));
                            ui.label(RichText::new(format!("• {}", &db.created_at[..10.min(db.created_at.len())]))
                                .size(11.0).color(AppColors::TEXT_MUTED));
                        });
                    });
                ui.add_space(Spacing::XS);
            }
        }
    }

    fn render_wizard_select_local_database(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(self.i18n.wizard_local_step_title())
            .size(18.0).strong().color(AppColors::TEXT_PRIMARY));
        ui.add_space(Spacing::SM);
        ui.label(RichText::new(self.i18n.wizard_local_step_desc())
            .color(AppColors::TEXT_SECONDARY));
        ui.add_space(Spacing::LG);

        if self.onboarding_state.local_db_scanning {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(self.i18n.wizard_scanning_local())
                    .color(AppColors::TEXT_MUTED));
            });
        } else if self.onboarding_state.local_databases.is_empty() {
            egui::Frame::new()
                .fill(AppColors::WARNING.gamma_multiply(0.2))
                .corner_radius(Radius::MD)
                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                .show(ui, |ui| {
                    ui.label(RichText::new(self.i18n.wizard_no_local_databases())
                        .color(AppColors::WARNING));
                });
            ui.add_space(Spacing::MD);

            ui.horizontal(|ui| {
                // Rescan button
                if theme::secondary_button(ui, self.i18n.wizard_rescan()).clicked() {
                    self.wizard_scan_local_databases();
                }
                ui.add_space(Spacing::SM);

                // Browse button for manual selection
                if theme::secondary_button(ui, self.i18n.wizard_browse_file()).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("SQLite Database", &["sqlite", "db", "sqlite3"])
                        .pick_file()
                    {
                        // Add as a manual local database
                        let local_db = crate::local_db::LocalD1Database {
                            name: path.file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Local DB".to_string()),
                            binding: "manual".to_string(),
                            path: path.clone(),
                            project_path: path.parent().unwrap_or(&path).to_path_buf(),
                        };
                        self.onboarding_state.local_databases.push(local_db);
                        self.onboarding_state.selected_local_db_idx = Some(self.onboarding_state.local_databases.len() - 1);
                        if self.onboarding_state.connection_name.is_empty() {
                            self.onboarding_state.connection_name = path.file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Local DB".to_string());
                        }
                    }
                }
            });

            ui.add_space(Spacing::LG);
            ui.separator();
            ui.add_space(Spacing::MD);

            // Create new database section
            ui.label(RichText::new(self.i18n.wizard_create_new_db_title())
                .size(14.0).strong().color(AppColors::TEXT_PRIMARY));
            ui.add_space(Spacing::SM);
            ui.label(RichText::new(self.i18n.wizard_create_new_db_desc())
                .size(12.0).color(AppColors::TEXT_SECONDARY));
            ui.add_space(Spacing::SM);

            if theme::primary_button(ui, self.i18n.wizard_create_new_db()).clicked() {
                self.wizard_create_new_local_database();
            }
        } else {
            // Display found databases
            for (i, db) in self.onboarding_state.local_databases.iter().enumerate() {
                let is_selected = self.onboarding_state.selected_local_db_idx == Some(i);
                let bg = if is_selected { AppColors::PRIMARY.gamma_multiply(0.2) } else { AppColors::BG_TERTIARY };
                let border = if is_selected { AppColors::PRIMARY } else { AppColors::BORDER };

                egui::Frame::new()
                    .fill(bg)
                    .stroke(Stroke::new(1.0, border))
                    .corner_radius(Radius::MD)
                    .inner_margin(egui::Margin::same(Spacing::SM as i8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if ui.add(egui::Button::new(
                            RichText::new(&db.name)
                                .color(if is_selected { AppColors::PRIMARY } else { AppColors::TEXT_PRIMARY })
                        ).frame(false)).clicked() {
                            self.onboarding_state.selected_local_db_idx = Some(i);
                            // Auto-fill connection name
                            if self.onboarding_state.connection_name.is_empty() {
                                self.onboarding_state.connection_name = db.binding.clone();
                            }
                        }
                        ui.label(RichText::new(db.path.display().to_string())
                            .size(11.0).monospace().color(AppColors::TEXT_MUTED));
                    });
                ui.add_space(Spacing::XS);
            }

            ui.add_space(Spacing::MD);

            // Rescan, browse, and create buttons
            ui.horizontal(|ui| {
                if theme::secondary_button(ui, self.i18n.wizard_rescan()).clicked() {
                    self.wizard_scan_local_databases();
                }
                ui.add_space(Spacing::SM);
                if theme::secondary_button(ui, self.i18n.wizard_browse_file()).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("SQLite Database", &["sqlite", "db", "sqlite3"])
                        .pick_file()
                    {
                        let local_db = crate::local_db::LocalD1Database {
                            name: path.file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Local DB".to_string()),
                            binding: "manual".to_string(),
                            path: path.clone(),
                            project_path: path.parent().unwrap_or(&path).to_path_buf(),
                        };
                        self.onboarding_state.local_databases.push(local_db);
                        self.onboarding_state.selected_local_db_idx = Some(self.onboarding_state.local_databases.len() - 1);
                        if self.onboarding_state.connection_name.is_empty() {
                            self.onboarding_state.connection_name = path.file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Local DB".to_string());
                        }
                    }
                }
                ui.add_space(Spacing::SM);
                if theme::primary_button(ui, self.i18n.wizard_create_new_db()).clicked() {
                    self.wizard_create_new_local_database();
                }
            });
        }
    }

    fn render_wizard_configure_env(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(self.i18n.wizard_step4_title())
            .size(18.0).strong().color(AppColors::TEXT_PRIMARY));
        ui.add_space(Spacing::SM);
        ui.label(RichText::new(self.i18n.wizard_step4_desc())
            .color(AppColors::TEXT_SECONDARY));
        ui.add_space(Spacing::LG);

        // Connection name
        ui.label(RichText::new(self.i18n.connection_name()).color(AppColors::TEXT_SECONDARY));
        ui.add(
            egui::TextEdit::singleline(&mut self.onboarding_state.connection_name)
                .hint_text("My Database")
                .desired_width(ui.available_width())
        );

        ui.add_space(Spacing::MD);

        // Environment type selector
        ui.label(RichText::new(self.i18n.environment()).size(13.0).color(AppColors::TEXT_SECONDARY));
        ui.add_space(Spacing::XS);
        ui.horizontal(|ui| {
            for env_type in [EnvironmentType::Development, EnvironmentType::Staging, EnvironmentType::Production] {
                let is_selected = self.onboarding_state.environment == env_type;
                let (bg, text_color) = if is_selected {
                    (env_type.color().gamma_multiply(0.3), env_type.color())
                } else {
                    (AppColors::BG_TERTIARY, AppColors::TEXT_SECONDARY)
                };

                let btn = egui::Button::new(
                    RichText::new(env_type.label()).color(text_color)
                )
                .fill(bg)
                .stroke(Stroke::new(1.0, if is_selected { env_type.color() } else { AppColors::BORDER }))
                .corner_radius(Radius::MD);

                if ui.add(btn).clicked() {
                    self.onboarding_state.environment = env_type;
                    // Auto-enable read-only for production
                    if env_type == EnvironmentType::Production {
                        self.onboarding_state.read_only = true;
                    }
                }
            }
        });
        ui.add_space(Spacing::XS);
        ui.label(RichText::new(self.i18n.env_confirmation_note()).size(11.0).color(AppColors::TEXT_MUTED));

        ui.add_space(Spacing::MD);

        // Read-only toggle
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.onboarding_state.read_only, "");
            ui.label(RichText::new(self.i18n.read_only_mode()).color(AppColors::TEXT_PRIMARY));
        });
        ui.label(RichText::new(self.i18n.read_only_description()).size(11.0).color(AppColors::TEXT_MUTED));
    }

    fn render_wizard_test_connection(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label(RichText::new(self.i18n.wizard_step5_title())
            .size(18.0).strong().color(AppColors::TEXT_PRIMARY));
        ui.add_space(Spacing::SM);
        ui.label(RichText::new(self.i18n.wizard_step5_desc())
            .color(AppColors::TEXT_SECONDARY));
        ui.add_space(Spacing::LG);

        // Summary
        egui::Frame::new()
            .fill(AppColors::BG_TERTIARY)
            .corner_radius(Radius::MD)
            .inner_margin(egui::Margin::same(Spacing::MD as i8))
            .show(ui, |ui| {
                ui.label(RichText::new(&self.onboarding_state.connection_name)
                    .size(16.0).strong().color(AppColors::TEXT_PRIMARY));
                ui.add_space(Spacing::XS);

                if let Some(idx) = self.onboarding_state.selected_account_idx {
                    if let Some(account) = self.onboarding_state.accounts.get(idx) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Account:").color(AppColors::TEXT_MUTED));
                            ui.label(RichText::new(&account.name).color(AppColors::TEXT_SECONDARY));
                        });
                    }
                }

                if let Some(idx) = self.onboarding_state.selected_database_idx {
                    if let Some(db) = self.onboarding_state.databases.get(idx) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Database:").color(AppColors::TEXT_MUTED));
                            ui.label(RichText::new(&db.name).color(AppColors::TEXT_SECONDARY));
                        });
                    }
                }

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Environment:").color(AppColors::TEXT_MUTED));
                    let env_color = self.onboarding_state.environment.color();
                    ui.label(RichText::new(self.onboarding_state.environment.label()).color(env_color));
                });

                if self.onboarding_state.read_only {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔒").size(12.0));
                        ui.label(RichText::new(self.i18n.read_only_mode()).size(12.0).color(AppColors::TEXT_MUTED));
                    });
                }
            });

        ui.add_space(Spacing::MD);

        // Test result
        if self.onboarding_state.test_in_progress {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new(self.i18n.wizard_testing()).color(AppColors::TEXT_MUTED));
            });
        } else if let Some((success, ref msg)) = self.onboarding_state.test_result {
            let (color, bg_color, icon) = if success {
                (AppColors::SUCCESS, AppColors::SUCCESS.gamma_multiply(0.2), "✓")
            } else {
                (AppColors::ERROR, AppColors::ERROR.gamma_multiply(0.2), "✗")
            };
            egui::Frame::new()
                .fill(bg_color)
                .corner_radius(Radius::MD)
                .inner_margin(egui::Margin::same(Spacing::SM as i8))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("{} {}", icon, msg)).color(color));
                });

            // Return true if test succeeded (allows completing setup)
            return success;
        }

        false
    }

    fn wizard_handle_next(&mut self) {
        match self.onboarding_state.current_step {
            OnboardingStep::Welcome => {
                if self.onboarding_state.wizard_mode == WizardMode::Local {
                    // Go to local database selection and scan for databases
                    self.wizard_scan_local_databases();
                    self.onboarding_state.current_step = OnboardingStep::SelectLocalDatabase;
                } else {
                    self.onboarding_state.current_step = OnboardingStep::ApiToken;
                }
            }
            OnboardingStep::ApiToken => {
                // Fetch accounts when moving to next step
                self.wizard_fetch_accounts();
                self.onboarding_state.current_step = OnboardingStep::SelectAccount;
            }
            OnboardingStep::SelectAccount => {
                // Fetch databases for selected account
                self.wizard_fetch_databases();
                self.onboarding_state.current_step = OnboardingStep::SelectDatabase;
            }
            OnboardingStep::SelectDatabase => {
                self.onboarding_state.current_step = OnboardingStep::ConfigureEnv;
            }
            OnboardingStep::SelectLocalDatabase => {
                // Proceed to environment config for local db
                self.onboarding_state.current_step = OnboardingStep::ConfigureEnv;
            }
            OnboardingStep::ConfigureEnv => {
                self.onboarding_state.current_step = OnboardingStep::TestConnection;
            }
            OnboardingStep::TestConnection => {
                // Test connection or complete setup
                if self.onboarding_state.test_result.as_ref().map(|(s, _)| *s).unwrap_or(false) {
                    // Test succeeded, complete setup
                    self.wizard_complete_setup();
                    self.show_onboarding_wizard = false;
                    self.onboarding_state = OnboardingWizardState::default();
                } else {
                    // Run test
                    self.wizard_test_connection();
                }
            }
        }
    }

    fn wizard_fetch_accounts(&mut self) {
        if self.onboarding_state.api_token.is_empty() {
            return;
        }

        self.onboarding_state.accounts_loading = true;
        self.onboarding_state.accounts_error = None;

        let token = self.onboarding_state.api_token.clone();
        let sender = self.sender.clone();

        self.runtime.spawn(async move {
            let result = crate::api::list_accounts(&token).await;
            let _ = sender.send(Message::AccountsLoaded(result.map(|accounts| {
                accounts.into_iter()
                    .map(|a| CloudflareAccount { id: a.id, name: a.name })
                    .collect()
            })));
        });
    }

    fn wizard_fetch_databases(&mut self) {
        let account_idx = match self.onboarding_state.selected_account_idx {
            Some(idx) => idx,
            None => return,
        };

        let account_id = match self.onboarding_state.accounts.get(account_idx) {
            Some(account) => account.id.clone(),
            None => return,
        };

        self.onboarding_state.databases_loading = true;
        self.onboarding_state.databases_error = None;

        let token = self.onboarding_state.api_token.clone();
        let sender = self.sender.clone();

        self.runtime.spawn(async move {
            let result = crate::api::list_databases(&token, &account_id).await;
            let _ = sender.send(Message::DatabasesListLoaded(result.map(|databases| {
                databases.into_iter()
                    .map(|d| CloudflareDatabase {
                        uuid: d.uuid,
                        name: d.name,
                        created_at: d.created_at,
                    })
                    .collect()
            })));
        });
    }

    fn wizard_test_connection(&mut self) {
        let is_local = self.onboarding_state.wizard_mode == WizardMode::Local;

        if is_local {
            // Test local connection
            self.wizard_test_local_connection();
        } else {
            // Test remote connection
            let account_idx = match self.onboarding_state.selected_account_idx {
                Some(idx) => idx,
                None => return,
            };
            let db_idx = match self.onboarding_state.selected_database_idx {
                Some(idx) => idx,
                None => return,
            };

            let account_id = match self.onboarding_state.accounts.get(account_idx) {
                Some(account) => account.id.clone(),
                None => return,
            };
            let database_id = match self.onboarding_state.databases.get(db_idx) {
                Some(db) => db.uuid.clone(),
                None => return,
            };

            self.onboarding_state.test_in_progress = true;
            self.onboarding_state.test_result = None;

            let token = self.onboarding_state.api_token.clone();
            let sender = self.sender.clone();

            self.runtime.spawn(async move {
                let client = crate::api::D1Client::new(account_id, database_id, token);
                let result = client.execute("SELECT 1", vec![]).await;
                let _ = sender.send(Message::OnboardingTestResult(
                    result.map(|_| "Connection successful!".to_string())
                ));
            });
        }
    }

    fn wizard_scan_local_databases(&mut self) {
        self.onboarding_state.local_db_scanning = true;
        self.onboarding_state.local_databases.clear();

        let sender = self.sender.clone();

        // Scan for local databases in a background thread
        std::thread::spawn(move || {
            let databases = crate::local_db::find_local_d1_databases();
            let _ = sender.send(Message::LocalDatabasesScanned(databases));
        });
    }

    fn wizard_create_new_local_database(&mut self) {
        // Let user pick a folder
        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Select folder for new database")
            .pick_folder()
        {
            // Generate a unique database name
            let db_name = format!("local_d1_{}.sqlite", chrono_timestamp());
            let db_path = folder.join(&db_name);

            // Create the database
            match crate::local_db::LocalD1Client::create_new(&db_path) {
                Ok(_) => {
                    // Add to the list
                    let local_db = crate::local_db::LocalD1Database {
                        name: db_path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "New DB".to_string()),
                        binding: "created".to_string(),
                        path: db_path.clone(),
                        project_path: folder,
                    };
                    self.onboarding_state.local_databases.push(local_db);
                    self.onboarding_state.selected_local_db_idx = Some(self.onboarding_state.local_databases.len() - 1);
                    self.onboarding_state.connection_name = db_path.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "New Local DB".to_string());
                }
                Err(e) => {
                    // Show error in keychain_error for now
                    self.keychain_error = Some(self.i18n.translate_local_db_error(&e));
                }
            }
        }
    }

    fn wizard_test_local_connection(&mut self) {
        let db_idx = match self.onboarding_state.selected_local_db_idx {
            Some(idx) => idx,
            None => return,
        };

        let db = match self.onboarding_state.local_databases.get(db_idx) {
            Some(db) => db.clone(),
            None => return,
        };

        self.onboarding_state.test_in_progress = true;
        self.onboarding_state.test_result = None;

        let sender = self.sender.clone();
        let success_msg = self.i18n.connection_success().to_string();

        std::thread::spawn(move || {
            let client = crate::local_db::LocalD1Client::new(db.path);
            let result = client.execute("SELECT 1", vec![]);
            let _ = sender.send(Message::OnboardingTestResult(
                result.map(|_| success_msg).map_err(|e| e.to_string())
            ));
        });
    }

    fn wizard_complete_setup(&mut self) {
        let is_local = self.onboarding_state.wizard_mode == WizardMode::Local;

        if is_local {
            // Local database setup
            let db_idx = match self.onboarding_state.selected_local_db_idx {
                Some(idx) => idx,
                None => return,
            };

            let local_db = match self.onboarding_state.local_databases.get(db_idx) {
                Some(db) => db.clone(),
                None => return,
            };

            let profile = ConnectionProfile {
                id: uuid_simple(),
                name: self.onboarding_state.connection_name.clone(),
                account_id: String::new(),
                database_id: String::new(),
                api_token: SecureString::new(String::new()),
                environment: self.onboarding_state.environment,
                read_only: self.onboarding_state.read_only,
                connection_type: ConnectionType::Local,
                local_path: Some(local_db.path.to_string_lossy().to_string()),
            };

            // No keychain needed for local connections

            // Add to metadata
            self.profile_metadata.push(profile.to_metadata());
            self.save_profile_metadata();

            // Open the connection immediately
            self.open_connection(&profile.to_metadata());
        } else {
            // Remote database setup
            let account_idx = match self.onboarding_state.selected_account_idx {
                Some(idx) => idx,
                None => return,
            };
            let db_idx = match self.onboarding_state.selected_database_idx {
                Some(idx) => idx,
                None => return,
            };

            let account_id = match self.onboarding_state.accounts.get(account_idx) {
                Some(account) => account.id.clone(),
                None => return,
            };
            let database_id = match self.onboarding_state.databases.get(db_idx) {
                Some(db) => db.uuid.clone(),
                None => return,
            };

            // Create and save the connection profile
            let profile = ConnectionProfile {
                id: uuid_simple(),
                name: self.onboarding_state.connection_name.clone(),
                account_id,
                database_id,
                api_token: SecureString::new(self.onboarding_state.api_token.clone()),
                environment: self.onboarding_state.environment,
                read_only: self.onboarding_state.read_only,
                connection_type: ConnectionType::Remote,
                local_path: None,
            };

            // Save token to keychain
            self.save_profile_to_keychain(&profile);

            // Add to metadata
            self.profile_metadata.push(profile.to_metadata());
            self.save_profile_metadata();

            // Optionally open the connection immediately
            self.open_connection(&profile.to_metadata());
        }
    }
}

impl eframe::App for D1ManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_initialized {
            theme::configure_theme(ctx);
            self.theme_initialized = true;
        }

        self.process_messages();

        if self.tabs.iter().any(|t| t.loading) {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::new().fill(AppColors::BG_SECONDARY).inner_margin(egui::Margin::symmetric(Spacing::MD as i8, Spacing::SM as i8)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("D1 Manager").size(18.0).strong().color(AppColors::TEXT_PRIMARY));
                    // Version badge
                    ui.label(RichText::new(format!("v{}", crate::version::VERSION)).size(10.0).color(AppColors::TEXT_MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // About button
                        if ui.add(egui::Button::new(RichText::new("ⓘ").size(14.0))
                            .fill(Color32::TRANSPARENT)
                            .frame(false))
                            .on_hover_text(self.i18n.about())
                            .clicked()
                        {
                            self.show_about_dialog = true;
                            self.update_info = None;
                        }
                        ui.add_space(Spacing::SM);
                        if theme::secondary_button(ui, "⚙ Connections").clicked() {
                            self.show_settings = !self.show_settings;
                            self.show_profile_editor = false;
                            self.keychain_error = None;
                        }
                    });
                });

                if !self.tabs.is_empty() && !self.show_settings {
                    ui.add_space(Spacing::SM);
                    self.render_connection_tabs(ui);
                }
            });

        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::new().fill(AppColors::BG_SECONDARY).inner_margin(egui::Margin::symmetric(Spacing::MD as i8, Spacing::XS as i8)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.active_tab < self.tabs.len() {
                        // Clone necessary data to avoid borrow issues
                        let tab_connected = self.tabs[self.active_tab].connected;
                        let tab_env = self.tabs[self.active_tab].profile.environment;
                        let tab_read_only = self.tabs[self.active_tab].profile.read_only;
                        let tab_status_message = self.tabs[self.active_tab].status_message.clone();
                        let tab_database_id = self.tabs[self.active_tab].profile.database_id.clone();

                        theme::status_dot(ui, tab_connected);
                        ui.add_space(Spacing::XS);

                        // Environment badge in status bar
                        let env_color = tab_env.color();
                        ui.label(RichText::new(format!("[{}]", tab_env.short_label())).size(11.0).color(env_color));
                        ui.add_space(Spacing::XS);

                        // Read-only indicator
                        if tab_read_only {
                            ui.label(RichText::new("🔒").size(11.0));
                            ui.add_space(Spacing::XS);
                        }

                        // Production Lock status for production environments
                        if tab_env == EnvironmentType::Production && !tab_read_only {
                            if let Some(remaining) = self.get_production_unlock_remaining(&tab_database_id) {
                                let remaining_str = if remaining >= 60 {
                                    format!("{}:{:02}", remaining / 60, remaining % 60)
                                } else {
                                    format!("{}s", remaining)
                                };

                                // Show unlocked status with remaining time
                                if ui.add(
                                    egui::Button::new(RichText::new(format!("🔓 {}", self.i18n.unlocked_until(&remaining_str)))
                                        .size(11.0).color(AppColors::WARNING))
                                        .frame(false)
                                ).clicked() {
                                    self.lock_production(&tab_database_id.clone());
                                }
                            } else {
                                // Show locked status with unlock button
                                if ui.add(
                                    egui::Button::new(RichText::new(format!("🔐 {}", self.i18n.production_lock()))
                                        .size(11.0).color(AppColors::SUCCESS))
                                        .frame(false)
                                ).on_hover_text(self.i18n.production_lock_desc()).clicked() {
                                    self.unlock_production(&tab_database_id.clone());
                                }
                            }
                            ui.add_space(Spacing::SM);
                        }

                        ui.label(RichText::new(&tab_status_message).size(12.0).color(AppColors::TEXT_SECONDARY));

                        // Rate limit info on the right (clickable to open metrics panel)
                        let cache_age = self.get_cache_age(&tab_database_id);
                        let remaining_info = self.rate_limit_info.api_info.remaining;
                        let limit_info = self.rate_limit_info.api_info.limit.unwrap_or(1200);
                        let metrics_label = self.i18n.metrics().to_string();
                        let metrics_hover = self.i18n.usage_metrics().to_string();

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (remaining, limit, color) = if let Some(remaining) = remaining_info {
                                let ratio = remaining as f32 / limit_info as f32;
                                let color = if ratio > 0.5 {
                                    AppColors::SUCCESS
                                } else if ratio > 0.2 {
                                    AppColors::WARNING
                                } else {
                                    AppColors::ERROR
                                };
                                (Some(remaining), limit_info, color)
                            } else {
                                (None, 1200, AppColors::TEXT_MUTED)
                            };

                            // Clickable metrics button
                            let metrics_text = if let Some(remaining) = remaining {
                                format!("📊 API: {}/{}", remaining, limit)
                            } else {
                                format!("📊 {}", metrics_label)
                            };
                            let response = ui.add(
                                egui::Button::new(RichText::new(&metrics_text).size(11.0).color(color))
                                    .frame(false)
                            );
                            if response.clicked() {
                                self.show_metrics_panel = !self.show_metrics_panel;
                            }
                            response.on_hover_text(&metrics_hover);

                            // Cache age indicator
                            if let Some(age) = cache_age {
                                let age_str = if age < 60 {
                                    format!("{}s", age)
                                } else if age < 3600 {
                                    format!("{}m", age / 60)
                                } else {
                                    format!("{}h", age / 3600)
                                };
                                ui.add_space(Spacing::MD);
                                ui.label(RichText::new(format!("Cache: {}", age_str)).size(11.0).color(AppColors::TEXT_MUTED));
                            }
                        });
                    } else {
                        ui.label(RichText::new("No active connection").size(12.0).color(AppColors::TEXT_MUTED));
                    }
                });
            });

        // Bottom bar with language selector and help button (always visible)
        egui::TopBottomPanel::bottom("footer_panel")
            .frame(egui::Frame::new().fill(AppColors::BG_SECONDARY).inner_margin(egui::Margin::symmetric(Spacing::MD as i8, Spacing::XS as i8)).stroke(Stroke::new(1.0, AppColors::BORDER)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // How to use button on left
                    if ui.add(egui::Button::new(RichText::new(format!("❓ {}", self.i18n.how_to_use())).size(12.0).color(AppColors::TEXT_SECONDARY)).frame(false)).clicked() {
                        self.show_tutorial = true;
                    }

                    ui.add_space(Spacing::MD);

                    // Execution Log button
                    let log_count = self.execution_log.len();
                    let log_label = if log_count > 0 {
                        format!("📋 {} ({})", self.i18n.execution_log(), log_count)
                    } else {
                        format!("📋 {}", self.i18n.execution_log())
                    };
                    if ui.add(egui::Button::new(RichText::new(log_label).size(12.0).color(AppColors::TEXT_SECONDARY)).frame(false)).clicked() {
                        self.show_execution_log_panel = !self.show_execution_log_panel;
                    }

                    ui.add_space(Spacing::MD);

                    // Audit Journal button
                    let audit_count = self.audit_journal.record_count();
                    let audit_label = if audit_count > 0 {
                        format!("🕐 {} ({})", self.i18n.audit_journal(), audit_count)
                    } else {
                        format!("🕐 {}", self.i18n.audit_journal())
                    };
                    if ui.add(egui::Button::new(RichText::new(audit_label).size(12.0).color(AppColors::TEXT_SECONDARY)).frame(false)).clicked() {
                        self.show_audit_panel = !self.show_audit_panel;
                    }

                    ui.add_space(Spacing::MD);

                    // Schema Compare button (only show when there are multiple connections)
                    if self.profile_metadata.len() > 1 {
                        let compare_btn_color = if self.show_schema_diff_panel {
                            AppColors::PRIMARY
                        } else {
                            AppColors::TEXT_SECONDARY
                        };
                        if ui.add(egui::Button::new(RichText::new(format!("⇄ {}", self.i18n.schema_compare())).size(12.0).color(compare_btn_color)).frame(false)).clicked() {
                            self.show_schema_diff_panel = !self.show_schema_diff_panel;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Language selector on right
                        for lang in Language::all().iter().rev() {
                            let is_selected = self.i18n.lang == *lang;
                            let text_color = if is_selected { AppColors::PRIMARY } else { AppColors::TEXT_MUTED };

                            if ui.add(egui::Button::new(RichText::new(lang.label()).size(12.0).color(text_color)).frame(false)).clicked() {
                                self.i18n = I18n::new(*lang);
                                self.save_language_setting();
                            }
                        }
                    });
                });
            });

        if self.show_settings || self.tabs.is_empty() {
            // Show connections page (settings) when no tabs or explicitly requested
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(AppColors::BG_PRIMARY).inner_margin(egui::Margin::same(Spacing::LG as i8)))
                .show(ctx, |ui| {
                    if self.show_profile_editor { self.render_profile_editor(ui); }
                    else { self.render_profile_list(ui); }
                });
        } else {
            egui::SidePanel::left("tables_panel")
                .default_width(220.0)
                .frame(egui::Frame::new().fill(AppColors::BG_SECONDARY).inner_margin(egui::Margin::symmetric(Spacing::MD as i8, Spacing::NONE as i8)).stroke(Stroke::new(1.0, AppColors::BORDER)))
                .show(ctx, |ui| { self.render_sidebar(ui); });

            // SQL panel at bottom
            egui::TopBottomPanel::bottom("sql_panel")
                .resizable(true)
                .default_height(200.0)
                .frame(egui::Frame::new().fill(AppColors::BG_PRIMARY).inner_margin(egui::Margin::symmetric(Spacing::MD as i8, Spacing::SM as i8)).stroke(Stroke::new(1.0, AppColors::BORDER)))
                .show(ctx, |ui| { self.render_sql_editor(ui); });

            // Action bar above SQL panel
            egui::TopBottomPanel::bottom("action_bar_panel")
                .resizable(false)
                .frame(egui::Frame::new().fill(AppColors::BG_SECONDARY).inner_margin(egui::Margin::symmetric(Spacing::MD as i8, Spacing::SM as i8)).stroke(Stroke::new(1.0, AppColors::BORDER)))
                .show(ctx, |ui| { self.render_action_bar(ui); });

            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(AppColors::BG_PRIMARY).inner_margin(egui::Margin::symmetric(Spacing::MD as i8, Spacing::NONE as i8)))
                .show(ctx, |ui| { self.render_table_view(ui); });
        }

        // Render modal dialogs
        self.render_export_dialog(ctx);
        self.render_import_dialog(ctx);
        self.render_row_editor_dialog(ctx);
        self.render_cell_editor_dialog(ctx);
        self.render_confirmation_dialog(ctx);
        self.render_history_panel(ctx);
        self.render_snippets_panel(ctx);
        self.render_metrics_panel(ctx);
        self.render_tutorial_dialog(ctx);
        self.render_settings_export_dialog(ctx);
        self.render_settings_import_dialog(ctx);
        self.render_dangerous_query_dialog(ctx);
        self.render_execution_log_panel(ctx);
        self.render_audit_panel(ctx);
        self.render_schema_explorer(ctx);
        self.render_schema_diff_panel(ctx);
        self.render_ai_suggest_panel(ctx);
        self.render_about_dialog(ctx);
        self.render_onboarding_wizard(ctx);
    }

}
