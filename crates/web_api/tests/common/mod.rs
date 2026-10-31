//! Web API のテストの共通部品。サーバーを起動せず、`tower::ServiceExt::oneshot` で受け口を呼ぶ。
//! 本物のメモリのデータストアを固定の時刻で組み立て、テストごとに新しく作る。

#![allow(dead_code)]

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use chrono::{DateTime, TimeZone, Utc};
use http_body_util::BodyExt;
use in_memory_store::{Store, StoreState, seed_state};
use serde_json::Value;
use tower::ServiceExt;
use web_api::{ApiConfig, Ports, purchase_recorder, router, state_reader};

pub const UI_ORIGIN: &str = "http://localhost:8080";

/// 2026-11-02T10:00:00+09:00
pub fn fixed_now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 2, 1, 0, 0).unwrap()
}

/// テスト用の Web API と、その中のデータストア。
pub struct TestApi {
    pub app: Router,
    pub store: Store,
}

impl TestApi {
    pub fn new() -> Self {
        let store = Store::new(seed_state(fixed_now()));
        let ports = Ports {
            read_state: state_reader(store.clone()),
            record_purchase: purchase_recorder(store.clone()),
            now: fixed_now,
        };
        let config = ApiConfig::from_lookup(|_| None).unwrap();
        Self {
            app: router(ports, &config),
            store,
        }
    }

    pub fn state(&self) -> Arc<StoreState> {
        self.store.snapshot()
    }

    pub async fn send(&self, request: Request<Body>) -> TestResponse {
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        TestResponse {
            status,
            headers,
            json,
        }
    }

    pub async fn get(&self, path: &str, employee: Option<&str>) -> TestResponse {
        self.send(get_request(path, employee)).await
    }

    /// 画面と同じ条件（Origin と専用ヘッダー）の書き込みの要求を送る。
    pub async fn post(&self, path: &str, employee: Option<&str>, body: &str) -> TestResponse {
        self.send(post_request(path, employee, body, Some(UI_ORIGIN), true))
            .await
    }

    pub async fn purchase(&self, employee: Option<&str>, session_id: &str) -> TestResponse {
        let body = format!(r#"{{"sessionId":"{session_id}"}}"#);
        self.post("/api/purchases", employee, &body).await
    }
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub json: Value,
}

impl TestResponse {
    pub fn set_cookie(&self) -> Option<String> {
        self.headers
            .get(header::SET_COOKIE)
            .map(|v| v.to_str().unwrap().to_owned())
    }

    pub fn reason(&self) -> &str {
        self.json["error"]["reason"].as_str().unwrap_or("")
    }
}

fn with_cookie(
    builder: axum::http::request::Builder,
    employee: Option<&str>,
) -> axum::http::request::Builder {
    match employee {
        Some(id) => builder.header(header::COOKIE, format!("aidlc_employee_id={id}")),
        None => builder,
    }
}

pub fn get_request(path: &str, employee: Option<&str>) -> Request<Body> {
    with_cookie(Request::builder().uri(path), employee)
        .body(Body::empty())
        .unwrap()
}

pub fn post_request(
    path: &str,
    employee: Option<&str>,
    body: &str,
    origin: Option<&str>,
    with_marker: bool,
) -> Request<Body> {
    let mut builder = with_cookie(
        Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json"),
        employee,
    );
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    if with_marker {
        builder = builder.header("X-Requested-With", "aidlc-handson-web");
    }
    builder.body(Body::from(body.to_owned())).unwrap()
}

/// 詳細の応答から、日程の残席数と状態を取り出す。
pub fn session_of(detail: &Value, session_id: &str) -> (u64, String) {
    let session = detail["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["sessionId"] == session_id)
        .unwrap_or_else(|| panic!("{session_id} がない"));
    (
        session["remainingSeats"].as_u64().unwrap(),
        session["status"].as_str().unwrap().to_owned(),
    )
}
