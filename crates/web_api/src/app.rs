//! 部品の組み立て（コンポジションルート）。
//!
//! 受け口が使うデータの読み書きは「関数の形」（[`Ports`]）で受け取り、起動時に
//! データストアの関数を部分適用して作る（NFR6.3。トレイトのオブジェクトは使わない）。
//! 受け口はクロージャで、ポートの関数を呼んで得た値を [`crate::handlers`] の関数に渡す。

use std::future::ready;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, Request};
use axum::http::{HeaderName, Method, StatusCode, header::CONTENT_TYPE};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use in_memory_store::{Store, StoreError, StoreState};
use purchasing::{PurchaseDecision, PurchaseRequest, TicketId};
use serde_json::json;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::config::ApiConfig;
use crate::csrf::{self, CSRF_HEADER};
use crate::errors::{ApiError, internal_error_response};
use crate::handlers;
use crate::session_cookie::{CookieEmployee, employee_cookie, read_employee};

/// 受け口が使う関数の組。
///
/// - `read_state`: 現在の状態の写しを返す
/// - `record_purchase`: 購入を記録する（記録の直前に決定をやり直す。BR5.4）
/// - `now`: Web API の時計の現在時刻（BR5.7）
#[derive(Clone)]
pub struct Ports<Read, Record, Now> {
    pub read_state: Read,
    pub record_purchase: Record,
    pub now: Now,
}

/// メモリのデータストアの読み出しを、現在の状態の写しを返す関数にする（部分適用）。
pub fn state_reader(store: Store) -> impl Fn() -> Arc<StoreState> + Clone + Send + Sync + 'static {
    move || store.snapshot()
}

/// メモリのデータストアの購入の記録を、要求と時刻だけを受け取る関数にする（部分適用）。
///
/// チケット番号の乱数には、OS の乱数を種にした暗号論的に安全な乱数（`rand::random_range`。
/// スレッドごとの ChaCha 乱数）を、チケット ID には UUID v4 を使う（BR5.6）。
pub fn purchase_recorder(
    store: Store,
) -> impl Fn(&PurchaseRequest, DateTime<Utc>) -> Result<PurchaseDecision, StoreError>
+ Clone
+ Send
+ Sync
+ 'static {
    move |request, at| {
        store.record_purchase(
            request,
            at,
            |n| rand::random_range(0..n),
            || {
                TicketId::parse(&uuid::Uuid::new_v4().to_string())
                    .expect("UUID v4 の文字列はチケット ID の形式に合う")
            },
        )
    }
}

/// 購入の承認・拒否をログに出す（NFR10.2。従業員の表示名は出さない。NFR10.3）。
fn log_decision(request: &PurchaseRequest, decision: &PurchaseDecision) {
    let employee = request.employee_id.as_ref().map_or("-", |id| id.as_str());
    let session = request.session_id.as_str();
    match decision {
        PurchaseDecision::Approved(approval) => tracing::info!(
            ticket_id = %approval.ticket.id,
            employee_id = %employee,
            session_id = %session,
            "購入を承認しました"
        ),
        PurchaseDecision::Rejected(reason) => tracing::info!(
            employee_id = %employee,
            session_id = %session,
            reason = %reason.as_str(),
            "購入を拒否しました"
        ),
    }
}

/// 要求ごとのログの範囲（メソッドとパス）。応答のログと同じ 1 行に出る（NFR10.1）。
fn request_span(request: &Request<Body>) -> Span {
    tracing::info_span!("request", method = %request.method(), path = %request.uri().path())
}

/// 応答のログ（状態コードと処理時間のミリ秒）。
fn log_response(response: &Response, latency: Duration, _span: &Span) {
    tracing::info!(
        status = response.status().as_u16(),
        elapsed_ms = latency.as_millis(),
        "応答しました"
    );
}

/// 画面のオリジンだけを資格情報付きで許可する CORS の層（NFR3.6）。
///
/// `AllowOrigin::list` を使い、要求のオリジンが一致したときだけ許可のヘッダーを返す
/// （固定値の指定では、どのオリジンにも同じ許可のヘッダーが付いてしまうため）。
fn cors_layer(config: &ApiConfig) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::list([config.ui_origin_header()]))
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE, HeaderName::from_static(CSRF_HEADER)])
}

/// 受け口の処理中に異常終了しても、5xx と一般的なメッセージを返して次の要求を受け付ける（NFR2.5）。
fn panic_response(_detail: Box<dyn std::any::Any + Send + 'static>) -> Response {
    tracing::error!("受け口の処理が異常終了しました");
    internal_error_response()
}

/// 部品を組み立てた Router を返す。
pub fn router<Read, Record, Now>(ports: Ports<Read, Record, Now>, config: &ApiConfig) -> Router
where
    Read: Fn() -> Arc<StoreState> + Clone + Send + Sync + 'static,
    Record: Fn(&PurchaseRequest, DateTime<Utc>) -> Result<PurchaseDecision, StoreError>
        + Clone
        + Send
        + Sync
        + 'static,
    Now: Fn() -> DateTime<Utc> + Clone + Send + Sync + 'static,
{
    let Ports {
        read_state: read,
        record_purchase: record,
        now,
    } = ports;

    let list_courses = {
        let (read, now) = (read.clone(), now.clone());
        move |Query(query): Query<Vec<(String, String)>>| {
            ready(course_list_response(&read(), &query, now()))
        }
    };
    let course_detail = {
        let (read, now) = (read.clone(), now.clone());
        move |jar: CookieJar, Path(course_id): Path<String>| {
            let employee = read_employee(&jar);
            ready(
                handlers::course_detail(&read(), &course_id, employee.employee_id(), now())
                    .into_response(),
            )
        }
    };
    let me = {
        let read = read.clone();
        move |jar: CookieJar| ready(me_response(&read(), jar))
    };
    let employees = {
        let read = read.clone();
        move || ready(handlers::employees(&read()).into_response())
    };
    let switch_employee = {
        let read = read.clone();
        move |jar: CookieJar, body: Bytes| ready(switch_response(&read(), jar, &body))
    };
    let purchase = {
        let (read, now) = (read.clone(), now.clone());
        move |jar: CookieJar, body: Bytes| {
            ready(purchase_response(&read, &record, now(), &jar, &body))
        }
    };
    let ticket = {
        let read = read.clone();
        move |jar: CookieJar, Path(ticket_id): Path<String>| {
            let employee = read_employee(&jar);
            ready(handlers::ticket(&read(), &ticket_id, employee.employee_id()).into_response())
        }
    };

    let allowed_origin = config.ui_origin_header();
    let writes = Router::new()
        .route("/api/session/employee", post(switch_employee))
        .route("/api/purchases", post(purchase))
        .route_layer(middleware::from_fn(move |request: Request, next: Next| {
            csrf::guard(allowed_origin.clone(), request, next)
        }));

    Router::new()
        .route(
            "/api/health",
            get(|| ready(axum::Json(json!({ "status": "ok" })))),
        )
        .route("/api/courses", get(list_courses))
        .route("/api/courses/{course_id}", get(course_detail))
        .route("/api/me", get(me))
        .route("/api/employees", get(employees))
        .route("/api/tickets/{ticket_id}", get(ticket))
        .merge(writes)
        .fallback(|| ready(ApiError::NotFound("受け口").into_response()))
        .layer(CatchPanicLayer::custom(panic_response))
        .layer(cors_layer(config))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(request_span)
                .on_request(())
                .on_response(log_response),
        )
}

/// `GET /api/courses`: クエリがなければ U1 と同じ一覧（BR3.3）、あれば探索（U2）。
///
/// ログにはクエリの値（キーワードや日付の文字列）を出さない（NFR10.1、NFR10.2）。
/// 400 は不正な項目名だけを情報レベルで、探索の件数と処理時間は詳細レベルでだけ出す（NFR10.3）。
fn course_list_response(
    state: &StoreState,
    query: &[(String, String)],
    now: DateTime<Utc>,
) -> Response {
    if query.is_empty() {
        return handlers::list_courses(state, now).into_response();
    }
    let started = std::time::Instant::now();
    match handlers::search_courses(state, query, now) {
        Ok(json) => {
            tracing::debug!(
                count = json.count,
                elapsed_us = started.elapsed().as_micros(),
                "コース一覧を探索しました"
            );
            json.into_response()
        }
        Err(error) => {
            if let ApiError::InvalidSearchCondition(errors) = &error {
                let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
                tracing::info!(fields = %fields.join(","), "探索条件が不正です");
            }
            error.into_response()
        }
    }
}

/// `GET /api/me`: Cookie がなければ先頭の従業員の Cookie を発行してその従業員を返す（BR6.2）。
fn me_response(state: &StoreState, jar: CookieJar) -> Response {
    match read_employee(&jar) {
        CookieEmployee::Missing => match handlers::default_employee(state) {
            Some(first) => {
                let body = handlers::me(state, Some(&first));
                (jar.add(employee_cookie(&first)), axum::Json(body)).into_response()
            }
            None => {
                tracing::error!("初期データに従業員がいません");
                internal_error_response()
            }
        },
        present => axum::Json(handlers::me(state, present.employee_id())).into_response(),
    }
}

/// `POST /api/session/employee`: 一覧にいる従業員なら Cookie を発行する（BR6.1）。
fn switch_response(state: &StoreState, jar: CookieJar, body: &[u8]) -> Response {
    match handlers::switch_target(state, body) {
        Ok(target) => {
            let me = handlers::me(state, Some(&target));
            (jar.add(employee_cookie(&target)), axum::Json(me)).into_response()
        }
        Err(error) => error.into_response(),
    }
}

/// `POST /api/purchases`: 購入を記録し、承認なら 201、拒否なら 409 を返す。
fn purchase_response<Read, Record>(
    read: &Read,
    record: &Record,
    now: DateTime<Utc>,
    jar: &CookieJar,
    body: &[u8],
) -> Response
where
    Read: Fn() -> Arc<StoreState>,
    Record: Fn(&PurchaseRequest, DateTime<Utc>) -> Result<PurchaseDecision, StoreError>,
{
    let session_id = match handlers::purchase_session_id(body) {
        Ok(id) => id,
        Err(error) => return error.into_response(),
    };
    let request = PurchaseRequest {
        employee_id: read_employee(jar).employee_id().cloned(),
        session_id,
    };
    match record(&request, now) {
        Ok(decision) => {
            log_decision(&request, &decision);
            match handlers::purchase_result(&read(), &decision) {
                Ok(json) => (StatusCode::CREATED, json).into_response(),
                Err(error) => error.into_response(),
            }
        }
        Err(error) => {
            tracing::error!(error = %error, "購入の記録に失敗しました");
            internal_error_response()
        }
    }
}
