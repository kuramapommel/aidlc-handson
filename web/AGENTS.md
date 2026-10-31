# Project Agents.md Guide

This is a [MoonBit](https://docs.moonbitlang.com) project.

## この画面のプロジェクトの決まり（研修コース e チケット）

monorepo 全体の構成はリポジトリのルートの [AGENTS.md](../AGENTS.md)、画面の作りとコマンドは [README.mbt.md](README.mbt.md) を参照する。コマンドはリポジトリのルートから `moon -C web ...` の形で実行する（`moon test --directory` はこの版にない）。

### TEA

- 画面は The Elm Architecture で作る。TEA の実行環境は自作せず、rabbita の `@rabbita.elmish` を使う。
- 状態（`Model`）は 1 つの不変な値にし、更新関数 `update : (Model, Msg) -> (Model, Array[Effect])` と描画関数 `view` はどちらも純粋関数にする。
- 通信・URL の書き換え・ページの移動は、更新関数が `Effect` の値として返すだけにし、`web.mbt` の `effect_to_cmd` だけがそれを rabbita のコマンドに変える。更新関数の中で通信しない。
- 応答のメッセージには、要求したときの従業員の世代（`generation`）と対象の ID を持たせ、今の画面・世代と合わない古い応答は捨てる。
- 購入確認の Ready／Blocked のように、ほかの状態から計算できる値は保存せずに `confirm_readiness` のような関数で求める。

### 関数型

- 可変な状態は持たない（ループの中の一時的な変数は除く）。値は `{ ..model, field: value }` で作り直す。
- 応答の読み取り（`decode.mbt`）・表示の形（`format.mbt`）・URL の変換（`route.mbt`）は純粋関数にして、`web_wbtest.mbt` でテストする。
- 更新関数のテスト（`web_test.mbt`）では、Web API の応答をメッセージとして直接与える。ブラウザや Web API は使わない。

### rabbita と mizchi/js_web の使い分け

- rabbita（0.17.0）: TEA の実行環境・描画（`@html`）・URL の購読（`@sub.on_url_changed`・`@sub.on_url_request`）・URL の書き換え（`@nav.push_url`・`@nav.replace_url`）。
- rabbita の通信の部品（`@http`）は使わない。別オリジンへの Cookie の送信・専用ヘッダー・2xx 以外の本文の受け取りができないため。
- 通信は `http.mbt` の `request_cmd` だけを使う。中身は `mizchi/js_web` の `@fetch.fetch` で、常に `credentials: "include"`、POST には `X-Requested-With: aidlc-handson-web` を付け、応答は状態コードと本文の文字列で返す。非同期の処理は `@rabbita.attempt` でコマンドにする。
- 金額は送らない（購入の本文は日程 ID だけ）。購入の可否と金額の最終判断は Web API が行う。

### 描画

- 操作できる部品には `data-testid`（小文字のケバブケース。繰り返す部品は業務の ID を末尾に付ける）を付ける。
- 状態は色だけでなく文字で示す。ボタン・日程の選択（ラジオボタンとラベル）・従業員の切り替えには名前を付け、キーボードで操作できるようにする。選べない日程は `disabled` にし、理由を `aria-describedby` で読み上げにも含める。
- Web API から受け取った文字列は文字（テキストノード）として描画し、HTML として解釈させない。
- 画面に出す文言は日本語、識別子は英語で書く。

You can browse and install extra skills here:
<https://github.com/moonbitlang/skills>

## Project Structure

- MoonBit packages are organized per directory; each directory contains a
  `moon.pkg` file listing its dependencies. Each package has its files and
  blackbox test files (ending in `_test.mbt`) and whitebox test files (ending in
  `_wbtest.mbt`).

- In the toplevel directory, there is a `moon.mod` file listing module
  metadata.

## Coding convention

- MoonBit code is organized in block style, each block is separated by `///|`,
  the order of each block is irrelevant. In some refactorings, you can process
  block by block independently.

- Try to keep deprecated blocks in file called `deprecated.mbt` in each
  directory.

## Tooling

- `moon fmt` is used to format your code properly.

- `moon ide` provides project navigation helpers like `peek-def`, `outline`, and
  `find-references`. See $moonbit-agent-guide for details.

- `moon info` is used to update the generated interface of the package, each
  package has a generated interface file `.mbti`, it is a brief formal
  description of the package. If nothing in `.mbti` changes, this means your
  change does not bring the visible changes to the external package users, it is
  typically a safe refactoring.

- In the last step, run `moon info && moon fmt` to update the interface and
  format the code. Check the diffs of `.mbti` file to see if the changes are
  expected.

- Run `moon test` to check tests pass. MoonBit supports snapshot testing; when
  changes affect outputs, run `moon test --update` to refresh snapshots.

- Prefer `assert_eq` or `assert_true(pattern is Pattern(...))` for results that
  are stable or very unlikely to change. For snapshot tests that record
  structured debugging output, derive `Debug` and use `debug_inspect`, rather
  than deriving `Show` for debugging. For solid, well-defined results (e.g.
  scientific computations), prefer assertion tests. You can use
  `moon coverage analyze > uncovered.log` to see which parts of your code are
  not covered by tests.
