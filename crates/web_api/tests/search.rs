//! コース一覧の探索の受け口のテスト（U2）。サーバーを起動せず `oneshot` で呼ぶ（U1 と同じ）。
//! 本物のメモリのデータストアを固定の時刻（2026-11-02T10:00:00+09:00）で組み立てて使う。
//! 実行: `cargo test -p web_api --test search -- --nocapture`（性能の測定値も表示される）

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use common::*;

const P95_LIMIT: Duration = Duration::from_millis(200);

/// クエリの値を URL 用にエンコードする（英数字と `-._~` 以外を UTF-8 のバイトごとに %XX にする）。
fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// 名前と値の組から `/api/courses?...` のパスを作る。
fn courses_path(params: &[(&str, &str)]) -> String {
    let query: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{k}={}", encode(v)))
        .collect();
    format!("/api/courses?{}", query.join("&"))
}

fn ids(response: &TestResponse) -> Vec<String> {
    response.json["courses"]
        .as_array()
        .expect("courses がある")
        .iter()
        .map(|c| c["courseId"].as_str().unwrap().to_owned())
        .collect()
}

fn error_fields(response: &TestResponse) -> Vec<String> {
    response.json["error"]["fields"]
        .as_array()
        .expect("fields がある")
        .iter()
        .map(|f| f["field"].as_str().unwrap().to_owned())
        .collect()
}

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

#[tokio::test]
async fn conditioned_list_returns_matching_rows_with_count() {
    let api = TestApi::new();
    // AC1.2.1・AC1.2.3: キーワードは全角・小文字でも同じ結果
    let ai = api.get(&courses_path(&[("q", "AI")]), None).await;
    assert_eq!(ai.status, StatusCode::OK);
    assert_eq!(ids(&ai), ["C007"]);
    assert_eq!(ai.json["count"], 1);
    for variant in ["ＡＩ", "ai"] {
        let response = api.get(&courses_path(&[("q", variant)]), None).await;
        assert_eq!(ids(&response), ids(&ai), "{variant}");
    }
    // AC1.3.1: カテゴリ。行の形は U1 と同じ
    let tech = api
        .get(&courses_path(&[("category", "technology")]), None)
        .await;
    let rows = tech.json["courses"].as_array().unwrap();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|r| r["category"] == "technology"));
    assert_eq!(tech.json["count"].as_u64().unwrap() as usize, rows.len());
    // AC1.2.2・AC1.9.1: 複数の語は AND で、合うものがなければ 0 件（200）
    let none = api.get(&courses_path(&[("q", "AI 入門")]), None).await;
    assert_eq!(none.status, StatusCode::OK);
    assert_eq!(none.json["count"], 0);
}

#[tokio::test]
async fn available_period_and_sort_follow_the_rules_on_seed_data() {
    let api = TestApi::new();
    // 満席だけの C014、受付終了の日程だけの C012、日程のない C013 は空席ありのみで出ない
    let available = ids(&api.get(&courses_path(&[("available", "true")]), None).await);
    for excluded in ["C012", "C013", "C014"] {
        assert!(!available.contains(&excluded.to_owned()), "{excluded}");
    }
    assert!(available.contains(&"C001".to_owned()));
    // 開始日 = 終了日（S0001 の 2026-11-09）: その日に開催開始する空席のある日程を持つコース
    let same_day = api
        .get(
            &courses_path(&[("from", "2026-11-09"), ("to", "2026-11-09")]),
            None,
        )
        .await;
    assert_eq!(ids(&same_day), ["C001"]);
    // 並べ替え: 受講料の安い順の先頭、コース名順（よみがなの五十音順）の先頭
    let fee = ids(&api.get(&courses_path(&[("sort", "fee-asc")]), None).await);
    assert_eq!(fee[0], "C011");
    let name = ids(&api.get(&courses_path(&[("sort", "name")]), None).await);
    assert_eq!(&name[..3], ["C005", "C010", "C013"]);
    // 開催日が近い順: 直近の開催日時が同じ C002 と C014 は ID 順で並ぶ
    let by_date = ids(&api.get(&courses_path(&[("sort", "date")]), None).await);
    let c002 = by_date.iter().position(|id| id == "C002").unwrap();
    assert_eq!(by_date[c002 + 1], "C014");
}

#[tokio::test]
async fn each_invalid_condition_returns_400_with_its_field() {
    let api = TestApi::new();
    let cases = [
        (("category", "cooking"), "category"),
        (("available", "yes"), "availableOnly"),
        (("from", "2026/11/09"), "periodFrom"),
        (("to", "2026-02-30"), "periodTo"),
        (("sort", "price"), "sort"),
    ];
    for ((name, value), field) in cases {
        let response = api.get(&courses_path(&[(name, value)]), None).await;
        assert_eq!(response.status, StatusCode::BAD_REQUEST, "{name}={value}");
        assert_eq!(response.json["error"]["code"], "invalid-search-condition");
        assert_eq!(error_fields(&response), [field]);
        assert!(response.json.get("courses").is_none(), "結果は返さない");
    }
    let reversed = api
        .get(
            &courses_path(&[("from", "2026-11-10"), ("to", "2026-11-09")]),
            None,
        )
        .await;
    assert_eq!(reversed.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_fields(&reversed), ["period"]);
}

#[tokio::test]
async fn all_invalid_conditions_are_listed_without_echoing_values() {
    let api = TestApi::new();
    let response = api
        .get(
            &courses_path(&[
                ("q", "研修"),
                ("category", "<script>x</script>"),
                ("available", "maybe"),
                ("from", "x"),
                ("to", "y"),
                ("sort", "zzz"),
            ]),
            None,
        )
        .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        error_fields(&response),
        [
            "category",
            "availableOnly",
            "periodFrom",
            "periodTo",
            "sort"
        ]
    );
    // 受け取った値そのものは応答に含めない（NFR3.6）
    let body = response.json.to_string();
    for echoed in ["<script>", "maybe", "zzz", "研修"] {
        assert!(!body.contains(echoed), "{echoed} が応答にある");
    }
    for field in response.json["error"]["fields"].as_array().unwrap() {
        assert!(field["reason"].as_str().is_some_and(|r| !r.is_empty()));
    }
}

#[tokio::test]
async fn keyword_never_returns_400_even_when_huge() {
    let api = TestApi::new();
    // HTML の記号も、数万文字のキーワードも拒否しない（BR8.1、NFR3.2）
    let tag = api.get(&courses_path(&[("q", "<b>研修</b>")]), None).await;
    assert_eq!(tag.status, StatusCode::OK);
    let huge = "研修 ".repeat(5) + &"x".repeat(40_000);
    let started = Instant::now();
    let response = api.get(&courses_path(&[("q", &huge)]), None).await;
    let elapsed = started.elapsed();
    assert_eq!(response.status, StatusCode::OK);
    // 先頭 200 文字（「研修」と「xxx…」）で探すので、「研修」を含み「xxx…」を含まない → 0 件
    assert_eq!(response.json["count"], 0);
    println!(
        "数万文字のキーワード: {:.3} ms",
        elapsed.as_secs_f64() * 1000.0
    );
    assert!(elapsed <= P95_LIMIT, "{elapsed:?}");
}

#[tokio::test]
async fn unconditioned_list_is_unchanged_from_u1() {
    let api = TestApi::new();
    let plain = api.get("/api/courses", None).await;
    assert_eq!(plain.status, StatusCode::OK);
    // 既定の値だけの条件・知らない名前のクエリでも、U1 の一覧と同じ応答
    for path in [
        "/api/courses?sort=date",
        "/api/courses?available=false&q=",
        "/api/courses?utm_source=mail",
    ] {
        let response = api.get(path, None).await;
        assert_eq!(response.status, StatusCode::OK, "{path}");
        assert_eq!(response.json, plain.json, "{path}");
    }
    assert_eq!(plain.json["count"], 14);
}

#[tokio::test]
async fn search_does_not_change_any_data() {
    let api = TestApi::new();
    let before = api.state();
    for path in [
        courses_path(&[("q", "入門"), ("available", "true")]),
        courses_path(&[("from", "2026-11-01"), ("sort", "name")]),
        courses_path(&[("category", "bad")]),
    ] {
        api.get(&path, Some("E001")).await;
    }
    let after = api.state();
    assert!(
        Arc::ptr_eq(&before, &after),
        "探索で状態を置き換えない（NFR2.1）"
    );
}

#[tokio::test]
async fn search_p95_with_varied_conditions_is_within_200ms() {
    let api = TestApi::new();
    let keywords = [
        "",
        "入門",
        "AI",
        "まねじめんと",
        "ﾃﾞｰﾀ 分析",
        "研修 リーダー",
    ];
    let categories = [
        "",
        "management",
        "technology",
        "business-skills",
        "compliance",
    ];
    let sorts = ["date", "fee-asc", "fee-desc", "name"];
    let periods = [
        ("", ""),
        ("2026-11-03", ""),
        ("", "2026-11-30"),
        ("2026-11-05", "2026-12-31"),
    ];
    let mut samples = Vec::with_capacity(100);
    for i in 0..100 {
        let (from, to) = periods[i % periods.len()];
        let available = if i % 2 == 0 { "true" } else { "false" };
        let path = courses_path(&[
            ("q", keywords[i % keywords.len()]),
            ("category", categories[i % categories.len()]),
            ("available", available),
            ("from", from),
            ("to", to),
            ("sort", sorts[i % sorts.len()]),
        ]);
        let started = Instant::now();
        let response = api.get(&path, None).await;
        samples.push(started.elapsed());
        assert_eq!(response.status, StatusCode::OK, "{path}");
        let count = response.json["count"].as_u64().unwrap() as usize;
        assert_eq!(
            count,
            response.json["courses"].as_array().map_or(0, Vec::len)
        );
    }
    let value = p95(samples);
    println!(
        "GET /api/courses（探索）: p95 = {:.3} ms",
        value.as_secs_f64() * 1000.0
    );
    assert!(value <= P95_LIMIT, "p95 {value:?}");
}
