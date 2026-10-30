# コード生成の計画（U2 コース探索）

## Summary

- Builds: コース一覧の探索（キーワード・カテゴリ・空席ありのみ・期間・4 つの並べ替え）、条件の検証と 400、よみがな、画面の探索フォーム・URL 同期・0 件と失敗の表示
- Touches: `crates/course_catalog/`（探索の関数）、`crates/in_memory_store/src/seed.rs`（よみがな）、`crates/web_api/`（一覧の受け口）、`web/`（探索の状態・更新・描画）、`README.md`
- Tests: Rust の探索のテスト 約 25 件（正規化・絞り込み・並べ替え・400・性能）、画面の更新関数のテスト 約 15 件

## 前提と進め方

- 設計の入力: U2 の機能設計（`../functional-design/` の entities.md・rules.md（BR8〜BR11）・functional-spec.md・frontend-components.md）、U2 の NFR 要件（`../nfr-requirements/`）、U1 の設計とコード（U1 が用意した共有部分に「足す」だけで、U1 の振る舞いは変えない）。
- テストの進め方は下の Testing Contract（test-after）に従う。層ごとに「実装 → その層のテストを書いて実行」の順にする。量は Minimal（要件ごとに 1 件、部品ごとに成功の場合を 1 件以上）に、境目と NFR の自動テストを加える。
- 関数型で書く（NFR6）。探索の関数は純粋関数とし、現在時刻は引数で受け取る。
- U1 のテスト（Rust 51 件・画面 32 件）はすべて成功のままにする。条件なしの一覧の応答は U1 と変えない。
- U2 のテストは U2 用のファイルにまとめ、U2 だけを指定して実行できるようにする（`unit-test-instructions.md`）。

## ストーリーとの対応

| ストーリー | 主な Step |
| --- | --- |
| US1.2 キーワードで探す | 3〜6、9〜10 |
| US1.3 カテゴリ／US1.4 空席ありのみ／US1.5 期間 | 3〜6、9〜10 |
| US1.6 開催日の近い順／US1.7 受講料・コース名順 | 3〜6、9〜10 |
| US1.8 URL で残す | 9〜10 |
| US1.9 見つからない・読み込めない | 7〜10 |

## Steps

- [ ] **Step 1: 構成と依存の準備**
  - `crates/course_catalog` に `unicode-normalization` の最新 stable 版を `cargo add` で追加する（U2 の tech-stack-decisions.md）。
  - 探索の Rust のコードは `crates/course_catalog/src/search.rs`（条件・検証・正規化・絞り込み・並べ替え）に置く。
- [ ] **Step 2: テストの実行の準備**
  - U2 用のテストの置き場所を作る: `crates/course_catalog/src/search_tests.rs`、`crates/web_api/tests/search.rs`、`web/search_test.mbt`。
  - `unit-test-instructions.md` の U2 だけを実行するコマンドが動くこと（0 件で成功）を確かめる。MoonBit のファイル指定のオプションが今の moon で使えない場合は、同じ意味の指定（パッケージやテスト名の絞り込み）に置き換えて、その旨を `code-summary.md` に書く。
- [ ] **Step 3: データの形（よみがな）を実装する**（BR10.4）
  - Course に titleReading を足し、検証（1〜120 文字）を足す。`crates/in_memory_store/src/seed.rs` のすべてのコースによみがなを書く（英字は読みに直す）。
  - 機能仕様 6 節の探索の確認用のデータがそろっているか確かめ、足りなければ U1 の振る舞い（U1 のテスト）を変えない範囲でコースや日程を足す。
- [ ] **Step 4: データの形のテストを書いて実行する**
  - すべてのコースによみがながあり、ひらがな・長音符・英数字・空白だけであること。U1 のテストが成功のままであること。
- [ ] **Step 5: 探索の業務ロジックを実装する**（BR8.1〜BR8.5、BR9.1〜BR9.5、BR10.1〜BR10.5）
  - 生の条件 → SearchCondition か不正な条件の一覧（BR8.2〜BR8.5）。キーワードは拒否しない（BR8.1）。
  - キーワードの正規化: 先頭 200 文字、NFKC、英字の小文字化、カタカナ → ひらがな、空白で区切る。
  - 絞り込み（キーワードの AND・カテゴリ・空席ありのみ・期間（両端を含む））と並べ替え（開催日が近い順は期間に関係なく、受講料の昇順・降順、よみがなの五十音順。どれも最後にコース ID）。発行済み数と現在時刻は引数で受け取る（ADR-005）。
  - PostgreSQL に移すときの注意をコメントに書く（NFR7.3）。
- [ ] **Step 6: 探索の業務ロジックのテストを書いて実行する**
  - 正規化（全角「ＡＩ」・小文字「ai」・「まねじめんと」・全角スペース・200 文字を超える入力・空白だけ）。検証の各不正値。絞り込み（AC1.2.1〜AC1.5.4 の各場面、受付終了の日程にだけ空席が残るコース、開始日 = 終了日、開始日だけ）。並べ替え（4 つ、同値の扱い、予定なしは最後、期間で絞っても直近の開催日は期間外を含む）。データストアを使わず、固定のデータを返す関数を渡す（NFR7.2）。
- [ ] **Step 7: Web API の受け口を実装する**（BR8.6、NFR3.1〜NFR3.6、NFR10.1〜NFR10.2）
  - `GET /api/courses` で `q`・`category`・`available`・`from`・`to`・`sort` を受け取り、検証して探索する。不正なら 400 と `{ error: { code: "invalid-search-condition", fields: [{ field, reason }] } }`。条件なしは U1 と同じ応答。
  - ログにクエリの値を残さない（400 は項目名だけ）。
- [ ] **Step 8: Web API のテストを書いて実行する**
  - 条件付きの応答（件数を含む）、各不正な条件で 400 と項目名、複数の不正を全部挙げる、キーワードでは 400 にならない（数万文字でも 200 ms 以内）、条件なしは U1 と同じ、条件の組み合わせを替えた 100 回の 95 パーセンタイルが 200 ms 以内（NFR1.1）、探索でデータが変わらない（NFR2.1）。
- [ ] **Step 9: 画面の探索を実装する**（BR11.1〜BR11.10、NFR1.2〜NFR1.3、NFR4、NFR5、NFR3.4〜NFR3.5）
  - `frontend-components.md` のとおり、SearchForm（キーワード・カテゴリ・空席ありのみ・開始日・終了日・並べ替え）、EmptyState、ConditionError、状態（searchInput・periodError・keywordDebounce・searchRequestSeq）、メッセージと更新を作る。
  - キーワードは rabbita の `@cmd.delay` で 150 ms 待ってから送る。最後の要求の応答だけを反映する。URL は履歴を増やさずに書き換え、URL から項目ごとに検証して復元し、不正な項目は取り除く。期間の前後が逆なら送らずに案内する。
  - 詳細の「コース一覧へ戻る」は条件付きの一覧の URL へ戻る。従業員の切り替えでは探索条件も既定に戻す。
  - ラベル・キーボード操作・ライブリージョン・`data-testid` を付ける。文字列は文字として表示する。
- [ ] **Step 10: 画面の更新関数のテストを書いて実行する**
  - キーワードの待ち（古い待ちの番号では送らない）、古い応答を捨てる、各条件の変更ですぐ送る、URL の書き出し（既定値は書かない）と復元（不正な項目だけ無視）、期間の前後が逆なら送らずに案内、0 件とリセット、失敗と再試行、400 の案内、詳細から戻ったときの条件の保持、切り替えで条件が既定に戻る。
- [ ] **Step 11: 環境とビルドの確認**
  - `bash scripts/verify.sh` が U1・U2 のすべてのテストと画面のビルドを含めて成功することを確かめる（確認の段階は緩めない）。
- [ ] **Step 12: ドキュメント**
  - `README.md` に探索のデモの手順（キーワード「AI 入門」、カテゴリ、空席ありのみ、期間、並べ替え、URL の共有、0 件とリセット）と、一覧の受け口の探索条件を書く。公開する関数に短いドキュメントコメントを付ける。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "org",
  "ordering": "implement each applicable testable layer, then write and run that layer's tests.",
  "scope": "workshop",
  "test_strategy": "minimal",
  "project_type": "greenfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra`, `classic` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    }
  ],
  "obligations": {
    "strategy": "minimal",
    "strategy_volume": [
      "One verifiable test per requirement at the narrowest effective level.",
      "At least one happy-path unit test per component.",
      "Unit tests are the default; a bugfix/security scope floor may require an integration or E2E regression when that is the narrowest level that reproduces the defect."
    ],
    "scope_floor": [
      "Keep the existing test suite green.",
      "This scope adds no extra new-test floor beyond the selected test strategy."
    ],
    "combination_rule": "Apply every selected-strategy obligation and every scope-floor obligation; neither replaces the other, and a targeted scope regression may add the narrowest necessary test type beyond the strategy default."
  },
  "plan_profile": {
    "methodology": "test-after",
    "runner_step": "Bootstrap the minimal test runner/configuration and record the exact unit-scoped command.",
    "runner_ready_before_first_test": true,
    "testable_layers": [
      "Data model / database behavior",
      "Repository / data access",
      "Business logic",
      "API / endpoint",
      "Frontend behavior"
    ],
    "steps": [
      "Project structure and production configuration skeleton.",
      "Bootstrap the minimal test runner/configuration and record the exact unit-scoped command.",
      "Data model / database behavior - implement.",
      "Data model / database behavior - write and run its tests after implementation.",
      "Repository / data access - implement.",
      "Repository / data access - write and run its tests after implementation.",
      "Business logic - implement.",
      "Business logic - write and run its tests after implementation.",
      "API / endpoint - implement.",
      "API / endpoint - write and run its tests after implementation.",
      "Frontend behavior - implement.",
      "Frontend behavior - write and run its tests after implementation.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:29f8c477061465cf54fa927df8f6275854a3c87b7dbe5fc917536f71c1c78e7d",
  "contract_sha256": "sha256:356b950a9fbda78149490c376363c0d24f8e5e696c003140de4bde2e0ae85d23"
}
```
