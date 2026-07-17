# VVcall Server

**言語:** [English](README.md) | [Русский](README.ru.md) | [日本語](README.ja.md)

Rust で構築されたピアツーピア音声チャットサーバー。WebSocket 通信、ユーザー認証、クラウド設定ストレージを備えています。

## 機能

- **WebSocket ベースのリアルタイム通信** - WebSocket を介して音声ルームに接続
- **ユーザー認証** - 安全なパスワードハッシュ化（bcrypt）による登録とログイン
- **ルームシステム** - 音声ルームの動的な作成と参加
- **クラウド設定** - ユーザー設定の保存と取得
- **SQLite データベース** - ユーザーと設定の永続ストレージ

## テックスタック

- **Rust** - コア言語
- **Axum** - WebSocket サポート付き Web フレームワーク
- **Tokio** - 非同期ランタイム
- **bcrypt** - パスワードハッシュ化
- **rusqlite** - SQLite データベースバインディング
- **serde/serde_json** - JSON シリアライゼーション

## インストール

### 前提条件

- Rust（edition 2024 以降または互換バージョン）
- Cargo パッケージマネージャー

### ビルドと実行

```bash
# リポジトリをクローン
git clone https://github.com/vvcall-dev/server.git
cd server

# プロジェクトをビルド
cargo build --release

# サーバーを実行
cargo run
```

サーバーはデフォルトでポート `3030` で起動します。

## 設定

データベースパスを指定するには、`DATABASE_URL` 環境変数を設定します：

```bash
export DATABASE_URL=/path/to/vvcall.db
```

設定しない場合、カレントディレクトリの `vvcall.db` がデフォルトで使用されます。

## API エンドポイント

### 認証

#### 登録
```http
POST /api/register
Content-Type: application/json

{
    "username": "user123",
    "password": "securepassword"
}
```

#### ログイン
```http
POST /api/login
Content-Type: application/json

{
    "username": "user123",
    "password": "securepassword"
}
```

レスポンスにはセッショントークンと保存された設定が含まれます。

### 設定

#### 設定の更新
```http
POST /api/config
Content-Type: application/json

{
    "token": "session-token",
    "config_json": "{\"volume\": 80, \"mic_gain\": 50}"
}
```

### WebSocket

#### ルームに接続
```
WS /ws/:room
```

`:room` を希望するルーム名に置き換えて音声ルームに接続します。ルームに送信されたメッセージは、接続中のすべてのクライアントにブロードキャストされます。

## データベーススキーマ

サーバーは以下の構造で `users` テーブルを作成します：

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    token TEXT,
    config_json TEXT
);
```

## Docker サポート

Docker Compose を使用してビルドおよび実行：

```bash
docker-compose up --build
```

## ライセンス

詳細は [LICENSE](LICENSE) ファイルをご覧ください。

## プロジェクト構造

```
p2p-server/
├── src/
│   └── main.rs      # メインアプリケーションコード
├── Cargo.toml       # Rust 依存関係
├── Dockerfile       # Docker 設定
├── docker-compose.yml
└── README.ja.md     # このファイル
```
