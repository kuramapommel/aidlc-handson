## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-10-30T23:56:16Z
**Iteration:** 1

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/course_catalog/src/search_tests.rs > fixture() と period_includes_both_ends_and_requires_an_open_upcoming_session | 期間は「日本時間の暦日」で判定する実装（`start_date_jst`）だが、テストの日程はすべて日本時間の 13 時（`jst_day`）で、日本時間の 0 時をまたぐ日程（例: UTC で前日 15:30 ＝ 日本時間の翌日 0:30、UTC で当日 14:59 ＝ 日本時間の 23:59）がない。`start_date_jst` が UTC の日付を使う誤りに変わってもテストが通る。 | 日本時間の 0 時前後（例: 日本時間 11/05 00:30 と 11/04 23:59）の日程を足し、開始日・終了日がそれぞれ 11/05・11/04 のときの含む・含まないを確かめるテストを追加する。 | New |
| R-02 | Minor | crates/course_catalog/src/search_tests.rs > four_sort_orders_break_ties_by_course_id（name） | コース名順（BR10.4）のテストは、ひらがなの五十音順と同じよみがなの ID 順しか確かめていない。ルールが明記する「長音符・濁点の違いも符号の順で決める」と、カタカナのよみがなを正規化して比べる分岐を通るデータがなく、`normalize_text` を外しても通る。 | 長音符を含むよみがな（例: 「かー」と「かあ」）、濁点の違い（「か」と「が」）を持つコースをテストのデータに足し、符号の順で並ぶことを確かめる。 | New |
| R-03 | Minor | web/update_search.mbt > on_course_list_loaded と request_course_list | 条件なしの要求は番号を持たない U1 の `FetchCourses` で送るため、「条件なし → 条件あり → 条件なし」と素早く送ると、最初の条件なしの要求の応答（特に通信の失敗）が後から届いたとき、最新の条件なしの結果を上書きして「読み込み失敗」にできる。ファイル冒頭の説明は「どれも同じ条件の結果」としているが、失敗の応答は同じとは言えない。BR11.2（最後に送った要求の応答だけを反映）を条件なしの場合は厳密には満たしていない。 | 条件なしの応答でも要求の番号を持たせて照合する（`CourseListLoaded` を変えずに済ませたい場合は、条件なしの失敗の応答だけでも最新の要求のものか確かめる）。または制約として `code-summary.md` に明記し、その並びのテストを追加する。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo test -p course_catalog search_tests | PASS（11 件） | BR8〜BR10 の検証・正規化・絞り込み・並べ替えのテストが成功。上の Minor 2 件のとおり境目の一部は未確認 |
| cargo test -p in_memory_store reading_ | PASS（2 件） | 初期データの全コースによみがながあり、探索の確認用データ（満席だけ・同額・同日時）がある |
| cargo test -p web_api --test search | PASS（8 件） | 400 は不正な項目をすべて挙げ値を含まない、キーワードは巨大でも 200、探索でデータは変わらない、条件なしは U1 と同一、95 パーセンタイル 200 ms 以内 |
| moon -C web test --target js | PASS（44 件） | 150 ms の待ち（コマンド）、最新の応答だけを反映、URL の書き出しと復元、期間の案内、リセット、再試行、400、戻る操作、従業員切り替えを確認 |
| bash scripts/verify.sh | PASS（5 段階すべて） | fmt・clippy・cargo test・moon test・画面のビルドが成功し、U1 のテストも成功のまま |

### Summary

BR8〜BR11 と U2 の NFR は実装とテストで満たされている。キーワードは拒否せず先頭 200 文字を NFKC・小文字・カタカナ→ひらがなで正規化し、400 は不正な項目をすべて挙げ値を含まない。期間は両端を含み日付順は期間に影響されない。よみがなによる名前順、コマンドによる 150 ms の待ち、番号による最新の応答のみの反映、URL の書き出しと復元、期間の案内、リセット、再試行、戻る操作も確認できた。U1 の振る舞いは変わらず、関数の受け渡しによる構成で、トレイトオブジェクトも使っていない。指摘は日本時間の日付境界・名前順の境目のテスト不足と、条件なしの古い失敗の応答の扱いの 3 件の Minor のみで、ブロックしない。
