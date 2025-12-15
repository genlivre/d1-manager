# D1 Manager - 実装内容まとめ

Production-safe な Cloudflare D1 データベース GUI クライアント（Rust/egui）

## 実装フェーズ

### Phase 1: 基本機能

#### Phase 1-1: 設定エクスポート/インポート

- 接続プロファイルの JSON 形式エクスポート
- API トークンの暗号化オプション
- パスワード保護付きエクスポート
- 既存設定とのマージ/上書きインポート

#### Phase 1-2: 事故防止セット

- **Production Lock**: 本番環境での DELETE/DROP/TRUNCATE をロック（タイムアウト付き解除機能）
- **危険クエリ検知**: 5段階リスク分析（Safe/Low/Medium/High/Critical）
- **実行ログ**: 全クエリの履歴を記録（タイムスタンプ、実行時間、影響行数、リスク評価）
- **環境バッジ**: DEV/STG/PROD の視覚的表示（色分け）

#### Phase 1-3: SQL 体験向上

- **SQL フォーマッター**: ワンクリックで整形
- **クエリ履歴**: 検索可能な履歴パネル（最大500件）
- **SQL スニペット**: よく使うクエリのテンプレート集
- **保存済みクエリ**: ユーザー定義クエリの保存・管理
- **バッチ実行**: 複数SQL文の連続実行と進捗追跡

#### Phase 1-4: クロスプラットフォーム対応

- macOS: Keychain
- Windows: Credential Manager
- Linux: Secret Service (libsecret)
- システム言語自動検出（日本語/英語）

---

### Phase 2: 高度な機能

#### Phase 2-1: Audit/Journal（擬似 Time-travel）

- INSERT/UPDATE/DELETE の変更履歴を自動記録
- 変更前後の値を保存
- ロールバック SQL 生成機能
- テーブル別フィルタリング

#### Phase 2-2: スキーマ探索強化

- 外部キー関連テーブルへのジャンプ
- 簡易 ER 図（ASCII 形式）
- テーブル関係の可視化

#### Phase 2-3: 複数 DB 比較

- 2 つの D1 データベース間のスキーマ差分表示
- テーブル追加/削除/変更の検出
- カラムレベルの差分表示
- マイグレーション SQL 自動生成

#### Phase 2-4: AI SQL 提案

- ローカル処理のみ（外部API呼び出しなし）
- スキーマコンテキスト対応
- カテゴリ別提案（基本クエリ、集計、結合、フィルタ、変更、スキーマ）
- 日本語/英語説明付き

---

### Phase 3: データ移行

#### Phase 3-1: MySQL ダンプインポート

- MySQL 形式の SQL ダンプを SQLite 互換に変換
- データ型マッピング（VARCHAR→TEXT, INT→INTEGER 等）
- CREATE TABLE 文の変換
- INSERT 文の変換
- 非対応構文の警告表示（TRIGGER, VIEW 等）

#### Phase 3-2: データ編集機能

- インラインセル編集（元に戻す/やり直し対応）
- 行操作（挿入、削除、複製）
- 高度なフィルタリング（=, !=, >, <, LIKE, IN, IS NULL 等）
- コンフリクト解決戦略（Fail/Replace/Skip）

---

### Phase 4: ローカル開発支援

#### Phase 4-1: ローカル D1 対応

- `.wrangler/state/v3/d1/` からローカルDB自動検出
- SQLite ベースのローカルデータベース対応
- バインディング認識

---

### Phase 5: モニタリング

#### Phase 5-1: 使用状況トラッキング

- Cloudflare API レート制限モニタリング
- セッション統計（クエリ数、読み書き行数、実行時間）
- バージョン確認・アップデートチェック（GitHub Releases）

---

## アーキテクチャ

```text
src/
├── main.rs            # エントリーポイント、eframe設定
├── app.rs             # メインアプリケーション（egui UI）
├── api.rs             # Cloudflare D1 API クライアント
├── i18n.rs            # 国際化（日本語/英語）
├── theme.rs           # UIテーマ・カラー・スタイルウィジェット
├── secure_storage.rs  # クロスプラットフォーム認証情報保存
├── settings_io.rs     # 設定のエクスポート/インポート
├── export.rs          # データエクスポート/インポート（CSV/JSON/SQL/MySQL）
├── sql_safety.rs      # SQLリスク分析・安全性チェック
├── sql_highlight.rs   # SQLシンタックスハイライト
├── audit.rs           # 変更履歴・Audit Journal・ロールバックSQL
├── schema_diff.rs     # スキーマ比較・マイグレーション生成
├── schema_explorer.rs # スキーマ探索・ER図生成
├── ai_suggest.rs      # ローカルSQL提案（外部API不使用）
├── local_db.rs        # Wranglerローカルデータベース検出
└── version.rs         # バージョン情報・アップデートチェック
```

---

## 技術スタック

- **UI**: egui 0.33 + eframe（ネイティブデスクトップ）
- **HTTP**: reqwest + tokio（非同期 API 通信）
- **認証情報保存**: keyring（OS ネイティブ）
- **ローカルDB**: rusqlite（SQLite接続）
- **シリアライズ**: serde + serde_json
- **言語検出**: sys-locale

---

## テスト

```bash
cargo test
```

ユニットテスト:

- Audit Journal
- CSV/JSON エクスポート
- MySQL 変換
- スキーマ差分
- SQL 安全性チェック
- 暗号化/復号化
- etc.

---

## セキュリティ設計

1. **API トークン**: OS ネイティブ認証情報ストアに保存（メモリ上は zeroize）
2. **設定エクスポート**: パスワード保護オプション
3. **Production Lock**: 本番環境での危険操作をブロック（タイムアウト付き解除）
4. **SQL 検証**: 5段階リスク分析と危険クエリの事前警告
5. **AI提案**: ローカル処理のみ（データは外部送信されない）
