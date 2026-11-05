// 購入済みチケットの一覧（U3）の更新関数。frontend-components.md の 4 節に対応する。
// 純粋関数で、通信は Effect（FetchMyTickets）として返すだけにする（U1・U2 と同じ）。

///|
/// 一覧を読み込み中にして取得する。開閉の状態は閉じた状態に戻す（BR14.1、BR14.4）。
fn request_my_tickets(model : Model) -> (Model, Array[Effect]) {
  (
    { ..model, my_tickets: Loading, open_ticket_ids: [] },
    [FetchMyTickets(model.generation)],
  )
}

///|
/// 一覧の応答。今の従業員の世代で、一覧の画面にいるときだけ反映する（BR14.1、BR14.8、NFR2.3）。
fn on_my_tickets_loaded(
  model : Model,
  from : Int32,
  result : Result[Array[OwnedTicket], FetchError],
) -> (Model, Array[Effect]) {
  if from == model.generation && model.route is MyTicketsPage {
    ({ ..model, my_tickets: to_remote(result) }, [])
  } else {
    (model, [])
  }
}

///|
/// 「再試行」。一覧の画面にいるときだけ、同じ一覧を読み込み直す（BR14.2）。
fn on_retry_my_tickets(model : Model) -> (Model, Array[Effect]) {
  if model.route is MyTicketsPage {
    request_my_tickets(model)
  } else {
    (model, [])
  }
}

///|
/// 詳細の開閉。開いていれば閉じ、閉じていれば開く。ほかのチケットの開閉は変えず、通信しない
/// （BR14.4、BR13.2、NFR1.2）。
fn on_ticket_detail_toggled(
  model : Model,
  ticket_id : String,
) -> (Model, Array[Effect]) {
  (
    {
      ..model,
      open_ticket_ids: toggle_ticket_id(model.open_ticket_ids, ticket_id),
    },
    [],
  )
}

///|
/// チケット ID の集まりで、その ID を入れるか外すかを切り替えた新しい集まりを返す（NFR6.2）。
pub fn toggle_ticket_id(ids : Array[String], ticket_id : String) -> Array[String] {
  if ids.contains(ticket_id) {
    ids.filter(id => id != ticket_id)
  } else {
    [..ids, ticket_id]
  }
}

///|
/// そのチケットの詳細を開いているか。
pub fn is_ticket_open(model : Model, ticket_id : String) -> Bool {
  model.open_ticket_ids.contains(ticket_id)
}
