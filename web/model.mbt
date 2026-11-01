// 画面の状態（Model）とメッセージ（Msg）と、更新関数が外に出す作用（Effect）の型。
// frontend-components.md の 2 節（Model）・3 節（Route）・4 節（Msg）に対応する。
// どれも不変の値で、等しさで比べられる（rabbita は状態が変わったかを Eq で判定する）。

///|
/// 読み込みの状態（BR7.1）。Missing は「見つからない」（404）。
pub(all) enum Remote[T] {
  NotAsked
  Loading
  Loaded(T)
  Missing
  Failed(FetchError)
} derive(Eq, Debug)

///|
/// 読み込みの失敗の種類。
pub(all) enum FetchError {
  /// 404（コース・チケットが見つからない）
  NotFound
  /// 2xx・404 以外の応答（状態コードと、本文に含まれていた説明）
  Http(Int32, String)
  /// 通信そのものの失敗（接続できないなど）
  Network(String)
  /// 応答の形が想定と違う
  Decode(String)
} derive(Eq, Debug)

///|
/// 利用資格（BR1.1）。
pub(all) enum Eligibility {
  Eligible
  CompanyNotActive
  EmployeeNotFound
  OtherEligibility(String)
} derive(Eq, Debug)

///|
/// 日程の表示用の状態（BR2.3・BR2.4）。
pub(all) enum SessionStatus {
  Available
  Full
  Closed
  Purchased
  OtherStatus(String)
} derive(Eq, Debug)

///|
/// 購入の拒否の理由（BR5.1）。
pub(all) enum RejectionReason {
  ReasonEmployeeNotFound
  ReasonCompanyNotActive
  ReasonSessionNotFound
  ReasonClosed
  ReasonAlreadyPurchased
  ReasonFull
  ReasonInsufficientBalance
  OtherReason(String, String)
} derive(Eq, Debug)

///|
/// 操作中の従業員（`GET /api/me`・従業員の切り替えの応答）。
pub(all) struct Me {
  employee_id : String?
  display_name : String?
  company_name : String?
  contract_status : String?
  eligibility : Eligibility
  balance : Int32
} derive(Eq, Debug)

///|
/// コース一覧の 1 行。
pub(all) struct CourseRow {
  course_id : String
  title : String
  category : String
  summary : String
  fee_points : Int32
  delivery_format : String
  next_session_starts_at : String?
  open_session_count : Int32
} derive(Eq, Debug)

///|
/// コース一覧（探索の結果も同じ形。件数は条件に合うコースの数）。
pub(all) struct CourseList {
  courses : Array[CourseRow]
  count : Int32
} derive(Eq, Debug)

///|
/// コースの全項目。
pub(all) struct Course {
  course_id : String
  title : String
  category : String
  summary : String
  description : String
  fee_points : Int32
  duration_minutes : Int32
  delivery_format : String
} derive(Eq, Debug)

///|
/// 日程。
pub(all) struct Session {
  session_id : String
  starts_at : String
  ends_at : String
  venue : String?
  capacity : Int32
  remaining_seats : Int32
  status : SessionStatus
} derive(Eq, Debug)

///|
/// コースの詳細（コースと日程の一覧）。
pub(all) struct CourseDetail {
  course : Course
  sessions : Array[Session]
} derive(Eq, Debug)

///|
/// 切り替え用の従業員。
pub(all) struct Employee {
  employee_id : String
  display_name : String
  company_name : String
  contract_status : String
} derive(Eq, Debug)

///|
/// e チケット（BR5.9）。
pub(all) struct Ticket {
  ticket_id : String
  ticket_number : String
  employee_name : String
  course_id : String
  course_title : String
  session_id : String
  starts_at : String
  ends_at : String
  venue : String?
  delivery_format : String
  purchased_at : String
  paid_points : Int32
} derive(Eq, Debug)

///|
/// 現在の画面（frontend-components.md の 3 節）。
pub(all) enum Route {
  CourseListPage
  CourseDetailPage(String)
  PurchaseConfirmPage(String, String)
  PurchaseCompletePage(String)
  NotFoundPage
} derive(Eq, Debug)

///|
/// 購入確認の中の状態のうち、保存するもの（functional-spec.md 5.3）。
/// Ready と Blocked は保存せず、[confirm_readiness] で描画のたびに計算する。
pub(all) enum PurchaseState {
  Idle
  Submitting
  Succeeded(Ticket)
  Rejected(RejectionReason)
  NetworkFailed
  /// 拒否（409）でも通信の失敗でもない失敗（403・400・5xx など）
  RequestFailed(String)
} derive(Eq, Debug)

///|
/// 従業員の切り替えの部品の状態。
pub(all) struct Switcher {
  is_open : Bool
  employees : Remote[Array[Employee]]
  is_switching : Bool
  error : String?
} derive(Eq, Debug)

///|
/// 画面全体の状態。
pub(all) struct Model {
  route : Route
  /// 操作中の従業員の世代。切り替えのたびに 1 増やし、前の従業員の要求の応答を捨てるのに使う
  generation : Int32
  me : Remote[Me]
  switcher : Switcher
  course_list : Remote[CourseList]
  course_detail : Remote[CourseDetail]
  selected_session_id : String?
  /// コースの詳細に出す案内（購入確認を開いたが日程が購入できなくなっていたときなど）
  notice : String?
  purchase : PurchaseState
  ticket : Remote[Ticket]
  // ---- U2: コース一覧の探索（frontend-components.md の 2 節） ----
  /// 探索条件の入力欄の今の値
  search_input : SearchInput
  /// 期間の前後が逆のときの案内（BR11.5）
  period_error : String?
  /// キーワードの入力ごとに進める番号。待ち終わりの番号がこれと同じときだけ送る（BR11.1）
  keyword_debounce : Int32
  /// 一覧の要求（条件なし・条件付きのどちらも）を送るたびに進める番号。
  /// 応答の番号がこれと同じときだけ反映する（BR11.2）
  search_request_seq : Int32
  /// 最後に表示できた一覧（400 のときに一覧をそれまでのままにするため。BR11.9）
  last_results : CourseList?
  /// 万一の 400 の案内（不正な条件の一覧。BR11.9）
  condition_errors : Array[ConditionFieldError]?
} derive(Eq, Debug)

///|
/// メッセージ。応答のメッセージは、要求したときの従業員の世代（Int）と対象の ID を持ち、
/// 画面の移動や切り替えの後に届いた古い応答を捨てられるようにする。
pub(all) enum Msg {
  /// URL が変わった（起動時・画面内の移動・戻る/進む）。パスを持つ
  UrlChanged(String)
  /// 画面内のリンクが押された（パス）
  LinkClicked(String)
  /// 別のサイトへのリンクが押された（URL）
  ExternalLinkClicked(String)
  MeLoaded(Int32, Result[Me, FetchError])
  RetryMe
  /// 条件なしの一覧の応答（従業員の世代, 要求の番号, 結果）
  CourseListLoaded(Int32, Int32, Result[CourseList, FetchError])
  RetryCourseList
  CourseDetailLoaded(Int32, String, Result[CourseDetail, FetchError])
  RetryCourseDetail
  SessionSelected(String)
  ProceedToConfirm
  BackToDetail
  ConfirmClicked
  PurchaseSucceeded(Int32, Ticket, Int32)
  PurchaseRejected(Int32, RejectionReason)
  PurchaseFailed(Int32, String)
  PurchaseNetworkFailed(Int32)
  RetryPurchase
  ChooseAnotherSession
  TicketLoaded(Int32, String, Result[Ticket, FetchError])
  RetryTicket
  SwitcherOpened
  SwitcherClosed
  EmployeesLoaded(Result[Array[Employee], FetchError])
  EmployeeChosen(String)
  EmployeeSwitched(Result[Me, FetchError])
  // ---- U2: コース一覧の探索（frontend-components.md の 4 節） ----
  /// キーワードの入力（打ったとおりの文字列）
  KeywordChanged(String)
  /// キーワードの待ちが終わった（待ちの番号）
  KeywordSettled(Int32)
  /// カテゴリの選択（値。空は「すべて」）
  CategoryChanged(String)
  AvailableOnlyChanged(Bool)
  /// 期間の入力（開始日, 終了日。空は指定なし）
  PeriodChanged(String, String)
  /// 並べ替えの選択（値）
  SortChanged(String)
  /// 探索の応答（従業員の世代, 要求の番号, 結果）
  SearchLoaded(Int32, Int32, Result[CourseList, SearchFailure])
  ResetConditions
  RetrySearch
} derive(Eq, Debug)

///|
/// 更新関数が外に出す作用（コマンドの中身）。[effect_to_cmd] が rabbita のコマンドにする。
pub(all) enum Effect {
  FetchMe(Int32)
  /// 条件なしのコース一覧の要求（従業員の世代, 要求の番号）。U1 と同じ `GET /api/courses` を送る
  FetchCourses(Int32, Int32)
  FetchCourseDetail(Int32, String)
  FetchTicket(Int32, String)
  FetchEmployees
  SwitchEmployee(String)
  /// 購入の要求（日程 ID だけを送る。BR5.2）
  SubmitPurchase(Int32, String)
  PushUrl(String)
  ReplaceUrl(String)
  LoadUrl(String)
  /// 探索条件付きのコース一覧の要求（従業員の世代, 要求の番号, 条件）。
  /// 条件なしの一覧は FetchCourses で要求する（番号は同じ search_request_seq から振る）
  SearchCourses(Int32, Int32, SearchInput)
  /// キーワードの待ち（keyword_debounce_ms 後に KeywordSettled(番号) を届ける。BR11.1）
  WaitKeyword(Int32)
} derive(Eq, Debug)
