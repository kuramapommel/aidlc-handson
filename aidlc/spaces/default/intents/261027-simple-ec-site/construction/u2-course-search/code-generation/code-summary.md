# コード生成のまとめ（U2 コース探索）

## 作ったもの・変えたもの

| 場所 | 中身 |
| --- | --- |
| `crates/course_catalog/src/search.rs` | 探索の純粋関数。条件の検証（不正な項目をすべて返す）、キーワードの正規化（先頭 200 文字・NFKC・小文字・カタカナ → ひらがな）、絞り込み（キーワードの AND・カテゴリ・空席ありのみ・期間）、4 つの並べ替え（どれも最後にコース ID）。PostgreSQL に移すときの注意をコメントに記載 |
| `crates/course_catalog/`（`model.rs`・`listing.rs`・`lib.rs`・`Cargo.toml`） | `Course.title_reading` とその検証、`Category::from_value`。`unicode-normalization` 0.1.26 を追加 |
| `crates/in_memory_store/src/seed.rs` | 全コースのよみがな。探索の確認用に C014「新任リーダー研修」と満席の日程 S0020 を追加（14 コース） |
| `crates/web_api/`（`app.rs`・`handlers.rs`・`errors.rs`） | `GET /api/courses` が `q`・`category`・`available`・`from`・`to`・`sort` を受け取る。不正なら 400 `invalid-search-condition` と不正な項目の一覧。ログはパスと項目名だけ。条件なしは U1 と同じ応答 |
| `web/search.mbt`・`update_search.mbt`・`view_search.mbt` | 探索条件の型、URL の書き出しと項目ごとの復元、150 ms の待ち、要求の番号で古い応答を捨てる、リセット・再試行、探索フォーム・0 件の表示・400 の案内 |
| `web/`（`model.mbt`・`update.mbt`・`web.mbt`・`view.mbt`・`view_pages.mbt`・`static/style.css` など） | 状態・メッセージ・作用の追加、詳細の「コース一覧へ戻る」を条件付きの URL に |
| テスト | `crates/course_catalog/src/search_tests.rs`、`crates/in_memory_store/src/tests.rs`（`reading_`）、`crates/web_api/tests/search.rs`、`web/search_test.mbt` |
| `README.md`・`web/README.mbt.md` | 探索のデモの手順、探索条件の表と 400 の形、U2 のテストのコマンド |

## 主な決定

- **U1 の画面のテストを変えずに足す**: 条件なしの一覧は U1 と同じ要求、条件付きの一覧は新しい要求で取得する。どちらも要求の番号を進め、最後に送った要求の応答だけを反映する（BR11.2）。
- **0 件は状態として持たない**: 件数 0 のときに表示を変える。400 のときに一覧を保つため、最後に表示できた一覧を持つ（BR11.9）。
- **URL からの復元**: 期間の前後が逆なら開始日と終了日の両方を捨てる。不正な項目・既定の値・知らない項目があれば、履歴を増やさずに正しい形の URL に 1 回だけ書き換える。
- **Web API**: クエリの空の値（`?category=` など）は省略と同じに扱う。日本時間の暦日は UTC+9 の固定の時差で求める（`chrono-tz` は足さない）。
- **表示の文言**: U1 の表示を変えないため、カテゴリの表示名は U1 の「テクノロジー」、件数は「N 件のコース」、失敗の文言は U1 の「コース一覧を読み込めませんでした」のままにした（設計の文書の例とは少し違う）。

## テストの結果

| コマンド | 結果 |
| --- | --- |
| `cargo test -p course_catalog search_tests` | 11 件成功 |
| `cargo test -p in_memory_store reading_` | 2 件成功 |
| `cargo test -p web_api --test search` | 8 件成功 |
| `moon -C web test --target js search_test.mbt` | 12 件成功 |
| 探索の応答時間の 95 パーセンタイル（条件を替えた 100 回、目標 200 ms） | 0.461 ms。40,000 文字のキーワードでも 7.17 ms |
| `bash scripts/verify.sh` | 成功（fmt・clippy `-D warnings`・cargo test 72 件（U1 51＋U2 21）・moon test 44 件（U1 32＋U2 12）・画面のビルド） |
| ブラウザ（Chromium）での確認 | 19 項目すべて OK（全角ＡＩと URL、0 件とリセット、カテゴリと空席ありのみ、受講料順、期間の前後が逆の案内、詳細から戻ったときの条件の保持、URL の不正な項目の除去、`<script>` を文字のまま表示、キーボード、ライブリージョン、ログにクエリの値が出ないこと） |

## 計画からのずれと注意

- **テストのコマンドの置き換え**: 計画の `moon -C web test --target js --file search_test.mbt` は、今の moon（0.1.20261028）に `--file` がないため動かない。同じ意味の `moon -C web test --target js search_test.mbt` に置き換えた（計画が許している置き換え）。
- **U1 のテスト用データ**: `Course` に必須の `title_reading` が増えたため、U1 のテスト 3 ファイルのコースを作る部分に 1 行ずつ足した。U1 の判定は変えていない。
- **初期データ**: C014 と S0020 を足したため、E005・E006 の初期の残高が 18,000 pt ずつ減る。U1 のテストは成功のまま。
- 確かめていないこと: スクリーンリーダーでの読み上げ（NFR5.3）と、入力中の文字の位置が保たれること（NFR1.3）の目視。
