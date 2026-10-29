//! データストアが持つ全データ（1 つの不変な状態）と、状態を変える純粋関数。

use chrono::{DateTime, Utc};
use course_catalog::{Course, Session, find_session_with_course};
use eligibility::{Company, Employee, company_finder, determine_eligibility, employee_finder};
use purchasing::{
    PurchaseDecision, PurchaseRequest, PurchaseSnapshot, Ticket, TicketIdentity, decide_purchase,
    has_purchased, issued_counts,
};
use training_budget::{BudgetEntry, EmployeeRef, balance_of};

/// 全データ。値として扱い、変更は新しい値を作って表す（NFR6.2）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StoreState {
    pub companies: Vec<Company>,
    /// 初期データの順。先頭の従業員は Cookie がないときの操作中の従業員（BR6.2）。
    pub employees: Vec<Employee>,
    pub courses: Vec<Course>,
    pub sessions: Vec<Session>,
    pub budget_entries: Vec<BudgetEntry>,
    pub tickets: Vec<Ticket>,
}

/// 状態と購入の要求から、購入の決定関数の入力（写し）を作る（BR5.4）。
pub fn purchase_snapshot(
    state: &StoreState,
    request: &PurchaseRequest,
    now: DateTime<Utc>,
) -> PurchaseSnapshot {
    let employee_id = request.employee_id.as_ref();
    let eligibility = determine_eligibility(
        employee_id,
        employee_finder(&state.employees),
        company_finder(&state.companies),
    );
    let session = find_session_with_course(&state.courses, &state.sessions, &request.session_id)
        .map(|(s, c)| (s.clone(), c.clone()));
    let issued = issued_counts(&state.tickets)
        .get(&request.session_id)
        .copied()
        .unwrap_or(0);
    let balance = employee_id.map_or(0, |id| {
        balance_of(&EmployeeRef(id.as_str().to_owned()), &state.budget_entries)
    });
    let already_purchased =
        employee_id.is_some_and(|id| has_purchased(&state.tickets, id, &request.session_id));
    PurchaseSnapshot {
        employee_id: request.employee_id.clone(),
        eligibility,
        session_id: request.session_id.clone(),
        session,
        issued,
        balance,
        already_purchased,
        now,
    }
}

/// 購入の状態遷移: 「現在の状態 → 新しい状態（承認のときだけ）＋決定」を返す純粋関数。
///
/// 承認ならチケットと消費の取引を同時に追加した新しい状態を返す（BR5.3）。
/// 拒否なら新しい状態を作らない（何も反映しない）。
pub fn apply_purchase(
    state: &StoreState,
    request: &PurchaseRequest,
    now: DateTime<Utc>,
    identity: TicketIdentity,
) -> (Option<StoreState>, PurchaseDecision) {
    let snapshot = purchase_snapshot(state, request, now);
    let decision = decide_purchase(&snapshot, identity);
    let next = match &decision {
        PurchaseDecision::Approved(approval) => Some(StoreState {
            tickets: append(&state.tickets, approval.ticket.clone()),
            budget_entries: append(&state.budget_entries, approval.consumption.clone()),
            ..state.clone()
        }),
        PurchaseDecision::Rejected(_) => None,
    };
    (next, decision)
}

fn append<T: Clone>(items: &[T], item: T) -> Vec<T> {
    items.iter().cloned().chain(std::iter::once(item)).collect()
}
