<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-10-27T08:49:36Z — 業務 4 コンポーネントに加え、Web API・データストア・Web 画面をアダプターとしてコンポーネントに含めた; ステージの定義は「自分たちで書くコード」をコンポーネントとするため、関数の形で依存を逆転させる六角形アーキテクチャとして明示した。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-10-27T08:49:36Z — コースカタログと購入の循環依存を、発行済み数を返す関数を引数で受け取る形（高階関数）で解消した; 探索専用のコンポーネントを別に作る案より小さく、関数型の方針にも合うため。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-10-27T08:49:36Z — MoonBit で TEA を実現するライブラリの有無と成熟度; 未確認のため、技術選定（要件 OQ1）で調べる。
