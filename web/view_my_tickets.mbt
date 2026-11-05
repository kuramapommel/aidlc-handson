// 購入済みチケットの一覧の画面（U3。frontend-components.md の 1 節・5 節）。
// - 並びは Web API の応答のまま描き、画面で並べ替えない（BR12.2）。
// - 行はコース名・開催日時・会場または形式・チケット番号と「受講済み」（BR12.4、BR14.5）。
// - 詳細はその場で開閉し、画面は移らない。開いたら完了画面と同じ 7 項目（BR14.4）。
// - Web API から受け取った文字列は文字として描画する（NFR3.4）。

///|
/// ヘッダーのリンクの文言（BR14.7）。
pub let my_tickets_link_label : String = "購入済みチケット"

///|
/// 完了画面のリンクの文言（BR14.6）。
pub let view_my_tickets_link_label : String = "購入済みチケットを見る"

///|
/// 0 件の案内の文言（BR14.3）。
pub let my_tickets_empty_message : String = "購入済みのチケットはありません"

///|
/// 読み込み中の文言（BR14.2。0 件の案内とは別の文言）。
pub let my_tickets_loading_message : String = "購入済みチケットを読み込んでいます"

///|
/// 受講の状態の表示（BR14.5）。受講済みは文字で「受講済み」と示し、これからは何も出さない。
pub fn attendance_label(attendance : Attendance) -> String? {
  match attendance {
    Attended => Some("受講済み")
    Upcoming | OtherAttendance(_) => None
  }
}

///|
/// 開閉のボタンの名前（BR14.4、NFR5.2）。
pub fn ticket_toggle_label(is_open : Bool) -> String {
  if is_open {
    "詳細を閉じる"
  } else {
    "詳細を見る"
  }
}

///|
/// 開閉のボタンの `aria-expanded` の値（NFR5.2）。
pub fn ticket_toggle_expanded(is_open : Bool) -> String {
  if is_open {
    "true"
  } else {
    "false"
  }
}

///|
/// 詳細の部分の要素 ID（開閉のボタンの `aria-controls` で結ぶ）。
pub fn ticket_detail_panel_id(ticket_id : String) -> String {
  "my-ticket-detail-\{ticket_id}"
}

///|
/// 一覧の行に出す項目（BR12.4）。
pub fn owned_ticket_summary_rows(ticket : Ticket) -> Array[(String, String)] {
  [
    ("開催日時", format_session_time(ticket.starts_at, ticket.ends_at)),
    ("会場または形式", venue_label(ticket.venue, ticket.delivery_format)),
    ("チケット番号", ticket.ticket_number),
  ]
}

///|
/// 購入済みチケットの一覧の画面（WF1〜WF5）。読み込み中・0 件・失敗は別の表示にする（BR14.2）。
fn view_my_tickets_page(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  let body = match model.my_tickets {
    NotAsked | Loading =>
      view_loading(my_tickets_loading_message, "my-tickets-loading")
    Failed(error) =>
      view_load_failed(
        "購入済みチケット",
        error,
        RetryMyTickets,
        "my-tickets-failed",
        emit,
      )
    Missing =>
      view_load_failed(
        "購入済みチケット",
        NotFound,
        RetryMyTickets,
        "my-tickets-failed",
        emit,
      )
    Loaded(tickets) if tickets.length() == 0 => view_my_tickets_empty()
    Loaded(tickets) =>
      @html.ul(
        class="owned-tickets",
        attrs=testid("my-tickets-list"),
        tickets.map(owned => {
          view_owned_ticket(
            owned,
            is_ticket_open(model, owned.ticket.ticket_id),
            emit,
          )
        }),
      )
  }
  @html.section(class="my-tickets-page", attrs=testid("my-tickets-page"), [
    @html.h1("購入済みチケット"),
    @html.div(
      class="my-tickets-results",
      attrs=testid("my-tickets-results").aria_live("polite"),
      [body],
    ),
  ])
}

///|
/// 0 件の案内と「コース一覧へ」（BR14.3）。
fn view_my_tickets_empty() -> @html.Html {
  @html.div(class="load-state empty", attrs=testid("my-tickets-empty"), [
    @html.p(my_tickets_empty_message),
    @html.a(
      href=route_to_path(CourseListPage),
      class="back-link",
      attrs=testid("my-tickets-course-list-link"),
      "コース一覧へ",
    ),
  ])
}

///|
/// 一覧のチケット 1 枚。開閉のボタンは名前と開閉の状態を支援技術に伝える（BR14.4、NFR5.2）。
fn view_owned_ticket(
  owned : OwnedTicket,
  is_open : Bool,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let ticket = owned.ticket
  let id = ticket.ticket_id
  let panel_id = ticket_detail_panel_id(id)
  let attended = match attendance_label(owned.attendance) {
    Some(label) =>
      @html.p(
        class="attendance-attended",
        attrs=testid("my-ticket-attended-\{id}"),
        label,
      )
    None => @html.nothing
  }
  let detail = if is_open {
    @html.section(
      id=panel_id,
      class="owned-ticket-detail",
      attrs=testid("my-ticket-detail-\{id}"),
      [field_list(ticket_field_rows(ticket), "my-ticket-fields-\{id}")],
    )
  } else {
    @html.nothing
  }
  @html.li(
    class="owned-ticket",
    @html.article(class="ticket", attrs=testid("my-ticket-\{id}"), [
      @html.h2(ticket.course_title),
      attended,
      field_list(owned_ticket_summary_rows(ticket), "my-ticket-summary-\{id}"),
      @html.button(
        type_="button",
        on_click=emit(TicketDetailToggled(id)),
        attrs=testid("my-ticket-toggle-\{id}")
          .aria_expanded(ticket_toggle_expanded(is_open))
          .aria_controls(panel_id),
        ticket_toggle_label(is_open),
      ),
      detail,
    ]),
  )
}
