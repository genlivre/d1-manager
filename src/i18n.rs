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

    pub fn lines(&self) -> &'static str {
        match self.lang {
            Language::English => "lines",
            Language::Japanese => "行",
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

    pub fn setup_guide(&self) -> &'static str {
        match self.lang {
            Language::English => "Setup Guide",
            Language::Japanese => "セットアップガイド",
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

    /// Explanation of what the diff shows
    pub fn schema_diff_explanation(&self) -> &'static str {
        match self.lang {
            Language::English => "Shows changes needed to make Target match Source",
            Language::Japanese => "ターゲットをソースに合わせるために必要な変更を表示",
        }
    }

    /// Table exists only in source (needs to be created in target)
    pub fn table_only_in_source(&self) -> &'static str {
        match self.lang {
            Language::English => "Only in Source",
            Language::Japanese => "ソースのみ",
        }
    }

    /// Table exists only in target (would be dropped)
    pub fn table_only_in_target(&self) -> &'static str {
        match self.lang {
            Language::English => "Only in Target",
            Language::Japanese => "ターゲットのみ",
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

    pub fn mysql_import_help_title(&self) -> &'static str {
        match self.lang {
            Language::English => "MySQL to SQLite Migration",
            Language::Japanese => "MySQL → SQLite 移行ガイド",
        }
    }

    pub fn mysql_import_supported_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Supported Syntax",
            Language::Japanese => "対応構文",
        }
    }

    pub fn mysql_import_supported(&self) -> &'static str {
        match self.lang {
            Language::English => "• CREATE TABLE (with type conversion)\n• INSERT statements\n• CREATE INDEX / UNIQUE INDEX\n• DROP TABLE",
            Language::Japanese => "• CREATE TABLE（型変換付き）\n• INSERT文\n• CREATE INDEX / UNIQUE INDEX\n• DROP TABLE",
        }
    }

    pub fn mysql_import_type_mapping_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Type Mapping",
            Language::Japanese => "型マッピング",
        }
    }

    pub fn mysql_import_type_mapping(&self) -> &'static str {
        match self.lang {
            Language::English => "VARCHAR/CHAR → TEXT\nINT/BIGINT/TINYINT → INTEGER\nDOUBLE/FLOAT/DECIMAL → REAL\nDATETIME/TIMESTAMP → TEXT\nBOOLEAN → INTEGER\nENUM → TEXT",
            Language::Japanese => "VARCHAR/CHAR → TEXT\nINT/BIGINT/TINYINT → INTEGER\nDOUBLE/FLOAT/DECIMAL → REAL\nDATETIME/TIMESTAMP → TEXT\nBOOLEAN → INTEGER\nENUM → TEXT",
        }
    }

    pub fn mysql_import_auto_skip_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Auto-skipped",
            Language::Japanese => "自動スキップ",
        }
    }

    pub fn mysql_import_auto_skip(&self) -> &'static str {
        match self.lang {
            Language::English => "SET, LOCK/UNLOCK TABLES, USE, CREATE/DROP DATABASE, Transaction controls",
            Language::Japanese => "SET, LOCK/UNLOCK TABLES, USE, CREATE/DROP DATABASE, トランザクション制御",
        }
    }

    pub fn mysql_import_not_supported_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Not Supported",
            Language::Japanese => "非対応",
        }
    }

    pub fn mysql_import_not_supported(&self) -> &'static str {
        match self.lang {
            Language::English => "VIEW, TRIGGER, PROCEDURE, FUNCTION, ALTER TABLE",
            Language::Japanese => "VIEW, TRIGGER, PROCEDURE, FUNCTION, ALTER TABLE",
        }
    }

    pub fn mysql_import_tip(&self) -> &'static str {
        match self.lang {
            Language::English => "Tip: Use mysqldump with --compatible=ansi for best results",
            Language::Japanese => "ヒント: mysqldump --compatible=ansi オプションで最適な結果が得られます",
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

    pub fn open_source_licenses(&self) -> &'static str {
        match self.lang {
            Language::English => "Open Source Licenses",
            Language::Japanese => "オープンソースライセンス",
        }
    }

    pub fn licenses_description(&self) -> &'static str {
        match self.lang {
            Language::English => "D1 Manager uses the following open source software:",
            Language::Japanese => "D1 Managerは以下のオープンソースソフトウェアを使用しています：",
        }
    }

    // =========================================================================
    // Onboarding Wizard
    // =========================================================================

    pub fn wizard_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Setup Wizard",
            Language::Japanese => "セットアップウィザード",
        }
    }

    pub fn wizard_welcome_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Welcome to D1 Manager",
            Language::Japanese => "D1 Manager へようこそ",
        }
    }

    pub fn wizard_welcome_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "A production-safe GUI client for Cloudflare D1 databases. Let's set up your first connection.",
            Language::Japanese => "Cloudflare D1 データベース用の本番環境対応 GUI クライアントです。最初の接続を設定しましょう。",
        }
    }

    pub fn wizard_choose_mode(&self) -> &'static str {
        match self.lang {
            Language::English => "Choose connection type:",
            Language::Japanese => "接続タイプを選択:",
        }
    }

    pub fn wizard_remote_mode(&self) -> &'static str {
        match self.lang {
            Language::English => "Remote (Cloudflare D1)",
            Language::Japanese => "リモート (Cloudflare D1)",
        }
    }

    pub fn wizard_remote_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Connect to your Cloudflare D1 databases via API",
            Language::Japanese => "API 経由で Cloudflare D1 データベースに接続",
        }
    }

    pub fn wizard_local_mode(&self) -> &'static str {
        match self.lang {
            Language::English => "Local (SQLite)",
            Language::Japanese => "ローカル (SQLite)",
        }
    }

    pub fn wizard_local_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Use local wrangler D1 databases for development",
            Language::Japanese => "開発用のローカル wrangler D1 データベースを使用",
        }
    }

    pub fn wizard_step1_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Step 1: API Token",
            Language::Japanese => "ステップ 1: API トークン",
        }
    }

    pub fn wizard_step1_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Enter your Cloudflare API token. You can create one in the Cloudflare Dashboard.",
            Language::Japanese => "Cloudflare API トークンを入力してください。Cloudflare ダッシュボードで作成できます。",
        }
    }

    pub fn wizard_required_scopes(&self) -> &'static str {
        match self.lang {
            Language::English => "Required API Token Scopes:",
            Language::Japanese => "必要な API トークンスコープ:",
        }
    }

    pub fn wizard_open_dashboard(&self) -> &'static str {
        match self.lang {
            Language::English => "Open Cloudflare Dashboard",
            Language::Japanese => "Cloudflare ダッシュボードを開く",
        }
    }

    pub fn wizard_step2_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Step 2: Select Account",
            Language::Japanese => "ステップ 2: アカウントを選択",
        }
    }

    pub fn wizard_step2_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Select the Cloudflare account containing your D1 database.",
            Language::Japanese => "D1 データベースがある Cloudflare アカウントを選択してください。",
        }
    }

    pub fn wizard_step3_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Step 3: Select Database",
            Language::Japanese => "ステップ 3: データベースを選択",
        }
    }

    pub fn wizard_step3_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Select the D1 database you want to connect to.",
            Language::Japanese => "接続する D1 データベースを選択してください。",
        }
    }

    pub fn wizard_step4_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Step 4: Environment Settings",
            Language::Japanese => "ステップ 4: 環境設定",
        }
    }

    pub fn wizard_step4_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Configure connection name and environment type. Production environments have additional safety locks.",
            Language::Japanese => "接続名と環境タイプを設定してください。本番環境には追加の安全ロックが適用されます。",
        }
    }

    pub fn wizard_step5_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Step 5: Connection Test",
            Language::Japanese => "ステップ 5: 接続テスト",
        }
    }

    pub fn wizard_step5_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Test the connection and complete setup.",
            Language::Japanese => "接続をテストしてセットアップを完了します。",
        }
    }

    pub fn wizard_loading_accounts(&self) -> &'static str {
        match self.lang {
            Language::English => "Loading accounts...",
            Language::Japanese => "アカウントを読み込み中...",
        }
    }

    pub fn wizard_loading_databases(&self) -> &'static str {
        match self.lang {
            Language::English => "Loading databases...",
            Language::Japanese => "データベースを読み込み中...",
        }
    }

    pub fn wizard_no_accounts(&self) -> &'static str {
        match self.lang {
            Language::English => "No accounts found. Please check your API token permissions.",
            Language::Japanese => "アカウントが見つかりません。API トークンの権限を確認してください。",
        }
    }

    pub fn wizard_no_databases(&self) -> &'static str {
        match self.lang {
            Language::English => "No D1 databases found in this account.",
            Language::Japanese => "このアカウントには D1 データベースがありません。",
        }
    }

    pub fn wizard_test_connection(&self) -> &'static str {
        match self.lang {
            Language::English => "Test Connection",
            Language::Japanese => "接続テスト",
        }
    }

    pub fn wizard_testing(&self) -> &'static str {
        match self.lang {
            Language::English => "Testing connection...",
            Language::Japanese => "接続をテスト中...",
        }
    }

    pub fn wizard_test_success(&self) -> &'static str {
        match self.lang {
            Language::English => "Connection successful!",
            Language::Japanese => "接続成功！",
        }
    }

    pub fn wizard_complete_setup(&self) -> &'static str {
        match self.lang {
            Language::English => "Complete Setup",
            Language::Japanese => "セットアップを完了",
        }
    }

    pub fn wizard_back(&self) -> &'static str {
        match self.lang {
            Language::English => "Back",
            Language::Japanese => "戻る",
        }
    }

    pub fn wizard_next(&self) -> &'static str {
        match self.lang {
            Language::English => "Next",
            Language::Japanese => "次へ",
        }
    }

    pub fn wizard_skip(&self) -> &'static str {
        match self.lang {
            Language::English => "Skip Wizard",
            Language::Japanese => "ウィザードをスキップ",
        }
    }

    pub fn wizard_retry(&self) -> &'static str {
        match self.lang {
            Language::English => "Retry",
            Language::Japanese => "再試行",
        }
    }

    pub fn wizard_scan_local(&self) -> &'static str {
        match self.lang {
            Language::English => "Scan for Local Databases",
            Language::Japanese => "ローカルデータベースをスキャン",
        }
    }

    pub fn wizard_scanning(&self) -> &'static str {
        match self.lang {
            Language::English => "Scanning...",
            Language::Japanese => "スキャン中...",
        }
    }

    pub fn wizard_no_local_dbs(&self) -> &'static str {
        match self.lang {
            Language::English => "No local wrangler D1 databases found.",
            Language::Japanese => "ローカルの wrangler D1 データベースが見つかりません。",
        }
    }

    pub fn wizard_local_db_hint(&self) -> &'static str {
        match self.lang {
            Language::English => "Tip: Run 'wrangler d1 execute --local' in a project directory to create a local database.",
            Language::Japanese => "ヒント: プロジェクトディレクトリで 'wrangler d1 execute --local' を実行してローカルデータベースを作成できます。",
        }
    }

    // Local database wizard strings
    pub fn wizard_local_step_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Select Local Database",
            Language::Japanese => "ローカルデータベースを選択",
        }
    }

    pub fn wizard_local_step_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a local SQLite database from your wrangler projects or browse for a file.",
            Language::Japanese => "wrangler プロジェクトからローカル SQLite データベースを選択するか、ファイルを参照してください。",
        }
    }

    pub fn wizard_scanning_local(&self) -> &'static str {
        match self.lang {
            Language::English => "Scanning for local databases...",
            Language::Japanese => "ローカルデータベースをスキャン中...",
        }
    }

    pub fn wizard_no_local_databases(&self) -> &'static str {
        match self.lang {
            Language::English => "No local wrangler D1 databases found. You can browse for any SQLite file or run 'wrangler d1 execute --local' to create one.",
            Language::Japanese => "ローカルの wrangler D1 データベースが見つかりません。任意の SQLite ファイルを参照するか、'wrangler d1 execute --local' を実行して作成できます。",
        }
    }

    pub fn wizard_rescan(&self) -> &'static str {
        match self.lang {
            Language::English => "Rescan",
            Language::Japanese => "再スキャン",
        }
    }

    pub fn wizard_or_browse(&self) -> &'static str {
        match self.lang {
            Language::English => "Or browse for a SQLite file:",
            Language::Japanese => "または SQLite ファイルを参照：",
        }
    }

    pub fn wizard_browse_file(&self) -> &'static str {
        match self.lang {
            Language::English => "Browse...",
            Language::Japanese => "参照...",
        }
    }

    pub fn wizard_create_new_db(&self) -> &'static str {
        match self.lang {
            Language::English => "Create New Database",
            Language::Japanese => "新規データベースを作成",
        }
    }

    pub fn wizard_create_new_db_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Create a New Database",
            Language::Japanese => "新しいデータベースを作成",
        }
    }

    pub fn wizard_create_new_db_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a folder to create a new empty SQLite database for local development.",
            Language::Japanese => "ローカル開発用に新しい空のSQLiteデータベースを作成するフォルダを選択してください。",
        }
    }

    // Profile editor - connection type tabs
    pub fn remote_d1(&self) -> &'static str {
        match self.lang {
            Language::English => "Remote D1",
            Language::Japanese => "リモート D1",
        }
    }

    pub fn local_sqlite(&self) -> &'static str {
        match self.lang {
            Language::English => "Local SQLite",
            Language::Japanese => "ローカル SQLite",
        }
    }

    pub fn database_file_path(&self) -> &'static str {
        match self.lang {
            Language::English => "Database File Path",
            Language::Japanese => "データベースファイルパス",
        }
    }

    pub fn no_file_selected(&self) -> &'static str {
        match self.lang {
            Language::English => "No file selected",
            Language::Japanese => "ファイルが選択されていません",
        }
    }

    pub fn browse(&self) -> &'static str {
        match self.lang {
            Language::English => "Browse...",
            Language::Japanese => "参照...",
        }
    }

    pub fn or_create_new(&self) -> &'static str {
        match self.lang {
            Language::English => "Or create a new database:",
            Language::Japanese => "または新規データベースを作成：",
        }
    }

    pub fn create_new_database(&self) -> &'static str {
        match self.lang {
            Language::English => "Create New Database",
            Language::Japanese => "新規データベースを作成",
        }
    }

    pub fn open_existing_database(&self) -> &'static str {
        match self.lang {
            Language::English => "Open Existing Database",
            Language::Japanese => "既存のデータベースを開く",
        }
    }

    pub fn select_folder_for_new_db(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a folder to create a new database",
            Language::Japanese => "新しいデータベースを作成するフォルダを選択",
        }
    }

    pub fn select_folder(&self) -> &'static str {
        match self.lang {
            Language::English => "Select Folder...",
            Language::Japanese => "フォルダを選択...",
        }
    }

    pub fn database_will_be_created(&self) -> &'static str {
        match self.lang {
            Language::English => "A new SQLite database will be created when you save.",
            Language::Japanese => "保存時に新しいSQLiteデータベースが作成されます。",
        }
    }

    pub fn select_sqlite_file(&self) -> &'static str {
        match self.lang {
            Language::English => "Select an existing SQLite database file",
            Language::Japanese => "既存のSQLiteデータベースファイルを選択",
        }
    }

    pub fn sql_file_hint_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Have a .sql file?",
            Language::Japanese => ".sqlファイルをお持ちですか？",
        }
    }

    pub fn sql_file_hint_description(&self) -> &'static str {
        match self.lang {
            Language::English => "SQL files (.sql) cannot be opened directly. Create a new database first, then use Import → MySQL to import your SQL file.",
            Language::Japanese => "SQLファイル（.sql）は直接開けません。まず新規データベースを作成し、インポート → MySQL からSQLファイルを読み込んでください。",
        }
    }

    // Local database error messages (with technical details preserved)
    pub fn local_db_not_found(&self, path: &str) -> String {
        match self.lang {
            Language::English => format!("Database file not found: {}", path),
            Language::Japanese => format!("データベースファイルが見つかりません: {}", path),
        }
    }

    pub fn local_db_open_error(&self, detail: &str) -> String {
        match self.lang {
            Language::English => format!("Failed to open database: {}", detail),
            Language::Japanese => format!("データベースを開けませんでした: {}", detail),
        }
    }

    pub fn local_db_create_error(&self, detail: &str) -> String {
        match self.lang {
            Language::English => format!("Failed to create database: {}", detail),
            Language::Japanese => format!("データベースの作成に失敗しました: {}", detail),
        }
    }

    pub fn local_db_query_error(&self, detail: &str) -> String {
        match self.lang {
            Language::English => format!("Query error: {}", detail),
            Language::Japanese => format!("クエリエラー: {}", detail),
        }
    }

    pub fn local_db_execute_error(&self, detail: &str) -> String {
        match self.lang {
            Language::English => format!("Execute error: {}", detail),
            Language::Japanese => format!("実行エラー: {}", detail),
        }
    }

    pub fn local_db_prepare_error(&self, detail: &str) -> String {
        match self.lang {
            Language::English => format!("Prepare error: {}", detail),
            Language::Japanese => format!("SQL準備エラー: {}", detail),
        }
    }

    pub fn local_db_path_not_configured(&self) -> &'static str {
        match self.lang {
            Language::English => "Local database path not configured",
            Language::Japanese => "ローカルデータベースのパスが設定されていません",
        }
    }

    pub fn query_success(&self) -> &'static str {
        match self.lang {
            Language::English => "Query executed successfully",
            Language::Japanese => "クエリが正常に実行されました",
        }
    }

    pub fn batch_execution_complete(&self) -> &'static str {
        match self.lang {
            Language::English => "Batch execution completed",
            Language::Japanese => "バッチ実行が完了しました",
        }
    }

    pub fn statements_count(&self, count: usize) -> String {
        match self.lang {
            Language::English => format!("{} statements", count),
            Language::Japanese => format!("{} ステートメント", count),
        }
    }

    pub fn connection_success(&self) -> &'static str {
        match self.lang {
            Language::English => "Connection successful",
            Language::Japanese => "接続に成功しました",
        }
    }

    // Saved Queries
    pub fn saved_queries(&self) -> &'static str {
        match self.lang {
            Language::English => "Saved Queries",
            Language::Japanese => "保存済みクエリ",
        }
    }

    pub fn save_query(&self) -> &'static str {
        match self.lang {
            Language::English => "Save query",
            Language::Japanese => "クエリを保存",
        }
    }

    pub fn queries_count(&self) -> &'static str {
        match self.lang {
            Language::English => "queries",
            Language::Japanese => "件のクエリ",
        }
    }

    pub fn save_current_query(&self) -> &'static str {
        match self.lang {
            Language::English => "Save current query",
            Language::Japanese => "現在のクエリを保存",
        }
    }

    pub fn label(&self) -> &'static str {
        match self.lang {
            Language::English => "Label:",
            Language::Japanese => "ラベル:",
        }
    }

    pub fn query_label_hint(&self) -> &'static str {
        match self.lang {
            Language::English => "e.g., Get active users",
            Language::Japanese => "例: アクティブユーザーを取得",
        }
    }

    pub fn description_optional(&self) -> &'static str {
        match self.lang {
            Language::English => "Optional description",
            Language::Japanese => "説明（任意）",
        }
    }

    pub fn no_saved_queries(&self) -> &'static str {
        match self.lang {
            Language::English => "No saved queries yet",
            Language::Japanese => "保存済みクエリはありません",
        }
    }

    pub fn clear_result(&self) -> &'static str {
        match self.lang {
            Language::English => "Clear result",
            Language::Japanese => "結果をクリア",
        }
    }

    pub fn description_label(&self) -> &'static str {
        match self.lang {
            Language::English => "Description:",
            Language::Japanese => "説明:",
        }
    }

    pub fn search_placeholder(&self) -> &'static str {
        match self.lang {
            Language::English => "Search...",
            Language::Japanese => "検索...",
        }
    }

    /// Translate a LocalDbError into a localized message
    pub fn translate_local_db_error(&self, error: &crate::local_db::LocalDbError) -> String {
        use crate::local_db::LocalDbError;
        match error {
            LocalDbError::NotFound(path) => self.local_db_not_found(path),
            LocalDbError::OpenFailed(detail) => self.local_db_open_error(detail),
            LocalDbError::CreateFailed(detail) => self.local_db_create_error(detail),
            LocalDbError::PrepareFailed(detail) => self.local_db_prepare_error(detail),
            LocalDbError::QueryFailed(detail) => self.local_db_query_error(detail),
            LocalDbError::ExecuteFailed(detail) => self.local_db_execute_error(detail),
        }
    }

    // =========================================================================
    // API Error Guidance (Onboarding improvement)
    // =========================================================================

    pub fn how_to_fix(&self) -> &'static str {
        match self.lang {
            Language::English => "How to fix this",
            Language::Japanese => "解決方法",
        }
    }

    pub fn try_demo_database(&self) -> &'static str {
        match self.lang {
            Language::English => "Try Demo Database",
            Language::Japanese => "デモデータベースを試す",
        }
    }

    pub fn demo_db_description(&self) -> &'static str {
        match self.lang {
            Language::English => "Start with a sample local database to explore features without setting up an API token.",
            Language::Japanese => "APIトークンを設定せずに機能を試すためのサンプルローカルデータベースで始めましょう。",
        }
    }

    pub fn demo_db_created(&self) -> &'static str {
        match self.lang {
            Language::English => "Demo database created successfully!",
            Language::Japanese => "デモデータベースが正常に作成されました！",
        }
    }

    pub fn demo_db_tables(&self) -> &'static str {
        match self.lang {
            Language::English => "Includes sample tables: users, products, orders",
            Language::Japanese => "サンプルテーブルを含む: users, products, orders",
        }
    }

    // =========================================================================
    // Database Templates (Local Mock)
    // =========================================================================

    pub fn select_template(&self) -> &'static str {
        match self.lang {
            Language::English => "Select Template",
            Language::Japanese => "テンプレートを選択",
        }
    }

    pub fn select_template_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Choose a template for your new local database:",
            Language::Japanese => "新しいローカルデータベースのテンプレートを選択:",
        }
    }

    pub fn template_name(&self, id: &str) -> &'static str {
        match id {
            "empty" => self.template_empty(),
            "ecommerce" => self.template_ecommerce(),
            "blog" => self.template_blog(),
            "tasks" => self.template_tasks(),
            _ => self.template_empty(),
        }
    }

    pub fn template_description(&self, id: &str) -> &'static str {
        match id {
            "empty" => self.template_empty_desc(),
            "ecommerce" => self.template_ecommerce_desc(),
            "blog" => self.template_blog_desc(),
            "tasks" => self.template_tasks_desc(),
            _ => self.template_empty_desc(),
        }
    }

    pub fn template_empty(&self) -> &'static str {
        match self.lang {
            Language::English => "Empty Database",
            Language::Japanese => "空のデータベース",
        }
    }

    pub fn template_empty_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Start with a blank SQLite database",
            Language::Japanese => "空のSQLiteデータベースから開始",
        }
    }

    pub fn template_ecommerce(&self) -> &'static str {
        match self.lang {
            Language::English => "E-Commerce",
            Language::Japanese => "Eコマース",
        }
    }

    pub fn template_ecommerce_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Users, products, orders with sample data",
            Language::Japanese => "ユーザー、商品、注文とサンプルデータ",
        }
    }

    pub fn template_blog(&self) -> &'static str {
        match self.lang {
            Language::English => "Blog Platform",
            Language::Japanese => "ブログプラットフォーム",
        }
    }

    pub fn template_blog_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Posts, comments, tags with sample content",
            Language::Japanese => "投稿、コメント、タグとサンプルコンテンツ",
        }
    }

    pub fn template_tasks(&self) -> &'static str {
        match self.lang {
            Language::English => "Task Management",
            Language::Japanese => "タスク管理",
        }
    }

    pub fn template_tasks_desc(&self) -> &'static str {
        match self.lang {
            Language::English => "Projects, tasks, users for project management",
            Language::Japanese => "プロジェクト管理用のプロジェクト、タスク、ユーザー",
        }
    }

    pub fn template_preview(&self) -> &'static str {
        match self.lang {
            Language::English => "Schema Preview",
            Language::Japanese => "スキーマプレビュー",
        }
    }

    pub fn create_with_template(&self) -> &'static str {
        match self.lang {
            Language::English => "Create with Template",
            Language::Japanese => "テンプレートで作成",
        }
    }

    // =========================================================================
    // Rollback Execution Flow
    // =========================================================================

    pub fn execute_rollback(&self) -> &'static str {
        match self.lang {
            Language::English => "Execute Rollback",
            Language::Japanese => "ロールバックを実行",
        }
    }

    pub fn rollback_confirmation(&self) -> &'static str {
        match self.lang {
            Language::English => "Confirm Rollback",
            Language::Japanese => "ロールバックの確認",
        }
    }

    pub fn rollback_warning(&self) -> &'static str {
        match self.lang {
            Language::English => "This action cannot be undone. The following SQL will be executed:",
            Language::Japanese => "この操作は元に戻せません。以下のSQLが実行されます：",
        }
    }

    pub fn rollback_impact(&self) -> &'static str {
        match self.lang {
            Language::English => "Impact",
            Language::Japanese => "影響",
        }
    }

    pub fn type_rollback_to_confirm(&self) -> &'static str {
        match self.lang {
            Language::English => "Type ROLLBACK to confirm:",
            Language::Japanese => "確認のため ROLLBACK と入力:",
        }
    }

    pub fn rollback_success(&self) -> &'static str {
        match self.lang {
            Language::English => "Rollback executed successfully",
            Language::Japanese => "ロールバックが正常に実行されました",
        }
    }

    pub fn changes_diff(&self) -> &'static str {
        match self.lang {
            Language::English => "Changes",
            Language::Japanese => "変更内容",
        }
    }

    pub fn before_value(&self) -> &'static str {
        match self.lang {
            Language::English => "Before",
            Language::Japanese => "変更前",
        }
    }

    pub fn after_value(&self) -> &'static str {
        match self.lang {
            Language::English => "After",
            Language::Japanese => "変更後",
        }
    }

    pub fn operation(&self) -> &'static str {
        match self.lang {
            Language::English => "Operation:",
            Language::Japanese => "操作:",
        }
    }

    pub fn table_label(&self) -> &'static str {
        match self.lang {
            Language::English => "Table:",
            Language::Japanese => "テーブル:",
        }
    }

    pub fn timestamp(&self) -> &'static str {
        match self.lang {
            Language::English => "Timestamp:",
            Language::Japanese => "タイムスタンプ:",
        }
    }

    pub fn change_preview(&self) -> &'static str {
        match self.lang {
            Language::English => "Change Preview",
            Language::Japanese => "変更プレビュー",
        }
    }

    pub fn sql_to_execute(&self) -> &'static str {
        match self.lang {
            Language::English => "SQL to Execute:",
            Language::Japanese => "実行するSQL:",
        }
    }

    pub fn production_locked(&self) -> &'static str {
        match self.lang {
            Language::English => "Production Environment Locked",
            Language::Japanese => "本番環境がロックされています",
        }
    }

    pub fn unlock_production_to_continue(&self) -> &'static str {
        match self.lang {
            Language::English => "Unlock the production environment to continue with the rollback.",
            Language::Japanese => "ロールバックを続行するには、本番環境のロックを解除してください。",
        }
    }
}
