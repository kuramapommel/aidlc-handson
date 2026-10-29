// 内部の補助関数のテスト（表示の形・応答の読み取り・要求のヘッダー）。

///|
test "ポイントは 3 桁区切りに pt を付ける（BR7.10）" {
  assert_eq(format_points(60000), "60,000 pt")
  assert_eq(format_points(0), "0 pt")
  assert_eq(format_points(999), "999 pt")
  assert_eq(format_points(1234567), "1,234,567 pt")
  assert_eq(format_points(-5000), "-5,000 pt")
}

///|
test "開催日時は日本時間で曜日を付けて表示する（BR7.10）" {
  assert_eq(
    format_session_time(
      "2026-11-05T13:00:00+09:00", "2026-11-05T17:00:00+09:00",
    ),
    "2026年11月5日（木）13:00〜17:00",
  )
  assert_eq(
    format_session_time(
      "2026-12-31T22:00:00+09:00", "2027-01-01T01:30:00+09:00",
    ),
    "2026年12月31日（木）22:00〜2027年1月1日（金）01:30",
  )
  assert_eq(
    format_date("2026-11-02T10:00:00+09:00"),
    "2026年11月2日（月）",
  )
  assert_eq(
    format_datetime("2026-03-01T09:05:00+09:00"),
    "2026年3月1日（日）09:05",
  )
}

///|
test "日本時間でない・形が違う日時は受け取った文字列のまま表示する" {
  assert_eq(format_date("2026-11-02T01:00:00Z"), "2026-11-02T01:00:00Z")
  assert_eq(format_datetime("not a date"), "not a date")
}

///|
test "書き込みの要求にだけ専用ヘッダーを付ける（NFR3.5）" {
  assert_eq(
    request_headers(Post).get("X-Requested-With"),
    Some("aidlc-handson-web"),
  )
  assert_eq(request_headers(Post).get("Content-Type"), Some("application/json"))
  assert_eq(request_headers(Get).get("X-Requested-With"), None)
}

///|
test "コースの詳細の応答を読み取る（null の会場と表示用の状態を含む）" {
  let body =
    #|{"course":{"courseId":"C002","title":"1on1 ミーティング実践","category":"management","summary":"s","description":"d","feePoints":18000,"durationMinutes":120,"deliveryFormat":"online"},
    #| "sessions":[{"sessionId":"S0005","startsAt":"2026-11-19T10:00:00+09:00","endsAt":"2026-11-19T12:00:00+09:00","venue":null,"capacity":1,"remainingSeats":0,"status":"purchased"}]}
  guard decode_course_detail({ status: 200, body, }) is Ok(detail) else {
    fail("読み取れるはず")
  }
  assert_eq(detail.course.fee_points, 18000)
  assert_eq(detail.sessions[0].venue, None)
  assert_eq(detail.sessions[0].status, Purchased)
  assert_eq(
    venue_label(detail.sessions[0].venue, detail.course.delivery_format),
    "オンライン",
  )
}

///|
test "404 は見つからない、そのほかの失敗は本文の説明を使い、形が違えば読み取りの失敗にする" {
  let not_found =
    #|{"error":{"code":"not-found","message":"コースが見つかりません"}}
  assert_eq(
    decode_course_detail({ status: 404, body: not_found, }),
    Err(NotFound),
  )
  let bad_request =
    #|{"error":{"code":"invalid-request","message":"courseId の形式が正しくありません"}}
  assert_eq(
    decode_course_detail({ status: 400, body: bad_request, }),
    Err(Http(400, "courseId の形式が正しくありません")),
  )
  assert_eq(decode_me({ status: 500, body: "", }), Err(Http(500, "HTTP 500")))
  assert_true(
    decode_me({ status: 200, body: "{\"balance\":\"x\"}", }) is Err(Decode(_)),
  )
}

///|
test "操作中の従業員が見つからない応答（名前などが null）を読み取る（BR6.2）" {
  let body =
    #|{"employeeId":null,"displayName":null,"companyName":null,"contractStatus":null,"eligibility":"employee-not-found","balance":0}
  guard decode_me({ status: 200, body, }) is Ok(me) else {
    fail("読み取れるはず")
  }
  assert_eq(me.display_name, None)
  assert_eq(me.eligibility, EmployeeNotFound)
}

///|
test "通信そのものの失敗は Network にする" {
  assert_eq(
    fetch_result(Err("TypeError: Failed to fetch"), decode_me),
    Err(Network("TypeError: Failed to fetch")),
  )
}

///|
test "購入の成功の応答から e チケットと購入後の残高を読み取る（BR5.9）" {
  let body =
    #|{"ticket":{"ticketId":"T-1","ticketNumber":"ABCD-EFGH","employeeName":"高橋 遥","courseId":"C001","courseTitle":"チームマネジメント入門","sessionId":"S0001","startsAt":"2026-11-05T13:00:00+09:00","endsAt":"2026-11-05T16:00:00+09:00","venue":"東京本社 研修室A","deliveryFormat":"venue","purchasedAt":"2026-11-02T10:00:00+09:00","paidPoints":25000},"balanceAfter":35000}
  guard purchase_message(0, { status: 201, body, })
    is PurchaseSucceeded(0, ticket, 35000) else {
    fail("成功のメッセージになるはず")
  }
  assert_eq(ticket.ticket_number, "ABCD-EFGH")
  assert_eq(ticket.paid_points, 25000)
}

///|
test "表示名の変換（カテゴリ・契約状態・日程の状態・所要時間）" {
  assert_eq(category_label("business-skills"), "ビジネススキル")
  assert_eq(contract_status_label("ended"), "契約終了")
  assert_eq(session_status_label(Closed), "受付終了")
  assert_eq(format_duration(180), "3 時間")
  assert_eq(format_duration(90), "1 時間 30 分")
  assert_eq(format_duration(45), "45 分")
}
