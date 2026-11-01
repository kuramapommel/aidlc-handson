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
    // 条件なしの一覧。応答に要求の番号を結び付けて返す（BR11.2）
    FetchCourses(generation, seq) =>
      request_cmd(Get, api_base + "/api/courses", None, result => {
        emit(
          CourseListLoaded(
            generation,
            seq,
            fetch_result(result, decode_course_list),
          ),
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
    // 探索の要求（U2）。応答に要求の番号を結び付けて返す（BR11.2）
    SearchCourses(generation, seq, input) =>
      request_cmd(
        Get,
        api_base + "/api/courses" + search_query_string(input),
        None,
        result => emit(SearchLoaded(generation, seq, search_result(result))),
      )
    // キーワードの待ち（rabbita の @cmd.delay。更新関数はタイマーを使わない。NFR6.2）
    WaitKeyword(ticket) =>
      @cmd.delay(emit(KeywordSettled(ticket)), keyword_debounce_ms)
  }
}

///|
/// rabbita の URL を、画面内の URL の文字列（`/パス?クエリ`）にする。
/// コース一覧の探索条件はクエリにあるため、クエリも残す（U2 の BR11.4）。
pub fn location_of(url : @url.Url) -> String {
  match url.query {
    Some(query) if query != "" => "/" + url.path + "?" + query
    _ => "/" + url.path
  }
}

///|
/// URL の変化と画面内のリンクの押下を受け取る（起動時の URL も UrlChanged で届く）。
/// `@nav.replace_url`・`@nav.push_url` の後にも UrlChanged が届く（rabbita の動き）。
fn subscriptions(emit : @cmd.Emit[Msg]) -> @sub.Sub {
  @sub.batch([
    @sub.on_url_changed(
      emit.map((url : @url.Url) => UrlChanged(location_of(url))),
    ),
    @sub.on_url_request(
      emit.map((request : @url.UrlRequest) => {
        match request {
          Internal(url) => LinkClicked(location_of(url))
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
