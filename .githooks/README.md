# コミット前の確認（Git のフック）

`pre-commit` は、コミットの前に monorepo 全体の次の確認を行う。どちらかが失敗するとコミットは止まる。

1. `cargo fmt --all --check`（Rust のフォーマット）
2. `moon -C web check`（MoonBit の画面の型の確認）

clippy・テスト・画面のビルドまで含めたすべての確認は、リポジトリのルートで `bash scripts/verify.sh` を実行する。

## 有効にする方法

Git はフックの置き場所の設定をリポジトリに含めないため、各自の手元で 1 回だけ、リポジトリのルートで次を実行する。

```bash
git config core.hooksPath .githooks
```

`pre-commit` に実行の権限がない環境では、あわせて `chmod +x .githooks/pre-commit` を実行する。
