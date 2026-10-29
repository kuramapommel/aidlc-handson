//! 応答時間のテスト（NFR1.1〜NFR1.3）: 各受け口に 100 回ずつ要求を送り、
//! 応答時間の 95 パーセンタイルが 200 ms 以内であることを確かめる。

mod common;

use std::time::{Duration, Instant};

use axum::http::StatusCode;
use common::*;

const REQUESTS: usize = 100;
const P95_LIMIT: Duration = Duration::from_millis(200);

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    let index = (samples.len() * 95).div_ceil(100) - 1;
    samples[index]
}

/// `make_request(i)` の要求を 100 回送り、95 パーセンタイルを返す。状態コードも確かめる。
async fn measure<F>(api: &TestApi, label: &str, make_request: F, allowed: &[StatusCode]) -> Duration
where
    F: Fn(usize) -> axum::http::Request<axum::body::Body>,
{
    let mut samples = Vec::with_capacity(REQUESTS);
    for i in 0..REQUESTS {
        let request = make_request(i);
        let started = Instant::now();
        let response = api.send(request).await;
        samples.push(started.elapsed());
        assert!(
            allowed.contains(&response.status),
            "{label}: {}",
            response.status
        );
    }
    let value = p95(samples);
    println!("{label}: p95 = {:.3} ms", value.as_secs_f64() * 1000.0);
    value
}

#[tokio::test]
async fn course_list_p95_is_within_200ms() {
    let api = TestApi::new();
    let p = measure(
        &api,
        "GET /api/courses",
        |_| get_request("/api/courses", Some("E001")),
        &[StatusCode::OK],
    )
    .await;
    assert!(p <= P95_LIMIT, "p95 {p:?}");
}

#[tokio::test]
async fn course_detail_p95_is_within_200ms() {
    let api = TestApi::new();
    let p = measure(
        &api,
        "GET /api/courses/{courseId}",
        |i| get_request(&format!("/api/courses/C{:03}", i % 13 + 1), Some("E001")),
        &[StatusCode::OK],
    )
    .await;
    assert!(p <= P95_LIMIT, "p95 {p:?}");
}

#[tokio::test]
async fn purchase_me_and_ticket_p95_are_within_200ms() {
    let api = TestApi::new();
    // 日程と従業員を替えながら購入する（承認と拒否の両方の経路を通る）
    let employees = ["E001", "E002", "E003", "E005", "E006"];
    let sessions = [
        "S0001", "S0004", "S0005", "S0006", "S0009", "S0010", "S0012", "S0013", "S0017", "S0018",
    ];
    let purchase = measure(
        &api,
        "POST /api/purchases",
        |i| {
            let body = format!(r#"{{"sessionId":"{}"}}"#, sessions[i % sessions.len()]);
            post_request(
                "/api/purchases",
                Some(employees[i / 20]),
                &body,
                Some(UI_ORIGIN),
                true,
            )
        },
        &[StatusCode::CREATED, StatusCode::CONFLICT],
    )
    .await;
    assert!(purchase <= P95_LIMIT, "購入の p95 {purchase:?}");

    let me = measure(
        &api,
        "GET /api/me",
        |_| get_request("/api/me", Some("E001")),
        &[StatusCode::OK],
    )
    .await;
    assert!(me <= P95_LIMIT, "操作中の従業員の p95 {me:?}");

    let ticket_id = api
        .state()
        .tickets
        .iter()
        .find(|t| t.employee_id.as_str() == "E001")
        .map(|t| t.id.as_str().to_owned())
        .expect("E001 のチケットがある");
    let path = format!("/api/tickets/{ticket_id}");
    let ticket = measure(
        &api,
        "GET /api/tickets/{ticketId}",
        |_| get_request(&path, Some("E001")),
        &[StatusCode::OK],
    )
    .await;
    assert!(ticket <= P95_LIMIT, "チケット詳細の p95 {ticket:?}");
}
