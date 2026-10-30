//! エラーの応答（共通の形: `{"error": {"code": ..., "message": ...}}`）。
//! 予期しない失敗では一般的なメッセージだけを返し、内部の情報は出さない（NFR3.10）。

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use purchasing::RejectionReason;
use serde_json::json;

/// Web API が返すエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// 入力の形式が正しくない（400）。
    BadRequest(String),
    /// 書き込みの要求が CSRF 対策の条件を満たさない（403）。
    Forbidden,
    /// 見つからない（404）。
    NotFound(&'static str),
    /// 購入の拒否（409）。
    Rejected(RejectionReason),
    /// 予期しない失敗（500）。詳細はログにだけ出す。
    Internal,
}

/// 拒否の理由ごとの一般的な説明（画面は理由の値から自分の文言を出す）。
pub fn rejection_message(reason: RejectionReason) -> &'static str {
    match reason {
        RejectionReason::EmployeeNotFound | RejectionReason::CompanyNotActive => {
            "ご所属の企業は現在このサービスをご利用いただけません"
        }
        RejectionReason::SessionNotFound => "この日程は見つかりません",
        RejectionReason::Closed => "この日程は受付を終了しました",
        RejectionReason::AlreadyPurchased => "この日程はすでに購入済みです",
        RejectionReason::Full => "この日程は満席になりました",
        RejectionReason::InsufficientBalance => "研修予算の残高が不足しています",
    }
}

/// 予期しない失敗の応答（内部の情報を含めない）。
pub fn internal_error_response() -> Response {
    ApiError::Internal.into_response()
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                json!({ "error": { "code": "invalid-request", "message": message } }),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                json!({ "error": { "code": "forbidden", "message": "この要求は受け付けられません" } }),
            ),
            Self::NotFound(what) => (
                StatusCode::NOT_FOUND,
                json!({ "error": { "code": "not-found", "message": format!("{what}が見つかりません") } }),
            ),
            Self::Rejected(reason) => (
                StatusCode::CONFLICT,
                json!({ "error": {
                    "code": "purchase-rejected",
                    "reason": reason.as_str(),
                    "message": rejection_message(reason),
                } }),
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": { "code": "internal-error", "message": "予期しないエラーが発生しました" } }),
            ),
        };
        (status, Json(body)).into_response()
    }
}
