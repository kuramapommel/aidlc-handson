// 画面（TEA）の組み立て。TEA の実行環境・描画・URL の扱いは rabbita を使い、
// 更新関数（update.mbt）が返す作用（Effect）を rabbita のコマンドに変える。
// 通信は http.mbt（mizchi/js_web の fetch）、応答の読み取りは decode.mbt の純粋関数で行う。

///|
/// Web API の既定のオリジン（Web API の既定のポート 3000。README を参照）。
pub let default_api_base : String = "http://localhost:3000"

///|
/// 作用を rabbita のコマンドにする。応答はメッセージにして更新関数へ戻す。
pub fn effect_to_cmd(
  effect : Effect,
  api_base : String,
  emit : @cmd.Emit[Msg],
) -> @cmd.Cmd {
  match effect {
    FetchMe(generation) =>
      request_cmd(Get, api_base + "/api/me", None, result => {
        emit(MeLoaded(generation, fetch_result(result, decode_me)))
      })
    FetchCourses(generation) =>
      request_cmd(Get, api_base + "/api/courses", None, result => {
        emit(
          CourseListLoaded(generation, fetch_result(result, decode_course_list)),
        )
      })
    FetchCourseDetail(generation, course_id) =>
      request_cmd(Get, api_base + "/api/courses/\{course_id}", None, result => {
        emit(
          CourseDetailLoaded(
            generation,
            course_id,
            fetch_result(result, decode_course_detail),
          ),
        )
      })
    FetchTicket(generation, ticket_id) =>
      request_cmd(Get, api_base + "/api/tickets/\{ticket_id}", None, result => {
        emit(
          TicketLoaded(
            generation,
            ticket_id,
            fetch_result(result, decode_ticket),
          ),
        )
      })
    FetchEmployees =>
      request_cmd(Get, api_base + "/api/employees", None, result => {
        emit(EmployeesLoaded(fetch_result(result, decode_employees)))
      })
    SwitchEmployee(employee_id) => {
      let body : Json = { "employeeId": Json::string(employee_id) }
      request_cmd(
        Post,
        api_base + "/api/session/employee",
        Some(body.stringify()),
        result => emit(EmployeeSwitched(fetch_result(result, decode_me))),
      )
    }
    SubmitPurchase(generation, session_id) => {
      // 送るのは日程 ID だけ。金額は送らない（BR5.2）
      let body : Json = { "sessionId": Json::string(session_id) }
      request_cmd(Post, api_base + "/api/purchases", Some(body.stringify()), result => {
        match result {
          Ok(response) => emit(purchase_message(generation, response))
          Err(_) => emit(PurchaseNetworkFailed(generation))
        }
      })
    }
    PushUrl(path) => @nav.push_url(path)
    ReplaceUrl(path) => @nav.replace_url(path)
    LoadUrl(url) => @nav.load(url)
  }
}

///|
/// URL の変化と画面内のリンクの押下を受け取る（起動時の URL も UrlChanged で届く）。
fn subscriptions(emit : @cmd.Emit[Msg]) -> @sub.Sub {
  @sub.batch([
    @sub.on_url_changed(
      emit.map((url : @url.Url) => UrlChanged("/" + url.path)),
    ),
    @sub.on_url_request(
      emit.map((request : @url.UrlRequest) => {
        match request {
          Internal(url) => LinkClicked("/" + url.path)
          External(href) => ExternalLinkClicked(href)
        }
      }),
    ),
  ])
}

///|
/// 画面のアプリを作る。`api_base` は Web API のオリジン（例: `http://localhost:3000`）。
pub fn app(api_base~ : String) -> @rabbita.App {
  @rabbita.elmish(
    model=init_model(),
    update=(model, msg, emit) => {
      let (next, effects) = update(model, msg)
      (next, @rabbita.batch(effects.map(e => effect_to_cmd(e, api_base, emit))))
    },
    view~,
    subscriptions=(_, emit) => subscriptions(emit),
  )
}
