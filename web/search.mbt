// コース一覧の探索（U2）の純粋関数: 探索条件の入力の形、URL のクエリとの相互の変換、
// 送る前の形の確認、Web API の探索の応答の読み取り。
// 絞り込みと並べ替えのロジックは持たない（Web API が行う。ADR-006）。
// U3 以降で条件を足すときは、SearchInput に項目を足し、search_params と restore_search_input に
// その項目の 1 行ずつを足す（既存の項目は変えない。NFR7.4）。

///|
/// キーワードの入力が止まってから探索を送るまでの待ち時間（ミリ秒。BR11.1）。
pub let keyword_debounce_ms : Int32 = 150

///|
/// 期間の前後が逆のときの案内（BR11.5）。
pub let period_error_message : String = "期間の指定が正しくありません（終了日を開始日以降にしてください）"

///|
/// 並べ替え（BR11.10）。
pub(all) enum SortOrder {
  /// 開催日が近い順（既定）
  ByDate
  ByFeeAsc
  ByFeeDesc
  ByName
} derive(Eq, Debug)

///|
/// 並べ替えの値（Web API・URL で使う）。
pub fn sort_value(sort : SortOrder) -> String {
  match sort {
    ByDate => "date"
    ByFeeAsc => "fee-asc"
    ByFeeDesc => "fee-desc"
    ByName => "name"
  }
}

///|
/// 並べ替えの表示名。
pub fn sort_label(sort : SortOrder) -> String {
  match sort {
    ByDate => "開催日が近い順"
    ByFeeAsc => "受講料の安い順"
    ByFeeDesc => "受講料の高い順"
    ByName => "コース名順"
  }
}

///|
/// 選べる並べ方（表示の順）。
pub let sort_orders : Array[SortOrder] = [ByDate, ByFeeAsc, ByFeeDesc, ByName]

///|
/// 値から並べ方を得る。決められた 4 つ以外なら None（BR8.5）。
pub fn sort_from_value(value : String) -> SortOrder? {
  for sort in sort_orders {
    if sort_value(sort) == value {
      return Some(sort)
    }
  }
  None
}

///|
/// 選べるカテゴリの値（表示の順。「すべて」は空の文字列で表す）。
pub let category_values : Array[String] = [
  "management", "technology", "business-skills", "compliance",
]

///|
/// 決められた 4 つのカテゴリのどれかか（BR8.2）。
pub fn is_category_value(value : String) -> Bool {
  category_values.contains(value)
}

///|
/// 探索条件の入力欄の今の値（frontend-components.md の searchInput）。
/// 送る条件はこの値から送るたびに計算し、別に保存しない。
pub(all) struct SearchInput {
  /// 利用者が打ったとおりのキーワード（正規化は Web API が行う。TD-U2-1）
  keyword : String
  /// カテゴリの値。空は「すべて」
  category : String
  available_only : Bool
  /// 開始日（YYYY-MM-DD）。空は指定なし
  period_from : String
  /// 終了日（YYYY-MM-DD）。空は指定なし
  period_to : String
  sort : SortOrder
} derive(Eq, Debug)

///|
/// 既定の探索条件（条件なし・開催日が近い順）。
pub fn default_search_input() -> SearchInput {
  {
    keyword: "",
    category: "",
    available_only: false,
    period_from: "",
    period_to: "",
    sort: ByDate,
  }
}

///|
/// 開始日と終了日の両方があり、終了日が開始日より前か（BR11.5）。
/// YYYY-MM-DD の形の文字列は、文字列の大小が日付の前後と一致する。
pub fn is_period_reversed(input : SearchInput) -> Bool {
  input.period_from != "" &&
  input.period_to != "" &&
  input.period_to < input.period_from
}

///|
fn is_leap_year(year : Int32) -> Bool {
  (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

///|
fn days_in_month(year : Int32, month : Int32) -> Int32 {
  match month {
    2 => if is_leap_year(year) { 29 } else { 28 }
    4 | 6 | 9 | 11 => 30
    _ => 31
  }
}

///|
/// `YYYY-MM-DD` の形で実在する暦日か（BR8.4 と同じ規則）。
pub fn is_calendar_date(text : String) -> Bool {
  let chars : Array[Char] = []
  for c in text {
    chars.push(c)
  }
  if chars.length() != 10 || chars[4] != '-' || chars[7] != '-' {
    return false
  }
  let digits = [0, 1, 2, 3, 5, 6, 8, 9]
  for i in digits {
    if chars[i] < '0' || chars[i] > '9' {
      return false
    }
  }
  let number = (from : Int32, to : Int32) => {
    let mut value = 0
    for i in from..<to {
      value = value * 10 + (chars[i].to_int() - '0'.to_int())
    }
    value
  }
  let year = number(0, 4)
  let month = number(5, 7)
  let day = number(8, 10)
  month >= 1 && month <= 12 && day >= 1 && day <= days_in_month(year, month)
}

///|
/// 送る（URL に書く）条件の名前と値の組。既定の値と空の値は含めない（BR11.3）。
pub fn search_params(input : SearchInput) -> Array[(String, String)] {
  let params : Array[(String, String)] = []
  if input.keyword != "" {
    params.push(("q", input.keyword))
  }
  if input.category != "" {
    params.push(("category", input.category))
  }
  if input.available_only {
    params.push(("available", "true"))
  }
  if input.period_from != "" {
    params.push(("from", input.period_from))
  }
  if input.period_to != "" {
    params.push(("to", input.period_to))
  }
  if input.sort != ByDate {
    params.push(("sort", sort_value(input.sort)))
  }
  params
}

///|
/// 条件の名前と値の組をクエリの文字列（先頭の `?` を含む）にする。条件がなければ空の文字列。
pub fn search_query_string(input : SearchInput) -> String {
  let params = search_params(input)
  if params.length() == 0 {
    return ""
  }
  let parts = params.map(pair => {
    percent_encode(pair.0) + "=" + percent_encode(pair.1)
  })
  "?" + parts.join("&")
}

///|
/// 条件付きのコース一覧の URL（例: `/?q=AI&sort=fee-asc`。条件がなければ `/`）。
pub fn list_path(input : SearchInput) -> String {
  "/" + search_query_string(input)
}

///|
fn hex_digit(value : Int32) -> Char {
  "0123456789ABCDEF"[value].to_char().unwrap_or('0')
}

///|
fn hex_value(c : Char) -> Int32? {
  if c >= '0' && c <= '9' {
    Some(c.to_int() - '0'.to_int())
  } else if c >= 'A' && c <= 'F' {
    Some(c.to_int() - 'A'.to_int() + 10)
  } else if c >= 'a' && c <= 'f' {
    Some(c.to_int() - 'a'.to_int() + 10)
  } else {
    None
  }
}

///|
/// URL のクエリの値のエンコード。英数字と `-._~` 以外を UTF-8 のバイトごとに `%XX` にする。
pub fn percent_encode(text : String) -> String {
  let out = StringBuilder()
  for byte in @utf8.encode(text) {
    let b = byte.to_int()
    let c = b.unsafe_to_char()
    if (c >= 'A' && c <= 'Z') ||
      (c >= 'a' && c <= 'z') ||
      (c >= '0' && c <= '9') ||
      c == '-' ||
      c == '.' ||
      c == '_' ||
      c == '~' {
      out.write_char(c)
    } else {
      out.write_char('%')
      out.write_char(hex_digit(b / 16))
      out.write_char(hex_digit(b % 16))
    }
  }
  out.to_string()
}

///|
/// URL のクエリの値のデコード。`%XX` をバイトに、`+` を空白にする。形の崩れた `%` はそのまま残し、
/// UTF-8 として正しくないバイトは置き換え文字にする（URL の改ざんでエラーにしない。NFR3.5）。
pub fn percent_decode(text : String) -> String {
  let chars : Array[Char] = []
  for c in text {
    chars.push(c)
  }
  let bytes : Array[Byte] = []
  let mut i = 0
  while i < chars.length() {
    let c = chars[i]
    if c == '%' && i + 2 < chars.length() {
      match (hex_value(chars[i + 1]), hex_value(chars[i + 2])) {
        (Some(high), Some(low)) => {
          bytes.push((high * 16 + low).to_byte())
          i += 3
          continue
        }
        _ => ()
      }
    }
    if c == '+' {
      bytes.push(b' ')
    } else {
      for b in @utf8.encode(c.to_string()) {
        bytes.push(b)
      }
    }
    i += 1
  }
  @utf8.decode_lossy(Bytes::from_array(bytes[:])[:])
}

///|
/// クエリの文字列（先頭の `?` はあってもなくてもよい）を名前と値の組の並びにする。
pub fn parse_query(query : String) -> Array[(String, String)] {
  let text = if query.has_prefix("?") {
    query.unsafe_substring(start=1, end=query.length())
  } else {
    query
  }
  let pairs : Array[(String, String)] = []
  for part in text.split("&") {
    let segment = part.to_owned()
    if segment == "" {
      continue
    }
    match segment.find("=") {
      Some(at) =>
        pairs.push(
          (
            percent_decode(segment.unsafe_substring(start=0, end=at)),
            percent_decode(
              segment.unsafe_substring(start=at + 1, end=segment.length()),
            ),
          ),
        )
      None => pairs.push((percent_decode(segment), ""))
    }
  }
  pairs
}

///|
/// 画面内の URL（`/パス?クエリ#…`）からクエリの部分（`?` より後、`#` より前）を取り出す。
pub fn query_of(location : String) -> String {
  let out = StringBuilder()
  let mut in_query = false
  for c in location {
    if c == '#' {
      break
    }
    if in_query {
      out.write_char(c)
    } else if c == '?' {
      in_query = true
    }
  }
  out.to_string()
}

///|
/// URL のクエリから探索条件を復元する（BR11.4）。項目ごとに BR8.2〜BR8.5 と同じ規則で確かめ、
/// 正しい項目だけを使い、不正な項目は無視する。キーワードはそのまま復元する。
/// 同じ名前が複数あれば先頭を使う。期間の前後が逆なら、期間（開始日と終了日）を無視する。
pub fn restore_search_input(query : String) -> SearchInput {
  let first = (name : String) => {
    let mut found : String? = None
    for pair in parse_query(query) {
      if pair.0 == name && found is None {
        found = Some(pair.1)
      }
    }
    found
  }
  let date_or_empty = (value : String?) => {
    match value {
      Some(text) if is_calendar_date(text) => text
      _ => ""
    }
  }
  let restored = {
    keyword: first("q").unwrap_or(""),
    category: match first("category") {
      Some(value) if is_category_value(value) => value
      _ => ""
    },
    available_only: first("available") == Some("true"),
    period_from: date_or_empty(first("from")),
    period_to: date_or_empty(first("to")),
    sort: match first("sort") {
      Some(value) => sort_from_value(value).unwrap_or(ByDate)
      None => ByDate
    },
  }
  if is_period_reversed(restored) {
    { ..restored, period_from: "", period_to: "", }
  } else {
    restored
  }
}

///|
/// 不正な条件の 1 項目（Web API の 400 の応答。BR8.6）。
pub(all) struct ConditionFieldError {
  field : String
  reason : String
} derive(Eq, Debug)

///|
/// 探索の失敗。400（不正な条件）と、それ以外の読み込みの失敗を分ける（BR11.7、BR11.9）。
pub(all) enum SearchFailure {
  InvalidCondition(Array[ConditionFieldError])
  FetchFailed(FetchError)
} derive(Eq, Debug)

///|
/// 不正な条件の項目名の表示名。
pub fn condition_field_label(field : String) -> String {
  match field {
    "category" => "カテゴリ"
    "availableOnly" => "空席ありのみ"
    "periodFrom" => "開始日"
    "periodTo" => "終了日"
    "period" => "期間"
    "sort" => "並べ替え"
    other => other
  }
}

///|
/// 400 の本文から、不正な条件の一覧を読む。形が違えば None。
fn condition_errors(body : String) -> Array[ConditionFieldError]? {
  try {
    let fields = as_object(@json.parse(body), "エラー")
    let error = get_object(fields, "error")
    guard get_string(error, "code") == "invalid-search-condition" else {
      return None
    }
    Some(
      get_array(error, "fields").map(item => {
        let entry = as_object(item, "不正な条件")
        {
          field: get_string(entry, "field"),
          reason: get_string(entry, "reason"),
        }
      }),
    )
  } catch {
    _ => None
  }
}

///|
/// `GET /api/courses?…`（探索）の応答。400 は不正な条件の一覧にする。
pub fn decode_search_response(
  response : ApiResponse,
) -> Result[CourseList, SearchFailure] {
  if response.status == 400 {
    match condition_errors(response.body) {
      Some(errors) => return Err(InvalidCondition(errors))
      None => ()
    }
  }
  match decode_course_list(response) {
    Ok(list) => Ok(list)
    Err(error) => Err(FetchFailed(error))
  }
}

///|
/// 通信の結果（応答、または通信の失敗）を探索の結果にする。
pub fn search_result(
  result : Result[ApiResponse, String],
) -> Result[CourseList, SearchFailure] {
  match result {
    Ok(response) => decode_search_response(response)
    Err(message) => Err(FetchFailed(Network(message)))
  }
}
