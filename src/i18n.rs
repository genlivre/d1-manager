#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    Japanese,
}

impl Language {
    pub fn label(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Japanese => "日本語",
        }
    }

    pub fn all() -> &'static [Language] {
        &[Language::English, Language::Japanese]
    }

    pub fn from_locale(locale: &str) -> Self {
        if locale.starts_with("ja") {
            Language::Japanese
        } else {
            Language::English
        }
    }

    pub fn detect_system_language() -> Self {
        sys_locale::get_locale()
            .map(|l| Language::from_locale(&l))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct I18n {
    pub lang: Language,
}

impl Default for I18n {
    fn default() -> Self {
        Self {
            lang: Language::detect_system_language(),
        }
    }
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    // App title and general
    pub fn app_title(&self) -> &'static str {
        match self.lang {
            Language::English => "D1 Manager",
            Language::Japanese => "D1 Manager",
        }
    }

    // Connection panel
    pub fn connections(&self) -> &'static str {
        match self.lang {
            Language::English => "Connections",
            Language::Japanese => "接続",
        }
    }

    pub fn new_connection(&self) -> &'static str {
        match self.lang {
            Language::English => "+ New Connection",
            Language::Japanese => "+ 新規接続",
        }
    }

    pub fn no_connections_yet(&self) -> &'static str {
        match self.lang {
            Language::English => "No connections yet",
            Language::Japanese => "接続がありません",
        }
    }

    pub fn click_new_connection(&self) -> &'static str {
        match self.lang {
            Language::English => "Click '+ New Connection' to get started",
            Language::Japanese => "「+ 新規接続」をクリックして開始",
        }
    }

    pub fn edit_connection(&self) -> &'static str {
        match self.lang {
            Language::English => "Edit Connection",
            Language::Japanese => "接続を編集",
        }
    }

    pub fn database(&self) -> &'static str {
        match self.lang {
            Language::English => "Database:",
            Language::Japanese => "データベース:",
        }
    }

    pub fn delete(&self) -> &'static str {
        match self.lang {
            Language::English => "Delete",
            Language::Japanese => "削除",
        }
    }

    pub fn edit(&self) -> &'static str {
        match self.lang {
            Language::English => "Edit",
            Language::Japanese => "編集",
        }
    }

    pub fn connect(&self) -> &'static str {
        match self.lang {
            Language::English => "Connect",
            Language::Japanese => "接続",
        }
    }

    // Profile editor
    pub fn connection_name(&self) -> &'static str {
        match self.lang {
            Language::English => "Connection Name",
            Language::Japanese => "接続名",
        }
    }

    pub fn account_id(&self) -> &'static str {
        match self.lang {
            Language::English => "Account ID",
            Language::Japanese => "アカウント ID",
        }
    }

    pub fn database_id(&self) -> &'static str {
        match self.lang {
            Language::English => "Database ID",
            Language::Japanese => "データベース ID",
        }
    }

    pub fn api_token(&self) -> &'static str {
        match self.lang {
            Language::English => "API Token",
            Language::Japanese => "API トークン",
        }
    }

    pub fn stored_in_keychain(&self) -> &'static str {
        match self.lang {
            Language::English => "Stored securely in system credential store",
            Language::Japanese => "システムの資格情報ストアに安全に保存されます",
        }
    }

    pub fn environment(&self) -> &'static str {
        match self.lang {
            Language::English => "Environment",
            Language::Japanese => "環境",
        }
    }

    pub fn environment_development(&self) -> &'static str {
        match self.lang {
            Language::English => "Development",
            Language::Japanese => "開発",
        }
    }

    pub fn environment_staging(&self) -> &'static str {
        match self.lang {
            Language::English => "Staging",
            Language::Japanese => "ステージング",
        }
    }

    pub fn environment_production(&self) -> &'static str {
        match self.lang {
            Language::English => "Production",
            Language::Japanese => "本番",
        }
    }

    pub fn env_confirmation_note(&self) -> &'static str {
        match self.lang {
            Language::English => "Production/Staging environments require confirmation for write operations",
            Language::Japanese => "本番/ステージング環境では書き込み操作に確認が必要です",
        }
    }

    pub fn read_only_mode(&self) -> &'static str {
        match self.lang {
            Language::English => "Read-only mode",
            Language::Japanese => "読み取り専用モード",
        }
    }

    pub fn read_only_description(&self) -> &'static str {
        match self.lang {
            Language::English => "Block all INSERT, UPDATE, DELETE operations",
            Language::Japanese => "INSERT、UPDATE、DELETE 操作をすべてブロック",
        }
    }

    pub fn api_token_permissions(&self) -> &'static str {
        match self.lang {
            Language::English => "API Token Permissions",
            Language::Japanese => "API トークン権限",
        }
    }

    pub fn recommended_permissions(&self) -> &'static str {
        match self.lang {
            Language::English => "Recommended minimum permissions:",
            Language::Japanese => "推奨される最小限の権限:",
        }
    }

    pub fn create_tokens_at(&self) -> &'static str {
        match self.lang {
            Language::English => "Create tokens at: dash.cloudflare.com > Profile > API Tokens",
            Language::Japanese => "トークン作成: dash.cloudflare.com > Profile > API Tokens",
        }
    }

    pub fn test_connection(&self) -> &'static str {
        match self.lang {
            Language::English => "Test Connection",
            Language::Japanese => "接続テスト",
        }
    }

    pub fn testing(&self) -> &'static str {
        match self.lang {
            Language::English => "Testing...",
            Language::Japanese => "テスト中...",
        }
    }

    pub fn save_connection(&self) -> &'static str {
        match self.lang {
            Language::English => "Save Connection",
            Language::Japanese => "接続を保存",
        }
    }

    pub fn cancel(&self) -> &'static str {
        match self.lang {
            Language::English => "Cancel",
            Language::Japanese => "キャンセル",
        }
    }

    // Status messages
    pub fn not_connected(&self) -> &'static str {
        match self.lang {
            Language::English => "Not connected",
            Language::Japanese => "未接続",
        }
    }

    pub fn connecting(&self) -> &'static str {
        match self.lang {
            Language::English => "Connecting...",
            Language::Japanese => "接続中...",
        }
    }

    pub fn connected(&self) -> &'static str {
        match self.lang {
            Language::English => "Connected",
            Language::Japanese => "接続済み",
        }
    }

    pub fn connection_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Connection failed: {}", error),
            Language::Japanese => format!("接続失敗: {}", error),
        }
    }

    pub fn query_executed(&self) -> &'static str {
        match self.lang {
            Language::English => "Query executed successfully",
            Language::Japanese => "クエリが正常に実行されました",
        }
    }

    pub fn query_failed(&self) -> &'static str {
        match self.lang {
            Language::English => "Query failed",
            Language::Japanese => "クエリ失敗",
        }
    }

    pub fn row_deleted(&self) -> &'static str {
        match self.lang {
            Language::English => "Row deleted",
            Language::Japanese => "行を削除しました",
        }
    }

    pub fn delete_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Delete failed: {}", error),
            Language::Japanese => format!("削除失敗: {}", error),
        }
    }

    pub fn row_inserted(&self) -> &'static str {
        match self.lang {
            Language::English => "Row inserted",
            Language::Japanese => "行を挿入しました",
        }
    }

    pub fn insert_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Insert failed: {}", error),
            Language::Japanese => format!("挿入失敗: {}", error),
        }
    }

    pub fn cell_updated(&self) -> &'static str {
        match self.lang {
            Language::English => "Cell updated",
            Language::Japanese => "セルを更新しました",
        }
    }

    pub fn update_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Update failed: {}", error),
            Language::Japanese => format!("更新失敗: {}", error),
        }
    }

    pub fn failed_to_load_schema(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Failed to load schema: {}", error),
            Language::Japanese => format!("スキーマ読み込み失敗: {}", error),
        }
    }

    pub fn keychain_error(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Credential store error: {}", error),
            Language::Japanese => format!("資格情報ストアエラー: {}", error),
        }
    }

    pub fn api_token_not_found(&self) -> &'static str {
        match self.lang {
            Language::English => "API Token not found",
            Language::Japanese => "API トークンが見つかりません",
        }
    }

    pub fn error(&self, error: &str) -> String {
        match self.lang {
            Language::English => format!("Error: {}", error),
            Language::Japanese => format!("エラー: {}", error),
        }
    }

    // Table view
    pub fn tables(&self) -> &'static str {
        match self.lang {
            Language::English => "Tables",
            Language::Japanese => "テーブル",
        }
    }

    pub fn select_table(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a table from the sidebar",
            Language::Japanese => "サイドバーからテーブルを選択",
        }
    }

    pub fn rows(&self, count: i64) -> String {
        match self.lang {
            Language::English => format!("{} rows", count),
            Language::Japanese => format!("{} 行", count),
        }
    }

    pub fn row_selected(&self, idx: usize) -> String {
        match self.lang {
            Language::English => format!("Row {} selected", idx + 1),
            Language::Japanese => format!("行 {} を選択中", idx + 1),
        }
    }

    pub fn filter(&self) -> &'static str {
        match self.lang {
            Language::English => "Filter",
            Language::Japanese => "フィルター",
        }
    }

    pub fn filter_with_count(&self, count: usize) -> String {
        match self.lang {
            Language::English => format!("Filter ({})", count),
            Language::Japanese => format!("フィルター ({})", count),
        }
    }

    pub fn column(&self) -> &'static str {
        match self.lang {
            Language::English => "Column:",
            Language::Japanese => "カラム:",
        }
    }

    pub fn operator(&self) -> &'static str {
        match self.lang {
            Language::English => "Op:",
            Language::Japanese => "演算子:",
        }
    }

    pub fn value(&self) -> &'static str {
        match self.lang {
            Language::English => "Value:",
            Language::Japanese => "値:",
        }
    }

    pub fn select(&self) -> &'static str {
        match self.lang {
            Language::English => "Select...",
            Language::Japanese => "選択...",
        }
    }

    pub fn add(&self) -> &'static str {
        match self.lang {
            Language::English => "Add",
            Language::Japanese => "追加",
        }
    }

    pub fn apply(&self) -> &'static str {
        match self.lang {
            Language::English => "Apply",
            Language::Japanese => "適用",
        }
    }

    pub fn clear(&self) -> &'static str {
        match self.lang {
            Language::English => "Clear",
            Language::Japanese => "クリア",
        }
    }

    // Operations
    pub fn delete_row(&self) -> &'static str {
        match self.lang {
            Language::English => "Delete Row",
            Language::Japanese => "行を削除",
        }
    }

    pub fn update_cell(&self) -> &'static str {
        match self.lang {
            Language::English => "Update Cell",
            Language::Japanese => "セルを更新",
        }
    }

    pub fn insert_row(&self) -> &'static str {
        match self.lang {
            Language::English => "Insert Row",
            Language::Japanese => "行を挿入",
        }
    }

    pub fn execute_query(&self) -> &'static str {
        match self.lang {
            Language::English => "Execute Query",
            Language::Japanese => "クエリを実行",
        }
    }

    pub fn import_data(&self) -> &'static str {
        match self.lang {
            Language::English => "Import Data",
            Language::Japanese => "データをインポート",
        }
    }

    // SQL Panel
    pub fn sql_query(&self) -> &'static str {
        match self.lang {
            Language::English => "SQL Query",
            Language::Japanese => "SQL クエリ",
        }
    }

    pub fn run(&self) -> &'static str {
        match self.lang {
            Language::English => "Run",
            Language::Japanese => "実行",
        }
    }

    pub fn history(&self) -> &'static str {
        match self.lang {
            Language::English => "History",
            Language::Japanese => "履歴",
        }
    }

    pub fn query_history(&self) -> &'static str {
        match self.lang {
            Language::English => "Query History",
            Language::Japanese => "クエリ履歴",
        }
    }

    pub fn close(&self) -> &'static str {
        match self.lang {
            Language::English => "Close",
            Language::Japanese => "閉じる",
        }
    }

    pub fn clear_history(&self) -> &'static str {
        match self.lang {
            Language::English => "Clear History",
            Language::Japanese => "履歴をクリア",
        }
    }

    pub fn no_history(&self) -> &'static str {
        match self.lang {
            Language::English => "No query history yet",
            Language::Japanese => "クエリ履歴がありません",
        }
    }

    // Export/Import
    pub fn export(&self) -> &'static str {
        match self.lang {
            Language::English => "Export",
            Language::Japanese => "エクスポート",
        }
    }

    pub fn import(&self) -> &'static str {
        match self.lang {
            Language::English => "Import",
            Language::Japanese => "インポート",
        }
    }

    pub fn export_data(&self) -> &'static str {
        match self.lang {
            Language::English => "Export Data",
            Language::Japanese => "データをエクスポート",
        }
    }

    pub fn export_format(&self) -> &'static str {
        match self.lang {
            Language::English => "Format:",
            Language::Japanese => "形式:",
        }
    }

    pub fn export_to_file(&self) -> &'static str {
        match self.lang {
            Language::English => "Export to File",
            Language::Japanese => "ファイルにエクスポート",
        }
    }

    pub fn import_from_file(&self) -> &'static str {
        match self.lang {
            Language::English => "Import from File",
            Language::Japanese => "ファイルからインポート",
        }
    }

    pub fn conflict_strategy(&self) -> &'static str {
        match self.lang {
            Language::English => "On Conflict:",
            Language::Japanese => "競合時:",
        }
    }

    pub fn conflict_fail(&self) -> &'static str {
        match self.lang {
            Language::English => "Fail",
            Language::Japanese => "失敗",
        }
    }

    pub fn conflict_ignore(&self) -> &'static str {
        match self.lang {
            Language::English => "Ignore",
            Language::Japanese => "無視",
        }
    }

    pub fn conflict_replace(&self) -> &'static str {
        match self.lang {
            Language::English => "Replace",
            Language::Japanese => "置換",
        }
    }

    pub fn preview(&self) -> &'static str {
        match self.lang {
            Language::English => "Preview",
            Language::Japanese => "プレビュー",
        }
    }

    pub fn select_file(&self) -> &'static str {
        match self.lang {
            Language::English => "Select File",
            Language::Japanese => "ファイルを選択",
        }
    }

    pub fn execute_import(&self) -> &'static str {
        match self.lang {
            Language::English => "Execute Import",
            Language::Japanese => "インポート実行",
        }
    }

    // Row editor
    pub fn new_row(&self) -> &'static str {
        match self.lang {
            Language::English => "New Row",
            Language::Japanese => "新規行",
        }
    }

    pub fn duplicate_row(&self) -> &'static str {
        match self.lang {
            Language::English => "Duplicate Row",
            Language::Japanese => "行を複製",
        }
    }

    pub fn add_row(&self) -> &'static str {
        match self.lang {
            Language::English => "+ Add Row",
            Language::Japanese => "+ 行を追加",
        }
    }

    pub fn save(&self) -> &'static str {
        match self.lang {
            Language::English => "Save",
            Language::Japanese => "保存",
        }
    }

    // Confirmation dialog
    pub fn confirm_operation(&self) -> &'static str {
        match self.lang {
            Language::English => "Confirm Operation",
            Language::Japanese => "操作の確認",
        }
    }

    pub fn warning_dangerous_env(&self, env: &str) -> String {
        match self.lang {
            Language::English => format!("⚠ You are about to modify data in a {} environment.", env),
            Language::Japanese => format!("⚠ {} 環境のデータを変更しようとしています。", env),
        }
    }

    pub fn confirm(&self) -> &'static str {
        match self.lang {
            Language::English => "Confirm",
            Language::Japanese => "確認",
        }
    }

    pub fn read_only_badge(&self) -> &'static str {
        match self.lang {
            Language::English => "READ-ONLY",
            Language::Japanese => "読み取り専用",
        }
    }

    // Pagination
    pub fn page(&self) -> &'static str {
        match self.lang {
            Language::English => "Page",
            Language::Japanese => "ページ",
        }
    }

    pub fn of(&self) -> &'static str {
        match self.lang {
            Language::English => "of",
            Language::Japanese => "/",
        }
    }

    pub fn rows_per_page(&self) -> &'static str {
        match self.lang {
            Language::English => "Rows per page:",
            Language::Japanese => "表示行数:",
        }
    }

    // Settings
    pub fn settings(&self) -> &'static str {
        match self.lang {
            Language::English => "Settings",
            Language::Japanese => "設定",
        }
    }

    pub fn language(&self) -> &'static str {
        match self.lang {
            Language::English => "Language",
            Language::Japanese => "言語",
        }
    }

    // Actions
    pub fn refresh(&self) -> &'static str {
        match self.lang {
            Language::English => "Refresh",
            Language::Japanese => "更新",
        }
    }

    pub fn copy(&self) -> &'static str {
        match self.lang {
            Language::English => "Copy",
            Language::Japanese => "コピー",
        }
    }

    pub fn loading(&self) -> &'static str {
        match self.lang {
            Language::English => "Loading...",
            Language::Japanese => "読み込み中...",
        }
    }

    // Rate limit info
    pub fn rate_limit(&self) -> &'static str {
        match self.lang {
            Language::English => "Rate Limit",
            Language::Japanese => "レート制限",
        }
    }

    pub fn requests_remaining(&self, count: i32) -> String {
        match self.lang {
            Language::English => format!("{} requests remaining", count),
            Language::Japanese => format!("残り {} リクエスト", count),
        }
    }

    // Schema view
    pub fn schema(&self) -> &'static str {
        match self.lang {
            Language::English => "Schema",
            Language::Japanese => "スキーマ",
        }
    }

    pub fn column_name(&self) -> &'static str {
        match self.lang {
            Language::English => "Column",
            Language::Japanese => "カラム名",
        }
    }

    pub fn data_type(&self) -> &'static str {
        match self.lang {
            Language::English => "Type",
            Language::Japanese => "型",
        }
    }

    pub fn nullable(&self) -> &'static str {
        match self.lang {
            Language::English => "Nullable",
            Language::Japanese => "NULL許可",
        }
    }

    pub fn primary_key(&self) -> &'static str {
        match self.lang {
            Language::English => "Primary Key",
            Language::Japanese => "主キー",
        }
    }

    pub fn default_value(&self) -> &'static str {
        match self.lang {
            Language::English => "Default",
            Language::Japanese => "デフォルト",
        }
    }

    pub fn yes(&self) -> &'static str {
        match self.lang {
            Language::English => "Yes",
            Language::Japanese => "はい",
        }
    }

    pub fn no(&self) -> &'static str {
        match self.lang {
            Language::English => "No",
            Language::Japanese => "いいえ",
        }
    }

    // Misc
    pub fn success(&self) -> &'static str {
        match self.lang {
            Language::English => "Success",
            Language::Japanese => "成功",
        }
    }

    pub fn failed(&self) -> &'static str {
        match self.lang {
            Language::English => "Failed",
            Language::Japanese => "失敗",
        }
    }

    pub fn result(&self) -> &'static str {
        match self.lang {
            Language::English => "Result",
            Language::Japanese => "結果",
        }
    }

    pub fn duration(&self, ms: f64) -> String {
        match self.lang {
            Language::English => format!("{:.2}ms", ms),
            Language::Japanese => format!("{:.2}ミリ秒", ms),
        }
    }

    pub fn rows_affected(&self, count: i64) -> String {
        match self.lang {
            Language::English => format!("{} rows affected", count),
            Language::Japanese => format!("{} 行が影響を受けました", count),
        }
    }

    // Welcome screen
    pub fn welcome_to(&self) -> &'static str {
        match self.lang {
            Language::English => "Welcome to D1 Manager",
            Language::Japanese => "D1 Manager へようこそ",
        }
    }

    pub fn welcome_subtitle(&self) -> &'static str {
        match self.lang {
            Language::English => "A native client for Cloudflare D1 databases",
            Language::Japanese => "Cloudflare D1 データベース用ネイティブクライアント",
        }
    }

    pub fn get_started(&self) -> &'static str {
        match self.lang {
            Language::English => "Get Started",
            Language::Japanese => "はじめる",
        }
    }

    // Cell editor
    pub fn edit_cell(&self) -> &'static str {
        match self.lang {
            Language::English => "Edit Cell",
            Language::Japanese => "セルを編集",
        }
    }

    pub fn editing_column(&self, col: &str) -> String {
        match self.lang {
            Language::English => format!("Editing: {}", col),
            Language::Japanese => format!("編集中: {}", col),
        }
    }

    // Table hints
    pub fn click_row_to_select(&self) -> &'static str {
        match self.lang {
            Language::English => "Click a row to select it",
            Language::Japanese => "行をクリックして選択",
        }
    }

    // SQL Editor
    pub fn execute(&self) -> &'static str {
        match self.lang {
            Language::English => "Execute",
            Language::Japanese => "実行",
        }
    }

    // Tutorial
    pub fn how_to_use(&self) -> &'static str {
        match self.lang {
            Language::English => "How to Use",
            Language::Japanese => "使い方を見る",
        }
    }

    pub fn tutorial_title(&self) -> &'static str {
        match self.lang {
            Language::English => "How to Use D1 Manager",
            Language::Japanese => "D1 Manager の使い方",
        }
    }

    pub fn tutorial_step1_title(&self) -> &'static str {
        match self.lang {
            Language::English => "1. Create a Connection",
            Language::Japanese => "1. 接続を作成",
        }
    }

    pub fn tutorial_step1_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Click '+ New Connection' and enter your Cloudflare Account ID, Database ID, and API Token.",
            Language::Japanese => "「+ 新規接続」をクリックし、Cloudflare アカウント ID、データベース ID、API トークンを入力します。",
        }
    }

    pub fn tutorial_step2_title(&self) -> &'static str {
        match self.lang {
            Language::English => "2. Connect to Database",
            Language::Japanese => "2. データベースに接続",
        }
    }

    pub fn tutorial_step2_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Click 'Connect' on your saved connection to connect to the D1 database.",
            Language::Japanese => "保存した接続の「接続」ボタンをクリックして、D1 データベースに接続します。",
        }
    }

    pub fn tutorial_step3_title(&self) -> &'static str {
        match self.lang {
            Language::English => "3. Browse and Edit Data",
            Language::Japanese => "3. データの閲覧・編集",
        }
    }

    pub fn tutorial_step3_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a table from the sidebar. Click rows to select, double-click cells to edit.",
            Language::Japanese => "サイドバーからテーブルを選択。行をクリックで選択、セルをダブルクリックで編集できます。",
        }
    }

    pub fn tutorial_step4_title(&self) -> &'static str {
        match self.lang {
            Language::English => "4. Run SQL Queries",
            Language::Japanese => "4. SQL クエリの実行",
        }
    }

    pub fn tutorial_step4_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Use the SQL panel at the bottom to run custom queries directly.",
            Language::Japanese => "画面下部の SQL パネルでカスタムクエリを直接実行できます。",
        }
    }

    pub fn tutorial_api_token_hint_title(&self) -> &'static str {
        match self.lang {
            Language::English => "How to get your API Token",
            Language::Japanese => "API トークンの取得方法",
        }
    }

    pub fn tutorial_api_token_hint_path(&self) -> &'static str {
        match self.lang {
            Language::English => "Cloudflare Dashboard > Profile > API Tokens",
            Language::Japanese => "Cloudflare ダッシュボード > プロフィール > API トークン",
        }
    }

    pub fn got_it(&self) -> &'static str {
        match self.lang {
            Language::English => "Got it!",
            Language::Japanese => "わかりました",
        }
    }

    // Metrics Panel
    pub fn metrics(&self) -> &'static str {
        match self.lang {
            Language::English => "Metrics",
            Language::Japanese => "メトリクス",
        }
    }

    pub fn usage_metrics(&self) -> &'static str {
        match self.lang {
            Language::English => "Usage Metrics",
            Language::Japanese => "使用状況メトリクス",
        }
    }

    pub fn rate_limit_status(&self) -> &'static str {
        match self.lang {
            Language::English => "Rate Limit Status",
            Language::Japanese => "レートリミット状況",
        }
    }

    pub fn api_requests_remaining(&self) -> &'static str {
        match self.lang {
            Language::English => "API Requests Remaining",
            Language::Japanese => "残りAPIリクエスト数",
        }
    }

    pub fn requests_last_5min(&self) -> &'static str {
        match self.lang {
            Language::English => "Requests (Last 5 min)",
            Language::Japanese => "リクエスト数 (過去5分)",
        }
    }

    pub fn rate_limit_resets_at(&self) -> &'static str {
        match self.lang {
            Language::English => "Resets at",
            Language::Japanese => "リセット時刻",
        }
    }

    pub fn session_statistics(&self) -> &'static str {
        match self.lang {
            Language::English => "Session Statistics",
            Language::Japanese => "セッション統計",
        }
    }

    pub fn session_duration(&self) -> &'static str {
        match self.lang {
            Language::English => "Session Duration",
            Language::Japanese => "セッション時間",
        }
    }

    pub fn total_api_requests(&self) -> &'static str {
        match self.lang {
            Language::English => "Total API Requests",
            Language::Japanese => "合計APIリクエスト",
        }
    }

    pub fn total_queries(&self) -> &'static str {
        match self.lang {
            Language::English => "Total Queries",
            Language::Japanese => "合計クエリ数",
        }
    }

    pub fn rows_read(&self) -> &'static str {
        match self.lang {
            Language::English => "Rows Read",
            Language::Japanese => "読み取り行数",
        }
    }

    pub fn rows_written(&self) -> &'static str {
        match self.lang {
            Language::English => "Rows Written",
            Language::Japanese => "書き込み行数",
        }
    }

    pub fn avg_query_duration(&self) -> &'static str {
        match self.lang {
            Language::English => "Avg Query Duration",
            Language::Japanese => "平均クエリ時間",
        }
    }

    pub fn total_query_time(&self) -> &'static str {
        match self.lang {
            Language::English => "Total Query Time",
            Language::Japanese => "合計クエリ時間",
        }
    }

    pub fn rate_limit_warning(&self) -> &'static str {
        match self.lang {
            Language::English => "Warning: Approaching rate limit!",
            Language::Japanese => "警告: レートリミットに近づいています!",
        }
    }

    pub fn rate_limit_critical(&self) -> &'static str {
        match self.lang {
            Language::English => "Critical: Rate limit nearly exhausted!",
            Language::Japanese => "危険: レートリミットがほぼ上限です!",
        }
    }

    pub fn cloudflare_limits_info(&self) -> &'static str {
        match self.lang {
            Language::English => "Cloudflare API: 1,200 requests per 5 minutes",
            Language::Japanese => "Cloudflare API: 5分あたり1,200リクエスト",
        }
    }

    pub fn no_metrics_yet(&self) -> &'static str {
        match self.lang {
            Language::English => "No metrics yet. Connect to a database to start collecting data.",
            Language::Japanese => "メトリクスがありません。データベースに接続して収集を開始してください。",
        }
    }

    // Settings Export/Import
    pub fn export_settings(&self) -> &'static str {
        match self.lang {
            Language::English => "Export Settings",
            Language::Japanese => "設定をエクスポート",
        }
    }

    pub fn import_settings(&self) -> &'static str {
        match self.lang {
            Language::English => "Import Settings",
            Language::Japanese => "設定をインポート",
        }
    }

    pub fn export_connections(&self) -> &'static str {
        match self.lang {
            Language::English => "Export Connections",
            Language::Japanese => "接続設定をエクスポート",
        }
    }

    pub fn import_connections(&self) -> &'static str {
        match self.lang {
            Language::English => "Import Connections",
            Language::Japanese => "接続設定をインポート",
        }
    }

    pub fn include_api_tokens(&self) -> &'static str {
        match self.lang {
            Language::English => "Include API Tokens (encrypted)",
            Language::Japanese => "APIトークンを含める（暗号化）",
        }
    }

    pub fn api_tokens_warning(&self) -> &'static str {
        match self.lang {
            Language::English => "Warning: API tokens will be encrypted but handle the file securely",
            Language::Japanese => "警告: APIトークンは暗号化されますが、ファイルは安全に管理してください",
        }
    }

    pub fn settings_exported(&self) -> &'static str {
        match self.lang {
            Language::English => "Settings exported successfully",
            Language::Japanese => "設定をエクスポートしました",
        }
    }

    pub fn settings_imported(&self) -> &'static str {
        match self.lang {
            Language::English => "Settings imported successfully",
            Language::Japanese => "設定をインポートしました",
        }
    }

    pub fn import_merge(&self) -> &'static str {
        match self.lang {
            Language::English => "Merge with existing",
            Language::Japanese => "既存の設定とマージ",
        }
    }

    pub fn import_replace(&self) -> &'static str {
        match self.lang {
            Language::English => "Replace all",
            Language::Japanese => "すべて置換",
        }
    }

    pub fn connections_count(&self, count: usize) -> String {
        match self.lang {
            Language::English => format!("{} connection(s)", count),
            Language::Japanese => format!("{} 件の接続", count),
        }
    }

    pub fn invalid_settings_file(&self) -> &'static str {
        match self.lang {
            Language::English => "Invalid settings file",
            Language::Japanese => "無効な設定ファイルです",
        }
    }

    pub fn encryption_password(&self) -> &'static str {
        match self.lang {
            Language::English => "Encryption Password",
            Language::Japanese => "暗号化パスワード",
        }
    }

    pub fn password_hint(&self) -> &'static str {
        match self.lang {
            Language::English => "Used to encrypt/decrypt API tokens",
            Language::Japanese => "APIトークンの暗号化/復号に使用",
        }
    }

    pub fn passwords_dont_match(&self) -> &'static str {
        match self.lang {
            Language::English => "Passwords don't match",
            Language::Japanese => "パスワードが一致しません",
        }
    }

    pub fn confirm_password(&self) -> &'static str {
        match self.lang {
            Language::English => "Confirm Password",
            Language::Japanese => "パスワード確認",
        }
    }

    pub fn decryption_failed(&self) -> &'static str {
        match self.lang {
            Language::English => "Decryption failed. Wrong password?",
            Language::Japanese => "復号に失敗しました。パスワードが違いませんか？",
        }
    }

    pub fn backup_restore(&self) -> &'static str {
        match self.lang {
            Language::English => "Backup & Restore",
            Language::Japanese => "バックアップと復元",
        }
    }

    // SQL Safety
    pub fn dangerous_query_detected(&self) -> &'static str {
        match self.lang {
            Language::English => "Dangerous Query Detected",
            Language::Japanese => "危険なクエリを検出",
        }
    }

    pub fn query_risk_level(&self, level: &str) -> String {
        match self.lang {
            Language::English => format!("Risk Level: {}", level),
            Language::Japanese => format!("リスクレベル: {}", level),
        }
    }

    pub fn production_environment_warning(&self) -> &'static str {
        match self.lang {
            Language::English => "This is a PRODUCTION environment. Extra confirmation required.",
            Language::Japanese => "これは本番環境です。追加の確認が必要です。",
        }
    }

    pub fn type_to_confirm(&self, text: &str) -> String {
        match self.lang {
            Language::English => format!("Type '{}' to confirm:", text),
            Language::Japanese => format!("確認のため '{}' と入力してください:", text),
        }
    }

    pub fn query_blocked_readonly(&self) -> &'static str {
        match self.lang {
            Language::English => "Query blocked: Connection is read-only",
            Language::Japanese => "クエリがブロックされました: 読み取り専用接続です",
        }
    }

    pub fn query_analysis(&self) -> &'static str {
        match self.lang {
            Language::English => "Query Analysis",
            Language::Japanese => "クエリ分析",
        }
    }

    pub fn affected_tables(&self) -> &'static str {
        match self.lang {
            Language::English => "Affected Tables",
            Language::Japanese => "影響を受けるテーブル",
        }
    }

    pub fn warnings(&self) -> &'static str {
        match self.lang {
            Language::English => "Warnings",
            Language::Japanese => "警告",
        }
    }

    pub fn execute_anyway(&self) -> &'static str {
        match self.lang {
            Language::English => "Execute Anyway",
            Language::Japanese => "それでも実行",
        }
    }

    pub fn production_lock(&self) -> &'static str {
        match self.lang {
            Language::English => "Production Lock",
            Language::Japanese => "本番ロック",
        }
    }

    pub fn production_lock_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Temporarily unlock write operations (5 min)",
            Language::Japanese => "書き込み操作を一時的に解除（5分間）",
        }
    }

    pub fn unlock(&self) -> &'static str {
        match self.lang {
            Language::English => "Unlock",
            Language::Japanese => "解除",
        }
    }

    pub fn lock(&self) -> &'static str {
        match self.lang {
            Language::English => "Lock",
            Language::Japanese => "ロック",
        }
    }

    pub fn unlocked_until(&self, remaining: &str) -> String {
        match self.lang {
            Language::English => format!("Unlocked ({} remaining)", remaining),
            Language::Japanese => format!("解除中（残り {}）", remaining),
        }
    }

    pub fn execution_log(&self) -> &'static str {
        match self.lang {
            Language::English => "Execution Log",
            Language::Japanese => "実行ログ",
        }
    }

    pub fn view_log(&self) -> &'static str {
        match self.lang {
            Language::English => "View Log",
            Language::Japanese => "ログを表示",
        }
    }

    pub fn no_logs_yet(&self) -> &'static str {
        match self.lang {
            Language::English => "No execution logs yet",
            Language::Japanese => "実行ログがありません",
        }
    }

    pub fn clear_logs(&self) -> &'static str {
        match self.lang {
            Language::English => "Clear Logs",
            Language::Japanese => "ログをクリア",
        }
    }

    pub fn export_logs(&self) -> &'static str {
        match self.lang {
            Language::English => "Export Logs",
            Language::Japanese => "ログをエクスポート",
        }
    }

    // SQL Editor improvements
    pub fn format_sql(&self) -> &'static str {
        match self.lang {
            Language::English => "Format SQL",
            Language::Japanese => "SQLを整形",
        }
    }

    pub fn snippets(&self) -> &'static str {
        match self.lang {
            Language::English => "SQL Snippets",
            Language::Japanese => "SQLスニペット",
        }
    }

    pub fn insert_snippet(&self) -> &'static str {
        match self.lang {
            Language::English => "Insert",
            Language::Japanese => "挿入",
        }
    }

    pub fn search_history(&self) -> &'static str {
        match self.lang {
            Language::English => "Search history...",
            Language::Japanese => "履歴を検索...",
        }
    }

    pub fn no_matching_history(&self) -> &'static str {
        match self.lang {
            Language::English => "No matching queries found",
            Language::Japanese => "一致するクエリが見つかりません",
        }
    }

    pub fn use_query(&self) -> &'static str {
        match self.lang {
            Language::English => "Use",
            Language::Japanese => "使用",
        }
    }

    // Audit Journal
    pub fn audit_journal(&self) -> &'static str {
        match self.lang {
            Language::English => "Audit Journal",
            Language::Japanese => "監査ジャーナル",
        }
    }

    pub fn audit_enabled(&self) -> &'static str {
        match self.lang {
            Language::English => "Audit Enabled",
            Language::Japanese => "監査有効",
        }
    }

    pub fn audit_disabled(&self) -> &'static str {
        match self.lang {
            Language::English => "Audit Disabled",
            Language::Japanese => "監査無効",
        }
    }

    pub fn filter_table(&self) -> &'static str {
        match self.lang {
            Language::English => "Filter by table:",
            Language::Japanese => "テーブルで絞込:",
        }
    }

    pub fn all_tables(&self) -> &'static str {
        match self.lang {
            Language::English => "All tables",
            Language::Japanese => "全テーブル",
        }
    }

    pub fn no_audit_records(&self) -> &'static str {
        match self.lang {
            Language::English => "No audit records yet",
            Language::Japanese => "監査レコードがありません",
        }
    }

    pub fn select_record(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a record to view details",
            Language::Japanese => "レコードを選択して詳細を表示",
        }
    }

    pub fn original_sql(&self) -> &'static str {
        match self.lang {
            Language::English => "Original SQL",
            Language::Japanese => "元のSQL",
        }
    }

    pub fn before_values(&self) -> &'static str {
        match self.lang {
            Language::English => "Before",
            Language::Japanese => "変更前",
        }
    }

    pub fn after_values(&self) -> &'static str {
        match self.lang {
            Language::English => "After",
            Language::Japanese => "変更後",
        }
    }

    pub fn rollback_sql(&self) -> &'static str {
        match self.lang {
            Language::English => "Rollback SQL",
            Language::Japanese => "ロールバックSQL",
        }
    }

    pub fn copy_to_editor(&self) -> &'static str {
        match self.lang {
            Language::English => "Copy to Editor",
            Language::Japanese => "エディタにコピー",
        }
    }

    // Schema Explorer
    pub fn schema_explorer(&self) -> &'static str {
        match self.lang {
            Language::English => "Schema Explorer",
            Language::Japanese => "スキーマエクスプローラー",
        }
    }

    pub fn relationships(&self) -> &'static str {
        match self.lang {
            Language::English => "Relationships",
            Language::Japanese => "リレーション",
        }
    }

    pub fn relationships_btn(&self) -> &'static str {
        match self.lang {
            Language::English => "Relations",
            Language::Japanese => "関連",
        }
    }

    pub fn no_relationships(&self) -> &'static str {
        match self.lang {
            Language::English => "No relationships found for this table",
            Language::Japanese => "このテーブルにはリレーションがありません",
        }
    }

    pub fn references_label(&self) -> &'static str {
        match self.lang {
            Language::English => "References (this table -> other)",
            Language::Japanese => "参照先 (このテーブル -> 他)",
        }
    }

    pub fn referenced_by_label(&self) -> &'static str {
        match self.lang {
            Language::English => "Referenced by (other -> this table)",
            Language::Japanese => "参照元 (他 -> このテーブル)",
        }
    }

    pub fn generate_join(&self) -> &'static str {
        match self.lang {
            Language::English => "JOIN SQL",
            Language::Japanese => "JOIN生成",
        }
    }

    pub fn er_diagram(&self) -> &'static str {
        match self.lang {
            Language::English => "ER Diagram",
            Language::Japanese => "ER図",
        }
    }

    pub fn select_table_first(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a table to view relationships",
            Language::Japanese => "テーブルを選択してリレーションを表示",
        }
    }

    pub fn clear_selection(&self) -> &'static str {
        match self.lang {
            Language::English => "Clear selection",
            Language::Japanese => "選択解除",
        }
    }

    // Schema Diff (DB Comparison)
    pub fn schema_compare(&self) -> &'static str {
        match self.lang {
            Language::English => "Schema Compare",
            Language::Japanese => "スキーマ比較",
        }
    }

    pub fn compare_databases(&self) -> &'static str {
        match self.lang {
            Language::English => "Compare Databases",
            Language::Japanese => "データベース比較",
        }
    }

    pub fn source_database(&self) -> &'static str {
        match self.lang {
            Language::English => "Source (current)",
            Language::Japanese => "ソース（現在）",
        }
    }

    pub fn target_database(&self) -> &'static str {
        match self.lang {
            Language::English => "Target",
            Language::Japanese => "ターゲット",
        }
    }

    pub fn compare(&self) -> &'static str {
        match self.lang {
            Language::English => "Compare",
            Language::Japanese => "比較",
        }
    }

    pub fn select_database(&self) -> &'static str {
        match self.lang {
            Language::English => "Select database...",
            Language::Japanese => "データベースを選択...",
        }
    }

    pub fn schema_diff_result(&self) -> &'static str {
        match self.lang {
            Language::English => "Schema Differences",
            Language::Japanese => "スキーマ差分",
        }
    }

    pub fn no_differences(&self) -> &'static str {
        match self.lang {
            Language::English => "No differences found",
            Language::Japanese => "差分なし",
        }
    }

    pub fn table_added(&self) -> &'static str {
        match self.lang {
            Language::English => "Added",
            Language::Japanese => "追加",
        }
    }

    pub fn table_removed(&self) -> &'static str {
        match self.lang {
            Language::English => "Removed",
            Language::Japanese => "削除",
        }
    }

    pub fn table_modified(&self) -> &'static str {
        match self.lang {
            Language::English => "Modified",
            Language::Japanese => "変更",
        }
    }

    pub fn column_changes(&self) -> &'static str {
        match self.lang {
            Language::English => "Column Changes",
            Language::Japanese => "カラム変更",
        }
    }

    pub fn migration_sql(&self) -> &'static str {
        match self.lang {
            Language::English => "Migration SQL",
            Language::Japanese => "マイグレーションSQL",
        }
    }

    pub fn generate_migration(&self) -> &'static str {
        match self.lang {
            Language::English => "Generate Migration",
            Language::Japanese => "マイグレーション生成",
        }
    }

    pub fn copy_sql(&self) -> &'static str {
        match self.lang {
            Language::English => "Copy SQL",
            Language::Japanese => "SQLをコピー",
        }
    }

    pub fn tables_count(&self, added: usize, removed: usize, modified: usize) -> String {
        match self.lang {
            Language::English => format!("{} added, {} removed, {} modified", added, removed, modified),
            Language::Japanese => format!("{} 追加, {} 削除, {} 変更", added, removed, modified),
        }
    }

    pub fn comparing(&self) -> &'static str {
        match self.lang {
            Language::English => "Comparing...",
            Language::Japanese => "比較中...",
        }
    }

    pub fn select_both_databases(&self) -> &'static str {
        match self.lang {
            Language::English => "Select both source and target databases to compare",
            Language::Japanese => "比較するソースとターゲットのデータベースを選択してください",
        }
    }

    pub fn old_definition(&self) -> &'static str {
        match self.lang {
            Language::English => "Old",
            Language::Japanese => "旧",
        }
    }

    pub fn new_definition(&self) -> &'static str {
        match self.lang {
            Language::English => "New",
            Language::Japanese => "新",
        }
    }

    // MySQL Dump Import
    pub fn mysql_dump_import(&self) -> &'static str {
        match self.lang {
            Language::English => "MySQL Dump Import",
            Language::Japanese => "MySQLダンプインポート",
        }
    }

    pub fn mysql_dump_description(&self) -> &'static str {
        match self.lang {
            Language::English => "Convert MySQL dump to SQLite and execute",
            Language::Japanese => "MySQLダンプをSQLiteに変換して実行",
        }
    }

    pub fn conversion_result(&self) -> &'static str {
        match self.lang {
            Language::English => "Conversion Result",
            Language::Japanese => "変換結果",
        }
    }

    pub fn tables_found(&self, count: usize) -> String {
        match self.lang {
            Language::English => format!("{} tables found", count),
            Language::Japanese => format!("{} テーブル検出", count),
        }
    }

    pub fn insert_statements(&self, count: usize) -> String {
        match self.lang {
            Language::English => format!("{} INSERT statements", count),
            Language::Japanese => format!("{} INSERT文", count),
        }
    }

    pub fn conversion_warnings(&self) -> &'static str {
        match self.lang {
            Language::English => "Conversion Warnings",
            Language::Japanese => "変換警告",
        }
    }

    pub fn no_warnings(&self) -> &'static str {
        match self.lang {
            Language::English => "No warnings",
            Language::Japanese => "警告なし",
        }
    }

    pub fn execute_all(&self) -> &'static str {
        match self.lang {
            Language::English => "Execute All",
            Language::Japanese => "すべて実行",
        }
    }

    pub fn preview_sql(&self) -> &'static str {
        match self.lang {
            Language::English => "Preview SQL",
            Language::Japanese => "SQLプレビュー",
        }
    }

    pub fn import_format(&self) -> &'static str {
        match self.lang {
            Language::English => "Import Format",
            Language::Japanese => "インポート形式",
        }
    }

    // AI SQL Suggestions
    pub fn sql_suggestions(&self) -> &'static str {
        match self.lang {
            Language::English => "SQL Suggestions",
            Language::Japanese => "SQL提案",
        }
    }

    pub fn ai_suggest_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Smart SQL Suggestions",
            Language::Japanese => "スマートSQL提案",
        }
    }

    pub fn ai_suggest_description(&self) -> &'static str {
        match self.lang {
            Language::English => "Context-aware query suggestions based on table schema",
            Language::Japanese => "テーブルスキーマに基づくコンテキスト認識クエリ提案",
        }
    }

    pub fn category_basic(&self) -> &'static str {
        match self.lang {
            Language::English => "Basic Query",
            Language::Japanese => "基本クエリ",
        }
    }

    pub fn category_aggregation(&self) -> &'static str {
        match self.lang {
            Language::English => "Aggregation",
            Language::Japanese => "集計",
        }
    }

    pub fn category_join(&self) -> &'static str {
        match self.lang {
            Language::English => "Join",
            Language::Japanese => "結合",
        }
    }

    pub fn category_filter_sort(&self) -> &'static str {
        match self.lang {
            Language::English => "Filter & Sort",
            Language::Japanese => "フィルタ/ソート",
        }
    }

    pub fn category_modification(&self) -> &'static str {
        match self.lang {
            Language::English => "Modification",
            Language::Japanese => "変更",
        }
    }

    pub fn category_schema(&self) -> &'static str {
        match self.lang {
            Language::English => "Schema",
            Language::Japanese => "スキーマ",
        }
    }

    pub fn show_unsafe_queries(&self) -> &'static str {
        match self.lang {
            Language::English => "Show modification queries",
            Language::Japanese => "変更クエリを表示",
        }
    }

    pub fn unsafe_query_warning(&self) -> &'static str {
        match self.lang {
            Language::English => "This query modifies data",
            Language::Japanese => "このクエリはデータを変更します",
        }
    }

    pub fn use_this_query(&self) -> &'static str {
        match self.lang {
            Language::English => "Use",
            Language::Japanese => "使用",
        }
    }

    pub fn no_suggestions_available(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a table to see suggestions",
            Language::Japanese => "提案を表示するにはテーブルを選択してください",
        }
    }

    pub fn generated_locally(&self) -> &'static str {
        match self.lang {
            Language::English => "Generated locally - no data sent externally",
            Language::Japanese => "ローカル生成 - データは外部に送信されません",
        }
    }

    // Version and Updates
    pub fn about(&self) -> &'static str {
        match self.lang {
            Language::English => "About",
            Language::Japanese => "バージョン情報",
        }
    }

    pub fn version(&self) -> &'static str {
        match self.lang {
            Language::English => "Version",
            Language::Japanese => "バージョン",
        }
    }

    pub fn check_for_updates(&self) -> &'static str {
        match self.lang {
            Language::English => "Check for Updates",
            Language::Japanese => "アップデートを確認",
        }
    }

    pub fn checking_updates(&self) -> &'static str {
        match self.lang {
            Language::English => "Checking for updates...",
            Language::Japanese => "アップデートを確認中...",
        }
    }

    pub fn update_available(&self, version: &str) -> String {
        match self.lang {
            Language::English => format!("Update available: v{}", version),
            Language::Japanese => format!("アップデートがあります: v{}", version),
        }
    }

    pub fn up_to_date(&self) -> &'static str {
        match self.lang {
            Language::English => "You're up to date!",
            Language::Japanese => "最新版です！",
        }
    }

    pub fn download_update(&self) -> &'static str {
        match self.lang {
            Language::English => "Download Update",
            Language::Japanese => "アップデートをダウンロード",
        }
    }

    pub fn view_release(&self) -> &'static str {
        match self.lang {
            Language::English => "View Release",
            Language::Japanese => "リリースを表示",
        }
    }

    pub fn release_notes(&self) -> &'static str {
        match self.lang {
            Language::English => "Release Notes",
            Language::Japanese => "リリースノート",
        }
    }

    pub fn current_version(&self) -> &'static str {
        match self.lang {
            Language::English => "Current Version",
            Language::Japanese => "現在のバージョン",
        }
    }

    pub fn latest_version(&self) -> &'static str {
        match self.lang {
            Language::English => "Latest Version",
            Language::Japanese => "最新バージョン",
        }
    }

    pub fn update_check_failed(&self) -> &'static str {
        match self.lang {
            Language::English => "Failed to check for updates",
            Language::Japanese => "アップデートの確認に失敗しました",
        }
    }
}
