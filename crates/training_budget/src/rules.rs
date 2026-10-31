//! 残高・支払い可否・不足額・購入後の残高の計算と、消費の取引の生成（BR4.1〜BR4.5）。

use chrono::{DateTime, Utc};

use crate::model::{BudgetEntry, EmployeeRef, EntryId, EntryKind, TicketRef};

/// 残高 = 付与の合計 − 消費の合計（BR4.1）。
///
/// `entries` には 1 人の従業員の取引を渡す（従業員の取引の取得は呼び出し側の関数で行う）。
pub fn balance<'a, I>(entries: I) -> i64
where
    I: IntoIterator<Item = &'a BudgetEntry>,
{
    entries
        .into_iter()
        .map(|e| match e.kind {
            EntryKind::Grant => i64::from(e.points),
            EntryKind::Consume { .. } => -i64::from(e.points),
        })
        .sum()
}

/// ある従業員の取引だけを取り出して残高を計算する。
pub fn balance_of(employee: &EmployeeRef, entries: &[BudgetEntry]) -> i64 {
    balance(entries.iter().filter(|e| &e.employee == employee))
}

/// 支払いの可否（BR4.2〜BR4.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentCheck {
    /// 支払える。購入後の残高を持つ（同額なら 0）。
    Payable { balance_after: i64 },
    /// 支払えない。不足額（受講料 − 残高）を持つ。
    Insufficient { shortfall: i64 },
}

/// 残高と受講料から、支払いの可否を判定する。
pub fn check_payment(balance: i64, fee_points: u32) -> PaymentCheck {
    let fee = i64::from(fee_points);
    if balance >= fee {
        PaymentCheck::Payable {
            balance_after: balance - fee,
        }
    } else {
        PaymentCheck::Insufficient {
            shortfall: fee - balance,
        }
    }
}

/// 消費の取引を作る（BR4.5）。受講料と同額で、発行するチケットを指す。
///
/// 支払えない（残高が負になる）ときは `None` を返し、取引を作らない。
pub fn consumption_entry(
    id: EntryId,
    employee: EmployeeRef,
    ticket: TicketRef,
    fee_points: u32,
    current_balance: i64,
    occurred_at: DateTime<Utc>,
) -> Option<BudgetEntry> {
    match check_payment(current_balance, fee_points) {
        PaymentCheck::Payable { .. } => Some(BudgetEntry {
            id,
            employee,
            kind: EntryKind::Consume { ticket },
            points: fee_points,
            occurred_at,
        }),
        PaymentCheck::Insufficient { .. } => None,
    }
}
