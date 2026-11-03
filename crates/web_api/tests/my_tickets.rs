//! 購入済みチケットの一覧の受け口 `GET /api/tickets` のテスト（U3。BR13.1、BR13.2、
//! NFR1.1、NFR2.1、NFR2.2、NFR3.1、NFR3.2、NFR10.3）。
//!
//! U1・U2 と同じく、サーバーを起動せず固定の時刻（2026-11-02T10:00:00+09:00）の
//! 本物のメモリのデータストアで組み立てる。初期データでは E005・E006 が 3 枚ずつ持つ。
//! 実行: `cargo test -p web_api --test my_tickets`

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use chrono::DateTime;
use common::*;
use in_memory_store::{Store, seed_state};
use serde_json::Value;
use web_api::{ApiConfig, Ports, purchase_recorder, router, state_reader};

fn tickets_of(response: &TestResponse) -> &Vec<Value> {
    response.json["tickets"].as_array().expect("tickets の並び")
}

fn starts(tickets: &[Value]) -> Vec<DateTime<chrono::FixedOffset>> {
    tickets
        .iter()
        .map(|t| DateTime::parse_from_rfc3339(t["startsAt"].as_str().unwrap()).unwrap())
        .collect()
}

#[tokio::test]
async fn list_has_only_own_tickets_nearest_first_with_all_fields() {
    let api = TestApi::new();
    let response = api.get("/api/tickets", Some("E005")).await;
    assert_eq!(response.status, StatusCode::OK);
    let tickets = tickets_of(&response);
    assert_eq!(tickets.len(), 3);
    // AC4.2.1: 開催日時の近い順（初期データの日程はすべてこれから）
    assert!(starts(tickets).windows(2).all(|w| w[0] <= w[1]));
    for ticket in tickets {
        // BR13.2: 開いたときの 7 項目と表示用の項目・受講の状態を含む（U1 の詳細と同じ項目名）
        for key in [
            "ticketId",
            "ticketNumber",
            "employeeName",
            "courseTitle",
            "startsAt",
            "endsAt",
            "venue",
            "deliveryFormat",
            "purchasedAt",
            "paidPoints",
            "attendance",
        ] {
            assert!(ticket.get(key).is_some(), "{key} がない");
        }
        assert_eq!(ticket["employeeName"], "伊藤 さくら");
        assert_eq!(ticket["attendance"], "upcoming");
        assert!(ticket["startsAt"].as_str().unwrap().ends_with("+09:00"));
    }
    // 他人（E006）のチケットを含めない（BR12.1）
    let own_ids: Vec<String> = api
        .state()
        .tickets
        .iter()
        .filter(|t| t.employee_id.as_str() == "E005")
        .map(|t| t.id.as_str().to_owned())
        .collect();
    assert!(
        tickets
            .iter()
            .all(|t| own_ids.contains(&t["ticketId"].as_str().unwrap().to_owned()))
    );
}

#[tokio::test]
async fn employee_given_in_query_is_ignored() {
    // NFR3.1: クエリで別の従業員を指定しても、Cookie の従業員の一覧のまま
    let api = TestApi::new();
    let plain = api.get("/api/tickets", Some("E005")).await;
    let with_query = api.get("/api/tickets?employeeId=E006", Some("E005")).await;
    assert_eq!(with_query.status, StatusCode::OK);
    assert_eq!(plain.json, with_query.json);
    let none = api.get("/api/tickets?employeeId=E005", Some("E001")).await;
    assert!(tickets_of(&none).is_empty());
}

#[tokio::test]
async fn no_tickets_unknown_employee_and_no_cookie_give_empty_list() {
    // AC4.2.2、BR12.1、NFR3.2
    let api = TestApi::new();
    for employee in [Some("E001"), Some("E999"), Some("bad id"), None] {
        let response = api.get("/api/tickets", employee).await;
        assert_eq!(response.status, StatusCode::OK, "{employee:?}");
        assert_eq!(response.json, serde_json::json!({ "tickets": [] }));
    }
}

#[tokio::test]
async fn just_purchased_ticket_is_in_the_list() {
    // NFR2.2: 購入の直後に一覧を引くと、いま購入したチケットが含まれる
    let api = TestApi::new();
    let purchased = api.purchase(Some("E001"), "S0001").await;
    assert_eq!(purchased.status, StatusCode::CREATED);
    let response = api.get("/api/tickets", Some("E001")).await;
    let tickets = tickets_of(&response);
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["ticketId"], purchased.json["ticket"]["ticketId"]);
    assert_eq!(
        tickets[0]["ticketNumber"],
        purchased.json["ticket"]["ticketNumber"]
    );
}

#[tokio::test]
async fn listing_does_not_change_any_data() {
    // NFR2.1: 何回引いても状態（残席・残高・チケット）は変わらない
    let api = TestApi::new();
    let before = api.state();
    for _ in 0..5 {
        assert_eq!(
            api.get("/api/tickets", Some("E005")).await.status,
            StatusCode::OK
        );
    }
    assert!(Arc::ptr_eq(&before, &api.state()));
}

#[tokio::test]
async fn missing_session_is_a_generic_500() {
    // NFR10.3: 日程が見つからないチケットはデータの不整合として 500（内部の情報は出さない）
    let mut state = seed_state(fixed_now());
    let broken = state
        .tickets
        .iter()
        .find(|t| t.employee_id.as_str() == "E005")
        .unwrap()
        .session_id
        .clone();
    state.sessions.retain(|s| s.id != broken);
    let store = Store::new(state);
    let ports = Ports {
        read_state: state_reader(store.clone()),
        record_purchase: purchase_recorder(store.clone()),
        now: fixed_now,
    };
    let app = router(ports, &ApiConfig::from_lookup(|_| None).unwrap());
    let api = TestApi { app, store };
    let response = api.get("/api/tickets", Some("E005")).await;
    assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(response.json["error"]["code"], "internal-error");
    assert!(!response.json.to_string().contains(broken.as_str()));
    // 不整合のない従業員の一覧はそのまま返す
    assert_eq!(
        api.get("/api/tickets", Some("E001")).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn list_p95_is_within_200ms() {
    // NFR1.1: 100 回送り、95 パーセンタイルが 200 ms 以内
    let api = TestApi::new();
    let mut samples = Vec::with_capacity(100);
    for i in 0..100 {
        let employee = if i % 2 == 0 { "E005" } else { "E006" };
        let started = Instant::now();
        let response = api.get("/api/tickets", Some(employee)).await;
        samples.push(started.elapsed());
        assert_eq!(response.status, StatusCode::OK);
    }
    samples.sort();
    let p95 = samples[(samples.len() * 95).div_ceil(100) - 1];
    println!(
        "GET /api/tickets: p95 = {:.3} ms",
        p95.as_secs_f64() * 1000.0
    );
    assert!(p95 <= Duration::from_millis(200), "p95 {p95:?}");
}
