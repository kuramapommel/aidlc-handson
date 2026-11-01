// 画面の公開の振る舞いのテスト（更新関数: メッセージ＋状態 → 新しい状態＋作用）。
// Web API の応答は、メッセージ（成功・失敗・拒否）として直接与える。ブラウザや Web API は使わない。
// 実行: リポジトリのルートで `moon -C web test --target js`

///|
fn me_fixture(
  employee_id~ : String = "E001",
  balance~ : Int32 = 60000,
  eligibility~ : @web.Eligibility = Eligible,
) -> @web.Me {
  {
    employee_id: Some(employee_id),
    display_name: Some("高橋 遥"),
    company_name: Some("株式会社サンプル商事"),
    contract_status: Some("active"),
    eligibility,
    balance,
  }
}

///|
fn session_fixture(id : String, status : @web.SessionStatus) -> @web.Session {
  {
    session_id: id,
    starts_at: "2026-11-05T13:00:00+09:00",
    ends_at: "2026-11-05T16:00:00+09:00",
    venue: Some("東京本社 研修室A"),
    capacity: 3,
    remaining_seats: if status == Full {
      0
    } else {
      3
    },
    status,
  }
}

///|
/// C001（受講料 25,000 pt）。S0001 は購入可、S0002 は満席、S0003 は受付終了。
fn detail_fixture(s0001~ : @web.SessionStatus = Available) -> @web.CourseDetail {
  {
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
      session_fixture("S0001", s0001),
      session_fixture("S0002", Full),
      session_fixture("S0003", Closed),
    ],
  }
}

///|
fn ticket_fixture() -> @web.Ticket {
  {
    ticket_id: "T-1",
    ticket_number: "ABCD-EFGH",
    employee_name: "高橋 遥",
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
/// メッセージを順に与えた後の状態。
fn run(model : @web.Model, messages : Array[@web.Msg]) -> @web.Model {
  let mut current = model
  for msg in messages {
    current = @web.update(current, msg).0
  }
  current
}

///|
/// コース C001 の詳細を開き、操作中の従業員と詳細を読み込んだ状態。
fn on_detail(me~ : @web.Me = me_fixture()) -> @web.Model {
  run(@web.init_model(), [
    UrlChanged("/courses/C001"),
    MeLoaded(0, Ok(me)),
    CourseDetailLoaded(0, "C001", Ok(detail_fixture())),
  ])
}

///|
/// C001 の S0001 の購入確認を開き、データを読み込んだ状態。
fn on_confirm(me~ : @web.Me = me_fixture()) -> @web.Model {
  run(@web.init_model(), [
    UrlChanged("/courses/C001/sessions/S0001/confirm"),
    MeLoaded(0, Ok(me)),
    CourseDetailLoaded(0, "C001", Ok(detail_fixture())),
  ])
}

///|
test "起動時は操作中の従業員とコース一覧を読み込み中にして取得する（WF1、BR7.1）" {
  let (model, effects) = @web.update(@web.init_model(), UrlChanged("/"))
  assert_eq(model.route, CourseListPage)
  assert_eq(model.me, Loading)
  assert_eq(model.course_list, Loading)
  assert_eq(effects, [FetchMe(0), FetchCourses(0, 1)])
}

///|
test "起動時の URL がコースの詳細なら、その詳細を取得する（URL から画面を復元）" {
  let (model, effects) = @web.update(
    @web.init_model(),
    UrlChanged("/courses/C001"),
  )
  assert_eq(model.route, CourseDetailPage("C001"))
  assert_eq(model.course_detail, Loading)
  assert_eq(effects, [FetchMe(0), FetchCourseDetail(0, "C001")])
}

///|
test "一覧の読み込みに失敗したら失敗にし、再試行で読み込み中に戻して取得し直す（BR7.1）" {
  let failed = run(@web.init_model(), [
    UrlChanged("/"),
    CourseListLoaded(0, 1, Err(Network("接続できません"))),
  ])
  assert_eq(failed.course_list, Failed(Network("接続できません")))
  let (retried, effects) = @web.update(failed, RetryCourseList)
  assert_eq(retried.course_list, Loading)
  assert_eq(effects, [FetchCourses(0, 2)])
}

///|
test "購入可の日程だけ選べる（BR2.5）" {
  let model = on_detail()
  assert_eq(run(model, [SessionSelected("S0002")]).selected_session_id, None)
  assert_eq(run(model, [SessionSelected("S0003")]).selected_session_id, None)
  assert_eq(run(model, [SessionSelected("S9999")]).selected_session_id, None)
  assert_eq(
    run(model, [SessionSelected("S0001")]).selected_session_id,
    Some("S0001"),
  )
}

///|
test "日程を選ぶまでは購入確認へ進めず、選ぶと購入確認の URL へ移る（BR7.2）" {
  let model = on_detail()
  let (unchanged, effects) = @web.update(model, ProceedToConfirm)
  assert_eq(unchanged, model)
  assert_eq(effects, [])
  let selected = run(model, [SessionSelected("S0001")])
  let (_, effects) = @web.update(selected, ProceedToConfirm)
  assert_eq(effects, [PushUrl("/courses/C001/sessions/S0001/confirm")])
}

///|
test "支払えて利用資格があれば確定でき、購入後の残高を計算できる（BR4.2、BR4.4）" {
  let model = on_confirm(me=me_fixture(balance=25000))
  guard @web.confirm_readiness(model) is Ready(summary) else {
    fail("確定できるはず")
  }
  assert_eq(summary.balance - summary.course.fee_points, 0)
}

///|
test "残高が足りなければ確定できず、不足額を示す（BR4.2、BR4.3）" {
  let model = on_confirm(me=me_fixture(balance=20000))
  guard @web.confirm_readiness(model) is Blocked(_, reason) else {
    fail("確定できないはず")
  }
  assert_eq(reason, InsufficientBalance(5000))
  let (after, effects) = @web.update(model, ConfirmClicked)
  assert_eq(after.purchase, Idle)
  assert_eq(effects, [])
}

///|
test "契約外の企業の従業員は確定できない（BR1.4）" {
  let model = on_confirm(me=me_fixture(eligibility=CompanyNotActive))
  guard @web.confirm_readiness(model) is Blocked(_, reason) else {
    fail("確定できないはず")
  }
  assert_eq(reason, NotEligible)
  let (_, effects) = @web.update(model, ConfirmClicked)
  assert_eq(effects, [])
}

///|
test "確定すると処理中になって要求を 1 回送り、処理中の 2 回目の押下は無視する（BR7.3、BR5.2）" {
  let (submitting, effects) = @web.update(on_confirm(), ConfirmClicked)
  assert_eq(submitting.purchase, Submitting)
  assert_eq(effects, [SubmitPurchase(0, "S0001")])
  let (again, effects) = @web.update(submitting, ConfirmClicked)
  assert_eq(again, submitting)
  assert_eq(effects, [])
}

///|
test "購入の成功でヘッダーの残高を購入後の値にし、完了画面へ移る（BR7.7、WF3）" {
  let submitting = run(on_confirm(), [ConfirmClicked])
  let ticket = ticket_fixture()
  let (succeeded, effects) = @web.update(
    submitting,
    PurchaseSucceeded(0, ticket, 35000),
  )
  guard succeeded.me is Loaded(me) else {
    fail("操作中の従業員があるはず")
  }
  assert_eq(me.balance, 35000)
  assert_eq(succeeded.purchase, Succeeded(ticket))
  assert_eq(effects, [PushUrl("/tickets/T-1")])
  // 完了画面では購入の応答の e チケットを使い、取得し直さない
  let (complete, effects) = @web.update(succeeded, UrlChanged("/tickets/T-1"))
  assert_eq(complete.route, PurchaseCompletePage("T-1"))
  assert_eq(complete.ticket, Loaded(ticket))
  assert_eq(effects, [])
}

///|
test "完了画面を開き直したときはチケット 1 枚の詳細を取得し、404 は見つからないにする（WF5）" {
  let (opened, effects) = @web.update(
    @web.init_model(),
    UrlChanged("/tickets/T-9"),
  )
  assert_eq(effects, [FetchMe(0), FetchTicket(0, "T-9")])
  let missing = run(opened, [TicketLoaded(0, "T-9", Err(NotFound))])
  assert_eq(missing.ticket, Missing)
}

///|
test "購入の拒否では理由を保ち、残高は変えない（BR7.4）" {
  let submitting = run(on_confirm(), [ConfirmClicked])
  let (rejected, effects) = @web.update(
    submitting,
    PurchaseRejected(0, ReasonFull),
  )
  assert_eq(rejected.purchase, Rejected(ReasonFull))
  assert_eq(rejected.me, submitting.me)
  assert_eq(effects, [])
}

///|
test "拒否の理由ごとの文言と次の操作（BR7.4）" {
  let cases : Array[(@web.RejectionReason, String, Bool)] = [
    (ReasonFull, "この日程は満席になりました", true),
    (ReasonClosed, "この日程は受付を終了しました", true),
    (ReasonAlreadyPurchased, "この日程はすでに購入済みです", true),
    (
      ReasonInsufficientBalance,
      "研修予算の残高が不足しています",
      true,
    ),
    (
      ReasonCompanyNotActive,
      "ご所属の企業は現在このサービスをご利用いただけません",
      false,
    ),
    (
      ReasonEmployeeNotFound,
      "ご所属の企業は現在このサービスをご利用いただけません",
      false,
    ),
    (ReasonSessionNotFound, "この日程は見つかりません", false),
  ]
  for case in cases {
    let (reason, message, offers_another) = case
    assert_eq(@web.rejection_message(reason), message)
    assert_eq(@web.rejection_offers_another_session(reason), offers_another)
  }
}

///|
test "409 の応答の本文から拒否の理由を読み取る（BR7.4）" {
  let body =
    #|{"error":{"code":"purchase-rejected","message":"この日程はすでに購入済みです","reason":"already-purchased"}}
  assert_eq(
    @web.purchase_message(3, { status: 409, body, }),
    PurchaseRejected(3, ReasonAlreadyPurchased),
  )
  let forbidden =
    #|{"error":{"code":"forbidden","message":"この要求は受け付けられません"}}
  assert_eq(
    @web.purchase_message(3, { status: 403, body: forbidden, }),
    PurchaseFailed(3, "この要求は受け付けられません"),
  )
}

///|
test "通信の失敗では NetworkFailed にし、もう一度試すで要求を送り直す（BR7.6）" {
  let submitting = run(on_confirm(), [ConfirmClicked])
  let failed = run(submitting, [PurchaseNetworkFailed(0)])
  assert_eq(failed.purchase, NetworkFailed)
  let (retried, effects) = @web.update(failed, RetryPurchase)
  assert_eq(retried.purchase, Submitting)
  assert_eq(effects, [SubmitPurchase(0, "S0001")])
}

///|
test "「別の日程を選ぶ」はコースの詳細へ移って詳細を取得し直す（BR7.5）" {
  let rejected = run(on_confirm(), [
    ConfirmClicked,
    PurchaseRejected(0, ReasonFull),
  ])
  let (model, effects) = @web.update(rejected, ChooseAnotherSession)
  assert_eq(model.route, CourseDetailPage("C001"))
  assert_eq(model.course_detail, Loading)
  assert_eq(model.selected_session_id, None)
  assert_eq(model.purchase, Idle)
  assert_eq(effects, [PushUrl("/courses/C001"), FetchCourseDetail(0, "C001")])
}

///|
test "「日程を選び直す」は要求を送らずにコースの詳細へ戻る（BR7.8）" {
  let (_, effects) = @web.update(on_confirm(), BackToDetail)
  assert_eq(effects, [PushUrl("/courses/C001")])
}

///|
test "購入確認を開いたときに日程が購入可でなくなっていれば、詳細へ戻して理由を案内する" {
  let model = run(@web.init_model(), [
    UrlChanged("/courses/C001/sessions/S0001/confirm"),
    MeLoaded(0, Ok(me_fixture())),
  ])
  let (back, effects) = @web.update(
    model,
    CourseDetailLoaded(0, "C001", Ok(detail_fixture(s0001=Full))),
  )
  assert_eq(back.route, CourseDetailPage("C001"))
  assert_eq(back.selected_session_id, None)
  assert_eq(
    back.notice,
    Some(
      "選んだ日程は「満席」のため購入できません。日程を選び直してください。",
    ),
  )
  assert_eq(effects, [ReplaceUrl("/courses/C001")])
}

///|
test "従業員の切り替えの成功で、どの画面からも一覧へ戻り前の情報を捨てる（BR6.4）" {
  let complete = run(on_confirm(), [
    ConfirmClicked,
    PurchaseSucceeded(0, ticket_fixture(), 35000),
    UrlChanged("/tickets/T-1"),
    SwitcherOpened,
    EmployeeChosen("E004"),
  ])
  assert_true(complete.switcher.is_switching)
  let next_me = me_fixture(employee_id="E004", eligibility=CompanyNotActive)
  let (switched, effects) = @web.update(complete, EmployeeSwitched(Ok(next_me)))
  assert_eq(switched.route, CourseListPage)
  assert_eq(switched.generation, 1)
  assert_eq(switched.me, Loaded(next_me))
  assert_eq(switched.course_list, Loading)
  assert_eq(switched.course_detail, NotAsked)
  assert_eq(switched.selected_session_id, None)
  assert_eq(switched.purchase, Idle)
  assert_eq(switched.ticket, NotAsked)
  assert_false(switched.switcher.is_open)
  assert_eq(effects, [PushUrl("/"), FetchCourses(1, 1)])
  // 切り替えの前の従業員の要求の応答は捨てる
  let stale = run(switched, [
    CourseListLoaded(0, 1, Ok({ courses: [], count: 0, })),
    MeLoaded(0, Ok(me_fixture())),
  ])
  assert_eq(stale, switched)
  // 一覧へ移ったことによる URL の変化では、取得を重ねない
  let (_, effects) = @web.update(switched, UrlChanged("/"))
  assert_eq(effects, [])
}

///|
test "従業員の切り替えに失敗したら、操作中の従業員は変えずに失敗を案内する（WF4）" {
  let opened = run(on_detail(), [SwitcherOpened, EmployeeChosen("E002")])
  let (failed, effects) = @web.update(
    opened,
    EmployeeSwitched(
      Err(Http(500, "予期しないエラーが発生しました")),
    ),
  )
  assert_eq(failed.me, opened.me)
  assert_eq(failed.route, CourseDetailPage("C001"))
  assert_false(failed.switcher.is_switching)
  assert_eq(
    failed.switcher.error,
    Some(
      "従業員を切り替えられませんでした（予期しないエラーが発生しました）",
    ),
  )
  assert_eq(effects, [])
}

///|
test "画面を移った後に届いた前の画面のコースの詳細は捨てる" {
  let moved = run(@web.init_model(), [
    UrlChanged("/courses/C001"),
    UrlChanged("/courses/C002"),
  ])
  let after = run(moved, [CourseDetailLoaded(0, "C001", Ok(detail_fixture()))])
  assert_eq(after.course_detail, Loading)
}

///|
test "URL と画面の対応（直接開いた URL から画面を復元する）" {
  assert_eq(@web.route_from_path("/"), CourseListPage)
  assert_eq(@web.route_from_path("/courses/C001"), CourseDetailPage("C001"))
  assert_eq(
    @web.route_from_path("/courses/C001/sessions/S0001/confirm"),
    PurchaseConfirmPage("C001", "S0001"),
  )
  assert_eq(
    @web.route_from_path("/tickets/T-1?x=1"),
    PurchaseCompletePage("T-1"),
  )
  assert_eq(@web.route_from_path("/unknown/path"), NotFoundPage)
  assert_eq(
    @web.route_to_path(PurchaseConfirmPage("C001", "S0001")),
    "/courses/C001/sessions/S0001/confirm",
  )
}
