// 画面の探索（U2）の更新関数のテスト。Web API の応答（成功・0 件・失敗・400）と待ち終わりは
// メッセージとして直接与える。ブラウザ・Web API・実際の時間待ちは使わない。
// 実行（U2 だけ）: リポジトリのルートで `moon -C web test --target js search_test.mbt`

///|
fn search_row(id : String, title : String) -> @web.CourseRow {
  {
    course_id: id,
    title,
    category: "technology",
    summary: "概要",
    fee_points: 25000,
    delivery_format: "online",
    next_session_starts_at: Some("2026-11-09T13:00:00+09:00"),
    open_session_count: 1,
  }
}

///|
fn search_list(rows : Array[@web.CourseRow]) -> @web.CourseList {
  { courses: rows, count: rows.length(), }
}

///|
fn search_me() -> @web.Me {
  {
    employee_id: Some("E001"),
    display_name: Some("高橋 遥"),
    company_name: Some("株式会社サンプル商事"),
    contract_status: Some("active"),
    eligibility: Eligible,
    balance: 60000,
  }
}

///|
fn search_run(model : @web.Model, messages : Array[@web.Msg]) -> @web.Model {
  let mut current = model
  for msg in messages {
    current = @web.update(current, msg).0
  }
  current
}

///|
/// 条件なしでコース一覧を開き、2 件を読み込んだ状態（要求の番号は 1）。
fn on_list() -> @web.Model {
  search_run(@web.init_model(), [
    UrlChanged("/"),
    MeLoaded(0, Ok(search_me())),
    CourseListLoaded(
      0,
      Ok(search_list([search_row("C001", "入門"), search_row("C007", "AI")])),
    ),
  ])
}

///|
fn input_with(
  keyword~ : String = "",
  category~ : String = "",
  available_only~ : Bool = false,
  period_from~ : String = "",
  period_to~ : String = "",
  sort~ : @web.SortOrder = ByDate,
) -> @web.SearchInput {
  { keyword, category, available_only, period_from, period_to, sort, }
}

///|
test "キーワードは入力ごとに待ち直し、最新の待ちが終わったときだけ送る（BR11.1）" {
  assert_eq(@web.keyword_debounce_ms, 150)
  let (typing, effects) = @web.update(on_list(), KeywordChanged("A"))
  let first = typing.keyword_debounce
  assert_eq(effects, [WaitKeyword(first)])
  assert_eq(typing.course_list, on_list().course_list)
  let (typed, effects) = @web.update(typing, KeywordChanged("AI 入門"))
  assert_eq(effects, [WaitKeyword(first + 1)])
  // 古い待ちの番号では送らない
  let (unchanged, effects) = @web.update(typed, KeywordSettled(first))
  assert_eq(effects, [])
  assert_eq(unchanged, typed)
  // 最新の待ちが終わると送り、URL を履歴を増やさずに書き換える
  let (sent, effects) = @web.update(typed, KeywordSettled(first + 1))
  assert_eq(sent.course_list, Loading)
  assert_eq(effects, [
    SearchCourses(0, sent.search_request_seq, input_with(keyword="AI 入門")),
    ReplaceUrl("/?q=AI%20%E5%85%A5%E9%96%80"),
  ])
  // 入力中の値は打ったとおりに保つ
  assert_eq(sent.search_input.keyword, "AI 入門")
}

///|
test "最後に送った要求の応答だけを反映し、古い応答は捨てる（BR11.2、NFR2.2）" {
  let (first, _) = @web.update(on_list(), CategoryChanged("technology"))
  let old_seq = first.search_request_seq
  let (second, _) = @web.update(first, SortChanged("fee-asc"))
  let new_seq = second.search_request_seq
  assert_true(new_seq > old_seq)
  let newest = search_list([search_row("C004", "Rust")])
  let applied = search_run(second, [SearchLoaded(0, new_seq, Ok(newest))])
  assert_eq(applied.course_list, Loaded(newest))
  // 古い番号の応答が後から届いても捨てる
  let late = search_run(applied, [SearchLoaded(0, old_seq, Ok(search_list([])))])
  assert_eq(late, applied)
  // 条件付きの要求の後に届いた、条件なしの古い応答も捨てる
  let stale_plain = search_run(second, [
    CourseListLoaded(0, Ok(search_list([]))),
  ])
  assert_eq(stale_plain, second)
}

///|
test "カテゴリ・空席ありのみ・並べ替えは変えたらすぐ送る。決められた値以外は無視する（BR11.1、BR11.10）" {
  let (category, effects) = @web.update(
    on_list(),
    CategoryChanged("technology"),
  )
  assert_eq(effects, [
    SearchCourses(
      0,
      category.search_request_seq,
      input_with(category="technology"),
    ),
    ReplaceUrl("/?category=technology"),
  ])
  let (available, effects) = @web.update(category, AvailableOnlyChanged(true))
  assert_eq(effects, [
    SearchCourses(
      0,
      available.search_request_seq,
      input_with(category="technology", available_only=true),
    ),
    ReplaceUrl("/?category=technology&available=true"),
  ])
  // 「すべて」はカテゴリの条件を外す（AC1.3.2）
  let (all, _) = @web.update(available, CategoryChanged(""))
  assert_eq(all.search_input.category, "")
  // 決められた値以外は無視する
  let (ignored, effects) = @web.update(on_list(), CategoryChanged("cooking"))
  assert_eq(effects, [])
  assert_eq(ignored, on_list())
  let (ignored_sort, effects) = @web.update(on_list(), SortChanged("random"))
  assert_eq(effects, [])
  assert_eq(ignored_sort, on_list())
}

///|
test "URL の書き出しは既定の値を書かず、キーワードを UTF-8 でエンコードする（BR11.3）" {
  assert_eq(@web.list_path(@web.default_search_input()), "/")
  assert_eq(@web.list_path(input_with(available_only=false, sort=ByDate)), "/")
  assert_eq(
    @web.list_path(
      input_with(
        keyword="AI 入門&x=1",
        category="technology",
        available_only=true,
        period_from="2026-11-10",
        period_to="2026-11-15",
        sort=ByFeeAsc,
      ),
    ),
    "/?q=AI%20%E5%85%A5%E9%96%80%26x%3D1&category=technology&available=true&from=2026-11-10&to=2026-11-15&sort=fee-asc",
  )
}

///|
test "URL から条件を項目ごとに復元し、不正な項目だけを無視して URL から取り除く（BR11.4、NFR3.5）" {
  let (model, effects) = @web.update(
    @web.init_model(),
    UrlChanged(
      "/?q=%E3%83%9E%E3%83%8D%E3%82%B8%E3%83%A1%E3%83%B3%E3%83%88&category=cooking&available=true&from=2026-02-30&to=2026-11-15&sort=price&utm=x",
    ),
  )
  let expected = input_with(
    keyword="マネジメント",
    available_only=true,
    period_to="2026-11-15",
  )
  assert_eq(model.route, CourseListPage)
  assert_eq(model.search_input, expected)
  assert_eq(effects, [
    FetchMe(0),
    SearchCourses(0, model.search_request_seq, expected),
    ReplaceUrl(
      "/?q=%E3%83%9E%E3%83%8D%E3%82%B8%E3%83%A1%E3%83%B3%E3%83%88&available=true&to=2026-11-15",
    ),
  ])
  // 書き換えた URL の変化（更新関数が自分で書き換えたもの）では何もしない
  let (_, echoed) = @web.update(
    model,
    UrlChanged(@web.list_path(model.search_input)),
  )
  assert_eq(echoed, [])
}

///|
test "URL の期間の前後が逆なら期間を無視し、崩れたエンコードでもエラーにしない（BR11.4）" {
  assert_eq(
    @web.restore_search_input("from=2026-11-10&to=2026-11-09&sort=name"),
    input_with(sort=ByName),
  )
  // `+` は空白、崩れた `%` はそのまま、正しくない UTF-8 は置き換え文字
  assert_eq(@web.restore_search_input("q=AI+%E5%85%A5").keyword, "AI 入")
  assert_eq(@web.percent_decode("AI+100%"), "AI 100%")
  assert_eq(@web.percent_decode("%zz%41"), "%zzA")
  assert_eq(@web.percent_decode("%E5%85%A5%E9%96%80"), "入門")
  assert_eq(@web.percent_decode("%FF"), "\u{FFFD}")
  // 同じ名前が複数あれば先頭を使う
  assert_eq(@web.restore_search_input("q=a&q=b").keyword, "a")
  assert_true(@web.is_calendar_date("2028-02-29"))
  assert_false(@web.is_calendar_date("2026-02-29"))
  assert_false(@web.is_calendar_date("2026-1-05"))
}

///|
test "期間の前後が逆なら送らずに案内し、一覧はそのまま。直すと案内を消して送る（BR11.5、NFR4.3）" {
  let list = on_list()
  let (reversed, effects) = @web.update(
    list,
    PeriodChanged("2026-11-10", "2026-11-09"),
  )
  assert_eq(effects, [])
  assert_eq(reversed.course_list, list.course_list)
  assert_eq(reversed.period_error, Some(@web.period_error_message))
  // 前後が逆の間はキーワードの待ちが終わっても送らない
  let (typed, _) = @web.update(reversed, KeywordChanged("AI"))
  let (_, effects) = @web.update(typed, KeywordSettled(typed.keyword_debounce))
  assert_eq(effects, [])
  // 直すと案内を消して送る
  let (fixed, effects) = @web.update(
    typed,
    PeriodChanged("2026-11-10", "2026-11-10"),
  )
  assert_eq(fixed.period_error, None)
  assert_eq(effects, [
    SearchCourses(
      0,
      fixed.search_request_seq,
      input_with(keyword="AI", period_from="2026-11-10", period_to="2026-11-10"),
    ),
    ReplaceUrl("/?q=AI&from=2026-11-10&to=2026-11-10"),
  ])
}

///|
test "0 件は Loaded の 0 件になり、条件をリセットで既定に戻して全コースを探索し直す（BR11.6、NFR4.2）" {
  let (searching, _) = @web.update(on_list(), CategoryChanged("compliance"))
  let empty = search_list([])
  let none = search_run(searching, [
    SearchLoaded(0, searching.search_request_seq, Ok(empty)),
  ])
  assert_eq(none.course_list, Loaded(empty))
  let (reset, effects) = @web.update(none, ResetConditions)
  assert_eq(reset.search_input, @web.default_search_input())
  assert_eq(reset.period_error, None)
  assert_eq(reset.course_list, Loading)
  assert_eq(effects, [FetchCourses(0), ReplaceUrl("/")])
  // リセット後の条件なしの応答は反映する
  let all = search_list([search_row("C001", "入門")])
  let shown = search_run(reset, [CourseListLoaded(0, Ok(all))])
  assert_eq(shown.course_list, Loaded(all))
}

///|
test "読み込みに失敗したら失敗にし、再試行は今の条件で送り直す（BR11.7、NFR2.3）" {
  let (searching, _) = @web.update(on_list(), KeywordChanged("AI"))
  let (sent, _) = @web.update(
    searching,
    KeywordSettled(searching.keyword_debounce),
  )
  let failed = search_run(sent, [
    SearchLoaded(
      0,
      sent.search_request_seq,
      Err(FetchFailed(Network("接続できません"))),
    ),
  ])
  assert_eq(failed.course_list, Failed(Network("接続できません")))
  let (retried, effects) = @web.update(failed, RetrySearch)
  assert_eq(retried.course_list, Loading)
  assert_eq(effects, [
    SearchCourses(0, retried.search_request_seq, input_with(keyword="AI")),
  ])
  assert_true(retried.search_request_seq > sent.search_request_seq)
}

///|
test "400 では一覧をそれまでのままにして、不正な条件を案内する（BR11.9）" {
  let body =
    #|{"error":{"code":"invalid-search-condition","message":"探索条件が正しくありません","fields":[{"field":"category","reason":"存在しないカテゴリです"}]}}
  let errors : Array[@web.ConditionFieldError] = [
    { field: "category", reason: "存在しないカテゴリです", },
  ]
  assert_eq(
    @web.decode_search_response({ status: 400, body, }),
    Err(InvalidCondition(errors)),
  )
  // 400 でも形が違う本文は通常の失敗にする
  assert_true(
    @web.decode_search_response({ status: 400, body: "x", })
    is Err(FetchFailed(Http(400, _))),
  )
  let list = on_list()
  let (searching, _) = @web.update(list, CategoryChanged("technology"))
  let shown = search_run(searching, [
    SearchLoaded(0, searching.search_request_seq, Err(InvalidCondition(errors))),
  ])
  assert_eq(shown.course_list, list.course_list)
  assert_eq(shown.condition_errors, Some(errors))
  assert_eq(@web.condition_field_label("period"), "期間")
}

///|
test "詳細から一覧へ戻ると、URL の条件を復元して同じ条件で探索する（BR11.8、AC1.8.4）" {
  let (filtered, _) = @web.update(on_list(), CategoryChanged("technology"))
  let (detail, effects) = @web.update(filtered, UrlChanged("/courses/C004"))
  assert_eq(detail.route, CourseDetailPage("C004"))
  assert_eq(effects, [FetchCourseDetail(0, "C004")])
  // 詳細にいる間も条件を保つ（「コース一覧へ戻る」はこの条件の URL を指す）
  assert_eq(@web.list_path(detail.search_input), "/?category=technology")
  let (back, effects) = @web.update(detail, UrlChanged("/?category=technology"))
  assert_eq(back.route, CourseListPage)
  assert_eq(back.search_input, input_with(category="technology"))
  assert_eq(effects, [
    SearchCourses(0, back.search_request_seq, input_with(category="technology")),
  ])
}

///|
test "従業員を切り替えると探索条件も既定に戻し、前の従業員の探索の応答は捨てる（BR6.4）" {
  let (filtered, _) = @web.update(on_list(), CategoryChanged("technology"))
  let old_seq = filtered.search_request_seq
  let opened = search_run(filtered, [SwitcherOpened, EmployeeChosen("E002")])
  let (switched, effects) = @web.update(
    opened,
    EmployeeSwitched(Ok({ ..search_me(), employee_id: Some("E002"), })),
  )
  assert_eq(switched.search_input, @web.default_search_input())
  assert_eq(effects, [PushUrl("/"), FetchCourses(1)])
  let stale = search_run(switched, [
    SearchLoaded(0, old_seq, Ok(search_list([]))),
    SearchLoaded(1, old_seq, Ok(search_list([]))),
  ])
  assert_eq(stale, switched)
}
