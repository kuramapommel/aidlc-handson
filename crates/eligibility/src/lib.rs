//! 業務の部品 Eligibility。
//!
//! 契約企業と従業員のデータの形、利用資格の判定、切り替え用の従業員一覧を純粋関数で提供する。
//! ログイン機能は持たず、要求に付いた従業員 ID を認証済みとして扱う前提（ADR-008。ローカル検証専用）。

pub mod model;
pub mod rules;

pub use model::{Company, CompanyId, ContractStatus, Employee, EmployeeId, IdFormatError};
pub use rules::{
    EligibilityResult, SwitchableEmployee, company_finder, determine_eligibility, employee_finder,
    switchable_employees,
};

#[cfg(test)]
mod tests;
