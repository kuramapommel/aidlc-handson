# コードのまとめ（U3 購入済みチケット）

## 作成・変更したファイル

新規

| ファイル | 内容 |
| --- | --- |
| `crates/purchasing/src/owned_tickets.rs` | `OwnedTicket`（U1 の `TicketView` と受講の状態）、`attendance_of`（開始日時 ≤ 現在時刻で受講済み）、`compare_owned_tickets`、`list_owned_tickets`（操作中の従業員で絞り、並べる。日程・コースが見つからないチケットは不整合として返す） |
| `crates/purchasing/src/owned_tickets_tests.rs` | 一覧の業務ロジックのテスト 7 件 |
| `crates/web_api/tests/my_tickets.rs` | `GET /api/tickets` のテスト 7 件 |
| `web/update_my_tickets.mbt` | 一覧の読み込み・受け取り・再試行・開閉の更新 |
| `web/view_my_tickets.mbt` | 一覧の画面（行・開閉のボタン・詳細・読み込み中・0 件・失敗） |
| `web/my_tickets_test.mbt` | 画面の更新関数と描画に使う値のテスト 11 件 |

変更

| ファイル | 内容 |
| --- | --- |
| `crates/purchasing/src/lib.rs` | `owned_tickets` の公開 |
| `crates/web_api/src/app.rs` | `GET /api/tickets` のルート（読み出し用のルーターに置き、CSRF の対象外） |
| `crates/web_api/src/handlers.rs` | `my_tickets` ハンドラ（不整合はチケット ID だけをログに出して 500） |
| `crates/web_api/src/dto.rs` | `OwnedTicketDto`（U1 の `TicketDto` に `attendance` を足す）・`OwnedTicketListDto`（`{"tickets": [...]}`） |
| `web/model.mbt`・`web/route.mbt`・`web/update.mbt`・`web/decode.mbt`・`web/web.mbt`・`web/view.mbt` | Route `MyTicketsPage`（`/my-tickets`）、状態 `my_tickets`・`open_ticket_ids`、メッセージ、通信のコマンド、応答の読み取り、従業員の切り替えでの破棄 |
| `web/view_header.mbt` | ヘッダーに「購入済みチケット」 |
| `web/view_pages.mbt` | 完了画面（購入直後）に「購入済みチケットを見る」。7 項目の行を `ticket_field_rows` に切り出して一覧の詳細と共有 |
| `web/static/style.css` | U3 の画面のスタイル |
| `README.md` | U3 の状況、デモの手順、一覧の受け口、U3 のテストのコマンド |

削除したファイルはない。

## 実装上の判断

- 一覧の値は `OwnedTicket { view: TicketView, attendance }` とし、U1 の `TicketView` をそのまま使った。応答は U1 の `TicketDto` を取り込んで `attendance` を足したので、項目名と日時の形式が U1 のチケット詳細と同じになる（`courseId`・`sessionId` も含む）。
- 応答の形は `{"tickets": [...]}`（entities.md の OwnedTicketList.tickets に合わせた）。
- Cookie がない・形式が不正・存在しない従業員のときは空の一覧にする。Cookie の発行は U1 の `/api/me` に任せる。
- 並びは、受講の状態のグループ → 開始日時（これからは早い順、受講済みは遅い順）→ 購入日時 → チケット番号。
- 画面の開閉の状態は不変の配列で持ち、`toggle_ticket_id` が新しい配列を返す。一覧の応答は、世代が今と同じで画面が一覧のときだけ反映する。一覧を開き直すたびに読み込み直し、開閉の状態も空に戻す。

## テストの結果

| コマンド | 結果 |
| --- | --- |
| `cargo test -p purchasing owned_tickets_tests` | 7 件成功 |
| `cargo test -p web_api --test my_tickets` | 7 件成功（一覧の 95 パーセンタイルは約 0.26 ms。目標は 200 ms） |
| `moon -C web test --target js my_tickets_test.mbt` | 11 件成功 |
| `bash scripts/verify.sh` | すべて成功（fmt・clippy・Rust の全テスト・画面のテスト 57 件・画面のビルド） |

## 計画からのずれ

- MoonBit のテストのコマンドは、計画の `--file my_tickets_test.mbt` を U2 と同じく `moon -C web test --target js my_tickets_test.mbt`（ファイル名を位置の引数で渡す）に置き換えた。README もこの形で書いた。
- 開発の途中では moon が入っておらず画面を確かめられなかったため、利用者の承認を得て公式の手順（`curl -fsSL https://cli.moonbitlang.com/install/unix.sh | bash`、moon 0.1.20261102）で導入してから、画面のテストと確認コマンドを実行した。新しい moon では既存のコードに `derive(...)` の書き方が古いという警告が出るが、エラーではない（U1・U2 の既存のコードも同じ）。
- rabbita の Html を文字列にして確かめる方法がないため、描画のテストは、描画に使う値（「受講済み」の文言、開閉のボタンの名前と `aria-expanded` の値、詳細の ID、行の項目、リンクの文言など）を純粋関数に切り出して確かめている。HTML の属性そのものを確かめるテストではない。
- 閉じているときは詳細の要素を描画しないので、その間 `aria-controls` が指す ID は存在しない。
- `web/pkg.generated.mbti`（`moon info` で作るファイル）は更新していない。確認コマンドの対象ではない。
- README の編集は、開発者がシェルの文字列置換で行った（決まりではファイルの編集の道具を使う）。差分は意図どおりであることを確認した。
- 初期データの E005 のチケットはすべて「これから」なので、「受講済み」の並びは単体テストでのみ確かめている（機能仕様の 6 章のとおり）。
