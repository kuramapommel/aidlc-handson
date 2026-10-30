//! 研修予算の取引のデータの形（entities.md の BudgetEntry）。追記だけで記録する（ADR-002）。
//!
//! この crate はほかの業務の crate に依存しない（Purchasing だけがほかの 3 つを使う構成のため）。
//! そのため従業員とチケットへの参照は、文字列を包んだ参照用の値の型
//! （[`EmployeeRef`]・[`TicketRef`]）で持つ。値は Eligibility の従業員 ID と
//! Purchasing のチケット ID の文字列と同じ。

use chrono::{DateTime, Utc};

/// 取引の ID。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryId(pub String);

/// 取引の持ち主の従業員への参照（従業員 ID の文字列）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EmployeeRef(pub String);

/// 消費の取引が指すチケットへの参照（チケット ID の文字列）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TicketRef(pub String);

/// 取引の種類。消費のときだけチケットを指す（entities.md の ticketId の制約を型で表す）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    /// 付与（初期データだけが作る）。
    Grant,
    /// 購入による消費。
    Consume { ticket: TicketRef },
}

/// 研修予算の取引 1 件。`points` は消費でも正の値で持ち、向きは `kind` で表す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetEntry {
    pub id: EntryId,
    pub employee: EmployeeRef,
    pub kind: EntryKind,
    pub points: u32,
    pub occurred_at: DateTime<Utc>,
}
