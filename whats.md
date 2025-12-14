# D1 Manager - 実装内容まとめ

Production-safe な Cloudflare D1 データベース GUI クライアント（Rust/egui）

## 実装フェーズ

### Phase 1: 基本機能

#### Phase 1-1: 設定エクスポート/インポート

- 接続プロファイルの JSON 形式エクスポート
- API トークンの暗号化オプション（AES-256-GCM）
- パスワード保護付きエクスポート
- 既存設定とのマージ/上書きインポート

#### Phase 1-2: 事故防止セット

- **Production Lock**: 本番環境での DELETE/DROP/TRUNCATE をロック
- **危険クエリ検知**: WHERE 句なし DELETE/UPDATE、DROP TABLE 等を警告
- **実行ログ**: 全クエリの履歴を記録（タイムスタンプ、実行時間、影響行数）
- **環境バッジ**: DEV/STG/PROD の視覚的表示

#### Phase 1-3: SQL 体験向上

- **SQL フォーマッター**: ワンクリックで整形
- **クエリ履歴**: 検索可能な履歴パネル
- **SQL スニペット**: よく使うクエリのテンプレート集

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
- 簡易 ER 図（ASCII/Mermaid 形式）
- テーブル関係の可視化

#### Phase 2-3: 複数 DB 比較

- 2 つの D1 データベース間のスキーマ差分表示
- テーブル追加/削除/変更の検出
- カラムレベルの差分表示
- マイグレーション SQL 自動生成

---

### Phase 3: データ移行

#### Phase 3-1: MySQL ダンプインポート

- MySQL 形式の SQL ダンプを SQLite 互換に変換
- データ型マッピング（VARCHAR→TEXT, INT→INTEGER 等）
- CREATE TABLE 文の変換
- INSERT 文の変換
- 非対応構文の警告表示（TRIGGER, VIEW 等）

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
└── local_db.rs        # ローカルキャッシュ
```

---

## 技術スタック

- **UI**: egui 0.33 + eframe（ネイティブデスクトップ）
- **HTTP**: reqwest + tokio（非同期 API 通信）
- **認証情報保存**: keyring（OS ネイティブ）
- **暗号化**: AES-256-GCM（設定エクスポート用）
- **正規表現**: regex-lite（軽量）
- **シリアライズ**: serde + serde_json

---

## テスト

```bash
cargo test
```

40 件のユニットテスト:

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
2. **設定エクスポート**: AES-256-GCM 暗号化（パスワード保護）
3. **Production Lock**: 本番環境での危険操作をブロック
4. **SQL 検証**: 危険クエリの事前警告
