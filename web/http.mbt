// Web API との通信（mizchi/js_web の fetch）。
//
// rabbita 0.17.0 の通信の部品（@http）は、別オリジンへの Cookie の送信（credentials）・
// 専用ヘッダーの付与・2xx 以外の応答の本文の受け取りに対応していないため、通信だけを
// mizchi/js_web の fetch で行い、rabbita の公開の拡張口（@rabbita.attempt）でコマンドにする。
// 応答は状態コードと本文の文字列（ApiResponse）のまま返し、中身の解釈は decode.mbt の
// 純粋関数で行う（テストできるようにするため）。

///|
/// 画面だけが付ける専用のヘッダー（CSRF 対策。NFR3.5）。
pub let csrf_header_name : String = "X-Requested-With"

///|
/// 専用のヘッダーの値。
pub let csrf_header_value : String = "aidlc-handson-web"

///|
/// 通信の方法。
pub(all) enum HttpMethod {
  Get
  Post
} derive(Eq, Debug)

///|
/// Web API の応答（状態コードと本文）。2xx 以外でも本文を持つ（409 の拒否の理由など）。
pub(all) struct ApiResponse {
  status : Int32
  body : String
} derive(Eq, Debug)

///|
/// 要求に付けるヘッダー。書き込み（POST）には本文の形式と専用ヘッダーを付ける。
pub fn request_headers(verb : HttpMethod) -> Map[String, String] {
  match verb {
    Get => { "Accept": "application/json" }
    Post =>
      {
        "Accept": "application/json",
        "Content-Type": "application/json",
        "X-Requested-With": csrf_header_value,
      }
  }
}

///|
fn method_name(verb : HttpMethod) -> String {
  match verb {
    Get => "GET"
    Post => "POST"
  }
}

///|
/// 要求を送り、状態コードと本文を返す。Cookie を送り受けするため credentials は常に include。
/// 通信そのものの失敗（接続できない・CORS で拒否されたなど）は例外になる。
async fn send(
  verb : HttpMethod,
  url : String,
  body : String?,
) -> ApiResponse raise Error {
  let response = @fetch.fetch(
    url,
    method_=method_name(verb),
    headers=request_headers(verb),
    credentials="include",
    body?=body.map(@js.any),
  )
  let text = response.text()
  { status: response.status, body: text, }
}

///|
/// 要求を送るコマンド。結果（応答、または通信の失敗の説明）を `on_result` でメッセージにする。
pub fn request_cmd(
  verb : HttpMethod,
  url : String,
  body : String?,
  on_result : (Result[ApiResponse, String]) -> @cmd.Cmd,
) -> @cmd.Cmd {
  @rabbita.attempt(
    (result : Result[ApiResponse, Error]) => {
      match result {
        Ok(response) => on_result(Ok(response))
        Err(error) => on_result(Err("\{error}"))
      }
    },
    async fn() { send(verb, url, body) },
  )
}
