// 表示の形を整える純粋関数（BR7.10: 日時は日本時間、ポイントは 3 桁区切りに「pt」）。
// Web API は日時を日本時間のタイムゾーン付きの形式（例: `2026-11-09T13:00:00+09:00`）で返すため、
// 文字列から年月日と時刻を読み取って表示する。形が違うときは受け取った文字列をそのまま出す。

///|
/// ポイントを「60,000 pt」の形にする。
pub fn format_points(points : Int32) -> String {
  "\{group_digits(points)} pt"
}

///|
/// 整数を 3 桁ごとにカンマで区切る。
fn group_digits(value : Int32) -> String {
  let digits = value.abs().to_string()
  let builder = StringBuilder()
  if value < 0 {
    builder.write_char('-')
  }
  let length = digits.length()
  let mut index = 0
  for c in digits {
    if index > 0 && (length - index) % 3 == 0 {
      builder.write_char(',')
    }
    builder.write_char(c)
    index += 1
  }
  builder.to_string()
}

///|
/// 日本時間の日時（読み取った値）。
struct JstDateTime {
  year : Int32
  month : Int32
  day : Int32
  hour : Int32
  minute : Int32
} derive(Eq, Debug)

///|
/// `YYYY-MM-DDTHH:MM:SS+09:00` の形を読み取る。日本時間（+09:00）でなければ None。
fn parse_jst(text : String) -> JstDateTime? {
  let chars = text.to_array()
  guard chars.length() == 25 && text.has_suffix("+09:00") else { return None }
  guard chars[4] == '-' &&
    chars[7] == '-' &&
    chars[10] == 'T' &&
    chars[13] == ':' else {
    return None
  }
  let year = read_number(chars, 0, 4)
  let month = read_number(chars, 5, 2)
  let day = read_number(chars, 8, 2)
  let hour = read_number(chars, 11, 2)
  let minute = read_number(chars, 14, 2)
  match (year, month, day, hour, minute) {
    (Some(year), Some(month), Some(day), Some(hour), Some(minute)) =>
      Some({ year, month, day, hour, minute, })
    _ => None
  }
}

///|
/// `chars[start]` から `length` 文字の 10 進数を読む。数字でない文字があれば None。
fn read_number(chars : Array[Char], start : Int32, length : Int32) -> Int32? {
  let mut value = 0
  for i in start..<(start + length) {
    let c = chars[i]
    guard c >= '0' && c <= '9' else { return None }
    value = value * 10 + (c.to_int() - '0'.to_int())
  }
  Some(value)
}

///|
/// 曜日（日本語の 1 文字）。Sakamoto の方法でグレゴリオ暦の曜日を求める。
fn weekday_label(year : Int32, month : Int32, day : Int32) -> String {
  let offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4]
  let y = if month < 3 { year - 1 } else { year }
  let index = (y + y / 4 - y / 100 + y / 400 + offsets[month - 1] + day) % 7
  ["日", "月", "火", "水", "木", "金", "土"][index]
}

///|
fn two_digits(value : Int32) -> String {
  if value < 10 {
    "0\{value}"
  } else {
    value.to_string()
  }
}

///|
fn date_label(at : JstDateTime) -> String {
  "\{at.year}年\{at.month}月\{at.day}日（\{weekday_label(at.year, at.month, at.day)}）"
}

///|
fn time_label(at : JstDateTime) -> String {
  "\{two_digits(at.hour)}:\{two_digits(at.minute)}"
}

///|
/// 日時を「2026年11月5日（木）13:00」の形にする。
pub fn format_datetime(text : String) -> String {
  match parse_jst(text) {
    Some(at) => "\{date_label(at)}\{time_label(at)}"
    None => text
  }
}

///|
/// 日付を「2026年11月5日（木）」の形にする。
pub fn format_date(text : String) -> String {
  match parse_jst(text) {
    Some(at) => date_label(at)
    None => text
  }
}

///|
/// 開催日時を「2026年11月5日（木）13:00〜17:00」の形にする。日をまたぐときは終わりの日付も出す。
pub fn format_session_time(starts_at : String, ends_at : String) -> String {
  match (parse_jst(starts_at), parse_jst(ends_at)) {
    (Some(start), Some(end)) => {
      let same_day = start.year == end.year &&
        start.month == end.month &&
        start.day == end.day
      let end_label = if same_day {
        time_label(end)
      } else {
        "\{date_label(end)}\{time_label(end)}"
      }
      "\{date_label(start)}\{time_label(start)}〜\{end_label}"
    }
    _ => "\{starts_at}〜\{ends_at}"
  }
}

///|
/// カテゴリの表示名。
pub fn category_label(category : String) -> String {
  match category {
    "management" => "マネジメント"
    "technology" => "テクノロジー"
    "business-skills" => "ビジネススキル"
    "compliance" => "コンプライアンス"
    other => other
  }
}

///|
/// 受講の形式の表示名。
pub fn delivery_format_label(format : String) -> String {
  match format {
    "online" => "オンライン"
    "venue" => "会場"
    other => other
  }
}

///|
/// 会場または形式（会場がなければ形式の表示名）。
pub fn venue_label(venue : String?, delivery_format : String) -> String {
  match venue {
    Some(name) => name
    None => delivery_format_label(delivery_format)
  }
}

///|
/// 契約状態の表示名（色に頼らず文字で示す。NFR5.3）。
pub fn contract_status_label(status : String) -> String {
  match status {
    "active" => "契約中"
    "ended" => "契約終了"
    other => other
  }
}

///|
/// 日程の表示用の状態の文字（BR2.5）。
pub fn session_status_label(status : SessionStatus) -> String {
  match status {
    Available => "購入可"
    Full => "満席"
    Closed => "受付終了"
    Purchased => "購入済み"
    OtherStatus(other) => other
  }
}

///|
/// 所要時間を「4 時間」「1 時間 30 分」「45 分」の形にする。
pub fn format_duration(minutes : Int32) -> String {
  let hours = minutes / 60
  let rest = minutes % 60
  if hours == 0 {
    "\{rest} 分"
  } else if rest == 0 {
    "\{hours} 時間"
  } else {
    "\{hours} 時間 \{rest} 分"
  }
}
