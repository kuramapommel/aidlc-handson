<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-10-28T05:43:33Z — 縦割りユニットには種別（service/ui など）を付けなかった; 各ユニットが Rust の実行ファイルと MoonBit の画面の両方にまたがり、1 つの種別に当てはまらないため、設計書一式を作る扱いにした。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-10-28T13:18:25Z — 受け入れ基準がユニットをまたがないよう、承認済みのユーザーストーリーを組み直した（AC2.1.3 → AC1.8.4、AC3.5.3 の案内先変更、AC4.1.3 の一部 → AC4.2.4、AC4.2.5 追加）; ユーザーから「後続ユニットで満たす形は疎結合に反する」と指摘があり、許可を得て修正した。ユニット分割では各ユニットが担当ストーリーの受け入れ基準を単独で満たせるかを確認する。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-10-28T05:43:33Z — 一部の受け入れ基準（AC2.1.3、AC3.5.3、AC4.1.3）は主担当ユニットではなく後続ユニットで満たす形にした; ストーリーを分割せずに主担当を 1 つに保つため、またがる部分をストーリー対応表に明記した。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-10-28T05:43:33Z — 画面用の静的ファイルサーバーの具体的な手段（自作の小さな Rust サーバーか、既存のツールか）; コンストラクションで決める。
