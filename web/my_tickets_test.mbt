// 購入済みチケットの一覧（U3）の画面のテスト（更新関数・応答の読み取り・描画に使う値）。
// Web API の応答（成功・0 件・失敗）はメッセージとして直接与える。ブラウザ・Web API は使わない。
// 実行（U3 だけ）: リポジトリのルートで `moon -C web test --target js my_tickets_test.mbt`

///|
fn mt_ticket(id : String, number : String) -> @web.Ticket {
  {
    ticket_id: id,
    ticket_number: number,
    employee_name: "伊藤 さくら",
    course_id: "C001",
    course_title: "チームマネジメント入門",
    session_id: "S0001",
    starts_at: "2026-11-05T13:00:00+09:00",
    ends_at: "2026-11-05T16:00:00+09:00",
    venue: Some("東京本社 研修室A"),
    delivery_format: "venue",
    purchased_at: "2026-11-02T10:00:00+09:00",
    paid_points: 25000,
  }
}

///|
fn mt_owned(
  id : String,
  number : String,
  attendance : @web.Attendance,
) -> @web.OwnedTicket {
  { ticket: mt_ticket(id, number), attendance }
}

///|
/// これから 1 枚と受講済み 1 枚（Web API が並べた順）。
fn mt_two_tickets() -> Array[@web.OwnedTicket] {
  [mt_owned("T-1", "AAAA-AAAA", Upcoming), mt_owned("T-2", "BBBB-BBBB", Attended)]
}

///|
fn mt_me(employee_id : String) -> @web.Me {
  {
    employee_id: Some(employee_id),
    display_name: Some("伊藤 さくら"),
    company_name: Some("株式会社サンプル商事"),
    contract_status: Some("active"),
    eligibility: Eligible,
    balance: 100000,
  }
}

///|
fn mt_run(model : @web.Model, messages : Array[@web.Msg]) -> @web.Model {
  let mut current = model
  for msg in messages {
    current = @web.update(current, msg).0
  }
  current
}

///|
/// コース一覧から開き、操作中の従業員を読み込んだ状態（世代 0）。
fn mt_started() -> @web.Model {
  mt_run(@web.init_model(), [UrlChanged("/"), MeLoaded(0, Ok(mt_me("E005")))])
}

///|
/// 一覧の画面を開いて 2 枚を読み込んだ状態。
fn mt_on_my_tickets() -> @web.Model {
  mt_run(mt_started(), [
    UrlChanged("/my-tickets"),
    MyTicketsLoaded(0, Ok(mt_two_tickets())),
  ])
}

///|
test "URL /my-tickets と一覧の画面を相互に変換する（BR14.1）" {
  assert_eq(@web.route_from_path("/my-tickets"), MyTicketsPage)
  assert_eq(@web.route_from_path("my-tickets/"), MyTicketsPage)
  assert_eq(@web.route_from_path("/my-tickets?employeeId=E006"), MyTicketsPage)
  assert_eq(@web.route_to_path(MyTicketsPage), "/my-tickets")
  assert_eq(@web.route_from_path("/my-tickets/T-1"), NotFoundPage)
  // U1 のチケット 1 枚の画面はそのまま
  assert_eq(@web.route_from_path("/tickets/T-1"), PurchaseCompletePage("T-1"))
}

///|
test "一覧の画面へ移ると読み込み中にして一覧を取得する。起動時の URL でも同じ（WF1、BR14.1、BR14.2）" {
  let (model, effects) = @web.update(mt_started(), UrlChanged("/my-tickets"))
  assert_eq(model.route, MyTicketsPage)
  assert_eq(model.my_tickets, Loading)
  assert_eq(model.open_ticket_ids, [])
  assert_eq(effects, [FetchMyTickets(0)])
  let (direct, direct_effects) = @web.update(
    @web.init_model(),
    UrlChanged("/my-tickets"),
  )
  assert_eq(direct.my_tickets, Loading)
  assert_eq(direct_effects, [FetchMe(0), FetchMyTickets(0)])
}

///|
test "成功は応答の並びのまま、0 件は空の一覧、失敗と見つからないは失敗の表示にする（AC4.2.1、AC4.2.2、BR14.2）" {
  assert_eq(mt_on_my_tickets().my_tickets, Loaded(mt_two_tickets()))
  let empty = mt_run(mt_started(), [
    UrlChanged("/my-tickets"),
    MyTicketsLoaded(0, Ok([])),
  ])
  assert_eq(empty.my_tickets, Loaded([]))
  let failed = mt_run(mt_started(), [
    UrlChanged("/my-tickets"),
    MyTicketsLoaded(0, Err(Http(500, "予期しないエラーが発生しました"))),
  ])
  assert_eq(failed.my_tickets, Failed(Http(500, "予期しないエラーが発生しました")))
  let network = mt_run(mt_started(), [
    UrlChanged("/my-tickets"),
    MyTicketsLoaded(0, Err(Network("TypeError"))),
  ])
  assert_eq(network.my_tickets, Failed(Network("TypeError")))
}

///|
test "再試行は一覧の画面でだけ読み込み中に戻して取得し直す（BR14.2、NFR2.3）" {
  let failed = mt_run(mt_started(), [
    UrlChanged("/my-tickets"),
    MyTicketsLoaded(0, Err(Network("TypeError"))),
  ])
  let (retried, effects) = @web.update(failed, RetryMyTickets)
  assert_eq(retried.my_tickets, Loading)
  assert_eq(effects, [FetchMyTickets(0)])
  // 一覧の画面でなければ何もしない
  let (elsewhere, none) = @web.update(mt_started(), RetryMyTickets)
  assert_eq(none, [])
  assert_eq(elsewhere.my_tickets, NotAsked)
}

///|
test "詳細はチケットごとに別々に開閉でき、開閉では通信しない（AC4.2.3、BR14.4、NFR1.2）" {
  let model = mt_on_my_tickets()
  let (opened, effects1) = @web.update(model, TicketDetailToggled("T-1"))
  assert_eq(effects1, [])
  assert_eq(opened.open_ticket_ids, ["T-1"])
  let (both, effects2) = @web.update(opened, TicketDetailToggled("T-2"))
  assert_eq(effects2, [])
  assert_true(@web.is_ticket_open(both, "T-1"))
  assert_true(@web.is_ticket_open(both, "T-2"))
  let (closed, effects3) = @web.update(both, TicketDetailToggled("T-1"))
  assert_eq(effects3, [])
  assert_false(@web.is_ticket_open(closed, "T-1"))
  assert_true(@web.is_ticket_open(closed, "T-2"))
  // 開閉しても一覧そのものは変わらない
  assert_eq(closed.my_tickets, model.my_tickets)
  assert_eq(@web.toggle_ticket_id([], "X"), ["X"])
  assert_eq(@web.toggle_ticket_id(["X", "Y"], "X"), ["Y"])
}

///|
test "一覧の画面でないときや前の世代の応答は表示に使わない（BR14.1、BR14.8）" {
  // 読み込み中にコース一覧へ移った後に届いた応答は捨てる
  let moved = mt_run(mt_started(), [UrlChanged("/my-tickets"), UrlChanged("/")])
  let (after, _) = @web.update(moved, MyTicketsLoaded(0, Ok(mt_two_tickets())))
  assert_false(after.my_tickets is Loaded(_))
  // 前の世代（切り替え前の従業員）の応答も捨てる
  let on_page = mt_run(mt_started(), [UrlChanged("/my-tickets")])
  let (stale, _) = @web.update(on_page, MyTicketsLoaded(-1, Ok(mt_two_tickets())))
  assert_eq(stale.my_tickets, Loading)
}

///|
test "一覧の画面を開き直すと読み込み直し、開閉は閉じた状態に戻る（BR14.1、BR14.4、NFR2.2）" {
  let opened = mt_run(mt_on_my_tickets(), [
    TicketDetailToggled("T-1"),
    UrlChanged("/"),
  ])
  let (reopened, effects) = @web.update(opened, UrlChanged("/my-tickets"))
  assert_eq(reopened.my_tickets, Loading)
  assert_eq(reopened.open_ticket_ids, [])
  assert_eq(effects, [FetchMyTickets(0)])
}

///|
test "従業員を切り替えると一覧と開閉の状態を捨て、次は切り替えた後の従業員の一覧を読む（BR14.8、NFR3.5）" {
  let opened = mt_run(mt_on_my_tickets(), [TicketDetailToggled("T-1")])
  let (switched, _) = @web.update(opened, EmployeeSwitched(Ok(mt_me("E001"))))
  assert_eq(switched.route, CourseListPage)
  assert_eq(switched.my_tickets, NotAsked)
  assert_eq(switched.open_ticket_ids, [])
  // 切り替え前に送った要求の応答（世代 0）は捨てる
  let (late, _) = @web.update(
    { ..switched, route: MyTicketsPage, my_tickets: Loading },
    MyTicketsLoaded(0, Ok(mt_two_tickets())),
  )
  assert_eq(late.my_tickets, Loading)
  // 次に開くと、新しい世代で取得する
  let (_, effects) = @web.update(switched, UrlChanged("/my-tickets"))
  assert_eq(effects, [FetchMyTickets(1)])
}

///|
test "購入直後の完了画面にだけ「購入済みチケットを見る」を出し、そこから一覧を読み込む（AC4.2.4、BR14.6）" {
  let detail : @web.CourseDetail = {
    course: {
      course_id: "C001",
      title: "チームマネジメント入門",
      category: "management",
      summary: "概要",
      description: "説明",
      fee_points: 25000,
      duration_minutes: 180,
      delivery_format: "venue",
    },
    sessions: [
      {
        session_id: "S0001",
        starts_at: "2026-11-05T13:00:00+09:00",
        ends_at: "2026-11-05T16:00:00+09:00",
        venue: Some("東京本社 研修室A"),
        capacity: 3,
        remaining_seats: 3,
        status: Available,
      },
    ],
  }
  let ticket = mt_ticket("T-9", "CCCC-CCCC")
  let complete = mt_run(@web.init_model(), [
    UrlChanged("/courses/C001/sessions/S0001/confirm"),
    MeLoaded(0, Ok(mt_me("E005"))),
    CourseDetailLoaded(0, "C001", Ok(detail)),
    ConfirmClicked,
    PurchaseSucceeded(0, ticket, 75000),
    UrlChanged("/tickets/T-9"),
  ])
  assert_eq(complete.route, PurchaseCompletePage("T-9"))
  assert_true(@web.is_just_purchased(complete, ticket))
  assert_eq(@web.view_my_tickets_link_label, "購入済みチケットを見る")
  // 完了画面から一覧へ移ると、読み込み直す（いま購入したチケットが含まれる。NFR2.2）
  let (_, effects) = @web.update(complete, UrlChanged("/my-tickets"))
  assert_eq(effects, [FetchMyTickets(0)])
  // 完了画面を開き直しただけのとき（購入直後でない）は出さない
  let reopened = mt_run(@web.init_model(), [
    UrlChanged("/tickets/T-9"),
    TicketLoaded(0, "T-9", Ok(ticket)),
  ])
  assert_false(@web.is_just_purchased(reopened, ticket))
}

///|
test "描画に使う値: 受講済みの文字、開閉のボタンの名前と aria-expanded、ヘッダーのリンク（AC4.2.5、BR14.5、NFR5.2）" {
  assert_eq(@web.attendance_label(Attended), Some("受講済み"))
  assert_eq(@web.attendance_label(Upcoming), None)
  assert_eq(@web.attendance_label(OtherAttendance("x")), None)
  assert_eq(@web.ticket_toggle_label(false), "詳細を見る")
  assert_eq(@web.ticket_toggle_label(true), "詳細を閉じる")
  assert_eq(@web.ticket_toggle_expanded(false), "false")
  assert_eq(@web.ticket_toggle_expanded(true), "true")
  assert_eq(@web.ticket_detail_panel_id("T-1"), "my-ticket-detail-T-1")
  assert_eq(@web.my_tickets_link_label, "購入済みチケット")
  assert_eq(@web.my_tickets_empty_message, "購入済みのチケットはありません")
  assert_true(
    @web.my_tickets_loading_message != @web.my_tickets_empty_message,
  )
  // 開いたときは完了画面と同じ 7 項目、行は 3 項目（コース名は見出し）（BR12.4）
  let ticket = mt_ticket("T-1", "AAAA-AAAA")
  let rows = @web.ticket_field_rows(ticket)
  assert_eq(rows.length(), 7)
  assert_eq(rows[0], ("チケット番号", "AAAA-AAAA"))
  assert_eq(rows[3], ("開催日時", "2026年11月5日（木）13:00〜16:00"))
  assert_eq(rows[6], ("支払ったポイント", "25,000 pt"))
  assert_eq(@web.owned_ticket_summary_rows(ticket), [
    ("開催日時", "2026年11月5日（木）13:00〜16:00"),
    ("会場または形式", "東京本社 研修室A"),
    ("チケット番号", "AAAA-AAAA"),
  ])
}

///|
test "一覧の応答を読み取る（受講の状態・0 件・失敗・形の違い）（BR13.2）" {
  let body =
    #|{"tickets":[
    #| {"ticketId":"T-1","ticketNumber":"AAAA-AAAA","employeeName":"伊藤 さくら","courseId":"C002","courseTitle":"1on1 ミーティング実践","sessionId":"S0005","startsAt":"2026-11-19T10:00:00+09:00","endsAt":"2026-11-19T12:00:00+09:00","venue":null,"deliveryFormat":"online","purchasedAt":"2026-11-01T10:00:00+09:00","paidPoints":18000,"attendance":"upcoming"},
    #| {"ticketId":"T-2","ticketNumber":"BBBB-BBBB","employeeName":"伊藤 さくら","courseId":"C001","courseTitle":"チームマネジメント入門","sessionId":"S0001","startsAt":"2026-10-01T13:00:00+09:00","endsAt":"2026-10-01T16:00:00+09:00","venue":"東京本社 研修室A","deliveryFormat":"venue","purchasedAt":"2026-09-01T10:00:00+09:00","paidPoints":25000,"attendance":"attended"}
    #|]}
  guard @web.decode_owned_tickets({ status: 200, body }) is Ok(tickets) else {
    fail("読み取れるはず")
  }
  assert_eq(tickets.length(), 2)
  assert_eq(tickets[0].ticket.ticket_number, "AAAA-AAAA")
  assert_eq(tickets[0].ticket.venue, None)
  assert_eq(tickets[0].attendance, Upcoming)
  assert_eq(tickets[1].attendance, Attended)
  assert_eq(
    @web.decode_owned_tickets({ status: 200, body: "{\"tickets\":[]}" }),
    Ok([]),
  )
  let error_body =
    #|{"error":{"code":"internal-error","message":"予期しないエラーが発生しました"}}
  assert_eq(
    @web.decode_owned_tickets({ status: 500, body: error_body }),
    Err(Http(500, "予期しないエラーが発生しました")),
  )
  assert_true(
    @web.decode_owned_tickets({ status: 200, body: "{\"tickets\":[{}]}" })
    is Err(Decode(_)),
  )
}
