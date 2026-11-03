# コード生成の計画（U3 購入済みチケット）

## Summary

- Builds: 操作中の従業員の購入済みチケットの一覧（受講済みを後ろに並べる）、一覧の受け口 `GET /api/tickets`、一覧の画面 `/my-tickets`（その場で開閉する詳細）、ヘッダーと完了画面からの導線
- Touches: `crates/purchasing/`（一覧の関数）、`crates/web_api/`（一覧の受け口）、`web/`（Route・状態・更新・描画・ヘッダー・完了画面）、`README.md`
- Tests: Rust の一覧のテスト 約 10 件（並び・受講済み・他人のチケット・0 件・受け口・性能）、画面の更新関数と描画のテスト 約 10 件

## 前提と進め方

- 設計の入力: U3 の機能設計（`../functional-design/` の entities.md・rules.md（BR12〜BR14）・functional-spec.md・frontend-components.md）、U3 の NFR 要件（`../nfr-requirements/`）、U1・U2 の設計とコード（U1 が用意した共有部分（ヘッダー・完了画面）に「足す」だけで、U1・U2 の振る舞いは変えない）。
- テストの進め方は下の Testing Contract（test-after）に従う。層ごとに「実装 → その層のテストを書いて実行」の順にする。量は Minimal（要件ごとに 1 件、部品ごとに成功の場合を 1 件以上）に、境目と NFR の自動テストを加える。
- 関数型で書く（NFR6）。一覧の関数は純粋関数とし、現在時刻・日程とコースを引く関数・従業員を引く関数は引数で受け取る（プロジェクトの決まり「依存の逆転は関数の受け渡しで行う」）。
- U1・U2 のテストはすべて成功のままにする。既存の受け口（`GET /api/tickets/{チケット ID}` など）の応答は変えない。
- 保存のデータの形（Ticket など）とデータストア（InMemoryStore）は変えない（U3 は新しい保存のエンティティを作らない）。
- U3 のテストは U3 用のファイルにまとめ、U3 だけを指定して実行できるようにする（`unit-test-instructions.md`）。

## ストーリーとの対応

| 受け入れ基準 | 主な Step |
| --- | --- |
| AC4.2.1 2 枚が開催日時の近い順 | 3〜6、7〜8 |
| AC4.2.2 チケットがないときの案内 | 3〜6、7〜8 |
| AC4.2.3 一覧から詳細（完了画面と同じ項目） | 5〜6、7〜8 |
| AC4.2.4 完了画面から一覧へ | 7〜8 |
| AC4.2.5 ヘッダーから一覧へ | 7〜8 |

## Steps

- [x] **Step 1: 構成の準備**
  - 新しい依存は足さない（U3 の tech-stack-decisions.md）。
  - 一覧の Rust のコードは `crates/purchasing/src/owned_tickets.rs`（OwnedTicket・受講済みの判定・並べ）に置き、`lib.rs` から公開する。画面は `web/` の既存のファイル構成に合わせ、一覧の画面の描画は `web/view_my_tickets.mbt`、更新は `web/update_my_tickets.mbt` に置く。
- [x] **Step 2: テストの実行の準備**
  - U3 用のテストの置き場所を作る: `crates/purchasing/src/owned_tickets_tests.rs`、`crates/web_api/tests/my_tickets.rs`、`web/my_tickets_test.mbt`。
  - `unit-test-instructions.md` の U3 だけを実行するコマンドが動くこと（0 件で成功）を確かめる。MoonBit のファイル指定のオプションが今の moon で使えない場合は、同じ意味の指定（U2 と同じ置き換え）にし、その旨を `code-summary.md` に書く。
- [x] **Step 3: 一覧の業務ロジックを実装する**（BR12.1〜BR12.3、NFR6.1、NFR6.3、NFR7.1〜NFR7.2）
  - OwnedTicket（AC4.1.1 の 7 項目、表示に使う終了日時・形式、attendance）と、受講済みの判定（開始日時 ≤ 現在時刻で attended）を作る。
  - 一覧の関数: (全チケット, 操作中の従業員 ID, 現在時刻, 日程とコースを引く関数, 従業員を引く関数) → OwnedTicket の並び か データの不整合（見つからない日程・コースのチケット ID）。操作中の従業員のチケットだけにし、存在しない従業員は 0 件にする。
  - 並べ: これからを開始日時の早い順、そのあと受講済みを開始日時の遅い順。同じ開始日時は購入日時の早い順、さらにチケット番号の昇順。
  - PostgreSQL に移すときの注意（従業員 ID で引く検索と索引）を関数のコメントに書く（NFR7.2）。
- [x] **Step 4: 一覧の業務ロジックのテストを書いて実行する**（NFR8.2〜NFR8.3）
  - 2 枚が開催日時の近い順（AC4.2.1）、他人のチケットを含めない、存在しない従業員は 0 件、受講済みは後ろに新しい順、開始日時ちょうどは受講済み、同じ開始日時は購入日時→チケット番号の順、日程が見つからないチケットはデータの不整合。データストアを使わず、固定の値と固定の時刻を返す関数を渡す（NFR7.3）。
- [x] **Step 5: Web API の受け口を実装する**（BR13.1〜BR13.2、NFR3.1〜NFR3.3、NFR10.1〜NFR10.3）
  - `GET /api/tickets` を足す。Cookie の操作中の従業員（U1 の受け取り方）について、U1 の「状態の読み出し」と時計を部分適用して一覧を作り、各チケットに 7 項目と `attendance`（`upcoming` / `attended`）を含めて返す。クエリや本文の従業員の指定は受け付けない。
  - データの不整合は 5xx にし、チケット ID をログに残す。チケットの内容（番号・従業員名・コース名）はログに出さない。
  - 応答の項目名と日時の形式は U1 のチケット詳細の応答と同じにする。
- [x] **Step 6: Web API のテストを書いて実行する**
  - 操作中の従業員の一覧と並び、他人のチケットを含めない（クエリに別の従業員 ID を付けても同じ）、チケットのない従業員は空の並び、存在しない従業員 ID の Cookie は空の並び、購入のあとに一覧を引くといま購入したチケットが含まれる（NFR2.2）、一覧を何回引いてもデータが変わらない（NFR2.1）、100 回の 95 パーセンタイルが 200 ms 以内（NFR1.1）。U1 と同じく、サーバーを起動せず固定の時刻で組み立てる。
- [x] **Step 7: 画面を実装する**（BR14.1〜BR14.8、BR12.4、NFR1.2、NFR4.1〜NFR4.3、NFR5.1〜NFR5.3、NFR3.4〜NFR3.5）
  - `frontend-components.md` のとおり、Route `MyTicketsPage`（`/my-tickets`）、状態（myTickets・openTicketIds）、メッセージと更新（画面へ移る・受け取った・再試行・開閉）、通信のコマンド（Cookie 付きの `GET /api/tickets`）を作る。
  - 一覧の行（コース名・開催日時・会場または形式・チケット番号・「受講済み」）、開閉のボタン（「詳細を見る」/「詳細を閉じる」、`aria-expanded`・`aria-controls`）、開いたときの 7 項目（U1 の完了画面と同じ表示の関数）、読み込み中・0 件（「購入済みのチケットはありません」と「コース一覧へ」）・失敗（理由と「再試行」）。
  - U1 のヘッダーに「購入済みチケット」のリンク、完了画面（購入直後）に「購入済みチケットを見る」のリンクを足す。U1 の「コース一覧へ戻る」は残す。
  - 従業員の切り替えで myTickets と openTicketIds を初期値に戻す（U1 のコース一覧へ戻る処理に足す）。一覧の画面でないときに届いた応答は表示に使わない。
  - `data-testid` を付ける。文字列は文字として表示する。
- [x] **Step 8: 画面の更新関数と描画のテストを書いて実行する**
  - URL `/my-tickets` と Route の相互変換、画面へ移ると読み込み中になり一覧の通信を返す、成功・0 件・失敗の状態、再試行、開閉（開閉のメッセージは通信のコマンドを返さない。複数を別々に開ける）、一覧の画面でないときの応答は捨てる、切り替えで一覧と開閉の状態が初期値に戻る、描画で「受講済み」の文字・開閉のボタンの名前と `aria-expanded`・ヘッダーと完了画面のリンクがある。
- [x] **Step 9: 環境とビルドの確認**
  - `bash scripts/verify.sh` が U1・U2・U3 のすべてのテストと画面のビルドを含めて成功することを確かめる（確認の段階は緩めない）。
- [x] **Step 10: ドキュメント**
  - `README.md` の現在の状況に U3 を足し、デモの手順に購入済みチケットの確認（ヘッダーから開く、E005 で 3 枚が並ぶ、詳細を開く、購入の完了画面から開く、チケットのない従業員での案内）と、一覧の受け口を書く。公開する関数に短いドキュメントコメントを付ける。

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
