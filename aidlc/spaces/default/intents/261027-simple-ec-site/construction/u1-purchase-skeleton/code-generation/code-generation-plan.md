# コード生成の計画（U1 購入の最小の流れ）

## Summary

- Builds: monorepo の土台、Rust の業務 4 部品・メモリのデータストア・Web API・静的ファイルサーバー、MoonBit（rabbita）の購入の画面、確認コマンド
- Touches: `Cargo.toml`、`crates/`（7 crate）、`web/`、`scripts/verify.sh`、`README.md`、`.gitignore`
- Tests: Rust の単体テストと Web API のテスト 約 45 件（同時購入・CSRF・Cookie を含む）、MoonBit の更新関数のテスト 約 12 件

## 前提と進め方

- 設計の入力: 機能設計（`../functional-design/` の entities.md・rules.md・functional-spec.md・frontend-components.md）、NFR 要件（`../nfr-requirements/` の 6 文書）、ユニット定義、要件定義書。
- テストの進め方は下の Testing Contract（test-after）に従う。層ごとに「実装 → その層のテストを書いて実行」の順にする。テストの量は Minimal（要件ごとに 1 件、部品ごとに成功の場合を 1 件以上）に、機能設計の境目（BR）と NFR の確認方法で「自動テスト」と書いたものを加える。
- 関数型で書く（NFR6）。業務の crate は純粋関数だけを持ち、時刻と乱数は引数の関数で受け取る。依存の逆転はトレイトのオブジェクトではなく関数（クロージャ）の受け渡しで行う。
- **MoonBit の前提（TQ-1）**: Step 13 に入る時点で MoonBit のツールの取得先（`cli.moonbitlang.com`・`mooncakes.io`・パッケージ本体の `download.mooncakes.io`）が許可されていなければ、Step 13〜16 は行わずに止め、状況を報告する。画面の TEA ライブラリは、rabbit-tea が廃止（全版取り下げ）だったため、2026-10-29 の判断で後継の `moonbit-community/rabbita` に変更した。rabbita が最新版で使えない場合も、TEA を自作せずに止め、原因と対策の案を報告して判断を仰ぐ（NFR 要件 Q1 の回答）。どちらの場合も、画面の部分ができたと偽らない。

## ストーリーとの対応

| ストーリー | 主な Step |
| --- | --- |
| US1.1 一覧で全体を見渡す | 3〜4、7〜8、10〜11、14〜15 |
| US2.1〜US2.3 コースの詳細・日程・選べない理由 | 3〜4、10〜11、14〜15 |
| US3.1・US3.2 残高と購入後の残高 | 5〜6、10〜11、14〜15 |
| US3.3〜US3.5 購入・二重購入の防止・拒否の理由 | 5〜6、7〜9、10〜11、14〜15 |
| US4.1 購入直後のチケット | 5〜6、10〜11、14〜15 |
| US5.1 契約外の企業 | 5〜6、10〜11、14〜15 |
| 横断（NFR3・NFR9・NFR10 など） | 1〜2、12、17〜18 |

## Steps

- [x] **Step 1: プロジェクトの構成と設定の土台**（US 横断、NFR9.1・NFR9.2、NFR3.12）
  - ルートに Rust のワークスペースの `Cargo.toml`（edition 2024、`crates/*` を members に）と `.gitignore`（`target/`、MoonBit のビルド結果）を作る。
  - `crates/` に `course_catalog`・`eligibility`・`training_budget`・`purchasing`（ライブラリ）、`in_memory_store`（ライブラリ）、`web_api`・`static_server`（実行ファイル）を作る。業務の 4 crate はデータストアと Web API の crate に依存しない（NFR7.1）。Purchasing だけがほかの業務 3 crate を使う。
  - 依存の crate は `tech-stack-decisions.md` の表のものを、`cargo add` でその時点の最新 stable 版で追加し、`Cargo.lock` に固定する。
- [x] **Step 2: テストの実行の準備**（NFR8.1）
  - 各 crate に空のテストのモジュールを置き、`cargo test -p <crate>` が動くことを確かめる。U1 のテストの実行コマンドを `unit-test-instructions.md` のとおりに確かめる（このユニットの crate と画面のプロジェクトだけを指定する）。
- [x] **Step 3: データの形と CourseCatalog の業務ロジックを実装する**（US1.1、US2.1〜US2.3。BR2.1〜BR2.4、BR3.1〜BR3.3）
  - 業務の 4 crate に `entities.md` の型（Course・Session・Company・Employee・BudgetEntry・Ticket と列挙の値、ID の値の型と形式の検証）を不変の値として定義する。
  - CourseCatalog: 日程の状態の判定（受付終了 → 満席 → 購入可）、残席数、コース一覧の集計（直近の開催日は満席も含む、空席のある日程の数）、既定の並び（直近の開催日 → コース ID、予定なしは最後）、コースの詳細。発行済み数は「日程 ID → 数」の関数で受け取る（ADR-005）。現在時刻は引数で受け取る。
- [x] **Step 4: データの形と CourseCatalog のテストを書いて実行する**
  - ID の形式の検証、開催開始ちょうどで受付終了、過去の満席は受付終了、残席 0 で満席、直近の開催日に満席の日程を含む、同じ直近の開催日はコース ID 順、予定なしは最後、存在しないコース。`cargo test -p course_catalog`。
- [x] **Step 5: Eligibility・TrainingBudget・Purchasing の業務ロジックを実装する**（US3.1〜US3.5、US4.1、US5.1。BR1.1、BR4.1〜BR4.5、BR5.1〜BR5.7、BR2.4）
  - Eligibility: 利用資格の判定（eligible・company-not-active・employee-not-found）、切り替え用の従業員一覧。
  - TrainingBudget: 残高（付与の合計 − 消費の合計）、支払い可否・不足額・購入後の残高、消費取引の生成。
  - Purchasing: 購入の決定関数（PurchaseSnapshot → 承認（チケット＋消費取引）か拒否の理由 1 つ。BR5.1 の順）、受講料はコースから（BR5.2）、チケット番号（紛らわしい文字を除いた英大文字と数字 8 文字を「XXXX-XXXX」に。候補を作る関数は引数で受け取り、重複したら作り直す。BR5.6）、日程ごとの発行済み数、表示用の日程の状態（購入済みを優先）、チケット 1 枚の詳細の組み立て。
- [x] **Step 6: Eligibility・TrainingBudget・Purchasing のテストを書いて実行する**
  - 利用資格の 3 つの結果。残高の計算、残高ちょうどで支払える（購入後 0）、不足額。購入の決定: 承認（チケットと消費取引の中身）、7 つの拒否の理由それぞれ、理由が重なったときに BR5.1 の順で最初の 1 つ、要求の金額を使わないこと、チケット番号の形式と使う文字、重複時の作り直し。`cargo test -p eligibility -p training_budget -p purchasing`。
- [x] **Step 7: メモリのデータストアと初期データを実装する**（US1.1、US3.3、US3.4。BR5.3、BR5.4、NFR2.1〜NFR2.5、FR7）
  - 全データを 1 つの不変な状態として持ち、排他ロックで守った現在の状態を置き換える（TD-1）。読み出しは写しを取り出してロックの外で計算する。
  - 購入の記録: ロックの中で最新の状態から PurchaseSnapshot を作り、決定関数を評価し直し、承認ならチケットと消費取引を同時に追加した新しい状態に置き換える。拒否なら状態を変えない。
  - 初期データ: `functional-spec.md` 6 節のとおり、起動日を基準に日本時間で生成する（企業 3 社、高橋 遥を先頭にした従業員 4 名以上、4 カテゴリ以上・12 コース以上、満席・受付終了・残席 1・残席 3・予定なしを含む）。時刻は引数で受け取る。
  - PostgreSQL に移すときに必要な保証を、記録の関数のコメントに書く（NFR7.3）。
- [x] **Step 8: データストアのテストを書いて実行する**
  - 承認でチケットと消費取引が同時に増える、拒否で何も変わらない、残席 1 に 2 件同時の購入で 1 件だけ成功し他方は満席、定員 3 に 20 件同時でも発行数は 3、同じ従業員の同じ日程への同時の購入で 1 枚だけ、初期データが FR7.2 の種類をそろえていること（固定の時刻で）。`cargo test -p in_memory_store`。
- [x] **Step 9: 業務の部品が差し替えられることのテストを加える**（NFR7.2）
  - Purchasing と CourseCatalog のテストの一部を、データストアを使わずにテスト用のクロージャ（固定のデータを返す関数）を渡して動かす形で書く（Step 4・6 のテストに含めてよい）。
- [x] **Step 10: Web API を実装する**（US1.1〜US5.1 の受け口。BR1.2、BR5.8、BR5.10、BR6.1〜BR6.3、BR7.9、NFR3.1〜NFR3.10、NFR10.1〜NFR10.7）
  - axum で `functional-spec.md` 3 節の受け口（`/api/courses`、`/api/courses/{courseId}`、`/api/me`、`/api/employees`、`/api/session/employee`、`/api/purchases`、`/api/tickets/{ticketId}`）と `/api/health` を作る。起動時にデータストアの関数を業務の関数に部分適用して組み立てる。
  - Cookie（HttpOnly・SameSite=Lax・Path=/）で従業員 ID を受け渡す。Cookie がなければ先頭の従業員の Cookie を発行する（`/api/me`）。ローカル検証専用であることをコメントに書く（NFR3.8）。
  - 書き込みの要求は Origin と専用ヘッダー（`X-Requested-With: aidlc-handson-web`）を確かめ、満たさなければ 403（NFR3.5）。CORS は画面のオリジンだけを資格情報付きで許可（NFR3.6）。
  - 入力の検証で 400、存在しないものは 404、購入の拒否は 409 と理由、予期しない失敗は 5xx と一般的なメッセージ（NFR3.10）。要求ごとのログと、購入の承認・拒否のログ（NFR10.1〜NFR10.3）。ポート・画面のオリジン・ログの詳しさは環境変数で変えられる（既定は 3000・`http://localhost:8080`・情報レベル）。
- [x] **Step 11: Web API のテストを書いて実行する**
  - サーバーを起動せずに受け口を呼ぶテスト（`tower` の `ServiceExt`）。一覧と詳細の応答の形、存在しないコースで 404、形式不正で 400、Cookie なしの `/api/me` で先頭の従業員の Cookie が発行される（属性を含む）、切り替えで Cookie が変わる・一覧にない従業員で 404、購入の成功（残席と残高の変化）、拒否で 409 と理由（契約外・存在しない従業員・満席・受付終了・購入済み・残高不足）、要求に金額を含めても受講料で差し引く、Origin なし・別の Origin・専用ヘッダーなしの書き込みで 403、許可しないオリジンのプリフライトで許可されない、他人のチケットで 404、`/api/health` が 200、一覧・詳細・購入の応答時間の 95 パーセンタイルが 200 ms 以内（各 100 回）。`cargo test -p web_api`。
- [x] **Step 12: 画面用の静的ファイルサーバーを実装し、テストする**（TD-2、NFR9.1）
  - `tower-http` で `web/` のビルド結果と HTML・CSS を配信する。存在しないパスには `index.html` を返す。ポートと配信するディレクトリは環境変数で変えられる（既定は 8080）。
  - テスト: 既存のファイルが返る、画面内の URL（`/courses/C001` など）で `index.html` が返る、ディレクトリの外のパス（`..`）は配信しない。`cargo test -p static_server`。
- [ ] **Step 13: MoonBit の画面のプロジェクトを用意する**（TQ-1、TQ-2）
  - MoonBit のツールを導入できることを確かめる。できなければここで止めて報告する（前提と進め方を参照）。
  - `web/` に MoonBit のプロジェクト（JavaScript 向け）を作り、rabbita（`moonbit-community/rabbita`）の最新版を追加する。資格情報（Cookie）付きの通信と専用ヘッダーの付与ができることを確かめる。できなければここで止め、原因と対策の案を報告する。
- [ ] **Step 14: 画面（TEA）を実装する**（US1.1、US2.1〜US2.3、US3.1〜US3.5、US4.1、US5.1。BR1.3、BR1.4、BR2.5、BR6.4、BR7.1〜BR7.11、NFR4、NFR5、NFR3.11）
  - `frontend-components.md` の部品・状態（Model）・画面と URL（Route）・メッセージと更新・描画のルール・Web API との連携のとおりに作る。
  - 操作できる部品に `data-testid` を付ける。状態は色だけでなく文字で示し、ボタンと選択の部品にラベルを付け、キーボードで操作できるようにする（NFR5）。Web API から受け取った文字列は文字として表示する（NFR3.11）。
  - `web/index.html` と CSS を作る。
- [ ] **Step 15: 画面の更新関数のテストを書いて実行する**
  - 起動時の読み込み中、一覧の失敗と再試行、購入可の日程だけ選べる、日程を選ぶまで「購入確認へ進む」は無効、支払えないとき確定は無効、契約外で確定は無効、確定中の 2 回目の押下は無視、成功でヘッダーの残高が購入後の値、拒否の理由ごとの表示、通信の失敗ともう一度試す、「別の日程を選ぶ」で詳細を読み込み直す、従業員の切り替えの成功でどの画面からも一覧へ戻り前の情報を捨てる。リポジトリのルートで `moon -C web test --target js`（今の MoonBit では `moon test --directory web` が使えないため、2026-10-29 に変更）。
- [ ] **Step 16: 画面のビルドの設定**
  - `moon build --target js` の結果と `index.html`・CSS を、静的ファイルサーバーが配信するディレクトリにそろえる手順を決め、スクリプトにする（例: `scripts/build-web.sh`）。
- [x] **Step 17: 環境とビルドの設定（確認コマンド）**（NFR9.3）
  - `scripts/verify.sh` を作る。Rust のフォーマットの確認（`cargo fmt --check`）・`cargo clippy`（警告をエラーに）・`cargo test`・MoonBit のテスト・画面のビルドを順に実行し、どれかが失敗すれば失敗で終わる。MoonBit のツールがない場合はその旨を表示して失敗で終わる（成功と偽らない）。
- [x] **Step 18: ドキュメントとトレーサビリティ**（NFR3.8、NFR9.2）
  - `README.md`: 前提のツール、ビルド・テスト・2 プロセスの起動の手順（リポジトリのルートから）、既定のポートと環境変数、デモの手順（高橋 遥での購入、残高不足と契約外の従業員への切り替え）、**ローカル検証専用で公開してはならないこと**。
  - 公開する関数に短いドキュメントコメントを付ける。

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
