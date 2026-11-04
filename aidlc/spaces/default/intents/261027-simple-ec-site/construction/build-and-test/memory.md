<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-03T13:40:00Z — 目視・手動確認と書かれた目標は、2 プロセスを起動して Playwright で操作し、ログを読んで確かめた; 人の目視の代わりの自動の確認で、確認できない観点（処理中の表示が一瞬で消える）は理由を書いて残した。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-04T06:21:24Z — Loop-back 1: 接続失敗時の画面の例外を直すため U1 のコード生成に戻す; 利用者の指示で、読み出しは自動で 3 回まで再試行し、だめなら時間をおいて試すよう案内する形にした。購入（POST）は二重の送信を避けるため自動で再送しない。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-03T13:40:00Z — Web API に接続できないときの画面の例外が、U1・U2 を作った時点の moon でも起きていたかは未確認; 今回入れた moon 0.1.20261102 でビルドした結果で確認した。
