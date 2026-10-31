//! TrainingBudget のテスト。

use chrono::{TimeZone, Utc};

use crate::*;

fn entry(employee: &str, kind: EntryKind, points: u32) -> BudgetEntry {
    BudgetEntry {
        id: EntryId(format!("{employee}-{points}")),
        employee: EmployeeRef(employee.into()),
        kind,
        points,
        occurred_at: Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap(),
    }
}

fn consume(employee: &str, points: u32) -> BudgetEntry {
    entry(
        employee,
        EntryKind::Consume {
            ticket: TicketRef("t".into()),
        },
        points,
    )
}

#[test]
fn balance_is_grants_minus_consumptions_per_employee() {
    let entries = vec![
        entry("E001", EntryKind::Grant, 50_000),
        entry("E001", EntryKind::Grant, 10_000),
        consume("E001", 25_000),
        entry("E002", EntryKind::Grant, 20_000),
    ];
    assert_eq!(balance_of(&EmployeeRef("E001".into()), &entries), 35_000);
    assert_eq!(balance_of(&EmployeeRef("E002".into()), &entries), 20_000);
    assert_eq!(balance_of(&EmployeeRef("E999".into()), &entries), 0);
}

#[test]
fn exact_balance_can_pay_and_leaves_zero() {
    assert_eq!(
        check_payment(25_000, 25_000),
        PaymentCheck::Payable { balance_after: 0 }
    );
    assert_eq!(
        check_payment(60_000, 25_000),
        PaymentCheck::Payable {
            balance_after: 35_000
        }
    );
}

#[test]
fn insufficient_balance_reports_shortfall() {
    assert_eq!(
        check_payment(20_000, 25_000),
        PaymentCheck::Insufficient { shortfall: 5_000 }
    );
    assert_eq!(
        check_payment(0, 1),
        PaymentCheck::Insufficient { shortfall: 1 }
    );
}

#[test]
fn consumption_entry_matches_fee_and_is_not_created_when_insufficient() {
    let at = Utc.with_ymd_and_hms(2026, 11, 2, 1, 0, 0).unwrap();
    let created = consumption_entry(
        EntryId("c1".into()),
        EmployeeRef("E001".into()),
        TicketRef("t1".into()),
        25_000,
        25_000,
        at,
    )
    .expect("同額なら作られる");
    assert_eq!(created.points, 25_000);
    assert_eq!(
        created.kind,
        EntryKind::Consume {
            ticket: TicketRef("t1".into())
        }
    );
    assert_eq!(created.occurred_at, at);
    let refused = consumption_entry(
        EntryId("c2".into()),
        EmployeeRef("E001".into()),
        TicketRef("t2".into()),
        25_000,
        24_999,
        at,
    );
    assert!(refused.is_none());
}
