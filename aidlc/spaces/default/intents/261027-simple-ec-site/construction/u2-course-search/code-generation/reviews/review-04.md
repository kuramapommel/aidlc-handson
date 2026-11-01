## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-01T14:45:11Z
**Iteration:** 1

審査の記録をやり直すための再レビューである。計画・まとめ・追跡表・ソース一覧と、上流の業務ルール（BR8〜BR11）を、実際のソースと突き合わせて確かめた。

### Prior findings

| ID | Now | Severity | Note |
| --- | --- | --- | --- |
| R-01 | Fixed | Major | 日本時間の 0 時をまたぐ期間のテスト `period_uses_the_jst_calendar_date_across_midnight`（`crates/course_catalog/src/search_tests.rs`）が存在し、実装 `start_date_jst`（`search.rs`）は UTC+9 の暦日で判定している。`verify.sh` で成功 |
| R-02 | Fixed | Major | 長音符・濁点・カタカナ混じりのコース名順のテスト `name_order_compares_normalized_hiragana_by_code_point` が存在し、実装は `normalize_text` 後のひらがなを符号の順で比べている（BR10.4）。成功 |
| R-03 | Fixed | Major | `web/update_search.mbt` の `request_course_list` が条件なし（`FetchCourses`）・条件あり（`SearchCourses`）のどちらにも同じ `search_request_seq` を振り、`on_course_list_loaded` も番号が最新でない応答を成功・失敗とも捨てる（BR11.2）。`web/search_test.mbt` に古い応答の破棄のテストがある。画面のテスト 46 件すべて成功 |

### New findings

| ID | Severity | Finding | Recommendation |
| --- | --- | --- | --- |
| N-01 | Minor | `unit-test-instructions.md` の MoonBit のコマンドが `--file search_test.mbt` のままで、今の moon（`--file` なし）では動かない。実際に使ったコマンド（`moon -C web test --target js search_test.mbt`）は `code-summary.md` にだけ書かれている | 次に手を入れるときに `unit-test-instructions.md` のコマンドを実際に動くものへ直す。判定を止める内容ではない |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| `bash scripts/verify.sh` | 成功（cargo fmt・clippy は警告なし。cargo test は course_catalog 21・eligibility 4・in_memory_store 8・purchasing 9・static_server 4・training_budget 4・web_api の api 13 / performance 3 / search 8 で全件成功。moon test は 46 件成功。画面のビルド成功） | U1・U2 の振る舞いと性能の目標（95 パーセンタイル 200 ms）が通っている |
| ソース一覧の存在確認 | 列挙された 30 ファイルがすべて存在 | `source-manifest.json` は実体と一致 |
| 追跡表の確認 | 上流の ID（AC・NFR・BR）にすべて OK か N/A（理由つき）が付いている | 欠けや宙に浮いた参照はない |

### Summary

指摘していた 3 件（R-01〜R-03）はいずれもコードとテストで手直しが確認でき、検証もすべて成功した。探索の業務ロジックは HTTP・保存・画面に依存しない純粋関数で、現在時刻と発行済み数を引数で受け取る構成になっており、設計どおりである。残るのは手順書の MoonBit コマンドの古さ（Minor）だけである。

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/course_catalog/src/search_tests.rs > fixture() と period_includes_both_ends_and_requires_an_open_upcoming_session | 期間は「日本時間の暦日」で判定する実装（`start_date_jst`）だが、テストの日程はすべて日本時間の 13 時（`jst_day`）で、日本時間の 0 時をまたぐ日程（例: UTC で前日 15:30 ＝ 日本時間の翌日 0:30、UTC で当日 14:59 ＝ 日本時間の 23:59）がない。`start_date_jst` が UTC の日付を使う誤りに変わってもテストが通る。 | 日本時間の 0 時前後（例: 日本時間 11/05 00:30 と 11/04 23:59）の日程を足し、開始日・終了日がそれぞれ 11/05・11/04 のときの含む・含まないを確かめるテストを追加する。 | Resolved (reviewer) |
| R-02 | Minor | crates/course_catalog/src/search_tests.rs > four_sort_orders_break_ties_by_course_id（name） | コース名順（BR10.4）のテストは、ひらがなの五十音順と同じよみがなの ID 順しか確かめていない。ルールが明記する「長音符・濁点の違いも符号の順で決める」と、カタカナのよみがなを正規化して比べる分岐を通るデータがなく、`normalize_text` を外しても通る。 | 長音符を含むよみがな（例: 「かー」と「かあ」）、濁点の違い（「か」と「が」）を持つコースをテストのデータに足し、符号の順で並ぶことを確かめる。 | Resolved (reviewer) |
| R-03 | Minor | web/update_search.mbt > on_course_list_loaded と request_course_list | 条件なしの要求は番号を持たない U1 の `FetchCourses` で送るため、「条件なし → 条件あり → 条件なし」と素早く送ると、最初の条件なしの要求の応答（特に通信の失敗）が後から届いたとき、最新の条件なしの結果を上書きして「読み込み失敗」にできる。ファイル冒頭の説明は「どれも同じ条件の結果」としているが、失敗の応答は同じとは言えない。BR11.2（最後に送った要求の応答だけを反映）を条件なしの場合は厳密には満たしていない。 | 条件なしの応答でも要求の番号を持たせて照合する（`CourseListLoaded` を変えずに済ませたい場合は、条件なしの失敗の応答だけでも最新の要求のものか確かめる）。または制約として `code-summary.md` に明記し、その並びのテストを追加する。 | Resolved (reviewer) |
