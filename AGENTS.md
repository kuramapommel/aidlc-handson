# AGENTS.md（リポジトリ全体の案内）

研修コース e チケット購入サイト（ローカル検証専用）の monorepo です。AI のエージェントがこのリポジトリで作業するときの道しるべです。人向けの説明はルートの [README.md](README.md) にあります。

## 構成

| 場所 | 中身 | 詳しい案内 |
| --- | --- | --- |
| `Cargo.toml`・`crates/` | Rust のワークスペース。業務の 4 crate（`course_catalog`・`eligibility`・`training_budget`・`purchasing`）、アダプターの `in_memory_store`・`web_api`、画面用の `static_server` | [README.md](README.md) の「リポジトリの構成」 |
| `web/` | MoonBit の画面（TEA。rabbita と mizchi/js_web） | [web/AGENTS.md](web/AGENTS.md)・[web/README.md](web/README.md) |
| `scripts/` | `verify.sh`（すべての確認）・`build-web.sh`（画面のビルド） | 各スクリプトの先頭のコメント |
| `.githooks/` | コミット前の確認（`cargo fmt --check` と `moon -C web check`） | [.githooks/README.md](.githooks/README.md) |
| `.github/workflows/` | GitHub Copilot のコーディングエージェント向けの環境の準備 | `copilot-setup-steps.yml` の先頭のコメント |
| `aidlc/` | AI-DLC の記録（要件・設計・計画など）と、チームの決まり | `.claude/CLAUDE.md` |

## 全体の決まり

- 構成は六角形アーキテクチャ。業務の crate は HTTP・データ保存・画面に依存させない。Web API・データストア・Web 画面はアダプターとして分け、依存の逆転はインターフェース（トレイトのオブジェクト）ではなく関数の受け渡しで行う。
- 関数型で書く。業務の crate は純粋関数だけを持ち、現在時刻と乱数は引数の関数で受け取る。画面の更新関数と描画関数も純粋関数にする。
- 秘密情報をコードに書かない。設定は環境変数で変えられるようにし、既定値をコードに持つ（README の「環境変数」）。
- 人が読む文章（README・コメント・画面の文言）は日本語、識別子は英語。

## コマンド（リポジトリのルートで実行）

```bash
cargo test -p course_catalog -p eligibility -p training_budget -p purchasing -p in_memory_store -p web_api -p static_server
moon -C web test --target js     # 画面のテスト（Node.js が必要）
bash scripts/build-web.sh        # 画面のビルド（web/dist）
bash scripts/verify.sh           # fmt → clippy（-D warnings）→ cargo test → moon test → 画面のビルド
```

確認の基準（clippy の `-D warnings`、テストの対象など）を、通すために弱めてはならない。
