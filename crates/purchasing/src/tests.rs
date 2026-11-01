//! Purchasing のテスト。データストアを使わず、固定のデータと固定の値を返すクロージャ
//! （従業員・日程の取得、乱数）を渡して動かす（NFR7.2、NFR8.3、NFR8.4）。

use chrono::{DateTime, Duration, TimeZone, Utc};
use course_catalog::{
    Category, Course, CourseId, DeliveryFormat, Session, SessionId, SessionStatus,
};
use eligibility::{CompanyId, EligibilityResult, Employee, EmployeeId};
use training_budget::{EntryKind, TicketRef};

use crate::*;

fn fixed_now() -> DateTime<Utc> {
    // 2026-11-02T10:00:00+09:00
    Utc.with_ymd_and_hms(2026, 11, 2, 1, 0, 0).unwrap()
}

fn course(fee: u32) -> Course {
    Course {
        id: CourseId::parse("C001").unwrap(),
        title: "チームマネジメント入門".into(),
        title_reading: "ちーむまねじめんとにゅうもん".into(),
        category: Category::Management,
        summary: "概要".into(),
        description: "説明".into(),
        fee_points: fee,
        duration_minutes: 180,
        delivery_format: DeliveryFormat::Online,
    }
}

fn session(starts_in: Duration, capacity: u32) -> Session {
    let starts_at = fixed_now() + starts_in;
    Session {
        id: SessionId::parse("S0001").unwrap(),
        course_id: CourseId::parse("C001").unwrap(),
        starts_at,
        ends_at: starts_at + Duration::hours(3),
        venue: None,
        capacity,
    }
}

fn employee_id() -> EmployeeId {
    EmployeeId::parse("E001").unwrap()
}

/// 購入できる状態の写し（残席 3・受講料 25,000 pt・残高 60,000 pt）。
fn payable_snapshot() -> PurchaseSnapshot {
    PurchaseSnapshot {
        employee_id: Some(employee_id()),
        eligibility: EligibilityResult::Eligible,
        session_id: SessionId::parse("S0001").unwrap(),
        session: Some((session(Duration::days(7), 3), course(25_000))),
        issued: 0,
        balance: 60_000,
        already_purchased: false,
        now: fixed_now(),
    }
}

fn identity() -> TicketIdentity {
    TicketIdentity {
        ticket_id: TicketId::parse("00000000-0000-4000-8000-000000000001").unwrap(),
        ticket_number: TicketNumber::parse("K7Q2-M9XD").unwrap(),
    }
}

fn decide(snapshot: &PurchaseSnapshot) -> PurchaseDecision {
    decide_purchase(snapshot, identity())
}

/// 決まった並びの添字を順に返す乱数の代わり。
fn sequence(values: Vec<usize>) -> impl FnMut(usize) -> usize {
    let mut position = 0;
    move |_| {
        let value = values[position % values.len()];
        position += 1;
        value
    }
}

#[test]
fn approval_creates_ticket_and_consumption_from_course_fee() {
    let decision = decide(&payable_snapshot());
    let PurchaseDecision::Approved(approval) = decision else {
        panic!("承認のはず: {decision:?}");
    };
    assert_eq!(approval.ticket.employee_id, employee_id());
    assert_eq!(approval.ticket.session_id.as_str(), "S0001");
    assert_eq!(approval.ticket.paid_points, 25_000);
    assert_eq!(approval.ticket.purchased_at, fixed_now());
    assert_eq!(approval.ticket.number.as_str(), "K7Q2-M9XD");
    assert_eq!(approval.consumption.points, 25_000);
    assert_eq!(approval.consumption.employee.0, "E001");
    assert_eq!(
        approval.consumption.kind,
        EntryKind::Consume {
            ticket: TicketRef("00000000-0000-4000-8000-000000000001".into())
        }
    );
    assert_eq!(approval.consumption.occurred_at, fixed_now());
    assert_eq!(approval.balance_after, 35_000);
}

#[test]
fn exact_balance_is_approved_with_zero_after() {
    let snapshot = PurchaseSnapshot {
        balance: 25_000,
        ..payable_snapshot()
    };
    let PurchaseDecision::Approved(approval) = decide(&snapshot) else {
        panic!("残高ちょうどは承認のはず");
    };
    assert_eq!(approval.balance_after, 0);
}

#[test]
fn each_rejection_reason_is_returned() {
    let base = payable_snapshot();
    let cases = [
        (
            PurchaseSnapshot {
                employee_id: None,
                eligibility: EligibilityResult::EmployeeNotFound,
                ..base.clone()
            },
            RejectionReason::EmployeeNotFound,
        ),
        (
            PurchaseSnapshot {
                eligibility: EligibilityResult::CompanyNotActive,
                ..base.clone()
            },
            RejectionReason::CompanyNotActive,
        ),
        (
            PurchaseSnapshot {
                session: None,
                ..base.clone()
            },
            RejectionReason::SessionNotFound,
        ),
        (
            PurchaseSnapshot {
                // 開催開始の時刻ちょうどは受付終了
                session: Some((session(Duration::zero(), 3), course(25_000))),
                ..base.clone()
            },
            RejectionReason::Closed,
        ),
        (
            PurchaseSnapshot {
                already_purchased: true,
                ..base.clone()
            },
            RejectionReason::AlreadyPurchased,
        ),
        (
            PurchaseSnapshot {
                issued: 3,
                ..base.clone()
            },
            RejectionReason::Full,
        ),
        (
            PurchaseSnapshot {
                balance: 20_000,
                ..base.clone()
            },
            RejectionReason::InsufficientBalance,
        ),
    ];
    for (snapshot, expected) in cases {
        assert_eq!(decide(&snapshot), PurchaseDecision::Rejected(expected));
    }
}

#[test]
fn overlapping_reasons_return_only_the_first_in_br5_1_order() {
    let base = payable_snapshot();
    let everything_wrong = PurchaseSnapshot {
        eligibility: EligibilityResult::CompanyNotActive,
        session: Some((session(-Duration::days(1), 3), course(25_000))),
        issued: 3,
        balance: 0,
        already_purchased: true,
        ..base.clone()
    };
    assert_eq!(
        first_rejection(&everything_wrong),
        Some(RejectionReason::CompanyNotActive)
    );
    let closed_and_more = PurchaseSnapshot {
        eligibility: EligibilityResult::Eligible,
        ..everything_wrong.clone()
    };
    assert_eq!(
        first_rejection(&closed_and_more),
        Some(RejectionReason::Closed)
    );
    let purchased_full_poor = PurchaseSnapshot {
        session: Some((session(Duration::days(1), 3), course(25_000))),
        ..closed_and_more
    };
    assert_eq!(
        first_rejection(&purchased_full_poor),
        Some(RejectionReason::AlreadyPurchased)
    );
    let full_and_poor = PurchaseSnapshot {
        already_purchased: false,
        ..purchased_full_poor
    };
    assert_eq!(first_rejection(&full_and_poor), Some(RejectionReason::Full));
}

#[test]
fn paid_points_always_come_from_the_course_fee() {
    // 要求（PurchaseRequest）は金額を持たない型。支払いはコースの受講料だけで決まる（BR5.2）
    let request = PurchaseRequest {
        employee_id: Some(employee_id()),
        session_id: SessionId::parse("S0001").unwrap(),
    };
    let snapshot = PurchaseSnapshot {
        session_id: request.session_id.clone(),
        session: Some((session(Duration::days(2), 3), course(12_345))),
        ..payable_snapshot()
    };
    let PurchaseDecision::Approved(approval) = decide(&snapshot) else {
        panic!("承認のはず");
    };
    assert_eq!(approval.ticket.paid_points, 12_345);
    assert_eq!(approval.consumption.points, 12_345);
}

#[test]
fn ticket_number_has_the_format_and_uses_only_unambiguous_characters() {
    let number = generate_ticket_number(sequence((0..31).collect()), |_| false).unwrap();
    assert_eq!(number.as_str(), "ABCD-EFGH");
    let last =
        generate_ticket_number(sequence(vec![30, 29, 28, 27, 26, 25, 24, 23]), |_| false).unwrap();
    assert_eq!(last.as_str(), "9876-5432");
    for ambiguous in ['0', 'O', '1', 'I', 'L'] {
        assert!(!TICKET_NUMBER_ALPHABET.contains(ambiguous));
    }
    assert!(is_valid_ticket_number("K7Q2-M9XD"));
    for bad in [
        "K7Q2M9XD",
        "K7Q2-M9X0",
        "k7q2-m9xd",
        "K7Q2-M9XDA",
        "K7Q-2M9XD",
    ] {
        assert!(!is_valid_ticket_number(bad), "{bad:?} は不正なはず");
    }
}

#[test]
fn duplicate_ticket_number_is_regenerated() {
    // 1 回目と 2 回目は同じ番号（AAAA-AAAA）、3 回目に別の番号（BBBB-BBBB）を返す乱数
    let mut values = vec![0; 16];
    values.extend(vec![1; 8]);
    let taken = TicketNumber::parse("AAAA-AAAA").unwrap();
    let number = generate_ticket_number(sequence(values), |candidate| candidate == &taken).unwrap();
    assert_eq!(number.as_str(), "BBBB-BBBB");
    // ずっと同じ番号しか出ないときは上限で諦める
    let exhausted = generate_ticket_number(sequence(vec![0]), |candidate| candidate == &taken);
    assert_eq!(exhausted, Err(TicketNumberExhausted));
}

fn ticket(id: &str, employee: &str, session_id: &str) -> Ticket {
    Ticket {
        id: TicketId::parse(id).unwrap(),
        number: TicketNumber::parse("K7Q2-M9XD").unwrap(),
        employee_id: EmployeeId::parse(employee).unwrap(),
        session_id: SessionId::parse(session_id).unwrap(),
        paid_points: 25_000,
        purchased_at: fixed_now(),
    }
}

#[test]
fn issued_counts_and_availability_prefer_purchased() {
    let tickets = vec![
        ticket("00000000-0000-4000-8000-000000000001", "E001", "S0001"),
        ticket("00000000-0000-4000-8000-000000000002", "E002", "S0001"),
        ticket("00000000-0000-4000-8000-000000000003", "E002", "S0002"),
    ];
    let counts = issued_counts(&tickets);
    let count = issued_counter(&counts);
    assert_eq!(count(&SessionId::parse("S0001").unwrap()), 2);
    assert_eq!(count(&SessionId::parse("S0009").unwrap()), 0);
    let mine = purchased_session_ids(&tickets, Some(&employee_id()));
    assert!(mine.contains(&SessionId::parse("S0001").unwrap()));
    assert!(!mine.contains(&SessionId::parse("S0002").unwrap()));
    assert_eq!(
        session_availability(SessionStatus::Full, true),
        SessionAvailability::Purchased
    );
    assert_eq!(
        session_availability(SessionStatus::Closed, false),
        SessionAvailability::Closed
    );
    assert!(has_purchased(
        &tickets,
        &employee_id(),
        &SessionId::parse("S0001").unwrap()
    ));
}

#[test]
fn ticket_detail_is_only_for_its_owner() {
    let tickets = vec![ticket(
        "00000000-0000-4000-8000-000000000001",
        "E001",
        "S0001",
    )];
    let id = TicketId::parse("00000000-0000-4000-8000-000000000001").unwrap();
    let other = EmployeeId::parse("E002").unwrap();
    assert!(find_own_ticket(&tickets, &id, Some(&other)).is_none());
    assert!(find_own_ticket(&tickets, &id, None).is_none());
    let own = find_own_ticket(&tickets, &id, Some(&employee_id())).expect("本人のチケット");
    // 従業員と日程の取得は、データストアの代わりに固定のデータを返すクロージャで渡す
    let view = ticket_view(
        own,
        |id: &EmployeeId| {
            Some(Employee {
                id: id.clone(),
                display_name: "高橋 遥".into(),
                company_id: CompanyId::parse("K001").unwrap(),
            })
        },
        |_: &SessionId| Some((session(Duration::days(7), 3), course(25_000))),
    )
    .expect("詳細を組み立てられる");
    assert_eq!(view.employee_name, "高橋 遥");
    assert_eq!(view.course.title, "チームマネジメント入門");
    assert!(TicketId::parse("not-a-uuid").is_err());
    assert!(TicketId::parse("00000000-0000-4000-8000-00000000000G").is_err());
}
