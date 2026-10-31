//! InMemoryStore のテスト。本物のメモリのデータストアを、テストごとに新しく作って使う。

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::{DateTime, Duration, TimeZone, Utc};
use course_catalog::{
    Category, Course, CourseId, DeliveryFormat, Session, SessionId, SessionStatus, list_courses,
    session_status, validate_course, validate_session,
};
use eligibility::{Company, CompanyId, ContractStatus, Employee, EmployeeId};
use purchasing::{
    PurchaseDecision, PurchaseRequest, RejectionReason, TicketId, issued_counter, issued_counts,
};
use training_budget::{BudgetEntry, EmployeeRef, EntryId, EntryKind, balance_of};

use crate::*;

fn fixed_now() -> DateTime<Utc> {
    // 2026-11-02T10:00:00+09:00
    Utc.with_ymd_and_hms(2026, 11, 2, 1, 0, 0).unwrap()
}

/// 呼ぶたびに違う値を返す乱数の代わり（テストどうしで共有しない）。
fn counting_random() -> impl FnMut(usize) -> usize + Send + Clone {
    let counter = Arc::new(AtomicUsize::new(0));
    move |n| counter.fetch_add(5, Ordering::SeqCst) % n
}

/// 呼ぶたびに新しいチケット ID を返す関数を作る。
fn ticket_ids() -> impl Fn() -> TicketId + Send + Sync + Clone {
    let counter = Arc::new(AtomicUsize::new(1000));
    move || {
        let n = counter.fetch_add(1, Ordering::SeqCst);
        TicketId::parse(&format!("00000000-0000-4000-9000-{n:012x}")).unwrap()
    }
}

fn request(employee: &str, session: &str) -> PurchaseRequest {
    PurchaseRequest {
        employee_id: Some(EmployeeId::parse(employee).unwrap()),
        session_id: SessionId::parse(session).unwrap(),
    }
}

/// 1 社・`employees` 名（各 100,000 pt）・定員 `capacity` の日程 1 件だけの状態。
fn contention_state(employees: usize, capacity: u32) -> StoreState {
    let starts_at = fixed_now() + Duration::days(3);
    StoreState {
        companies: vec![Company {
            id: CompanyId::parse("K001").unwrap(),
            name: "株式会社サンプル商事".into(),
            contract_status: ContractStatus::Active,
        }],
        employees: (1..=employees)
            .map(|n| Employee {
                id: EmployeeId::parse(&format!("E{n:03}")).unwrap(),
                display_name: format!("従業員 {n}"),
                company_id: CompanyId::parse("K001").unwrap(),
            })
            .collect(),
        courses: vec![Course {
            id: CourseId::parse("C001").unwrap(),
            title: "チームマネジメント入門".into(),
            category: Category::Management,
            summary: "概要".into(),
            description: "説明".into(),
            fee_points: 25_000,
            duration_minutes: 180,
            delivery_format: DeliveryFormat::Online,
        }],
        sessions: vec![Session {
            id: SessionId::parse("S0001").unwrap(),
            course_id: CourseId::parse("C001").unwrap(),
            starts_at,
            ends_at: starts_at + Duration::hours(3),
            venue: None,
            capacity,
        }],
        budget_entries: (1..=employees)
            .map(|n| BudgetEntry {
                id: EntryId(format!("grant-{n}")),
                employee: EmployeeRef(format!("E{n:03}")),
                kind: EntryKind::Grant,
                points: 100_000,
                occurred_at: fixed_now() - Duration::days(30),
            })
            .collect(),
        tickets: Vec::new(),
    }
}

/// 同じデータストアに、複数のスレッドから同時に購入を送り、決定を集める。
async fn purchase_concurrently(
    store: &Store,
    requests: Vec<PurchaseRequest>,
) -> Vec<PurchaseDecision> {
    let random = counting_random();
    let new_id = ticket_ids();
    let handles: Vec<_> = requests
        .into_iter()
        .map(|req| {
            let (store, random, new_id) = (store.clone(), random.clone(), new_id.clone());
            tokio::task::spawn_blocking(move || {
                store
                    .record_purchase(&req, fixed_now(), random, new_id)
                    .expect("予期しない失敗はない")
            })
        })
        .collect();
    let mut decisions = Vec::new();
    for handle in handles {
        decisions.push(handle.await.expect("タスクが終わる"));
    }
    decisions
}

fn approved_count(decisions: &[PurchaseDecision]) -> usize {
    decisions
        .iter()
        .filter(|d| matches!(d, PurchaseDecision::Approved(_)))
        .count()
}

#[test]
fn approval_adds_ticket_and_consumption_together() {
    let store = Store::new(seed_state(fixed_now()));
    let before = store.snapshot();
    let decision = store
        .record_purchase(
            &request("E001", "S0001"),
            fixed_now(),
            counting_random(),
            ticket_ids(),
        )
        .unwrap();
    let PurchaseDecision::Approved(approval) = decision else {
        panic!("承認のはず: {decision:?}");
    };
    let after = store.snapshot();
    assert_eq!(after.tickets.len(), before.tickets.len() + 1);
    assert_eq!(after.budget_entries.len(), before.budget_entries.len() + 1);
    assert!(after.tickets.contains(&approval.ticket));
    assert!(after.budget_entries.contains(&approval.consumption));
    // 残高 60,000 − 受講料 25,000、残席 3 → 2（AC3.3.1）
    assert_eq!(
        balance_of(&EmployeeRef("E001".into()), &after.budget_entries),
        35_000
    );
    let counts = issued_counts(&after.tickets);
    assert_eq!(
        issued_counter(&counts)(&SessionId::parse("S0001").unwrap()),
        1
    );
}

#[test]
fn rejection_changes_nothing() {
    let store = Store::new(seed_state(fixed_now()));
    let before = store.snapshot();
    let cases = [
        (
            request("E003", "S0001"),
            RejectionReason::InsufficientBalance,
        ),
        (request("E004", "S0001"), RejectionReason::CompanyNotActive),
        (request("E999", "S0001"), RejectionReason::EmployeeNotFound),
        (request("E001", "S0002"), RejectionReason::Full),
        (request("E001", "S0003"), RejectionReason::Closed),
        (request("E001", "S9999"), RejectionReason::SessionNotFound),
    ];
    for (req, expected) in cases {
        let decision = store
            .record_purchase(&req, fixed_now(), counting_random(), ticket_ids())
            .unwrap();
        assert_eq!(decision, PurchaseDecision::Rejected(expected));
    }
    let after = store.snapshot();
    assert!(Arc::ptr_eq(&before, &after), "拒否では状態を置き換えない");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn last_seat_with_two_concurrent_purchases_admits_only_one() {
    let store = Store::new(contention_state(2, 1));
    let decisions = purchase_concurrently(
        &store,
        vec![request("E001", "S0001"), request("E002", "S0001")],
    )
    .await;
    assert_eq!(approved_count(&decisions), 1);
    assert!(decisions.contains(&PurchaseDecision::Rejected(RejectionReason::Full)));
    let state = store.snapshot();
    assert_eq!(state.tickets.len(), 1);
    let consumed = state
        .budget_entries
        .iter()
        .filter(|e| matches!(e.kind, EntryKind::Consume { .. }))
        .count();
    assert_eq!(consumed, 1, "消費の取引も 1 件だけ");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn twenty_concurrent_purchases_never_exceed_capacity_three() {
    let store = Store::new(contention_state(20, 3));
    let requests = (1..=20)
        .map(|n| request(&format!("E{n:03}"), "S0001"))
        .collect();
    let decisions = purchase_concurrently(&store, requests).await;
    assert_eq!(approved_count(&decisions), 3);
    let state = store.snapshot();
    assert_eq!(state.tickets.len(), 3);
    let numbers: HashSet<&str> = state.tickets.iter().map(|t| t.number.as_str()).collect();
    assert_eq!(numbers.len(), 3, "チケット番号は重複しない");
    let rejected_full = decisions
        .iter()
        .filter(|d| **d == PurchaseDecision::Rejected(RejectionReason::Full))
        .count();
    assert_eq!(rejected_full, 17);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn same_employee_concurrent_purchases_issue_one_ticket() {
    let store = Store::new(contention_state(1, 10));
    let requests = (0..10).map(|_| request("E001", "S0001")).collect();
    let decisions = purchase_concurrently(&store, requests).await;
    assert_eq!(approved_count(&decisions), 1);
    let already = decisions
        .iter()
        .filter(|d| **d == PurchaseDecision::Rejected(RejectionReason::AlreadyPurchased))
        .count();
    assert_eq!(already, 9);
    let state = store.snapshot();
    assert_eq!(state.tickets.len(), 1);
    assert_eq!(
        balance_of(&EmployeeRef("E001".into()), &state.budget_entries),
        75_000
    );
}

#[test]
fn seed_data_covers_the_fr7_2_kinds() {
    let now = fixed_now();
    let state = seed_state(now);
    // 企業: 契約中 2 社・契約終了 1 社
    let active = state
        .companies
        .iter()
        .filter(|c| c.contract_status == ContractStatus::Active)
        .count();
    assert_eq!((active, state.companies.len()), (2, 3));
    // 従業員: 先頭は高橋 遥（60,000 pt）、4 名以上、25,000 pt ちょうど・20,000 pt の従業員
    assert!(state.employees.len() >= 4);
    assert_eq!(state.employees[0].display_name, "高橋 遥");
    let balance = |id: &str| balance_of(&EmployeeRef(id.into()), &state.budget_entries);
    assert_eq!(balance("E001"), 60_000);
    let balances: Vec<i64> = state
        .employees
        .iter()
        .map(|e| balance(e.id.as_str()))
        .collect();
    assert!(balances.contains(&25_000) && balances.contains(&20_000));
    assert!(balances.iter().all(|b| *b >= 0));
    // コース: 4 カテゴリ以上・12 件以上、25,000 pt、オンラインと会場
    let categories: HashSet<&str> = state.courses.iter().map(|c| c.category.as_str()).collect();
    assert!(categories.len() >= 4 && state.courses.len() >= 12);
    assert!(state.courses.iter().any(|c| c.fee_points == 25_000));
    assert!(
        state
            .courses
            .iter()
            .any(|c| c.delivery_format == DeliveryFormat::Online)
    );
    assert!(
        state
            .courses
            .iter()
            .any(|c| c.delivery_format == DeliveryFormat::Venue)
    );
    // 日程: 満席・受付終了・残席 1・残席 3、今後の日程がないコース
    let counts = issued_counts(&state.tickets);
    let issued = issued_counter(&counts);
    let statuses: Vec<(SessionStatus, u32)> = state
        .sessions
        .iter()
        .map(|s| {
            (
                session_status(s, issued(&s.id), now),
                s.capacity.saturating_sub(issued(&s.id)),
            )
        })
        .collect();
    assert!(statuses.iter().any(|(st, _)| *st == SessionStatus::Full));
    assert!(statuses.iter().any(|(st, _)| *st == SessionStatus::Closed));
    assert!(statuses.contains(&(SessionStatus::Available, 1)));
    assert!(statuses.contains(&(SessionStatus::Available, 3)));
    let rows = list_courses(&state.courses, &state.sessions, &issued, now);
    assert!(rows.iter().any(|r| r.next_session_starts_at.is_none()));
    // データの形の制約と、初期データのチケットと消費の取引の対応（BR5.3〜BR5.6）
    for course in &state.courses {
        validate_course(course).unwrap();
    }
    for session in &state.sessions {
        let course = state
            .courses
            .iter()
            .find(|c| c.id == session.course_id)
            .unwrap();
        validate_session(session, course).unwrap();
    }
    for ticket in &state.tickets {
        let paired = state.budget_entries.iter().filter(
            |e| matches!(&e.kind, EntryKind::Consume { ticket: t } if t.0 == ticket.id.as_str()),
        );
        assert_eq!(paired.count(), 1);
    }
    // 同じ時刻なら毎回同じデータになる（NFR8.4）
    assert_eq!(seed_state(now), state);
}
