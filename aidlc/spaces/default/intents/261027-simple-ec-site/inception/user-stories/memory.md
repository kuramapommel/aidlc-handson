<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-10-27T08:36:40Z — ペルソナ例の「動画形式」をライブ配信・会場の数時間研修に読み替えた; 要件 FR3（日程と定員）と矛盾しないようにし、確認時に合意を得た。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-10-27T08:36:40Z — アカウント切り替え（FR1.2）と初期データ（FR7）はストーリーにせず機能設計へ送った; ペルソナを 1 人に絞ったため、検証用機能を利用者の価値として書くと不自然になるので、受け入れ基準を確認する手段として扱った。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
