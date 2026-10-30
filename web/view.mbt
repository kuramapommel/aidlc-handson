// 描画関数（状態 → 画面）。純粋関数で、操作はメッセージを送るコマンドとして結びつけるだけにする。
// - Web API から受け取った文字列は、すべて文字（テキストノード）として描画する（NFR3.11）。
// - 操作できる部品には data-testid を付ける。状態は色だけでなく文字で示す（NFR5）。
// - 部品ごとの描画は view_header.mbt・view_pages.mbt に分ける。

///|
/// data-testid を付けた属性の組を作る。
fn testid(id : String) -> @html.Attrs {
  @html.Attrs::build().data_set("testid", id)
}

///|
/// 画面全体。
pub fn view(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  @html.div(class="app", [
    view_header(model, emit),
    @html.main_(class="page", attrs=testid("page"), [
      view_eligibility_notice(model),
      view_page(model, emit),
    ]),
  ])
}

///|
/// 契約外の企業の案内（BR1.3）。一覧・詳細・購入確認の上部に出す。
fn view_eligibility_notice(model : Model) -> @html.Html {
  let shows_notice = match model.route {
    CourseListPage | CourseDetailPage(_) | PurchaseConfirmPage(_, _) => true
    _ => false
  }
  match model.me {
    Loaded(me) if shows_notice && me.eligibility != Eligible =>
      @html.div(
        class="notice notice-warning",
        attrs=testid("eligibility-notice").role("status"),
        [
          @html.p([
            @html.strong("ご利用いただけません: "),
            @html.text(
              "ご所属の企業は現在このサービスをご利用いただけません",
            ),
          ]),
          @html.p(
            "コースの閲覧は続けられますが、購入はできません。",
          ),
        ],
      )
    _ => @html.nothing
  }
}

///|
fn view_page(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  match model.route {
    CourseListPage => view_course_list_page(model, emit)
    CourseDetailPage(_) => view_course_detail_page(model, emit)
    PurchaseConfirmPage(_, _) => view_purchase_confirm_page(model, emit)
    PurchaseCompletePage(_) => view_purchase_complete_page(model, emit)
    NotFoundPage =>
      view_not_found("ページが見つかりません", "not-found-page")
  }
}

///|
/// 読み込み中の表示（BR7.1）。
fn view_loading(text : String, id : String) -> @html.Html {
  @html.p(class="load-state loading", attrs=testid(id).role("status"), text)
}

///|
/// 読み込みの失敗と「再試行」（BR7.1）。
fn view_load_failed(
  what : String,
  error : FetchError,
  retry : Msg,
  id : String,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  @html.div(class="load-state failed", attrs=testid(id).role("alert"), [
    @html.p(
      "\{what}を読み込めませんでした（\{fetch_error_text(error)}）。",
    ),
    @html.button(
      type_="button",
      on_click=emit(retry),
      attrs=testid("\{id}-retry-button"),
      "再試行",
    ),
  ])
}

///|
/// コース・チケット・ページが見つからないときの表示（BR7.9、BR5.10）。
fn view_not_found(message : String, id : String) -> @html.Html {
  @html.section(class="not-found", attrs=testid(id), [
    @html.h1(message),
    back_to_list_link("\{id}-back-link"),
  ])
}

///|
/// 「コース一覧へ戻る」のリンク。
fn back_to_list_link(id : String) -> @html.Html {
  @html.a(
    href="/",
    class="back-link",
    attrs=testid(id),
    "コース一覧へ戻る",
  )
}

///|
/// 項目名と値の組（定義リストの 1 行）。
fn field_row(name : String, value : String) -> Array[@html.Html] {
  [@html.dt(name), @html.dd(value)]
}

///|
fn field_list(rows : Array[(String, String)], id : String) -> @html.Html {
  let items : Array[@html.Html] = []
  for row in rows {
    items.append(field_row(row.0, row.1))
  }
  @html.dl(class="fields", attrs=testid(id), items)
}
