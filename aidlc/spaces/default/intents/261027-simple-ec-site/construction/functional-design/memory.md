<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-03T10:58:00Z — [unit u3-my-tickets] 「受講済み」の判定は開始日時が現在時刻以前とした（BR12.3）; 確認事項 Q1 は「開催日時が過ぎた」とだけ言っていたため、U1 の受付終了と同じ時点（開始日時）にそろえた。受講中のチケットも受講済みに並ぶ。
- 2026-11-03T10:58:00Z — [unit u3-my-tickets] 一覧の詳細をその場で開く（Q2: B）ため、一覧の応答に 7 項目まで含めた（BR13.2）; 開くたびに通信しないことで、詳細の受け口を一覧から使わずに済む。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-03T10:58:00Z — [unit u3-my-tickets] 前のセッションで作った未回答の確認事項ファイルを一度上書きしてしまい、コミット済みの版に戻してから前提を 1 行足した; 回答はまだなかったので失われた内容はない。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-03T10:58:00Z — [unit u3-my-tickets] 開閉は複数同時に開ける形にし、状態は URL に残さないとした; 1 つずつに限ると比べにくく、URL に残すと U1 の Route を広げる必要があるため。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
