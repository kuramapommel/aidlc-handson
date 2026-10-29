// ブラウザで画面（TEA）を起動する薄い入り口。画面の本体は親のパッケージ（aidlc-handson/web）にある。
// Web API のオリジンは、index.html の前に読み込む config.js（scripts/build-web.sh が作る）の
// `globalThis.AIDLC_API_BASE` で変えられる。なければ既定値（http://localhost:3000）を使う。

///|
extern "js" fn configured_api_base() -> String =
  #| () => (typeof globalThis.AIDLC_API_BASE === "string" ? globalThis.AIDLC_API_BASE : "")

///|
fn main {
  let configured = configured_api_base()
  let api_base = if configured == "" {
    @app.default_api_base
  } else {
    configured
  }
  @app.app(api_base~).mount("app")
}
