// 更新関数（メッセージ＋状態 → 新しい状態＋作用）。純粋関数で、通信や URL の書き換えは
// Effect の値として返すだけにする（frontend-components.md の 4 節）。
// 古い応答の扱い: 応答のメッセージに要求したときの従業員の世代と対象の ID を持たせ、
// 今の世代・今の画面と合わないものは捨てる。

///|
/// 起動直後の状態（まだ何も取得していない）。最初の UrlChanged で読み込みを始める。
pub fn init_model() -> Model {
  {
    route: CourseListPage,
    generation: 0,
    me: NotAsked,
    switcher: closed_switcher(),
    course_list: NotAsked,
    course_detail: NotAsked,
    selected_session_id: None,
    notice: None,
    purchase: Idle,
    ticket: NotAsked,
    search_input: default_search_input(),
    period_error: None,
    keyword_debounce: 0,
    search_request_seq: 0,
    last_results: None,
    condition_errors: None,
  }
}

///|
fn closed_switcher() -> Switcher {
  { is_open: false, employees: NotAsked, is_switching: false, error: None, }
}

///|
/// 購入確認で確定できるかどうか（描画のたびに計算する。functional-spec.md 5.3 の Ready／Blocked）。
pub(all) enum Readiness {
  /// 必要なデータがまだそろっていない
  Waiting
  Ready(ConfirmSummary)
  Blocked(ConfirmSummary, BlockReason)
} derive(Eq, Debug)

///|
/// 購入確認に表示する内容。
pub(all) struct ConfirmSummary {
  course : Course
  session : Session
  balance : Int32
} derive(Eq, Debug)

///|
/// 確定できない理由。
pub(all) enum BlockReason {
  /// 利用資格がない（BR1.4）
  NotEligible
  /// 日程が購入可でない
  Unavailable(SessionStatus)
  /// 残高が足りない（不足額。BR4.3）
  InsufficientBalance(Int32)
} derive(Eq, Debug)

///|
/// 購入確認で確定できるかを、操作中の従業員・コースの詳細・選んだ日程から求める（BR1.4、BR4.2〜BR4.4）。
pub fn confirm_readiness(model : Model) -> Readiness {
  guard model.route is PurchaseConfirmPage(_, session_id) else {
    return Waiting
  }
  guard model.me is Loaded(me) && model.course_detail is Loaded(detail) else {
    return Waiting
  }
  guard find_session(detail, session_id) is Some(session) else {
    return Waiting
  }
  let summary = { course: detail.course, session, balance: me.balance, }
  if me.eligibility != Eligible {
    Blocked(summary, NotEligible)
  } else if session.status != Available {
    Blocked(summary, Unavailable(session.status))
  } else if me.balance < detail.course.fee_points {
    Blocked(summary, InsufficientBalance(detail.course.fee_points - me.balance))
  } else {
    Ready(summary)
  }
}

///|
/// コースの詳細から日程を探す。
pub fn find_session(detail : CourseDetail, session_id : String) -> Session? {
  for session in detail.sessions {
    if session.session_id == session_id {
      return Some(session)
    }
  }
  None
}

///|
fn is_session_available(
  detail : Remote[CourseDetail],
  session_id : String,
) -> Bool {
  match detail {
    Loaded(detail) =>
      find_session(detail, session_id) is Some(session) &&
      session.status == Available
    _ => false
  }
}

///|
/// 画面が対象にしているコースの ID（コースの詳細と購入確認）。
fn route_course_id(route : Route) -> String? {
  match route {
    CourseDetailPage(course_id) | PurchaseConfirmPage(course_id, _) =>
      Some(course_id)
    _ => None
  }
}

///|
fn[T] to_remote(result : Result[T, FetchError]) -> Remote[T] {
  match result {
    Ok(value) => Loaded(value)
    Err(NotFound) => Missing
    Err(error) => Failed(error)
  }
}

///|
/// 更新関数。
pub fn update(model : Model, msg : Msg) -> (Model, Array[Effect]) {
  let generation = model.generation
  match msg {
    UrlChanged(path) => on_url_changed(model, path)
    LinkClicked(path) => (model, [PushUrl(path)])
    ExternalLinkClicked(href) => (model, [LoadUrl(href)])
    MeLoaded(from, result) =>
      if from == generation {
        ({ ..model, me: to_remote(result), }, [])
      } else {
        (model, [])
      }
    RetryMe => ({ ..model, me: Loading, }, [FetchMe(generation)])
    CourseListLoaded(from, seq, result) =>
      on_course_list_loaded(model, from, seq, result)
    // 一覧の「再試行」は今の探索条件で送り直す（条件なしなら U1 と同じ要求。BR11.7）
    RetryCourseList | RetrySearch => on_retry_search(model)
    CourseDetailLoaded(from, course_id, result) =>
      on_course_detail_loaded(model, from, course_id, result)
    RetryCourseDetail =>
      match route_course_id(model.route) {
        Some(course_id) =>
          (
            { ..model, course_detail: Loading, },
            [FetchCourseDetail(generation, course_id)],
          )
        None => (model, [])
      }
    SessionSelected(session_id) => on_session_selected(model, session_id)
    ProceedToConfirm => on_proceed_to_confirm(model)
    BackToDetail =>
      match (model.route, model.purchase) {
        (PurchaseConfirmPage(_, _), Submitting) => (model, [])
        (PurchaseConfirmPage(course_id, _), _) =>
          (
            { ..model, purchase: Idle, },
            [PushUrl(route_to_path(CourseDetailPage(course_id)))],
          )
        _ => (model, [])
      }
    ConfirmClicked =>
      match (model.purchase, confirm_readiness(model), model.route) {
        (Idle, Ready(_), PurchaseConfirmPage(_, session_id)) =>
          (
            { ..model, purchase: Submitting, },
            [SubmitPurchase(generation, session_id)],
          )
        _ => (model, [])
      }
    PurchaseSucceeded(from, ticket, balance_after) =>
      on_purchase_succeeded(model, from, ticket, balance_after)
    PurchaseRejected(from, reason) =>
      settle_purchase(model, from, Rejected(reason))
    PurchaseFailed(from, message) =>
      settle_purchase(model, from, RequestFailed(message))
    PurchaseNetworkFailed(from) => settle_purchase(model, from, NetworkFailed)
    RetryPurchase =>
      match (model.purchase, model.route) {
        (NetworkFailed | RequestFailed(_), PurchaseConfirmPage(_, session_id)) =>
          (
            { ..model, purchase: Submitting, },
            [SubmitPurchase(generation, session_id)],
          )
        _ => (model, [])
      }
    ChooseAnotherSession => on_choose_another_session(model)
    TicketLoaded(from, ticket_id, result) =>
      if from == generation && model.route == PurchaseCompletePage(ticket_id) {
        ({ ..model, ticket: to_remote(result), }, [])
      } else {
        (model, [])
      }
    RetryTicket =>
      match model.route {
        PurchaseCompletePage(ticket_id) =>
          ({ ..model, ticket: Loading, }, [FetchTicket(generation, ticket_id)])
        _ => (model, [])
      }
    SwitcherOpened =>
      (
        {
          ..model,
          switcher: {
            ..model.switcher,
            is_open: true,
            employees: Loading,
            error: None,
          },
        },
        [FetchEmployees],
      )
    SwitcherClosed =>
      if model.switcher.is_switching {
        (model, [])
      } else {
        ({ ..model, switcher: closed_switcher(), }, [])
      }
    EmployeesLoaded(result) =>
      if model.switcher.is_open {
        (
          {
            ..model,
            switcher: { ..model.switcher, employees: to_remote(result), },
          },
          [],
        )
      } else {
        (model, [])
      }
    EmployeeChosen(employee_id) =>
      if model.switcher.is_switching {
        (model, [])
      } else {
        (
          {
            ..model,
            switcher: { ..model.switcher, is_switching: true, error: None, },
          },
          [SwitchEmployee(employee_id)],
        )
      }
    EmployeeSwitched(result) => on_employee_switched(model, result)
    // ---- U2: コース一覧の探索（update_search.mbt） ----
    KeywordChanged(keyword) => on_keyword_changed(model, keyword)
    KeywordSettled(ticket) => on_keyword_settled(model, ticket)
    CategoryChanged(value) =>
      if value == "" || is_category_value(value) {
        apply_search_input(model, { ..model.search_input, category: value, })
      } else {
        (model, [])
      }
    AvailableOnlyChanged(checked) =>
      apply_search_input(model, {
        ..model.search_input,
        available_only: checked,
      })
    PeriodChanged(from, to) => {
      // 日付の入力欄は空か正しい日付だけを返す。それ以外の値は指定なしとして扱う
      let date_or_empty = (text : String) => {
        if is_calendar_date(text) {
          text
        } else {
          ""
        }
      }
      apply_search_input(model, {
        ..model.search_input,
        period_from: date_or_empty(from),
        period_to: date_or_empty(to),
      })
    }
    SortChanged(value) =>
      match sort_from_value(value) {
        Some(sort) => apply_search_input(model, { ..model.search_input, sort, })
        None => (model, [])
      }
    SearchLoaded(from, seq, result) =>
      on_search_loaded(model, from, seq, result)
    ResetConditions => on_reset_conditions(model)
  }
}

///|
/// URL が変わったとき。最初の 1 回（me が未取得）は起動（AppStarted）として、操作中の従業員も取得する。
/// 同じ画面への URL の変化（更新関数が自分で移動と取得を済ませたとき）は何もしない。
/// コース一覧の URL（`/?q=…`）では、クエリから探索条件を復元する（U2 の BR11.4・BR11.8）。
fn on_url_changed(model : Model, path : String) -> (Model, Array[Effect]) {
  let route = route_from_path(path)
  let is_starting = model.me is NotAsked
  let (base, rewrite) = if route is CourseListPage {
    match restore_list_from_url(model, path, is_starting) {
      Some(restored) => restored
      None => return (model, [])
    }
  } else if !is_starting && route == model.route {
    return (model, [])
  } else {
    (model, [])
  }
  let (entered, effects) = enter_route({ ..base, route, }, route)
  if is_starting {
    (
      { ..entered, me: Loading, },
      [FetchMe(model.generation), ..effects, ..rewrite],
    )
  } else {
    (entered, [..effects, ..rewrite])
  }
}

///|
/// 画面に入ったときに、その画面のデータを読み込み中にして取得する（frontend-components.md の 3 節）。
fn enter_route(model : Model, route : Route) -> (Model, Array[Effect]) {
  let generation = model.generation
  match route {
    // 今の探索条件で一覧を取得する（条件なしなら U1 と同じ要求の FetchCourses。どちらも番号付き）
    CourseListPage =>
      request_course_list({ ..model, purchase: Idle, notice: None, })
    CourseDetailPage(course_id) =>
      (
        { ..model, course_detail: Loading, purchase: Idle, notice: None, },
        [FetchCourseDetail(generation, course_id)],
      )
    PurchaseConfirmPage(course_id, session_id) =>
      (
        {
          ..model,
          course_detail: Loading,
          selected_session_id: Some(session_id),
          purchase: Idle,
          notice: None,
        },
        [FetchCourseDetail(generation, course_id)],
      )
    PurchaseCompletePage(ticket_id) =>
      match model.purchase {
        Succeeded(ticket) if ticket.ticket_id == ticket_id =>
          ({ ..model, ticket: Loaded(ticket), }, [])
        _ =>
          ({ ..model, ticket: Loading, }, [FetchTicket(generation, ticket_id)])
      }
    NotFoundPage => (model, [])
  }
}

///|
/// コースの詳細が届いたとき。購入確認を開いていて、選んだ日程がもう購入可でなければ、
/// 詳細へ戻して理由を案内する（frontend-components.md の 3 節）。
fn on_course_detail_loaded(
  model : Model,
  from : Int32,
  course_id : String,
  result : Result[CourseDetail, FetchError],
) -> (Model, Array[Effect]) {
  guard from == model.generation &&
    route_course_id(model.route) == Some(course_id) else {
    return (model, [])
  }
  guard result is Ok(detail) else {
    return ({ ..model, course_detail: to_remote(result), }, [])
  }
  let loaded = Loaded(detail)
  match model.route {
    PurchaseConfirmPage(_, session_id) =>
      if is_session_available(loaded, session_id) {
        ({ ..model, course_detail: loaded, }, [])
      } else {
        let detail_route = CourseDetailPage(course_id)
        (
          {
            ..model,
            route: detail_route,
            course_detail: loaded,
            selected_session_id: None,
            notice: Some(unavailable_notice(find_session(detail, session_id))),
            purchase: Idle,
          },
          [ReplaceUrl(route_to_path(detail_route))],
        )
      }
    _ => {
      let selected = match model.selected_session_id {
        Some(id) if is_session_available(loaded, id) => Some(id)
        _ => None
      }
      ({ ..model, course_detail: loaded, selected_session_id: selected, }, [])
    }
  }
}

///|
/// 選んだ日程が購入できなくなっていたときの案内。
pub fn unavailable_notice(session : Session?) -> String {
  match session {
    Some(session) =>
      "選んだ日程は「\{session_status_label(session.status)}」のため購入できません。日程を選び直してください。"
    None =>
      "選んだ日程は見つかりません。日程を選び直してください。"
  }
}

///|
/// 日程を選ぶ。購入可の日程のときだけ選べる（BR2.5）。
fn on_session_selected(
  model : Model,
  session_id : String,
) -> (Model, Array[Effect]) {
  if model.route is CourseDetailPage(_) &&
    is_session_available(model.course_detail, session_id) {
    ({ ..model, selected_session_id: Some(session_id), notice: None, }, [])
  } else {
    (model, [])
  }
}

///|
/// 「購入確認へ進む」。購入可の日程を選んでいるときだけ進める（BR7.2）。
fn on_proceed_to_confirm(model : Model) -> (Model, Array[Effect]) {
  match (model.route, model.selected_session_id) {
    (CourseDetailPage(course_id), Some(session_id)) if is_session_available(
        model.course_detail,
        session_id,
      ) =>
      (
        model,
        [PushUrl(route_to_path(PurchaseConfirmPage(course_id, session_id)))],
      )
    _ => (model, [])
  }
}

///|
/// 購入が承認されたとき。ヘッダーの残高を購入後の値にし（BR7.7）、完了画面へ移る。
fn on_purchase_succeeded(
  model : Model,
  from : Int32,
  ticket : Ticket,
  balance_after : Int32,
) -> (Model, Array[Effect]) {
  guard from == model.generation && model.purchase is Submitting else {
    return (model, [])
  }
  let me = match model.me {
    Loaded(me) => Loaded({ ..me, balance: balance_after, })
    other => other
  }
  let next = {
    ..model,
    me,
    purchase: Succeeded(ticket),
    ticket: Loaded(ticket),
  }
  if model.route is PurchaseConfirmPage(_, _) {
    (next, [PushUrl(route_to_path(PurchaseCompletePage(ticket.ticket_id)))])
  } else {
    (next, [])
  }
}

///|
/// 購入の拒否・失敗を反映する。残高は変えない（BR7.4）。
fn settle_purchase(
  model : Model,
  from : Int32,
  outcome : PurchaseState,
) -> (Model, Array[Effect]) {
  if from == model.generation && model.purchase is Submitting {
    ({ ..model, purchase: outcome, }, [])
  } else {
    (model, [])
  }
}

///|
/// 「別の日程を選ぶ」。同じコースの詳細へ移り、最新の残席を取得し直す（BR7.5）。
fn on_choose_another_session(model : Model) -> (Model, Array[Effect]) {
  guard route_course_id(model.route) is Some(course_id) else {
    return (model, [])
  }
  let detail_route = CourseDetailPage(course_id)
  (
    {
      ..model,
      route: detail_route,
      course_detail: Loading,
      selected_session_id: None,
      purchase: Idle,
      notice: None,
    },
    [
      PushUrl(route_to_path(detail_route)),
      FetchCourseDetail(model.generation, course_id),
    ],
  )
}

///|
/// 従業員の切り替えの結果。成功なら前の従業員の情報を捨て、どの画面からでもコース一覧へ戻る（BR6.4）。
/// 探索条件も既定に戻す（前の従業員の操作を残さない。U2）。失敗なら操作中の従業員は変えず、失敗を案内する。
fn on_employee_switched(
  model : Model,
  result : Result[Me, FetchError],
) -> (Model, Array[Effect]) {
  match result {
    Ok(me) => {
      let generation = model.generation + 1
      // 待っているキーワードの送信と、前の従業員の一覧・探索の応答を無効にする
      let seq = model.search_request_seq + 1
      (
        {
          route: CourseListPage,
          generation,
          me: Loaded(me),
          switcher: closed_switcher(),
          course_list: Loading,
          course_detail: NotAsked,
          selected_session_id: None,
          notice: None,
          purchase: Idle,
          ticket: NotAsked,
          search_input: default_search_input(),
          period_error: None,
          keyword_debounce: model.keyword_debounce + 1,
          search_request_seq: seq,
          last_results: None,
          condition_errors: None,
        },
        [PushUrl(route_to_path(CourseListPage)), FetchCourses(generation, seq)],
      )
    }
    Err(error) =>
      (
        {
          ..model,
          switcher: {
            ..model.switcher,
            is_switching: false,
            error: Some(
              "従業員を切り替えられませんでした（\{fetch_error_text(error)}）",
            ),
          },
        },
        [],
      )
  }
}

///|
/// 読み込みの失敗の説明（画面に文字で出す）。
pub fn fetch_error_text(error : FetchError) -> String {
  match error {
    NotFound => "見つかりません"
    Http(_, message) => message
    Network(_) => "通信に失敗しました"
    Decode(_) => "応答を読み取れませんでした"
  }
}
