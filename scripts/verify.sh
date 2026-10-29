#!/usr/bin/env bash
# Construction の確認コマンド（NFR9.3）。リポジトリのルートで `bash scripts/verify.sh` と実行する。
#
# 次を順に実行し、どれかが失敗すれば失敗（終了コード 1）で終わる。
#   1. Rust のフォーマットの確認（cargo fmt --check）
#   2. cargo clippy（警告をエラーとして扱う）
#   3. Rust のテスト（cargo test）
#   4. MoonBit の画面のテスト（moon test）
#   5. 画面のビルド（scripts/build-web.sh）
# MoonBit のツールや画面のプロジェクトがない場合は、その旨を表示して失敗で終わる（成功と偽らない）。

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# 公式のインストーラーの既定の場所にある MoonBit のツールも使えるようにする
if [ -d "${MOON_HOME:-$HOME/.moon}/bin" ]; then
  PATH="${MOON_HOME:-$HOME/.moon}/bin:$PATH"
fi

step() {
  printf '\n==> %s\n' "$1"
}

fail() {
  printf '\n[失敗] %s\n' "$1" >&2
  exit 1
}

step "1/5 Rust のフォーマットの確認（cargo fmt --check）"
cargo fmt --all --check || fail "フォーマットが整っていません。cargo fmt --all を実行してください"

step "2/5 cargo clippy（警告をエラーとして扱う）"
cargo clippy --workspace --all-targets -- -D warnings || fail "clippy の指摘があります"

step "3/5 Rust のテスト（cargo test）"
cargo test --workspace || fail "Rust のテストが失敗しました"

step "4/5 MoonBit の画面のテスト（moon test）"
if ! command -v moon >/dev/null 2>&1; then
  fail "MoonBit のツール（moon）が見つかりません。README の「前提のツール」に従って導入してください"
fi
if [ ! -f web/moon.mod ] && [ ! -f web/moon.mod.json ]; then
  fail "画面のプロジェクト（web/ の moon.mod）がありません。画面（MoonBit・rabbit-tea）はまだ作られていません"
fi
moon test --directory web || fail "MoonBit のテストが失敗しました"

step "5/5 画面のビルド（scripts/build-web.sh）"
if [ ! -f scripts/build-web.sh ]; then
  fail "画面のビルドのスクリプト（scripts/build-web.sh）がありません"
fi
bash scripts/build-web.sh || fail "画面のビルドが失敗しました"

printf '\n[成功] すべての確認が通りました\n'
