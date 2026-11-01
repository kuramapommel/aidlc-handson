## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-01T02:28:01Z
**Iteration:** 1

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/course_catalog/src/search_tests.rs > fixture() と period_includes_both_ends_and_requires_an_open_upcoming_session | 期間は「日本時間の暦日」で判定する実装（`start_date_jst`）だが、テストの日程はすべて日本時間の 13 時（`jst_day`）で、日本時間の 0 時をまたぐ日程（例: UTC で前日 15:30 ＝ 日本時間の翌日 0:30、UTC で当日 14:59 ＝ 日本時間の 23:59）がない。`start_date_jst` が UTC の日付を使う誤りに変わってもテストが通る。 | 日本時間の 0 時前後（例: 日本時間 11/05 00:30 と 11/04 23:59）の日程を足し、開始日・終了日がそれぞれ 11/05・11/04 のときの含む・含まないを確かめるテストを追加する。 | Resolved (reviewer) |
| R-02 | Minor | crates/course_catalog/src/search_tests.rs > four_sort_orders_break_ties_by_course_id（name） | コース名順（BR10.4）のテストは、ひらがなの五十音順と同じよみがなの ID 順しか確かめていない。ルールが明記する「長音符・濁点の違いも符号の順で決める」と、カタカナのよみがなを正規化して比べる分岐を通るデータがなく、`normalize_text` を外しても通る。 | 長音符を含むよみがな（例: 「かー」と「かあ」）、濁点の違い（「か」と「が」）を持つコースをテストのデータに足し、符号の順で並ぶことを確かめる。 | Resolved (reviewer) |
| R-03 | Minor | web/update_search.mbt > on_course_list_loaded と request_course_list | 条件なしの要求は番号を持たない U1 の `FetchCourses` で送るため、「条件なし → 条件あり → 条件なし」と素早く送ると、最初の条件なしの要求の応答（特に通信の失敗）が後から届いたとき、最新の条件なしの結果を上書きして「読み込み失敗」にできる。ファイル冒頭の説明は「どれも同じ条件の結果」としているが、失敗の応答は同じとは言えない。BR11.2（最後に送った要求の応答だけを反映）を条件なしの場合は厳密には満たしていない。 | 条件なしの応答でも要求の番号を持たせて照合する（`CourseListLoaded` を変えずに済ませたい場合は、条件なしの失敗の応答だけでも最新の要求のものか確かめる）。または制約として `code-summary.md` に明記し、その並びのテストを追加する。 | Resolved (reviewer) |

> R-01 Reviewer note: `midnight_fixture` と `period_uses_the_jst_calendar_date_across_midnight` を追加した。日本時間の 0:30、23:30、23:59:59、0:00:00 ちょうどの日程で、開始日 = 終了日、終了日だけ、開始日だけを確かめている。テスト内で UTC の日付が日本時間の日付と違うことも確認しており、`start_date_jst` が UTC の日付を使う誤りに変わると失敗する。

> R-02 Reviewer note: `name_order_compares_normalized_hiragana_by_code_point` を追加した。長音符（かあど < かーど）、濁音・半濁音（か < が、は < ば < ぱ）、カタカナ「カード」、混在「きカク」、半角カタカナの濁点「ｶｲｾﾞﾝ」を含む。`normalize_text` を外すとカタカナが五十音の後ろに並び、期待値と食い違うので失敗する。`normalize_text` の直接のアサートもある。

> R-03 Reviewer note: `FetchCourses(generation, seq)` と `CourseListLoaded(generation, seq, result)` に番号を持たせ、`request_course_list`・`enter_route`・`on_employee_switched` が同じ `search_request_seq` の列から番号を振る。`on_course_list_loaded` は番号が最新でない応答を成功・失敗とも捨てる。`search_test.mbt` に「条件なし → 条件あり → 条件なし」の順で古い失敗と古い成功を捨てるテストがある。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `cargo test -p course_catalog search_tests` | PASS: 13 件成功 | R-01 と R-02 のテストを含めて成功している。 |
| `moon -C web test --target js` | PASS: 46 件成功 | R-03 の更新関数のテストと U1 の画面のテストが成功している。 |
| `bash scripts/verify.sh` | PASS: 「すべての確認が通りました」 | U1・U2 のテストと画面のビルドが通っている。確認の段階は緩められていない。 |

`cargo test -p in_memory_store reading_` と `cargo test -p web_api --test search` は、今回の手直しがこれらのコードに触れておらず、`verify.sh` に含まれるので個別には再実行していない。

### Summary

前回の 3 件はすべて直っている。U1 の画面のテスト（`web_test.mbt`）の変更は `FetchCourses` と `CourseListLoaded` の番号の引数を足しただけで、判定は変わっていない。R-03 の変更による後退は見つからなかった。
