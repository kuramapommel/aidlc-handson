# ビルドの手順

リポジトリのルートで実行する。手順は `README.md`（ビルドとテスト・起動）と同じ。

## 前提のツール

| ツール | 確認した版 | 導入 |
| --- | --- | --- |
| Rust（rustc・cargo・rustfmt・clippy） | 最新の stable | <https://rustup.rs/> |
| MoonBit（moon） | 0.1.20261102（この確認で導入） | `curl -fsSL https://cli.moonbitlang.com/install/unix.sh \| bash`（`~/.moon/bin` に入る。シェルの PATH に足す） |
| Node.js | 22 系 | 画面のテスト（`moon test --target js`）にだけ使う |

moon を初めて入れた環境では、画面の依存を取得する前に `moon update`（パッケージの一覧の取得）が必要だった。

## 環境の準備

- 環境変数は既定値のままでよい（`API_PORT=3000`・`UI_ORIGIN=http://localhost:8080`・`STATIC_PORT=8080`・`STATIC_DIR=web/dist`・`RUST_LOG=info`）。ログを詳しくするときは `RUST_LOG=debug`。
- 秘密情報やローカルのサービス（DB など）は使わない。データはメモリ上にあり、Web API を再起動すると初期データに戻る。

## ビルド

```bash
cargo build --workspace          # Rust（業務の 4 crate・データストア・Web API・静的ファイルサーバー）
bash scripts/build-web.sh        # 画面（moon -C web build --target js --release → web/dist）
```

## ビルドの確認

```bash
bash scripts/verify.sh           # fmt → clippy（警告はエラー）→ cargo test → moon test → 画面のビルド
cargo run -p web_api             # http://localhost:3000（GET /api/health が 200）
cargo run -p static_server       # http://localhost:8080（別の端末で）
```

## よくある問題

| 症状 | 原因と対処 |
| --- | --- |
| `[失敗] MoonBit のツール（moon）が見つかりません` | moon が PATH にない。上の手順で導入し、`~/.moon/bin` を PATH に足す |
| `module was not found in the registry` | moon のパッケージの一覧が古い。`moon update` を実行する |
| 静的ファイルサーバーが起動直後に終了する | `web/dist/index.html` がない。先に `bash scripts/build-web.sh` |
| 新しい moon で `derive(...)` の警告が出る | 書き方が古いという警告で、エラーではない（ビルドとテストは成功する） |
