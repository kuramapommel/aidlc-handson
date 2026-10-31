//! Web API の受け口のテスト（応答の形、400・403・404・409、Cookie、CORS、金額の改ざん、5xx）。

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use chrono::{DateTime, Utc};
use common::*;
use in_memory_store::{Store, StoreError, seed_state};
use purchasing::{PurchaseDecision, PurchaseRequest, TicketNumberExhausted};
use web_api::{ApiConfig, Ports, router, state_reader};

#[tokio::test]
async fn health_returns_200() {
    let api = TestApi::new();
    let response = api.get("/api/health", None).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.json["status"], "ok");
}

#[tokio::test]
async fn course_list_has_rows_count_and_default_order() {
    let api = TestApi::new();
    let response = api.get("/api/courses", None).await;
    assert_eq!(response.status, StatusCode::OK);
    let courses = response.json["courses"].as_array().unwrap();
    assert_eq!(
        response.json["count"].as_u64().unwrap() as usize,
        courses.len()
    );
    assert!(courses.len() >= 12);
    for key in [
        "courseId",
        "title",
        "category",
        "summary",
        "feePoints",
        "nextSessionStartsAt",
        "openSessionCount",
    ] {
        assert!(courses[0].get(key).is_some(), "{key} がない");
    }
    // 直近の開催日の昇順で、今後の日程がないコースは最後（BR3.3）
    let dates: Vec<Option<DateTime<Utc>>> = courses
        .iter()
        .map(|c| {
            c["nextSessionStartsAt"]
                .as_str()
                .map(|s| s.parse().unwrap())
        })
        .collect();
    let scheduled: Vec<_> = dates.iter().take_while(|d| d.is_some()).collect();
    assert!(scheduled.windows(2).all(|w| w[0] <= w[1]));
    assert!(dates[scheduled.len()..].iter().all(Option::is_none));
    assert!(
        courses[0]["nextSessionStartsAt"]
            .as_str()
            .unwrap()
            .ends_with("+09:00")
    );
}

#[tokio::test]
async fn course_detail_shows_sessions_with_status_and_errors() {
    let api = TestApi::new();
    let detail = api.get("/api/courses/C001", Some("E001")).await;
    assert_eq!(detail.status, StatusCode::OK);
    assert_eq!(detail.json["course"]["feePoints"], 25_000);
    assert_eq!(detail.json["course"]["deliveryFormat"], "venue");
    assert_eq!(session_of(&detail.json, "S0001"), (3, "available".into()));
    assert_eq!(session_of(&detail.json, "S0002"), (0, "full".into()));
    assert_eq!(session_of(&detail.json, "S0003").1, "closed");
    // 存在しないコースは 404（BR7.9）、形式不正は 400（NFR3.1）
    assert_eq!(
        api.get("/api/courses/C999", None).await.status,
        StatusCode::NOT_FOUND
    );
    let bad = api.get("/api/courses/C01", None).await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad.json["error"]["code"], "invalid-request");
}

#[tokio::test]
async fn me_without_cookie_issues_cookie_for_first_employee() {
    let api = TestApi::new();
    let response = api.get("/api/me", None).await;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.json["employeeId"], "E001");
    assert_eq!(response.json["displayName"], "高橋 遥");
    assert_eq!(response.json["balance"], 60_000);
    assert_eq!(response.json["eligibility"], "eligible");
    let cookie = response.set_cookie().expect("Cookie が発行される");
    assert!(cookie.starts_with("aidlc_employee_id=E001"));
    for attribute in ["HttpOnly", "SameSite=Lax", "Path=/"] {
        assert!(cookie.contains(attribute), "{attribute} がない: {cookie}");
    }
    assert!(!cookie.contains("Secure"));
    // Cookie の従業員がいなくても閲覧には応え、Cookie は変えない（BR6.2）
    let unknown = api.get("/api/me", Some("E999")).await;
    assert_eq!(unknown.status, StatusCode::OK);
    assert_eq!(unknown.json["eligibility"], "employee-not-found");
    assert!(unknown.set_cookie().is_none());
}

#[tokio::test]
async fn switching_employee_changes_cookie_and_rejects_unknown() {
    let api = TestApi::new();
    let list = api.get("/api/employees", None).await;
    assert_eq!(list.json["employees"][0]["employeeId"], "E001");
    assert!(list.json["employees"].as_array().unwrap().len() >= 4);

    let switched = api
        .post(
            "/api/session/employee",
            Some("E001"),
            r#"{"employeeId":"E004"}"#,
        )
        .await;
    assert_eq!(switched.status, StatusCode::OK);
    assert!(
        switched
            .set_cookie()
            .unwrap()
            .starts_with("aidlc_employee_id=E004")
    );
    assert_eq!(switched.json["contractStatus"], "ended");
    assert_eq!(switched.json["eligibility"], "company-not-active");

    let unknown = api
        .post(
            "/api/session/employee",
            Some("E001"),
            r#"{"employeeId":"E099"}"#,
        )
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert!(unknown.set_cookie().is_none());
    let bad_format = api
        .post("/api/session/employee", None, r#"{"employeeId":"X1"}"#)
        .await;
    assert_eq!(bad_format.status, StatusCode::BAD_REQUEST);
    let bad_json = api.post("/api/session/employee", None, "{").await;
    assert_eq!(bad_json.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn purchase_succeeds_and_updates_seats_balance_and_status() {
    let api = TestApi::new();
    let response = api.purchase(Some("E001"), "S0001").await;
    assert_eq!(response.status, StatusCode::CREATED);
    let ticket = &response.json["ticket"];
    assert_eq!(response.json["balanceAfter"], 35_000);
    assert_eq!(ticket["paidPoints"], 25_000);
    assert_eq!(ticket["employeeName"], "高橋 遥");
    assert_eq!(ticket["courseTitle"], "チームマネジメント入門");
    assert!(purchasing::is_valid_ticket_number(
        ticket["ticketNumber"].as_str().unwrap()
    ));
    for key in ["startsAt", "venue", "purchasedAt"] {
        assert!(!ticket[key].is_null(), "{key} がない");
    }
    let detail = api.get("/api/courses/C001", Some("E001")).await;
    assert_eq!(session_of(&detail.json, "S0001"), (2, "purchased".into()));
    let other = api.get("/api/courses/C001", Some("E002")).await;
    assert_eq!(session_of(&other.json, "S0001"), (2, "available".into()));
    assert_eq!(
        api.get("/api/me", Some("E001")).await.json["balance"],
        35_000
    );
}

#[tokio::test]
async fn rejections_return_409_with_reason_and_change_nothing() {
    let api = TestApi::new();
    assert_eq!(
        api.purchase(Some("E001"), "S0001").await.status,
        StatusCode::CREATED
    );
    let before = api.state();
    let cases = [
        (Some("E004"), "S0001", "company-not-active"),
        (Some("E999"), "S0001", "employee-not-found"),
        (None, "S0001", "employee-not-found"),
        (Some("E001"), "S0002", "full"),
        (Some("E001"), "S0003", "closed"),
        (Some("E001"), "S0001", "already-purchased"),
        (Some("E003"), "S0001", "insufficient-balance"),
        (Some("E001"), "S9999", "session-not-found"),
    ];
    for (employee, session, reason) in cases {
        let response = api.purchase(employee, session).await;
        assert_eq!(
            response.status,
            StatusCode::CONFLICT,
            "{employee:?} {session}"
        );
        assert_eq!(response.reason(), reason);
        assert!(response.json["error"]["message"].is_string());
    }
    assert!(
        Arc::ptr_eq(&before, &api.state()),
        "拒否では状態が変わらない"
    );
    // 形式不正は 400
    let bad = api
        .post("/api/purchases", Some("E001"), r#"{"sessionId":"S1"}"#)
        .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    let not_json = api
        .post("/api/purchases", Some("E001"), "sessionId=S0001")
        .await;
    assert_eq!(not_json.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn tampered_amount_in_request_is_ignored() {
    let api = TestApi::new();
    let response = api
        .post(
            "/api/purchases",
            Some("E001"),
            r#"{"sessionId":"S0001","feePoints":1,"paidPoints":1,"amount":1}"#,
        )
        .await;
    assert_eq!(response.status, StatusCode::CREATED);
    assert_eq!(response.json["ticket"]["paidPoints"], 25_000);
    assert_eq!(response.json["balanceAfter"], 35_000);
}

#[tokio::test]
async fn writes_without_csrf_conditions_are_forbidden() {
    let api = TestApi::new();
    let before = api.state();
    let body = r#"{"sessionId":"S0001"}"#;
    let cases = [
        post_request("/api/purchases", Some("E001"), body, None, true),
        post_request(
            "/api/purchases",
            Some("E001"),
            body,
            Some("http://evil.example"),
            true,
        ),
        post_request("/api/purchases", Some("E001"), body, Some(UI_ORIGIN), false),
        post_request(
            "/api/session/employee",
            Some("E001"),
            r#"{"employeeId":"E004"}"#,
            None,
            true,
        ),
    ];
    for request in cases {
        let response = api.send(request).await;
        assert_eq!(response.status, StatusCode::FORBIDDEN);
        assert!(response.set_cookie().is_none());
    }
    assert!(Arc::ptr_eq(&before, &api.state()), "403 では何も変わらない");
}

fn preflight(origin: &str) -> Request<Body> {
    Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/purchases")
        .header(header::ORIGIN, origin)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .header(
            header::ACCESS_CONTROL_REQUEST_HEADERS,
            "content-type,x-requested-with",
        )
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn cors_allows_only_the_ui_origin_with_credentials() {
    let api = TestApi::new();
    let denied = api.send(preflight("http://evil.example")).await;
    assert!(
        denied
            .headers
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );
    let allowed = api.send(preflight(UI_ORIGIN)).await;
    assert_eq!(
        allowed.headers[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        UI_ORIGIN
    );
    assert_eq!(
        allowed.headers[header::ACCESS_CONTROL_ALLOW_CREDENTIALS],
        "true"
    );
}

#[tokio::test]
async fn ticket_detail_is_only_returned_to_its_owner() {
    let api = TestApi::new();
    let purchased = api.purchase(Some("E001"), "S0001").await;
    let ticket_id = purchased.json["ticket"]["ticketId"]
        .as_str()
        .unwrap()
        .to_owned();
    let path = format!("/api/tickets/{ticket_id}");
    let own = api.get(&path, Some("E001")).await;
    assert_eq!(own.status, StatusCode::OK);
    assert_eq!(
        own.json["ticketNumber"],
        purchased.json["ticket"]["ticketNumber"]
    );
    assert_eq!(
        api.get(&path, Some("E002")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(api.get(&path, None).await.status, StatusCode::NOT_FOUND);
    assert_eq!(
        api.get("/api/tickets/not-a-uuid", Some("E001"))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn unexpected_failure_returns_generic_500_and_keeps_serving() {
    // 記録の関数を「必ず失敗する関数」に差し替える（関数の形の差し替え。NFR7.2）
    let store = Store::new(seed_state(fixed_now()));
    let failing = |_: &PurchaseRequest, _: DateTime<Utc>| -> Result<PurchaseDecision, StoreError> {
        Err(StoreError::TicketNumber(TicketNumberExhausted))
    };
    let ports = Ports {
        read_state: state_reader(store.clone()),
        record_purchase: failing,
        now: fixed_now,
    };
    let app = router(ports, &ApiConfig::from_lookup(|_| None).unwrap());
    let api = TestApi { app, store };
    let before = api.state();
    let response = api.purchase(Some("E001"), "S0001").await;
    assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response.json["error"]["code"], "internal-error");
    assert!(
        !response.json.to_string().contains("TicketNumber"),
        "内部の情報を出さない"
    );
    assert!(Arc::ptr_eq(&before, &api.state()));
    assert_eq!(api.get("/api/health", None).await.status, StatusCode::OK);
    assert_eq!(
        api.get("/api/unknown", None).await.status,
        StatusCode::NOT_FOUND
    );
}

#[test]
fn config_defaults_and_validation() {
    let defaults = ApiConfig::from_lookup(|_| None).unwrap();
    assert_eq!(
        (defaults.port, defaults.ui_origin.as_str()),
        (3000, UI_ORIGIN)
    );
    let custom = ApiConfig::from_lookup(|key| match key {
        "API_PORT" => Some("3100".into()),
        "UI_ORIGIN" => Some("http://127.0.0.1:9000".into()),
        _ => None,
    })
    .unwrap();
    assert_eq!(
        (custom.port, custom.ui_origin.as_str()),
        (3100, "http://127.0.0.1:9000")
    );
    assert!(ApiConfig::from_lookup(|k| (k == "API_PORT").then(|| "abc".into())).is_err());
    assert!(ApiConfig::from_lookup(|k| (k == "UI_ORIGIN").then(|| "*".into())).is_err());
    assert!(
        ApiConfig::from_lookup(|k| (k == "UI_ORIGIN").then(|| "http://localhost:8080/".into()))
            .is_err()
    );
}
