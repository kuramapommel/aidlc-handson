<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-10-27T08:12:26Z — 「シンプルな EC サイト」を研修コースの e チケット購入（LMS 型）として具体化した; Q1・Q2 の回答で商材と利用者が大きく変わったため、カートを持たず 1 回 1 日程の購入（前提 A1）と解釈した。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-10-27T08:12:26Z — 回答の途中で Q3〜Q5・Q8 の選択肢を書き換えた; 当初は物販の EC を想定した質問だったが、Q1 で商材が研修コースに変わったため、支払い（研修予算）や在庫（日程と定員）に合う選択肢へ置き換えてから質問した。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-10-27T08:12:26Z — MoonBit の画面構築に使える UI ライブラリが 2026 年 10 月時点で何か; 未確認のため OQ1 としてアプリケーション設計に持ち越した。
