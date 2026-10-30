//! 書き込みの要求のなりすまし送信（CSRF）の対策（BR6.3、NFR3.5）。
//!
//! 送信元（`Origin`）が許可した画面のオリジンであり、かつ画面だけが付ける専用のヘッダー
//! （`X-Requested-With: aidlc-handson-web`）がある要求だけを通す。満たさなければ 403 で
//! 拒否し、受け口の処理（状態の変更）には進まない。

use axum::extract::Request;
use axum::http::{HeaderMap, HeaderValue, header::ORIGIN};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::errors::ApiError;

/// 画面だけが付ける専用のヘッダーの名前。
pub const CSRF_HEADER: &str = "x-requested-with";
/// 専用のヘッダーの値。
pub const CSRF_HEADER_VALUE: &str = "aidlc-handson-web";

/// 要求のヘッダーが CSRF 対策の条件を満たすか。
pub fn passes_csrf_check(headers: &HeaderMap, allowed_origin: &HeaderValue) -> bool {
    let origin_ok = headers.get(ORIGIN) == Some(allowed_origin);
    let marker_ok = headers
        .get(CSRF_HEADER)
        .is_some_and(|v| v.as_bytes() == CSRF_HEADER_VALUE.as_bytes());
    origin_ok && marker_ok
}

/// 書き込みの受け口の前に置く処理。条件を満たさなければ 403 を返す。
pub async fn guard(allowed_origin: HeaderValue, request: Request, next: Next) -> Response {
    if passes_csrf_check(request.headers(), &allowed_origin) {
        next.run(request).await
    } else {
        tracing::warn!(
            path = %request.uri().path(),
            "CSRF 対策の条件を満たさない書き込みの要求を拒否しました"
        );
        ApiError::Forbidden.into_response()
    }
}
