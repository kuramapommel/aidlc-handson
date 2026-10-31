// ヘッダー（操作中の従業員名・企業名・残高。BR7.11）と従業員の切り替え（BR6.1、BR6.4）の描画。

///|
fn view_header(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  @html.header(class="site-header", attrs=testid("header"), [
    @html.a(
      href="/",
      class="site-title",
      attrs=testid("header-home-link"),
      "研修コース e チケット",
    ),
    view_me(model.me, emit),
    view_switcher(model, emit),
  ])
}

///|
/// 操作中の従業員の表示。
fn view_me(me : Remote[Me], emit : @cmd.Emit[Msg]) -> @html.Html {
  let content : Array[@html.Html] = match me {
    NotAsked | Loading =>
      [@html.text("従業員の情報を読み込んでいます")]
    Failed(error) =>
      [
        @html.text(
          "従業員の情報を読み込めませんでした（\{fetch_error_text(error)}）。",
        ),
        @html.button(
          type_="button",
          on_click=emit(RetryMe),
          attrs=testid("header-me-retry-button"),
          "再試行",
        ),
      ]
    Missing => [@html.text("従業員が見つかりません")]
    Loaded(me) => me_fields(me)
  }
  @html.div(
    class="header-me",
    attrs=testid("header-me").aria_live("polite"),
    content,
  )
}

///|
fn me_fields(me : Me) -> Array[@html.Html] {
  let name = match me.display_name {
    Some(name) => name
    None => "従業員が見つかりません"
  }
  let company = match me.company_name {
    Some(company) => "（\{company}）"
    None => ""
  }
  [
    @html.span(class="me-name", attrs=testid("header-me-name"), [
      @html.text("操作中の従業員: "),
      @html.strong(name),
      @html.text(company),
    ]),
    @html.span(class="me-balance", attrs=testid("header-me-balance"), [
      @html.text("残高: "),
      @html.strong(format_points(me.balance)),
    ]),
  ]
}

///|
/// 従業員の切り替えの部品。名前の付いたボタンで開閉し、一覧はボタンの並び（キーボードで選べる。NFR5）。
fn view_switcher(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  let switcher = model.switcher
  let toggle = if switcher.is_open {
    @html.button(
      type_="button",
      on_click=emit(SwitcherClosed),
      attrs=testid("employee-switcher-toggle")
        .aria_expanded("true")
        .aria_controls("employee-switcher-panel"),
      "従業員の切り替えを閉じる",
    )
  } else {
    @html.button(
      type_="button",
      on_click=emit(SwitcherOpened),
      attrs=testid("employee-switcher-toggle")
        .aria_expanded("false")
        .aria_controls("employee-switcher-panel"),
      "従業員を切り替える",
    )
  }
  let error = match switcher.error {
    Some(message) =>
      @html.p(
        class="switcher-error",
        attrs=testid("employee-switcher-error").role("alert"),
        message,
      )
    None => @html.nothing
  }
  let panel = if switcher.is_open {
    view_switcher_panel(model, emit)
  } else {
    @html.nothing
  }
  @html.div(class="switcher", attrs=testid("employee-switcher"), [
    toggle, error, panel,
  ])
}

///|
fn view_switcher_panel(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  let switcher = model.switcher
  let current_id = match model.me {
    Loaded(me) => me.employee_id
    _ => None
  }
  let body = match switcher.employees {
    NotAsked | Loading =>
      view_loading(
        "従業員の一覧を読み込んでいます", "employee-switcher-loading",
      )
    Failed(error) =>
      view_load_failed(
        "従業員の一覧",
        error,
        SwitcherOpened,
        "employee-switcher-failed",
        emit,
      )
    Missing => @html.p("従業員の一覧が見つかりません")
    Loaded(employees) =>
      @html.ul(
        class="switcher-list",
        attrs=testid("employee-switcher-list"),
        employees.map(employee => {
          view_switcher_option(
            employee,
            current_id,
            switcher.is_switching,
            emit,
          )
        }),
      )
  }
  let switching = if switcher.is_switching {
    @html.p(
      attrs=testid("employee-switcher-switching").role("status"),
      "切り替えています",
    )
  } else {
    @html.nothing
  }
  @html.section(
    id="employee-switcher-panel",
    class="switcher-panel",
    attrs=testid("employee-switcher-panel").aria_label(
      "切り替える従業員を選ぶ",
    ),
    [
      @html.h2("切り替える従業員を選んでください"),
      switching,
      body,
    ],
  )
}

///|
fn view_switcher_option(
  employee : Employee,
  current_id : String?,
  is_switching : Bool,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let is_current = current_id == Some(employee.employee_id)
  let current_label = if is_current { "・操作中" } else { "" }
  let label = "\{employee.display_name}（\{employee.company_name}・\{contract_status_label(employee.contract_status)}\{current_label}）"
  let attrs = testid("employee-switcher-option-\{employee.employee_id}")
  let attrs = if is_current { attrs.aria_current("true") } else { attrs }
  @html.li(
    @html.button(
      type_="button",
      disabled=is_switching,
      on_click=emit(EmployeeChosen(employee.employee_id)),
      attrs~,
      label,
    ),
  )
}
