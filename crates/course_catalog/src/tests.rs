//! CourseCatalog のテスト。データストアを使わず、固定の値を返すクロージャ（発行済み数）と
//! 固定の時刻を渡して動かす（NFR7.2、NFR8.4）。

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::*;

fn fixed_now() -> DateTime<Utc> {
    // 2026-11-02T10:00:00+09:00
    Utc.with_ymd_and_hms(2026, 11, 2, 1, 0, 0).unwrap()
}

fn course(id: &str, format: DeliveryFormat) -> Course {
    Course {
        id: CourseId::parse(id).unwrap(),
        title: format!("コース {id}"),
        category: Category::Technology,
        summary: "概要".into(),
        description: "説明".into(),
        fee_points: 25_000,
        duration_minutes: 120,
        delivery_format: format,
    }
}

fn session(id: &str, course_id: &str, starts_at: DateTime<Utc>, capacity: u32) -> Session {
    Session {
        id: SessionId::parse(id).unwrap(),
        course_id: CourseId::parse(course_id).unwrap(),
        starts_at,
        ends_at: starts_at + Duration::hours(2),
        venue: None,
        capacity,
    }
}

/// 日程 ID と発行済み数の組から、発行済み数を返す関数を作る（データストアの代わり）。
fn issued_from(pairs: &[(&str, u32)]) -> impl Fn(&SessionId) -> u32 {
    let pairs: Vec<(String, u32)> = pairs.iter().map(|(id, n)| (id.to_string(), *n)).collect();
    move |id: &SessionId| {
        pairs
            .iter()
            .find(|(sid, _)| sid == id.as_str())
            .map_or(0, |(_, n)| *n)
    }
}

#[test]
fn id_formats_are_validated() {
    assert!(CourseId::parse("C001").is_ok());
    assert!(SessionId::parse("S0001").is_ok());
    for bad in ["C01", "C0001", "c001", "X001", "C00a", "", "Ｃ001", " C001"] {
        let err = CourseId::parse(bad).unwrap_err();
        assert_eq!(err.kind, "courseId", "{bad:?} は不正なはず");
    }
    for bad in ["S001", "S00001", "s0001", "S000x"] {
        assert!(SessionId::parse(bad).is_err(), "{bad:?} は不正なはず");
    }
}

#[test]
fn session_is_closed_exactly_at_start_time() {
    let now = fixed_now();
    let at_start = session("S0001", "C001", now, 5);
    let just_after = session("S0002", "C001", now + Duration::seconds(1), 5);
    assert_eq!(session_status(&at_start, 0, now), SessionStatus::Closed);
    assert_eq!(
        session_status(&just_after, 0, now),
        SessionStatus::Available
    );
}

#[test]
fn past_full_session_is_closed_and_zero_remaining_is_full() {
    let now = fixed_now();
    let past = session("S0001", "C001", now - Duration::days(1), 2);
    let future = session("S0002", "C001", now + Duration::days(1), 2);
    assert_eq!(session_status(&past, 2, now), SessionStatus::Closed);
    assert_eq!(session_status(&future, 2, now), SessionStatus::Full);
    assert_eq!(session_status(&future, 1, now), SessionStatus::Available);
    assert_eq!(remaining_seats(&future, 1), 1);
    // 発行済み数が定員を超えても残席は 0 未満にならない（BR2.1）
    assert_eq!(remaining_seats(&future, 5), 0);
}

#[test]
fn next_session_date_includes_full_sessions_and_open_count_excludes_them() {
    let now = fixed_now();
    let courses = vec![course("C001", DeliveryFormat::Online)];
    let sessions = vec![
        session("S0001", "C001", now + Duration::days(3), 2), // 満席
        session("S0002", "C001", now + Duration::days(7), 3), // 空席あり
        session("S0003", "C001", now - Duration::days(1), 3), // 過去
    ];
    let rows = list_courses(&courses, &sessions, issued_from(&[("S0001", 2)]), now);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].next_session_starts_at,
        Some(now + Duration::days(3))
    );
    assert_eq!(rows[0].open_session_count, 1);
}

#[test]
fn default_order_is_next_date_then_course_id_and_unscheduled_last() {
    let now = fixed_now();
    let courses = vec![
        course("C004", DeliveryFormat::Online),
        course("C003", DeliveryFormat::Online),
        course("C002", DeliveryFormat::Online),
        course("C001", DeliveryFormat::Online),
        course("C005", DeliveryFormat::Online),
    ];
    let same_day = now + Duration::days(5);
    let sessions = vec![
        session("S0001", "C003", same_day, 5),
        session("S0002", "C002", same_day, 5),
        session("S0003", "C004", now + Duration::days(1), 5),
        session("S0004", "C001", now - Duration::days(2), 5), // 過去だけ → 予定なし
    ];
    let rows = list_courses(&courses, &sessions, |_: &SessionId| 0, now);
    let order: Vec<&str> = rows.iter().map(|r| r.course.id.as_str()).collect();
    assert_eq!(order, ["C004", "C002", "C003", "C001", "C005"]);
    assert_eq!(rows[3].next_session_starts_at, None);
    assert_eq!(rows[4].open_session_count, 0);
}

#[test]
fn course_detail_returns_sessions_in_start_order_with_status() {
    let now = fixed_now();
    let courses = vec![
        course("C001", DeliveryFormat::Online),
        course("C002", DeliveryFormat::Online),
    ];
    let sessions = vec![
        session("S0002", "C001", now + Duration::days(9), 3),
        session("S0001", "C001", now + Duration::days(2), 1),
        session("S0003", "C002", now + Duration::days(1), 1),
    ];
    let detail = course_detail(
        &courses,
        &sessions,
        &CourseId::parse("C001").unwrap(),
        issued_from(&[("S0001", 1)]),
        now,
    )
    .expect("コースがあるはず");
    let ids: Vec<&str> = detail
        .sessions
        .iter()
        .map(|v| v.session.id.as_str())
        .collect();
    assert_eq!(ids, ["S0001", "S0002"]);
    assert_eq!(detail.sessions[0].status, SessionStatus::Full);
    assert_eq!(detail.sessions[1].remaining_seats, 3);
    assert_eq!(detail.sessions[1].status, SessionStatus::Available);
}

#[test]
fn course_detail_of_unknown_course_is_none() {
    let courses = vec![course("C001", DeliveryFormat::Online)];
    let found = course_detail(
        &courses,
        &[],
        &CourseId::parse("C999").unwrap(),
        |_: &SessionId| 0,
        fixed_now(),
    );
    assert!(found.is_none());
}

#[test]
fn validation_rejects_venue_mismatch_and_bad_times() {
    let now = fixed_now();
    let online = course("C001", DeliveryFormat::Online);
    let venue = course("C002", DeliveryFormat::Venue);
    let mut ok = session("S0001", "C001", now, 3);
    assert!(validate_course(&online).is_ok());
    assert!(validate_session(&ok, &online).is_ok());
    // 会場の形式なのに会場名がない
    let mut no_venue = session("S0002", "C002", now, 3);
    assert!(validate_session(&no_venue, &venue).is_err());
    no_venue.venue = Some("東京本社".into());
    assert!(validate_session(&no_venue, &venue).is_ok());
    // 終了が開始より前
    ok.ends_at = ok.starts_at - Duration::minutes(1);
    assert!(validate_session(&ok, &online).is_err());
}
