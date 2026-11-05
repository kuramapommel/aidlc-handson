//! 購入済みチケットの一覧のテスト（U3。BR12.1〜BR12.3、NFR8.2〜NFR8.3）。
//!
//! データストアを使わず、チケットは値として、日程とコース・従業員は固定の値を返す
//! クロージャで、現在時刻は固定の値で渡す（NFR7.3）。
//! 実行: `cargo test -p purchasing owned_tickets_tests`

use chrono::{DateTime, Duration, TimeZone, Utc};
use course_catalog::{Category, Course, CourseId, DeliveryFormat, Session, SessionId};
use eligibility::{CompanyId, Employee, EmployeeId};

use crate::*;

/// 2026-11-02T10:00:00+09:00
fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 2, 1, 0, 0).unwrap()
}

fn course() -> Course {
    Course {
        id: CourseId::parse("C001").unwrap(),
        title: "チームマネジメント入門".into(),
        title_reading: "ちーむまねじめんとにゅうもん".into(),
        category: Category::Management,
        summary: "概要".into(),
        description: "説明".into(),
        fee_points: 25_000,
        duration_minutes: 180,
        delivery_format: DeliveryFormat::Online,
    }
}

/// 日程 ID と、現在時刻からの開始のずれ。
fn session(id: &str, starts_in: Duration) -> Session {
    let starts_at = now() + starts_in;
    Session {
        id: SessionId::parse(id).unwrap(),
        course_id: CourseId::parse("C001").unwrap(),
        starts_at,
        ends_at: starts_at + Duration::hours(3),
        venue: None,
        capacity: 10,
    }
}

fn employee(id: &str, name: &str) -> Employee {
    Employee {
        id: EmployeeId::parse(id).unwrap(),
        display_name: name.into(),
        company_id: CompanyId::parse("K001").unwrap(),
    }
}

fn eid(id: &str) -> EmployeeId {
    EmployeeId::parse(id).unwrap()
}

/// チケット（ID の末尾の番号、番号、従業員、日程、購入日時の現在時刻からのずれ）。
fn ticket(ordinal: u32, number: &str, employee: &str, session: &str, bought: Duration) -> Ticket {
    Ticket {
        id: TicketId::parse(&format!("00000000-0000-4000-8000-{ordinal:012}")).unwrap(),
        number: TicketNumber::parse(number).unwrap(),
        employee_id: eid(employee),
        session_id: SessionId::parse(session).unwrap(),
        paid_points: 25_000,
        purchased_at: now() + bought,
    }
}

/// 日程の集まりから「日程 ID → 日程とコース」の関数を作る。
fn session_finder(sessions: Vec<Session>) -> impl Fn(&SessionId) -> Option<(Session, Course)> {
    move |id| {
        sessions
            .iter()
            .find(|s| &s.id == id)
            .map(|s| (s.clone(), course()))
    }
}

/// E001（高橋 遥）と E002（佐藤 健）だけがいる従業員の取得。
fn find_employee(id: &EmployeeId) -> Option<Employee> {
    [employee("E001", "高橋 遥"), employee("E002", "佐藤 健")]
        .into_iter()
        .find(|e| &e.id == id)
}

/// 一覧のチケット番号の並び。
fn numbers(list: &[OwnedTicket]) -> Vec<&str> {
    list.iter().map(|t| t.view.ticket.number.as_str()).collect()
}

fn list_for(
    tickets: &[Ticket],
    sessions: Vec<Session>,
    employee_id: Option<&EmployeeId>,
) -> Result<Vec<OwnedTicket>, OwnedTicketsInconsistency> {
    list_owned_tickets(
        tickets,
        employee_id,
        now(),
        session_finder(sessions),
        find_employee,
    )
}

#[test]
fn two_upcoming_tickets_are_listed_nearest_first_with_display_fields() {
    // AC4.2.1: 2 枚が開催日時の近い順
    let sessions = vec![
        session("S0001", Duration::days(14)),
        session("S0002", Duration::days(3)),
    ];
    let tickets = [
        ticket(1, "AAAA-AAAA", "E001", "S0001", Duration::hours(-2)),
        ticket(2, "BBBB-BBBB", "E001", "S0002", Duration::hours(-1)),
    ];
    let list = list_for(&tickets, sessions, Some(&eid("E001"))).unwrap();
    assert_eq!(numbers(&list), ["BBBB-BBBB", "AAAA-AAAA"]);
    let first = &list[0];
    assert_eq!(first.attendance, Attendance::Upcoming);
    assert_eq!(first.view.employee_name, "高橋 遥");
    assert_eq!(first.view.course.title, "チームマネジメント入門");
    assert_eq!(first.view.session.id.as_str(), "S0002");
    assert_eq!(first.view.ticket.paid_points, 25_000);
}

#[test]
fn other_employees_tickets_are_not_included() {
    // BR12.1
    let sessions = vec![session("S0001", Duration::days(7))];
    let tickets = [
        ticket(1, "AAAA-AAAA", "E002", "S0001", Duration::hours(-2)),
        ticket(2, "BBBB-BBBB", "E001", "S0001", Duration::hours(-1)),
    ];
    let list = list_for(&tickets, sessions.clone(), Some(&eid("E001"))).unwrap();
    assert_eq!(numbers(&list), ["BBBB-BBBB"]);
    let other = list_for(&tickets, sessions, Some(&eid("E002"))).unwrap();
    assert_eq!(numbers(&other), ["AAAA-AAAA"]);
    assert_eq!(other[0].view.employee_name, "佐藤 健");
}

#[test]
fn unknown_or_missing_employee_and_no_tickets_give_empty_list() {
    // BR12.1、AC4.2.2: 存在しない従業員・Cookie なし・チケットなしは 0 件
    let sessions = vec![session("S0001", Duration::days(7))];
    // E009 は従業員の取得で見つからないが、チケットだけが残っている（不整合の手前の場合）
    let tickets = [ticket(1, "AAAA-AAAA", "E009", "S0001", Duration::hours(-1))];
    assert!(
        list_for(&tickets, sessions.clone(), Some(&eid("E009")))
            .unwrap()
            .is_empty()
    );
    assert!(
        list_for(&tickets, sessions.clone(), None)
            .unwrap()
            .is_empty()
    );
    assert!(
        list_for(&tickets, sessions, Some(&eid("E001")))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn attended_tickets_come_after_upcoming_newest_first() {
    // BR12.2: これから（近い順）→ 受講済み（新しい順）
    let sessions = vec![
        session("S0001", Duration::days(-10)),
        session("S0002", Duration::days(5)),
        session("S0003", Duration::days(-2)),
        session("S0004", Duration::days(1)),
    ];
    let tickets = [
        ticket(1, "AAAA-AAAA", "E001", "S0001", Duration::days(-20)),
        ticket(2, "BBBB-BBBB", "E001", "S0002", Duration::days(-20)),
        ticket(3, "CCCC-CCCC", "E001", "S0003", Duration::days(-20)),
        ticket(4, "DDDD-DDDD", "E001", "S0004", Duration::days(-20)),
    ];
    let list = list_for(&tickets, sessions, Some(&eid("E001"))).unwrap();
    assert_eq!(
        numbers(&list),
        ["DDDD-DDDD", "BBBB-BBBB", "CCCC-CCCC", "AAAA-AAAA"]
    );
    let attendance: Vec<Attendance> = list.iter().map(|t| t.attendance).collect();
    assert_eq!(
        attendance,
        [
            Attendance::Upcoming,
            Attendance::Upcoming,
            Attendance::Attended,
            Attendance::Attended
        ]
    );
}

#[test]
fn ticket_starting_exactly_now_is_attended() {
    // BR12.3: 開始日時 ≤ 現在時刻で受講済み（境目）
    assert_eq!(attendance_of(now(), now()), Attendance::Attended);
    assert_eq!(
        attendance_of(now() + Duration::seconds(1), now()),
        Attendance::Upcoming
    );
    assert_eq!(
        attendance_of(now() - Duration::seconds(1), now()),
        Attendance::Attended
    );
    let sessions = vec![
        session("S0001", Duration::zero()),
        session("S0002", Duration::seconds(1)),
    ];
    let tickets = [
        ticket(1, "AAAA-AAAA", "E001", "S0001", Duration::days(-1)),
        ticket(2, "BBBB-BBBB", "E001", "S0002", Duration::days(-1)),
    ];
    let list = list_for(&tickets, sessions, Some(&eid("E001"))).unwrap();
    assert_eq!(numbers(&list), ["BBBB-BBBB", "AAAA-AAAA"]);
    assert_eq!(list[1].attendance, Attendance::Attended);
    assert_eq!(Attendance::Attended.as_str(), "attended");
    assert_eq!(Attendance::Upcoming.as_str(), "upcoming");
}

#[test]
fn same_start_is_ordered_by_purchase_time_then_ticket_number() {
    // BR12.2、NFR6.3: 同じ開始日時は購入日時の早い順、さらにチケット番号の昇順（両グループ）
    let sessions = vec![
        session("S0001", Duration::days(3)),
        session("S0002", Duration::days(3)),
        session("S0003", Duration::days(3)),
        session("S0004", Duration::days(-3)),
        session("S0005", Duration::days(-3)),
    ];
    let tickets = [
        ticket(1, "ZZZZ-ZZZZ", "E001", "S0001", Duration::hours(-1)),
        ticket(2, "MMMM-MMMM", "E001", "S0002", Duration::hours(-5)),
        ticket(3, "AAAA-AAAA", "E001", "S0003", Duration::hours(-1)),
        ticket(4, "QQQQ-QQQQ", "E001", "S0004", Duration::days(-9)),
        ticket(5, "PPPP-PPPP", "E001", "S0005", Duration::days(-9)),
    ];
    let list = list_for(&tickets, sessions, Some(&eid("E001"))).unwrap();
    assert_eq!(
        numbers(&list),
        [
            "MMMM-MMMM",
            "AAAA-AAAA",
            "ZZZZ-ZZZZ",
            "PPPP-PPPP",
            "QQQQ-QQQQ"
        ]
    );
    // 入力の順に依らず同じ並びになる（全順序）
    let mut reversed = tickets.to_vec();
    reversed.reverse();
    let sessions_again = vec![
        session("S0001", Duration::days(3)),
        session("S0002", Duration::days(3)),
        session("S0003", Duration::days(3)),
        session("S0004", Duration::days(-3)),
        session("S0005", Duration::days(-3)),
    ];
    let again = list_for(&reversed, sessions_again, Some(&eid("E001"))).unwrap();
    assert_eq!(numbers(&again), numbers(&list));
}

#[test]
fn ticket_whose_session_is_missing_is_a_data_inconsistency() {
    // 日程（またはコース）が見つからないチケットは不整合として ID を返す
    let sessions = vec![session("S0001", Duration::days(3))];
    let tickets = [
        ticket(1, "AAAA-AAAA", "E001", "S0001", Duration::hours(-1)),
        ticket(2, "BBBB-BBBB", "E001", "S0099", Duration::hours(-1)),
        // 他人のチケットの不整合は、操作中の従業員の一覧には関係しない
        ticket(3, "CCCC-CCCC", "E002", "S0098", Duration::hours(-1)),
    ];
    let error = list_for(&tickets, sessions, Some(&eid("E001"))).unwrap_err();
    assert_eq!(
        error.ticket_ids,
        [TicketId::parse("00000000-0000-4000-8000-000000000002").unwrap()]
    );
    assert!(
        error
            .to_string()
            .contains("00000000-0000-4000-8000-000000000002")
    );
}
