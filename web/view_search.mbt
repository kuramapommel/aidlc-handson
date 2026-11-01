// コース一覧の探索（U2）の描画（frontend-components.md の 1 節・5 節）。
// どの入力にもラベルを付け、キーボードだけで操作できる標準の部品（入力欄・選択・チェックボックス・
// ボタン）だけを使う。文字列はすべて文字（テキストノード）として描画する（NFR3.4）。

///|
/// 探索条件の入力（SearchForm）。送信の操作はなく、入力のたびにメッセージを送る（BR11.1）。
fn view_search_form(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  let input = model.search_input
  @html.div(
    class="search-form",
    attrs=testid("course-search-form")
      .role("search")
      .aria_label("コースの探索条件"),
    [
      search_field("course-search-keyword", "キーワード", [
        @html.input(
          input_type=Search,
          id="course-search-keyword",
          name="q",
          value=input.keyword,
          auto_complete="off",
          on_input=emit.map((text : String) => KeywordChanged(text)),
          attrs=testid("course-search-keyword-input"),
        ),
      ]),
      search_field("course-search-category", "カテゴリ", [
        @html.select(
          id="course-search-category",
          name="category",
          on_change=emit.map((value : String) => CategoryChanged(value)),
          attrs=testid("course-search-category-select"),
          [
            @html.option(value="", selected=input.category == "", "すべて"),
            ..category_values.map(value => {
              @html.option(
                value~,
                selected=input.category == value,
                category_label(value),
              )
            }),
          ],
        ),
      ]),
      view_available_only(input, emit),
      view_period_inputs(model, emit),
      search_field("course-search-sort", "並べ替え", [
        @html.select(
          id="course-search-sort",
          name="sort",
          on_change=emit.map((value : String) => SortChanged(value)),
          attrs=testid("course-search-sort-select"),
          sort_orders.map(sort => {
            @html.option(
              value=sort_value(sort),
              selected=input.sort == sort,
              sort_label(sort),
            )
          }),
        ),
      ]),
    ],
  )
}

///|
/// ラベルと入力の組。
fn search_field(
  id : String,
  label : String,
  control : Array[@html.Html],
) -> @html.Html {
  @html.div(
    class="search-field",
    [@html.label(for_=id, class="search-label", label), ..control],
  )
}

///|
/// 空席ありのみ（チェックボックス）。押すたびに今の値を反転したメッセージを送る。
fn view_available_only(
  input : SearchInput,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let next = !input.available_only
  @html.div(class="search-field search-field-checkbox", [
    @html.input(
      input_type=Checkbox,
      id="course-search-available",
      name="available",
      checked=input.available_only,
      on_change=emit.map((_ : String) => AvailableOnlyChanged(next)),
      attrs=testid("course-search-available-checkbox"),
    ),
    @html.label(for_="course-search-available", "空席ありのみ"),
  ])
}

///|
/// 期間（開始日・終了日の日付の入力欄）と、前後が逆のときの案内（BR11.5）。
/// 案内は入力欄のすぐ下に文字で出し、aria-describedby で入力欄に結び付ける（NFR5.3）。
fn view_period_inputs(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  let input = model.search_input
  let error_id = "course-search-period-error"
  let date_attrs = (id : String) => {
    let attrs = testid(id)
    match model.period_error {
      Some(_) => attrs.aria_describedby(error_id).aria_invalid("true")
      None => attrs
    }
  }
  let error = match model.period_error {
    Some(message) =>
      @html.p(
        id=error_id,
        class="field-error",
        attrs=testid(error_id).role("alert"),
        message,
      )
    None => @html.nothing
  }
  @html.fieldset(class="search-period", attrs=testid("course-search-period"), [
    @html.legend("期間（開催開始日）"),
    search_field("course-search-from", "開始日", [
      @html.input(
        input_type=Date,
        id="course-search-from",
        name="from",
        value=input.period_from,
        on_change=emit.map((text : String) => {
          PeriodChanged(text, input.period_to)
        }),
        attrs=date_attrs("course-search-from-input"),
      ),
    ]),
    search_field("course-search-to", "終了日", [
      @html.input(
        input_type=Date,
        id="course-search-to",
        name="to",
        value=input.period_to,
        on_change=emit.map((text : String) => {
          PeriodChanged(input.period_from, text)
        }),
        attrs=date_attrs("course-search-to-input"),
      ),
    ]),
    error,
  ])
}

///|
/// 0 件の表示と「条件をリセット」（EmptyState。BR11.6）。読み込み中・失敗とは文言を分ける。
fn view_empty_results(emit : @cmd.Emit[Msg]) -> @html.Html {
  @html.div(class="load-state empty", attrs=testid("course-search-empty"), [
    @html.p("条件に合うコースがありません"),
    @html.button(
      type_="button",
      on_click=emit(ResetConditions),
      attrs=testid("course-search-reset-button"),
      "条件をリセット",
    ),
  ])
}

///|
/// 万一の 400 の案内（ConditionError。BR11.9）。一覧はそれまでのまま。
fn view_condition_errors(model : Model) -> @html.Html {
  match model.condition_errors {
    Some(errors) =>
      @html.div(
        class="notice notice-warning",
        attrs=testid("course-search-condition-error").role("alert"),
        [
          @html.p("条件を確かめてください"),
          @html.ul(
            errors.map(error => {
              @html.li("\{condition_field_label(error.field)}: \{error.reason}")
            }),
          ),
        ],
      )
    None => @html.nothing
  }
}
