<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-10-28T13:40:15Z — 「並行 bolt で対応したい」という前段のご発言を、進め方と同時進行の質問への回答として記録し、改めて質問しなかった; すでに意思が示されていたため、同じことを聞き直さない。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-10-28T13:40:15Z — 「外部の要因はない」との回答後に、MoonBit のツールの取得先がネットワーク設定でブロックされていることを実測で見つけ、外部依存マップに記録した; 開発環境で使うツールの取得可否は、デリバリー計画の質問の前に実際に確かめておくとよい。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-10-28T13:40:15Z — 1 セッションでの並行実行（stage-major＋swarm）から、チームごとのユニット担当（unit-major・チーム所有）に切り替えた; 利用者がチームごとの独立した進行と承認を選んだため。U2・U3 の同時進行はチーム単位で実現する。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
