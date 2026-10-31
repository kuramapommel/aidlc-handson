//! 業務の部品 TrainingBudget。
//!
//! 研修予算を「付与」「消費」の取引として追記だけで記録し（ADR-002）、残高・支払い可否・
//! 不足額・購入後の残高を純粋関数で求める。

pub mod model;
pub mod rules;

pub use model::{BudgetEntry, EmployeeRef, EntryId, EntryKind, TicketRef};
pub use rules::{PaymentCheck, balance, balance_of, check_payment, consumption_entry};

#[cfg(test)]
mod tests;
