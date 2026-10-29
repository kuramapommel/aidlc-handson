// Web API の応答（状態コードと本文）を画面の型に変換する純粋関数。
// 応答の形は crates/web_api/src/dto.rs と errors.rs に合わせる。
// 本文の文字列は値として読み取るだけで、HTML として解釈しない（NFR3.11）。

///|
/// 応答の形が想定と違うときの失敗。
priv suberror DecodeError {
  DecodeError(String)
}

///|
fn as_object(json : Json, what : String) -> Map[String, Json] raise DecodeError {
  match json {
    Object(fields) => fields
    _ => raise DecodeError("\{what} がオブジェクトではありません")
  }
}

///|
fn get_string(
  fields : Map[String, Json],
  key : String,
) -> String raise DecodeError {
  match fields.get(key) {
    Some(String(value)) => value
    _ => raise DecodeError("\{key} が文字列ではありません")
  }
}

///|
fn get_optional_string(
  fields : Map[String, Json],
  key : String,
) -> String? raise DecodeError {
  match fields.get(key) {
    None | Some(Null) => None
    Some(String(value)) => Some(value)
    _ =>
      raise DecodeError("\{key} が文字列でも null でもありません")
  }
}

///|
fn get_int(fields : Map[String, Json], key : String) -> Int32 raise DecodeError {
  match fields.get(key) {
    Some(Number(value, ..)) => value.to_int()
    _ => raise DecodeError("\{key} が数ではありません")
  }
}

///|
fn get_array(
  fields : Map[String, Json],
  key : String,
) -> Array[Json] raise DecodeError {
  match fields.get(key) {
    Some(Array(items)) => items
    _ => raise DecodeError("\{key} が配列ではありません")
  }
}

///|
fn get_object(
  fields : Map[String, Json],
  key : String,
) -> Map[String, Json] raise DecodeError {
  match fields.get(key) {
    Some(value) => as_object(value, key)
    None => raise DecodeError("\{key} がありません")
  }
}

///|
/// 利用資格の値。
pub fn eligibility_from_string(value : String) -> Eligibility {
  match value {
    "eligible" => Eligible
    "company-not-active" => CompanyNotActive
    "employee-not-found" => EmployeeNotFound
    other => OtherEligibility(other)
  }
}

///|
/// 日程の表示用の状態の値。
pub fn session_status_from_string(value : String) -> SessionStatus {
  match value {
    "available" => Available
    "full" => Full
    "closed" => Closed
    "purchased" => Purchased
    other => OtherStatus(other)
  }
}

///|
/// 拒否の理由の値（不明な値は、Web API の説明の文と一緒に持つ）。
pub fn rejection_reason_from_string(
  value : String,
  message : String,
) -> RejectionReason {
  match value {
    "employee-not-found" => ReasonEmployeeNotFound
    "company-not-active" => ReasonCompanyNotActive
    "session-not-found" => ReasonSessionNotFound
    "closed" => ReasonClosed
    "already-purchased" => ReasonAlreadyPurchased
    "full" => ReasonFull
    "insufficient-balance" => ReasonInsufficientBalance
    other => OtherReason(other, message)
  }
}

///|
fn me_from_json(json : Json) -> Me raise DecodeError {
  let fields = as_object(json, "操作中の従業員")
  {
    employee_id: get_optional_string(fields, "employeeId"),
    display_name: get_optional_string(fields, "displayName"),
    company_name: get_optional_string(fields, "companyName"),
    contract_status: get_optional_string(fields, "contractStatus"),
    eligibility: eligibility_from_string(get_string(fields, "eligibility")),
    balance: get_int(fields, "balance"),
  }
}

///|
fn course_row_from_json(json : Json) -> CourseRow raise DecodeError {
  let fields = as_object(json, "コースの行")
  {
    course_id: get_string(fields, "courseId"),
    title: get_string(fields, "title"),
    category: get_string(fields, "category"),
    summary: get_string(fields, "summary"),
    fee_points: get_int(fields, "feePoints"),
    delivery_format: get_string(fields, "deliveryFormat"),
    next_session_starts_at: get_optional_string(fields, "nextSessionStartsAt"),
    open_session_count: get_int(fields, "openSessionCount"),
  }
}

///|
fn course_list_from_json(json : Json) -> CourseList raise DecodeError {
  let fields = as_object(json, "コース一覧")
  let courses = get_array(fields, "courses").map(course_row_from_json)
  { courses, count: get_int(fields, "count"), }
}

///|
fn course_from_json(fields : Map[String, Json]) -> Course raise DecodeError {
  {
    course_id: get_string(fields, "courseId"),
    title: get_string(fields, "title"),
    category: get_string(fields, "category"),
    summary: get_string(fields, "summary"),
    description: get_string(fields, "description"),
    fee_points: get_int(fields, "feePoints"),
    duration_minutes: get_int(fields, "durationMinutes"),
    delivery_format: get_string(fields, "deliveryFormat"),
  }
}

///|
fn session_from_json(json : Json) -> Session raise DecodeError {
  let fields = as_object(json, "日程")
  {
    session_id: get_string(fields, "sessionId"),
    starts_at: get_string(fields, "startsAt"),
    ends_at: get_string(fields, "endsAt"),
    venue: get_optional_string(fields, "venue"),
    capacity: get_int(fields, "capacity"),
    remaining_seats: get_int(fields, "remainingSeats"),
    status: session_status_from_string(get_string(fields, "status")),
  }
}

///|
fn course_detail_from_json(json : Json) -> CourseDetail raise DecodeError {
  let fields = as_object(json, "コースの詳細")
  {
    course: course_from_json(get_object(fields, "course")),
    sessions: get_array(fields, "sessions").map(session_from_json),
  }
}

///|
fn employee_from_json(json : Json) -> Employee raise DecodeError {
  let fields = as_object(json, "従業員")
  {
    employee_id: get_string(fields, "employeeId"),
    display_name: get_string(fields, "displayName"),
    company_name: get_string(fields, "companyName"),
    contract_status: get_string(fields, "contractStatus"),
  }
}

///|
fn employees_from_json(json : Json) -> Array[Employee] raise DecodeError {
  get_array(as_object(json, "従業員一覧"), "employees").map(
    employee_from_json,
  )
}

///|
fn ticket_from_fields(fields : Map[String, Json]) -> Ticket raise DecodeError {
  {
    ticket_id: get_string(fields, "ticketId"),
    ticket_number: get_string(fields, "ticketNumber"),
    employee_name: get_string(fields, "employeeName"),
    course_id: get_string(fields, "courseId"),
    course_title: get_string(fields, "courseTitle"),
    session_id: get_string(fields, "sessionId"),
    starts_at: get_string(fields, "startsAt"),
    ends_at: get_string(fields, "endsAt"),
    venue: get_optional_string(fields, "venue"),
    delivery_format: get_string(fields, "deliveryFormat"),
    purchased_at: get_string(fields, "purchasedAt"),
    paid_points: get_int(fields, "paidPoints"),
  }
}

///|
fn ticket_from_json(json : Json) -> Ticket raise DecodeError {
  ticket_from_fields(as_object(json, "e チケット"))
}

///|
/// エラーの応答の本文 `{"error": {"code", "message", "reason"?}}` から (code, message, reason) を取り出す。
fn error_body(body : String) -> (String, String, String?) {
  try {
    let fields = as_object(@json.parse(body), "エラー")
    let error = get_object(fields, "error")
    (
      get_string(error, "code"),
      get_string(error, "message"),
      get_optional_string(error, "reason"),
    )
  } catch {
    _ => ("", "", None)
  }
}

///|
/// 2xx 以外の応答の説明（本文に説明があればそれ、なければ状態コード）。
fn http_error(response : ApiResponse) -> FetchError {
  let (_, message, _) = error_body(response.body)
  if message == "" {
    Http(response.status, "HTTP \{response.status}")
  } else {
    Http(response.status, message)
  }
}

///|
/// 応答を読み取る共通の手順。2xx なら本文を `decode` で変換し、404 は NotFound にする。
fn[T] decode_response(
  response : ApiResponse,
  decode : (Json) -> T raise DecodeError,
) -> Result[T, FetchError] {
  if response.status == 404 {
    return Err(NotFound)
  }
  if response.status < 200 || response.status >= 300 {
    return Err(http_error(response))
  }
  Ok(decode(@json.parse(response.body))) catch {
    DecodeError(message) => Err(Decode(message))
    error => Err(Decode("\{error}"))
  }
}

///|
/// `GET /api/me` と `POST /api/session/employee` の応答。
pub fn decode_me(response : ApiResponse) -> Result[Me, FetchError] {
  decode_response(response, me_from_json)
}

///|
/// `GET /api/courses` の応答。
pub fn decode_course_list(
  response : ApiResponse,
) -> Result[CourseList, FetchError] {
  decode_response(response, course_list_from_json)
}

///|
/// `GET /api/courses/{courseId}` の応答。
pub fn decode_course_detail(
  response : ApiResponse,
) -> Result[CourseDetail, FetchError] {
  decode_response(response, course_detail_from_json)
}

///|
/// `GET /api/employees` の応答。
pub fn decode_employees(
  response : ApiResponse,
) -> Result[Array[Employee], FetchError] {
  decode_response(response, employees_from_json)
}

///|
/// `GET /api/tickets/{ticketId}` の応答。
pub fn decode_ticket(response : ApiResponse) -> Result[Ticket, FetchError] {
  decode_response(response, ticket_from_json)
}

///|
/// `POST /api/purchases` の応答をメッセージにする。201 は成功、409 は拒否の理由、
/// それ以外（403・400・5xx、読み取れない応答）は失敗の説明（BR7.4）。
pub fn purchase_message(generation : Int32, response : ApiResponse) -> Msg {
  if response.status == 409 {
    let (_, message, reason) = error_body(response.body)
    return match reason {
      Some(reason) =>
        PurchaseRejected(
          generation,
          rejection_reason_from_string(reason, message),
        )
      None =>
        PurchaseFailed(
          generation, "購入の結果を読み取れませんでした",
        )
    }
  }
  let decoded = decode_response(response, json => {
    let fields = as_object(json, "購入の結果")
    (
      ticket_from_fields(get_object(fields, "ticket")),
      get_int(fields, "balanceAfter"),
    )
  })
  match decoded {
    Ok((ticket, balance_after)) =>
      PurchaseSucceeded(generation, ticket, balance_after)
    Err(Http(_, message)) => PurchaseFailed(generation, message)
    Err(NotFound) =>
      PurchaseFailed(generation, "購入の受け口が見つかりません")
    Err(Network(message)) | Err(Decode(message)) =>
      PurchaseFailed(generation, message)
  }
}

///|
/// 通信の結果（応答、または通信の失敗）を読み込みの結果にする。
pub fn[T] fetch_result(
  result : Result[ApiResponse, String],
  decode : (ApiResponse) -> Result[T, FetchError],
) -> Result[T, FetchError] {
  match result {
    Ok(response) => decode(response)
    Err(message) => Err(Network(message))
  }
}
