// コース一覧の探索（U2）の更新関数の部分（純粋関数）。frontend-components.md の 4 節。
// 待ち（150 ms）・通信・URL の書き換えは Effect の値として返すだけで、時計やタイマーは使わない（NFR6.2）。
//
// 古い応答の扱い（BR11.2）: 一覧の要求を送るたびに search_request_seq を進める。
// - 条件付きの要求（SearchCourses）は番号を持ち、応答の番号が最新のときだけ反映する。
// - 条件なしの要求は U1 と同じ FetchCourses で送る（U1 の振る舞いを変えないため）。その応答
//   （CourseListLoaded）は番号を持たないので、最後に送った要求が条件なしのときだけ反映する。
//   条件なしの応答はどれも同じ条件の結果なので、最後に送った条件の結果になる（NFR2.2）。

///|
/// 一覧の要求を送る。番号を進め、件数と一覧の部分だけを読み込み中にする（全画面は読み込み直さない）。
/// 待っているキーワードの送信は取り消す（同じ条件を 2 回送らないため）。
fn request_course_list(model : Model) -> (Model, Array[Effect]) {
  let seq = model.search_request_seq + 1
  let unconditioned = model.search_input == default_search_input()
  let next = {
    ..model,
    search_request_seq: seq,
    keyword_debounce: model.keyword_debounce + 1,
    latest_request_unconditioned: unconditioned,
    course_list: Loading,
    condition_errors: None,
  }
  let effect = if unconditioned {
    FetchCourses(model.generation)
  } else {
    SearchCourses(model.generation, seq, model.search_input)
  }
  (next, [effect])
}

///|
/// 探索条件を変えたとき（キーワード以外。BR11.1）。期間の前後が逆なら送らずに案内し、一覧はそのまま
/// （BR11.5）。正しければすぐに送り、履歴を増やさずに URL を書き換える（BR11.3）。
fn apply_search_input(
  model : Model,
  input : SearchInput,
) -> (Model, Array[Effect]) {
  let changed = { ..model, search_input: input, }
  if !(model.route is CourseListPage) {
    return (changed, [])
  }
  if is_period_reversed(input) {
    return ({ ..changed, period_error: Some(period_error_message), }, [])
  }
  let (sent, effects) = request_course_list({ ..changed, period_error: None, })
  (sent, [..effects, ReplaceUrl(list_path(input))])
}

///|
/// キーワードの入力。値を変えて待ちの番号を進め、150 ms の待ちを返す（BR11.1）。
fn on_keyword_changed(
  model : Model,
  keyword : String,
) -> (Model, Array[Effect]) {
  let ticket = model.keyword_debounce + 1
  (
    {
      ..model,
      search_input: { ..model.search_input, keyword, },
      keyword_debounce: ticket,
    },
    [WaitKeyword(ticket)],
  )
}

///|
/// キーワードの待ちが終わったとき。番号が最新でなければ（その後に入力があった・ほかの条件で
/// 送った）何もしない。期間の前後が逆のときも送らない（BR11.5）。
fn on_keyword_settled(model : Model, ticket : Int32) -> (Model, Array[Effect]) {
  guard ticket == model.keyword_debounce &&
    model.route is CourseListPage &&
    !is_period_reversed(model.search_input) else {
    return (model, [])
  }
  let (sent, effects) = request_course_list(model)
  (sent, [..effects, ReplaceUrl(list_path(model.search_input))])
}

///|
/// 探索の応答。番号が最新でない・画面を移った・従業員を切り替えた後の応答は捨てる（BR11.2）。
/// 400 は一覧をそれまでのままにして、不正な条件を案内する（BR11.9）。
fn on_search_loaded(
  model : Model,
  from : Int32,
  seq : Int32,
  result : Result[CourseList, SearchFailure],
) -> (Model, Array[Effect]) {
  guard from == model.generation &&
    seq == model.search_request_seq &&
    model.route is CourseListPage else {
    return (model, [])
  }
  match result {
    Ok(list) =>
      (
        {
          ..model,
          course_list: Loaded(list),
          last_results: Some(list),
          condition_errors: None,
        },
        [],
      )
    Err(InvalidCondition(errors)) => {
      let kept = match model.last_results {
        Some(list) => Loaded(list)
        None => Failed(Http(400, "条件を確かめてください"))
      }
      ({ ..model, course_list: kept, condition_errors: Some(errors), }, [])
    }
    Err(FetchFailed(error)) =>
      ({ ..model, course_list: to_remote(Err(error)), }, [])
  }
}

///|
/// 条件なしの一覧の応答（U1 の CourseListLoaded）。最後に送った要求が条件なしのときだけ反映する。
fn on_course_list_loaded(
  model : Model,
  from : Int32,
  result : Result[CourseList, FetchError],
) -> (Model, Array[Effect]) {
  guard from == model.generation &&
    model.route is CourseListPage &&
    model.latest_request_unconditioned else {
    return (model, [])
  }
  let last_results = match result {
    Ok(list) => Some(list)
    Err(_) => model.last_results
  }
  (
    {
      ..model,
      course_list: to_remote(result),
      last_results,
      condition_errors: None,
    },
    [],
  )
}

///|
/// 「条件をリセット」: すべての条件を既定に戻し、URL のクエリを消して全コースを探索し直す（BR11.6）。
fn on_reset_conditions(model : Model) -> (Model, Array[Effect]) {
  guard model.route is CourseListPage else { return (model, []) }
  let (sent, effects) = request_course_list({
    ..model,
    search_input: default_search_input(),
    period_error: None,
  })
  (sent, [..effects, ReplaceUrl(list_path(default_search_input()))])
}

///|
/// 「再試行」: 今の条件で送り直す（BR11.7）。期間の前後が逆なら送らない。
fn on_retry_search(model : Model) -> (Model, Array[Effect]) {
  if !(model.route is CourseListPage) || is_period_reversed(model.search_input) {
    return (model, [])
  }
  request_course_list(model)
}

///|
/// 一覧の URL が変わったとき（起動・再読み込み・戻る・リンク）。URL から条件を項目ごとに復元し、
/// 不正な項目は無視する（BR11.4）。不正な項目・既定の値・知らない項目があれば、履歴を増やさずに
/// 正しい形の URL へ書き換える。今の一覧と同じ条件なら何もしない（更新関数が自分で書き換えた URL）。
/// 戻り値は (条件を入れた状態, 追加の作用) で、一覧に入るなら Some。
fn restore_list_from_url(
  model : Model,
  location : String,
  is_starting : Bool,
) -> (Model, Array[Effect])? {
  let input = restore_search_input(query_of(location))
  if !is_starting &&
    model.route is CourseListPage &&
    input == model.search_input {
    return None
  }
  let canonical = list_path(input)
  let rewrite = if canonical != without_fragment(location) {
    [ReplaceUrl(canonical)]
  } else {
    []
  }
  Some(({ ..model, search_input: input, period_error: None, }, rewrite))
}

///|
/// URL から `#` 以降を取り除く。
fn without_fragment(location : String) -> String {
  let out = StringBuilder()
  for c in location {
    if c == '#' {
      break
    }
    out.write_char(c)
  }
  out.to_string()
}
