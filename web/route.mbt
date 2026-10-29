// URL と画面（Route）の相互の変換（純粋関数）。操作中の従業員 ID は URL に含めない（BR6.1）。

///|
/// パス（先頭の `/` はあってもなくてもよい。`?` 以降と `#` 以降は無視）から画面を復元する。
pub fn route_from_path(path : String) -> Route {
  let segments = path_segments(path)
  match segments {
    [] => CourseListPage
    ["courses", course_id] => CourseDetailPage(course_id)
    ["courses", course_id, "sessions", session_id, "confirm"] =>
      PurchaseConfirmPage(course_id, session_id)
    ["tickets", ticket_id] => PurchaseCompletePage(ticket_id)
    _ => NotFoundPage
  }
}

///|
/// 画面の URL のパス。
pub fn route_to_path(route : Route) -> String {
  match route {
    CourseListPage => "/"
    CourseDetailPage(course_id) => "/courses/\{course_id}"
    PurchaseConfirmPage(course_id, session_id) =>
      "/courses/\{course_id}/sessions/\{session_id}/confirm"
    PurchaseCompletePage(ticket_id) => "/tickets/\{ticket_id}"
    NotFoundPage => "/not-found"
  }
}

///|
/// パスを `/` で区切った空でない部分の並び。
fn path_segments(path : String) -> Array[String] {
  let chars : Array[Char] = []
  for c in path {
    if c == '?' || c == '#' {
      break
    }
    chars.push(c)
  }
  let segments : Array[String] = []
  let current = StringBuilder()
  let mut length = 0
  for c in chars {
    if c == '/' {
      if length > 0 {
        segments.push(current.to_string())
        current.reset()
        length = 0
      }
    } else {
      current.write_char(c)
      length += 1
    }
  }
  if length > 0 {
    segments.push(current.to_string())
  }
  segments
}
