// Web API に接続できないときの扱い（Build and Test の Loop-back 1）のテスト。
// 通信の失敗の文字列化で例外になっていた不具合の修正と、読み出しの再試行の規則を確かめる。
// 実行（このファイルだけ）: リポジトリのルートで `moon -C web test --target js network_retry_test.mbt`

///|
test "読み出し（GET）は 3 回まで試す（2 回目の失敗までは試し直し、3 回目の失敗で止める）" {
  assert_eq(@web.max_read_attempts, 3)
  assert_true(@web.should_retry(Get, 1))
  assert_true(@web.should_retry(Get, 2))
  assert_false(@web.should_retry(Get, 3))
}

///|
test "書き込み（POST）は二重に送らないよう再試行しない" {
  assert_false(@web.should_retry(Post, 1))
  assert_false(@web.should_retry(Post, 2))
}

///|
test "試し直すまでの待ちは回を重ねるほど長い" {
  assert_eq(@web.retry_delay_ms(1), 300)
  assert_eq(@web.retry_delay_ms(2), 600)
  assert_true(@web.retry_delay_ms(2) > @web.retry_delay_ms(1))
}

///|
test "通信の失敗は決まった文言で、時間をおいて試すよう案内する" {
  let text = @web.fetch_error_text(Network(@web.network_error_message))
  assert_eq(text, @web.network_error_message)
  assert_true(text.contains("時間をおいて再度お試しください"))
}

///|
test "通信の失敗の中身にかかわらず同じ文言を出す（中身を文字列にしない）" {
  assert_eq(
    @web.fetch_error_text(Network("TypeError")),
    @web.fetch_error_text(Network("")),
  )
}

///|
test "通信の失敗の結果は Network として一覧の失敗になる（「再試行」の表示へ）" {
  let result : Result[Array[Int], @web.FetchError] = @web.fetch_result(
    Err(@web.network_error_message),
    _ => Ok([]),
  )
  assert_eq(result, Err(Network(@web.network_error_message)))
}
