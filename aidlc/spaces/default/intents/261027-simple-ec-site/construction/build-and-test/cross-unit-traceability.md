# ユニットをまたぐ最終の網羅の確認

要件定義書の FR・NFR と、ユーザーストーリーの受け入れ基準（AC）のすべてについて、各ユニットのコード生成の `traceability.json`（`../*/code-generation/traceability.json`）に status `OK` の記載があり、対象のファイルが存在するかを確かめた。

## 判定: 不合格（網羅の記録に不足あり）

- 受け入れ基準 73 件: 72 件 OK、1 件 N/A（AC2.1.3 は欠番。AC1.8.4 へ移動）。
- NFR 10 件: すべて OK（各ユニットの NFRx.y 経由）。
- FR 36 件: **traceability.json に FR の ID を直接記載したものはない**。33 件は、その FR を要件に持つストーリーの受け入れ基準（FR1.2 は U1 の業務ルール BR6.2・BR6.4）経由で間接にカバーされる。**FR7.1〜FR7.3（初期データ）は、どの traceability.json にも ID の記載がない**（実装は `crates/in_memory_store/src/seed.rs` と、そのテストにある）。

どれも実装やテストが欠けているのではなく、網羅の記録（traceability.json）の書き方の不足である。承認のときに確認してください。

## ID ごとの網羅

| ID | 判定 | ユニット | 対象のファイル／経路 |
| --- | --- | --- | --- |
| FR1.1 | 間接（直接の記載なし） | u1-purchase-skeleton | US1.1 の AC1.1.1 経由: `web/view_header.mbt` |
| FR1.2 | 間接（直接の記載なし） | u1-purchase-skeleton | U1 の BR6.2・BR6.4 経由: `web/update.mbt` |
| FR1.3 | 間接（直接の記載なし） | u1-purchase-skeleton | US5.1 の AC5.1.1 経由: `web/view.mbt` |
| FR1.4 | 間接（直接の記載なし） | u1-purchase-skeleton | US5.1 の AC5.1.1 経由: `web/view.mbt` |
| FR2.1 | 間接（直接の記載なし） | u1-purchase-skeleton | US1.1 の AC1.1.1 経由: `web/view_header.mbt` |
| FR2.2 | 間接（直接の記載なし） | u2-course-search | US1.2 の AC1.2.1 経由: `crates/course_catalog/src/search.rs` |
| FR2.3 | 間接（直接の記載なし） | u2-course-search | US1.3 の AC1.3.1 経由: `crates/course_catalog/src/search.rs` |
| FR2.4 | 間接（直接の記載なし） | u2-course-search | US1.4 の AC1.4.1 経由: `crates/course_catalog/src/search.rs` |
| FR2.5 | 間接（直接の記載なし） | u2-course-search | US1.5 の AC1.5.1 経由: `crates/course_catalog/src/search.rs` |
| FR2.6 | 間接（直接の記載なし） | u2-course-search | US1.6 の AC1.6.1 経由: `crates/course_catalog/src/search.rs` |
| FR2.7 | 間接（直接の記載なし） | u1-purchase-skeleton | US1.1 の AC1.1.1 経由: `web/view_header.mbt` |
| FR2.8 | 間接（直接の記載なし） | u2-course-search | US1.8 の AC1.8.1 経由: `web/search.mbt` |
| FR2.9 | 間接（直接の記載なし） | u2-course-search | US1.9 の AC1.9.1 経由: `web/view_search.mbt` |
| FR3.1 | 間接（直接の記載なし） | u1-purchase-skeleton | US2.1 の AC2.1.1 経由: `web/view_pages.mbt` |
| FR3.2 | 間接（直接の記載なし） | u1-purchase-skeleton | US2.2 の AC2.2.1 経由: `crates/course_catalog/src/status.rs` |
| FR3.3 | 間接（直接の記載なし） | u1-purchase-skeleton | US2.2 の AC2.2.1 経由: `crates/course_catalog/src/status.rs` |
| FR3.4 | 間接（直接の記載なし） | u1-purchase-skeleton | US2.3 の AC2.3.1 経由: `crates/course_catalog/src/status.rs` |
| FR3.5 | 間接（直接の記載なし） | u1-purchase-skeleton | US2.1 の AC2.1.1 経由: `web/view_pages.mbt` |
| FR4.1 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.1 の AC3.1.1 経由: `web/view_header.mbt` |
| FR4.2 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.2 の AC3.2.1 経由: `crates/training_budget/src/rules.rs` |
| FR4.3 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.2 の AC3.2.1 経由: `crates/training_budget/src/rules.rs` |
| FR4.4 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.2 の AC3.2.1 経由: `crates/training_budget/src/rules.rs` |
| FR5.1 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.2 の AC3.2.1 経由: `crates/training_budget/src/rules.rs` |
| FR5.2 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.3 の AC3.3.1 経由: `crates/in_memory_store/src/store.rs` |
| FR5.3 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.3 の AC3.3.1 経由: `crates/in_memory_store/src/store.rs` |
| FR5.4 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.3 の AC3.3.1 経由: `crates/in_memory_store/src/store.rs` |
| FR5.5 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.3 の AC3.3.1 経由: `crates/in_memory_store/src/store.rs` |
| FR5.6 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.4 の AC3.4.1 経由: `web/update.mbt` |
| FR5.7 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.5 の AC3.5.1 経由: `web/view_pages.mbt` |
| FR5.8 | 間接（直接の記載なし） | u1-purchase-skeleton | US3.5 の AC3.5.1 経由: `web/view_pages.mbt` |
| FR6.1 | 間接（直接の記載なし） | u1-purchase-skeleton | US4.1 の AC4.1.1 経由: `web/view_pages.mbt` |
| FR6.2 | 間接（直接の記載なし） | u1-purchase-skeleton | US4.1 の AC4.1.1 経由: `web/view_pages.mbt` |
| FR6.3 | 間接（直接の記載なし） | u3-my-tickets | US4.2 の AC4.2.1 経由: `crates/web_api/tests/my_tickets.rs` |
| FR7.1 | 未カバー（traceability に記載なし） | u1-purchase-skeleton | 実装は `crates/in_memory_store/src/seed.rs`（traceability.json に ID がない） |
| FR7.2 | 未カバー（traceability に記載なし） | u1-purchase-skeleton | 実装は `crates/in_memory_store/src/seed.rs`（traceability.json に ID がない） |
| FR7.3 | 未カバー（traceability に記載なし） | u1-purchase-skeleton | 実装は `crates/in_memory_store/src/seed.rs`（traceability.json に ID がない） |
| NFR1 | OK（NFR1.1 ほか） | u2-course-search | `crates/web_api/tests/search.rs` |
| NFR2 | OK（NFR2.1 ほか） | u2-course-search | `crates/web_api/tests/search.rs` |
| NFR3 | OK（NFR3.1 ほか） | u2-course-search | `crates/web_api/src/handlers.rs` |
| NFR4 | OK（NFR4.1 ほか） | u2-course-search | `web/view_pages.mbt` |
| NFR5 | OK（NFR5.1 ほか） | u2-course-search | `web/view_search.mbt` |
| NFR6 | OK（NFR6.1 ほか） | u2-course-search | `crates/course_catalog/src/search.rs` |
| NFR7 | OK（NFR7.1 ほか） | u2-course-search | `crates/course_catalog/src/search.rs` |
| NFR8 | OK（NFR8.1 ほか） | u2-course-search | `crates/course_catalog/src/search_tests.rs` |
| NFR9 | OK（NFR9.1 ほか） | u2-course-search | `README.md` |
| NFR10 | OK（NFR10.1 ほか） | u3-my-tickets | `crates/web_api/src/app.rs` |
| AC1.1.1 | OK | u1-purchase-skeleton | `web/view_header.mbt` |
| AC1.1.2 | OK | u1-purchase-skeleton | `crates/course_catalog/src/listing.rs` |
| AC1.1.3 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC1.1.4 | OK | u1-purchase-skeleton | `web/update.mbt` |
| AC1.2.1 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.2.2 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.2.3 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.2.4 | OK | u2-course-search | `web/update_search.mbt` |
| AC1.2.5 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.3.1 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.3.2 | OK | u2-course-search | `web/view_search.mbt` |
| AC1.3.3 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.4.1 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.4.2 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.4.3 | OK | u2-course-search | `web/update_search.mbt` |
| AC1.5.1 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.5.2 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.5.3 | OK | u2-course-search | `web/update_search.mbt` |
| AC1.5.4 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.6.1 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.6.2 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.6.3 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.7.1 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.7.2 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.7.3 | OK | u2-course-search | `crates/in_memory_store/src/seed.rs` |
| AC1.7.4 | OK | u2-course-search | `crates/course_catalog/src/search.rs` |
| AC1.8.1 | OK | u2-course-search | `web/search.mbt` |
| AC1.8.2 | OK | u2-course-search | `web/update_search.mbt` |
| AC1.8.3 | OK | u2-course-search | `web/search.mbt` |
| AC1.8.4 | OK | u2-course-search | `web/view_pages.mbt` |
| AC1.9.1 | OK | u2-course-search | `web/view_search.mbt` |
| AC1.9.2 | OK | u2-course-search | `web/update_search.mbt` |
| AC1.9.3 | OK | u2-course-search | `web/view_pages.mbt` |
| AC1.9.4 | OK | u2-course-search | `web/update_search.mbt` |
| AC2.1.1 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC2.1.2 | OK | u1-purchase-skeleton | `crates/web_api/src/handlers.rs` |
| AC2.1.3 | N/A（欠番。AC1.8.4 へ移動） | — | — |
| AC2.2.1 | OK | u1-purchase-skeleton | `crates/course_catalog/src/status.rs` |
| AC2.2.2 | OK | u1-purchase-skeleton | `web/update.mbt` |
| AC2.2.3 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC2.3.1 | OK | u1-purchase-skeleton | `crates/course_catalog/src/status.rs` |
| AC2.3.2 | OK | u1-purchase-skeleton | `crates/course_catalog/src/status.rs` |
| AC2.3.3 | OK | u1-purchase-skeleton | `crates/purchasing/src/tickets.rs` |
| AC2.3.4 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC3.1.1 | OK | u1-purchase-skeleton | `web/view_header.mbt` |
| AC3.1.2 | OK | u1-purchase-skeleton | `web/update.mbt` |
| AC3.2.1 | OK | u1-purchase-skeleton | `crates/training_budget/src/rules.rs` |
| AC3.2.2 | OK | u1-purchase-skeleton | `crates/training_budget/src/rules.rs` |
| AC3.2.3 | OK | u1-purchase-skeleton | `crates/training_budget/src/rules.rs` |
| AC3.2.4 | OK | u1-purchase-skeleton | `web/update.mbt` |
| AC3.3.1 | OK | u1-purchase-skeleton | `crates/in_memory_store/src/store.rs` |
| AC3.3.2 | OK | u1-purchase-skeleton | `crates/purchasing/src/decision.rs` |
| AC3.3.3 | OK | u1-purchase-skeleton | `crates/in_memory_store/src/state.rs` |
| AC3.3.4 | OK | u1-purchase-skeleton | `crates/in_memory_store/src/store.rs` |
| AC3.4.1 | OK | u1-purchase-skeleton | `web/update.mbt` |
| AC3.4.2 | OK | u1-purchase-skeleton | `crates/purchasing/src/decision.rs` |
| AC3.4.3 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC3.5.1 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC3.5.2 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC3.5.3 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC3.5.4 | OK | u1-purchase-skeleton | `web/update.mbt` |
| AC4.1.1 | OK | u1-purchase-skeleton | `web/view_pages.mbt` |
| AC4.1.2 | OK | u1-purchase-skeleton | `crates/purchasing/src/ticket_number.rs` |
| AC4.1.3 | OK | u1-purchase-skeleton | `web/update.mbt` |
| AC4.2.1 | OK | u3-my-tickets | `crates/web_api/tests/my_tickets.rs` |
| AC4.2.2 | OK | u3-my-tickets | `web/view_my_tickets.mbt` |
| AC4.2.3 | OK | u3-my-tickets | `web/view_my_tickets.mbt` |
| AC4.2.4 | OK | u3-my-tickets | `web/view_pages.mbt` |
| AC4.2.5 | OK | u3-my-tickets | `web/view_header.mbt` |
| AC5.1.1 | OK | u1-purchase-skeleton | `web/view.mbt` |
| AC5.1.2 | OK | u1-purchase-skeleton | `web/update.mbt` |
| AC5.1.3 | OK | u1-purchase-skeleton | `crates/eligibility/src/rules.rs` |
| AC5.1.4 | OK | u1-purchase-skeleton | `crates/eligibility/src/rules.rs` |
