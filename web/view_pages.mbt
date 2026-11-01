// 各画面の描画（コース一覧・コースの詳細・購入確認・完了と e チケット）。

///|
/// コース一覧（WF1）。U2 で探索条件の入力・0 件・400 の案内を足した。
/// 条件を変えても全画面は描き直さず、件数と一覧の部分だけが変わる（NFR1.3）。
/// 件数・0 件・失敗の変化はライブリージョンで支援技術に知らせる（NFR5.3）。
fn view_course_list_page(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  let body = match model.course_list {
    NotAsked | Loading =>
      view_loading(
        "コース一覧を読み込んでいます", "course-list-loading",
      )
    Failed(error) =>
      view_load_failed(
        "コース一覧",
        error,
        RetrySearch,
        "course-list-failed",
        emit,
      )
    Missing =>
      view_load_failed(
        "コース一覧",
        NotFound,
        RetrySearch,
        "course-list-failed",
        emit,
      )
    Loaded(list) if list.count == 0 => view_empty_results(emit)
    Loaded(list) => view_course_list(list)
  }
  @html.section(class="course-list-page", attrs=testid("course-list-page"), [
    @html.h1("コース一覧"),
    view_search_form(model, emit),
    view_condition_errors(model),
    @html.div(
      class="search-results",
      attrs=testid("course-search-results").aria_live("polite"),
      [body],
    ),
  ])
}

///|
fn view_course_list(list : CourseList) -> @html.Html {
  if list.count == 0 {
    return @html.p(
      class="load-state empty",
      attrs=testid("course-list-empty"),
      "表示できるコースはありません",
    )
  }
  @html.div([
    @html.p(
      class="result-count",
      attrs=testid("course-list-count"),
      "\{list.count} 件のコース",
    ),
    @html.ul(
      class="course-cards",
      attrs=testid("course-list"),
      list.courses.map(view_course_card),
    ),
  ])
}

///|
/// コースのカード。カード全体をコースの詳細へのリンクにする。
fn view_course_card(row : CourseRow) -> @html.Html {
  let next_session = match row.next_session_starts_at {
    Some(starts_at) => format_date(starts_at)
    None => "今後の開催予定なし"
  }
  @html.li(
    class="course-card",
    @html.a(
      href=route_to_path(CourseDetailPage(row.course_id)),
      class="course-card-link",
      attrs=testid("course-card-link-\{row.course_id}"),
      [
        @html.h2(row.title),
        @html.p(class="course-summary", row.summary),
        field_list(
          [
            ("カテゴリ", category_label(row.category)),
            ("受講料", format_points(row.fee_points)),
            ("形式", delivery_format_label(row.delivery_format)),
            ("直近の開催日", next_session),
            ("空席のある日程", "\{row.open_session_count} 日程"),
          ],
          "course-card-fields-\{row.course_id}",
        ),
      ],
    ),
  )
}

///|
/// コースの詳細と日程の選択（WF2）。
fn view_course_detail_page(model : Model, emit : @cmd.Emit[Msg]) -> @html.Html {
  match model.course_detail {
    NotAsked | Loading =>
      view_loading(
        "コースの詳細を読み込んでいます", "course-detail-loading",
      )
    Missing =>
      view_not_found("コースが見つかりません", "course-not-found")
    Failed(error) =>
      view_load_failed(
        "コースの詳細",
        error,
        RetryCourseDetail,
        "course-detail-failed",
        emit,
      )
    Loaded(detail) => view_course_detail(model, detail, emit)
  }
}

///|
fn view_course_detail(
  model : Model,
  detail : CourseDetail,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let course = detail.course
  let notice = match model.notice {
    Some(message) =>
      @html.p(
        class="notice notice-warning",
        attrs=testid("course-detail-notice").role("alert"),
        message,
      )
    None => @html.nothing
  }
  @html.section(
    class="course-detail-page",
    attrs=testid("course-detail-page"),
    [
      // 詳細へ移る前の一覧の条件を持った URL へ戻る（U2 の BR11.8）
      back_to_list_link(
        "course-detail-back-link",
        href=list_path(model.search_input),
      ),
      @html.h1(course.title),
      field_list(
        [
          ("カテゴリ", category_label(course.category)),
          ("概要", course.summary),
          ("説明", course.description),
          ("受講料", format_points(course.fee_points)),
          ("所要時間", format_duration(course.duration_minutes)),
          ("形式", delivery_format_label(course.delivery_format)),
        ],
        "course-detail-fields",
      ),
      notice,
      view_session_list(detail, model.selected_session_id, emit),
      view_proceed(detail, model.selected_session_id, emit),
    ],
  )
}

///|
/// 日程の一覧（単一選択の部品のまとまり）。
fn view_session_list(
  detail : CourseDetail,
  selected : String?,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let body = if detail.sessions.length() == 0 {
    [
      @html.p(
        attrs=testid("session-list-empty"),
        "このコースには日程がありません",
      ),
    ]
  } else {
    detail.sessions.map(session => {
      view_session_option(detail.course, session, selected, emit)
    })
  }
  @html.fieldset(
    class="session-list",
    attrs=testid("session-list"),
    [@html.legend("日程を選んでください"), ..body],
  )
}

///|
/// 日程 1 つ。受付終了・満席・購入済みは選べない状態にし、理由を文字で示して
/// 読み上げ用の説明（aria-describedby）にも含める（BR2.5、NFR5.3、NFR5.4）。
fn view_session_option(
  course : Course,
  session : Session,
  selected : String?,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let id = session.session_id
  let is_available = session.status == Available
  let is_selected = selected == Some(id)
  let status_id = "session-\{id}-status"
  let class = if is_selected {
    "session-option selected"
  } else if is_available {
    "session-option"
  } else {
    "session-option unavailable"
  }
  @html.div(class~, attrs=testid("session-option-\{id}"), [
    @html.input(
      input_type=Radio,
      id="session-\{id}",
      name="session",
      value=id,
      checked=is_selected,
      on_change=emit.map((_ : String) => SessionSelected(id)),
      attrs=testid("session-option-radio-\{id}")
        .disabled(!is_available)
        .aria_describedby(status_id),
    ),
    @html.label(for_="session-\{id}", class="session-label", [
      @html.span(
        class="session-time",
        format_session_time(session.starts_at, session.ends_at),
      ),
      @html.span(
        class="session-venue",
        "会場・形式: \{venue_label(session.venue, course.delivery_format)}",
      ),
      @html.span(class="session-capacity", "定員 \{session.capacity} 名"),
      @html.span(class="session-seats", "残席 \{session.remaining_seats} 席"),
      @html.span(
        id=status_id,
        class="session-status status-\{status_class(session.status)}",
        attrs=testid("session-status-\{id}"),
        "状態: \{session_status_label(session.status)}",
      ),
    ]),
  ])
}

///|
fn status_class(status : SessionStatus) -> String {
  match status {
    Available => "available"
    Full => "full"
    Closed => "closed"
    Purchased => "purchased"
    OtherStatus(_) => "other"
  }
}

///|
/// 「購入確認へ進む」。購入可の日程を選ぶまでは無効にして案内する（BR7.2）。
fn view_proceed(
  detail : CourseDetail,
  selected : String?,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let chosen = match selected {
    Some(id) =>
      match find_session(detail, id) {
        Some(session) if session.status == Available => Some(session)
        _ => None
      }
    None => None
  }
  let hint = match chosen {
    Some(session) =>
      "選んだ日程: \{format_session_time(session.starts_at, session.ends_at)}"
    None => "日程を選んでください"
  }
  @html.div(class="proceed", [
    @html.p(
      id="proceed-hint",
      attrs=testid("proceed-hint").aria_live("polite"),
      hint,
    ),
    @html.button(
      type_="button",
      class="primary",
      disabled=chosen is None,
      on_click=emit(ProceedToConfirm),
      attrs=testid("proceed-to-confirm-button").aria_describedby("proceed-hint"),
      "購入確認へ進む",
    ),
  ])
}

///|
/// 購入確認（WF3）。
fn view_purchase_confirm_page(
  model : Model,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  match model.course_detail {
    NotAsked | Loading =>
      view_loading(
        "購入確認の内容を読み込んでいます", "purchase-confirm-loading",
      )
    Missing =>
      view_not_found("コースが見つかりません", "course-not-found")
    Failed(error) =>
      view_load_failed(
        "購入確認の内容",
        error,
        RetryCourseDetail,
        "purchase-confirm-failed",
        emit,
      )
    Loaded(_) =>
      match confirm_readiness(model) {
        Waiting =>
          view_loading(
            "購入確認の内容を準備しています", "purchase-confirm-loading",
          )
        Ready(summary) => view_confirm(model, summary, None, emit)
        Blocked(summary, reason) =>
          view_confirm(model, summary, Some(reason), emit)
      }
  }
}

///|
fn view_confirm(
  model : Model,
  summary : ConfirmSummary,
  blocked : BlockReason?,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let session = summary.session
  let is_submitting = model.purchase is Submitting
  let can_confirm = blocked is None && model.purchase is Idle
  let confirm_label = if is_submitting {
    "購入を処理しています"
  } else {
    "購入を確定する"
  }
  @html.section(
    class="purchase-confirm-page",
    attrs=testid("purchase-confirm-page"),
    [
      @html.h1("購入確認"),
      field_list(
        [
          ("コース名", summary.course.title),
          (
            "開催日時",
            format_session_time(session.starts_at, session.ends_at),
          ),
          (
            "会場または形式",
            venue_label(session.venue, summary.course.delivery_format),
          ),
          ("受講料", format_points(summary.course.fee_points)),
        ],
        "purchase-summary",
      ),
      view_budget_preview(summary, blocked),
      @html.div(class="confirm-actions", [
        @html.button(
          type_="button",
          class="primary",
          disabled=!can_confirm,
          on_click=emit(ConfirmClicked),
          attrs=testid("purchase-confirm-button").aria_busy(
            if is_submitting {
              "true"
            } else {
              "false"
            },
          ),
          confirm_label,
        ),
        @html.button(
          type_="button",
          disabled=is_submitting,
          on_click=emit(BackToDetail),
          attrs=testid("purchase-back-to-detail-button"),
          "日程を選び直す",
        ),
      ]),
      if is_submitting {
        @html.p(
          attrs=testid("purchase-submitting").role("status"),
          "購入を処理しています",
        )
      } else {
        @html.nothing
      },
      view_purchase_outcome(model.purchase, summary, emit),
    ],
  )
}

///|
/// 現在の残高・購入後の残高 または 不足額（BR4.3、BR4.4）。
fn view_budget_preview(
  summary : ConfirmSummary,
  blocked : BlockReason?,
) -> @html.Html {
  let fee = summary.course.fee_points
  let rows = [("現在の残高", format_points(summary.balance))]
  let message = match blocked {
    None => {
      rows.push(("購入後の残高", format_points(summary.balance - fee)))
      @html.nothing
    }
    Some(InsufficientBalance(shortfall)) => {
      rows.push(("不足額", format_points(shortfall)))
      blocked_message(
        "研修予算の残高が不足しているため、購入を確定できません。",
      )
    }
    Some(NotEligible) =>
      blocked_message(
        "ご所属の企業は現在このサービスをご利用いただけないため、購入を確定できません。",
      )
    Some(Unavailable(status)) =>
      blocked_message(
        "この日程は「\{session_status_label(status)}」のため、購入を確定できません。",
      )
  }
  @html.div(class="budget-preview", attrs=testid("budget-preview"), [
    field_list(rows, "budget-preview-fields"),
    message,
  ])
}

///|
fn blocked_message(text : String) -> @html.Html {
  @html.p(
    class="notice notice-warning",
    attrs=testid("purchase-blocked-reason").role("status"),
    text,
  )
}

///|
/// 拒否・通信失敗などのメッセージと次の操作（BR7.4、BR7.6）。
fn view_purchase_outcome(
  purchase : PurchaseState,
  summary : ConfirmSummary,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  match purchase {
    Rejected(reason) => view_rejection(reason, summary, emit)
    NetworkFailed =>
      outcome_box("purchase-network-failed", [
        @html.p("通信に失敗しました。"),
        @html.p(
          "購入できたかどうかは、コースの詳細の日程の表示（「購入済み」かどうか）で確かめられます。",
        ),
        retry_purchase_button(emit),
        @html.button(
          type_="button",
          on_click=emit(ChooseAnotherSession),
          attrs=testid("purchase-check-detail-button"),
          "コースの詳細で確かめる",
        ),
      ])
    RequestFailed(message) =>
      outcome_box("purchase-request-failed", [
        @html.p("購入を完了できませんでした（\{message}）。"),
        retry_purchase_button(emit),
        back_to_list_link("purchase-outcome-back-link"),
      ])
    Idle | Submitting | Succeeded(_) => @html.nothing
  }
}

///|
fn retry_purchase_button(emit : @cmd.Emit[Msg]) -> @html.Html {
  @html.button(
    type_="button",
    on_click=emit(RetryPurchase),
    attrs=testid("purchase-retry-button"),
    "もう一度試す",
  )
}

///|
fn outcome_box(id : String, children : Array[@html.Html]) -> @html.Html {
  @html.div(class="purchase-outcome", attrs=testid(id).role("alert"), children)
}

///|
/// 拒否の理由ごとの文言。
pub fn rejection_message(reason : RejectionReason) -> String {
  match reason {
    ReasonFull => "この日程は満席になりました"
    ReasonClosed => "この日程は受付を終了しました"
    ReasonAlreadyPurchased => "この日程はすでに購入済みです"
    ReasonInsufficientBalance => "研修予算の残高が不足しています"
    ReasonCompanyNotActive | ReasonEmployeeNotFound =>
      "ご所属の企業は現在このサービスをご利用いただけません"
    ReasonSessionNotFound => "この日程は見つかりません"
    OtherReason(_, message) =>
      if message == "" {
        "購入できませんでした"
      } else {
        message
      }
  }
}

///|
/// 拒否の理由のあとの操作が「別の日程を選ぶ」か（そうでなければ「コース一覧へ戻る」）。
pub fn rejection_offers_another_session(reason : RejectionReason) -> Bool {
  match reason {
    ReasonFull
    | ReasonClosed
    | ReasonAlreadyPurchased
    | ReasonInsufficientBalance => true
    _ => false
  }
}

///|
fn view_rejection(
  reason : RejectionReason,
  summary : ConfirmSummary,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  let shortfall = summary.course.fee_points - summary.balance
  let shortfall_line = if reason == ReasonInsufficientBalance && shortfall > 0 {
    @html.p(
      attrs=testid("purchase-rejected-shortfall"),
      "不足額: \{format_points(shortfall)}",
    )
  } else {
    @html.nothing
  }
  let next_action = if rejection_offers_another_session(reason) {
    @html.button(
      type_="button",
      on_click=emit(ChooseAnotherSession),
      attrs=testid("purchase-choose-another-button"),
      "別の日程を選ぶ",
    )
  } else {
    back_to_list_link("purchase-outcome-back-link")
  }
  outcome_box("purchase-rejected", [
    @html.p(
      attrs=testid("purchase-rejected-message"),
      rejection_message(reason),
    ),
    shortfall_line,
    next_action,
  ])
}

///|
/// 完了と e チケット（WF3・WF5）。
fn view_purchase_complete_page(
  model : Model,
  emit : @cmd.Emit[Msg],
) -> @html.Html {
  match model.ticket {
    NotAsked | Loading =>
      view_loading(
        "e チケットを読み込んでいます", "ticket-loading",
      )
    Missing =>
      view_not_found("チケットが見つかりません", "ticket-not-found")
    Failed(error) =>
      view_load_failed(
        "e チケット",
        error,
        RetryTicket,
        "ticket-failed",
        emit,
      )
    Loaded(ticket) => {
      let is_just_purchased = match model.purchase {
        Succeeded(t) => t.ticket_id == ticket.ticket_id
        _ => false
      }
      let heading = if is_just_purchased {
        "購入が完了しました"
      } else {
        "e チケット"
      }
      @html.section(
        class="purchase-complete-page",
        attrs=testid("purchase-complete-page"),
        [
          @html.h1(heading),
          @html.article(class="ticket", attrs=testid("ticket-view"), [
            @html.h2("e チケット"),
            field_list(
              [
                ("チケット番号", ticket.ticket_number),
                ("従業員名", ticket.employee_name),
                ("コース名", ticket.course_title),
                (
                  "開催日時",
                  format_session_time(ticket.starts_at, ticket.ends_at),
                ),
                (
                  "会場または形式",
                  venue_label(ticket.venue, ticket.delivery_format),
                ),
                ("購入日時", format_datetime(ticket.purchased_at)),
                ("支払ったポイント", format_points(ticket.paid_points)),
              ],
              "ticket-fields",
            ),
          ]),
          back_to_list_link("ticket-back-link"),
        ],
      )
    }
  }
}
