# D1 Manager - 実装内容まとめ

Production-safe な Cloudflare D1 データベース GUI クライアント（Rust/egui）

## 実装フェーズ

### Phase 1: 基本機能

#### Phase 1-1: 設定エクスポート/インポート

- 接続プロファイルのJSON形式エクスポート
- APIトークンの暗号化オプション（AES-256-GCM）
- パスワード保護付きエクスポート
- 既存設定とのマージ/上書きインポート

#### Phase 1-2: 事故防止セット

- **Production Lock**: 本番環境でのDELETE/DROP/TRUNCATEをロック
- **危険クエリ検知**: WHERE句なしDELETE/UPDATE、DROP TABLE等を警告
- **実行ログ**: 全クエリの履歴を記録（タイムスタンプ、実行時間、影響行数）
- **環境バッジ**: DEV/STG/PROD の視覚的表示

#### Phase 1-3: SQL体験向上

- **SQLフォーマッター**: ワンクリックで整形
- **クエリ履歴**: 検索可能な履歴パネル
- **SQLスニペット**: よく使うクエリのテンプレート集

#### Phase 1-4: クロスプラットフォーム対応

- macOS: Keychain
- Windows: Credential Manager
- Linux: Secret Service (libsecret)
- システム言語自動検出（日本語/英語）

---

### Phase 2: 高度な機能

#### Phase 2-1: Audit/Journal（擬似Time-travel）

- INSERT/UPDATE/DELETEの変更履歴を自動記録
- 変更前後の値を保存
- ロールバックSQL生成機能
- テーブル別フィルタリング

#### Phase 2-2: スキーマ探索強化

- 外部キー関連テーブルへのジャンプ
- 簡易ER図（ASCII/Mermaid形式）
- テーブル関係の可視化

#### Phase 2-3: 複数DB比較

- 2つのD1データベース間のスキーマ差分表示
- テーブル追加/削除/変更の検出
- カラムレベルの差分表示
- マイグレーションSQL自動生成

---

### Phase 3: データ移行・AI機能

#### Phase 3-1: AI SQL提案（安全設計付き）

- **ローカル処理のみ**: 外部APIへのデータ送信なし
- スキーマベースの提案カテゴリ:
  - 基本クエリ（SELECT, COUNT, DISTINCT）
  - 集計（SUM, AVG, GROUP BY）
  - フィルタ/ソート（WHERE, LIKE, 日付フィルタ）
  - 結合（外部キーベースJOIN）
  - スキーマ（PRAGMA, テーブル情報）
  - 変更（INSERT/UPDATE/DELETE）※デフォルト非表示
- カラム型自動判定（数値、テキスト、日付）
- 日本語/英語対応

#### Phase 3-2: MySQLダンプインポート

- MySQL形式のSQLダンプをSQLite互換に変換
- データ型マッピング（VARCHAR→TEXT, INT→INTEGER等）
- CREATE TABLE文の変換
- INSERT文の変換
- 非対応構文の警告表示（TRIGGER, VIEW等）

---

## アーキテクチャ

```text
src/
├── main.rs            # エントリーポイント
├── app.rs             # メインアプリケーション（egui UI）
├── api.rs             # Cloudflare D1 API クライアント
├── i18n.rs            # 国際化（日本語/英語）
├── theme.rs           # UIテーマ・カラー定義
├── secure_storage.rs  # クロスプラットフォーム認証情報保存
├── settings_io.rs     # 設定のエクスポート/インポート
├── export.rs          # データエクスポート（CSV/JSON/SQL/MySQL）
├── sql_safety.rs      # SQL安全性チェック
├── sql_highlight.rs   # SQLシンタックスハイライト・フォーマット
├── audit.rs           # 変更履歴・Audit Journal
├── schema_diff.rs     # スキーマ比較・マイグレーション生成
├── schema_explorer.rs # スキーマ探索・ER図
├── ai_suggest.rs      # AI SQL提案
└── local_db.rs        # ローカルキャッシュ
```

---

## 技術スタック

- **UI**: egui 0.33 + eframe（ネイティブデスクトップ）
- **HTTP**: reqwest + tokio（非同期API通信）
- **認証情報保存**: keyring（OS ネイティブ）
- **暗号化**: AES-256-GCM（設定エクスポート用）
- **正規表現**: regex-lite（軽量）
- **シリアライズ**: serde + serde_json

---

## テスト

```bash
cargo test
```

40件のユニットテスト:

- AI提案生成
- Audit Journal
- CSV/JSONエクスポート
- MySQL変換
- スキーマ差分
- SQL安全性チェック
- 暗号化/復号化
- etc.

---

## セキュリティ設計

1. **APIトークン**: OSネイティブ認証情報ストアに保存（メモリ上はzeroize）
2. **設定エクスポート**: AES-256-GCM暗号化（パスワード保護）
3. **Production Lock**: 本番環境での危険操作をブロック
4. **AI機能**: ローカル処理のみ（外部送信なし）
5. **SQL検証**: 危険クエリの事前警告
