#!/usr/bin/env bash
# 画面のビルド（Step 16）。リポジトリのルートで `bash scripts/build-web.sh` と実行する。
#
# 1. MoonBit の画面を JavaScript 向けにビルドする（moon -C web build --target js --release）
# 2. 静的ファイルサーバーが配信するディレクトリ（既定は web/dist。環境変数 WEB_DIST で変えられる）に、
#    ビルド結果の main.js と、web/static/ の index.html・style.css をそろえる
# 3. Web API のオリジンの設定 config.js を作る（環境変数 API_BASE。既定は http://localhost:3000）
#
# MoonBit のツールがなければ、その旨を表示して失敗で終わる。

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# 公式のインストーラーの既定の場所にある MoonBit のツールも使えるようにする
if [ -d "${MOON_HOME:-$HOME/.moon}/bin" ]; then
  PATH="${MOON_HOME:-$HOME/.moon}/bin:$PATH"
fi

if ! command -v moon >/dev/null 2>&1; then
  printf '[失敗] MoonBit のツール（moon）が見つかりません。README の「前提のツール」に従って導入してください\n' >&2
  exit 1
fi

DIST="${WEB_DIST:-web/dist}"
API_BASE="${API_BASE:-http://localhost:3000}"

# API_BASE は JavaScript の文字列に入れるため、http(s)://host[:port] の形だけを受け付ける
if ! printf '%s' "$API_BASE" | grep -Eq '^https?://[A-Za-z0-9.-]+(:[0-9]+)?$'; then
  printf '[失敗] API_BASE は http://host:port の形で指定してください（末尾の / は付けない）: %s\n' "$API_BASE" >&2
  exit 1
fi

echo "画面をビルドします（moon -C web build --target js --release）"
moon -C web build --target js --release

MAIN_JS="web/_build/js/release/build/cmd/main/main.js"
if [ ! -f "$MAIN_JS" ]; then
  printf '[失敗] ビルド結果 %s が見つかりません\n' "$MAIN_JS" >&2
  exit 1
fi

mkdir -p "$DIST"
cp "$MAIN_JS" "$DIST/main.js"
cp web/static/index.html "$DIST/index.html"
cp web/static/style.css "$DIST/style.css"
printf '// scripts/build-web.sh が作るファイル（手で編集しない）\nglobalThis.AIDLC_API_BASE = "%s";\n' "$API_BASE" > "$DIST/config.js"

echo "画面を $DIST にそろえました（Web API: $API_BASE）"
