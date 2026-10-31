//! 日程ごとの発行済み数、表示用の日程の状態（BR2.4）、チケット 1 枚の詳細（BR5.9、BR5.10）。

use std::collections::{HashMap, HashSet};

use course_catalog::{Course, Session, SessionId, SessionStatus};
use eligibility::{Employee, EmployeeId};

use crate::model::{SessionAvailability, Ticket, TicketId, TicketNumber};

/// 日程ごとの発行済みチケット数を 1 回だけ数える（NFR1.5）。
pub fn issued_counts(tickets: &[Ticket]) -> HashMap<SessionId, u32> {
    tickets.iter().fold(HashMap::new(), |mut counts, ticket| {
        *counts.entry(ticket.session_id.clone()).or_insert(0) += 1;
        counts
    })
}

/// 数えた結果から「日程 ID → 発行済み数」の関数を作る（CourseCatalog に渡す。ADR-005）。
pub fn issued_counter(counts: &HashMap<SessionId, u32>) -> impl Fn(&SessionId) -> u32 + '_ {
    move |id| counts.get(id).copied().unwrap_or(0)
}

/// その従業員が購入済みの日程 ID の集まり。
pub fn purchased_session_ids(
    tickets: &[Ticket],
    employee_id: Option<&EmployeeId>,
) -> HashSet<SessionId> {
    employee_id.map_or_else(HashSet::new, |employee| {
        tickets
            .iter()
            .filter(|t| &t.employee_id == employee)
            .map(|t| t.session_id.clone())
            .collect()
    })
}

/// その従業員がその日程を購入済みか（BR5.5）。
pub fn has_purchased(tickets: &[Ticket], employee_id: &EmployeeId, session_id: &SessionId) -> bool {
    tickets
        .iter()
        .any(|t| &t.employee_id == employee_id && &t.session_id == session_id)
}

/// 番号がすでに発行済みか。
pub fn is_number_taken(tickets: &[Ticket], number: &TicketNumber) -> bool {
    tickets.iter().any(|t| &t.number == number)
}

/// 表示用の日程の状態。購入済みならほかの状態より優先する（BR2.4）。
pub fn session_availability(status: SessionStatus, purchased: bool) -> SessionAvailability {
    if purchased {
        return SessionAvailability::Purchased;
    }
    match status {
        SessionStatus::Available => SessionAvailability::Available,
        SessionStatus::Full => SessionAvailability::Full,
        SessionStatus::Closed => SessionAvailability::Closed,
    }
}

/// 操作中の従業員のチケットだけを引く。他人のチケットは存在しないものとして扱う（BR5.10）。
pub fn find_own_ticket<'a>(
    tickets: &'a [Ticket],
    ticket_id: &TicketId,
    employee_id: Option<&EmployeeId>,
) -> Option<&'a Ticket> {
    let employee = employee_id?;
    tickets
        .iter()
        .find(|t| &t.id == ticket_id && &t.employee_id == employee)
}

/// e チケットの表示に使う項目（BR5.9）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketView {
    pub ticket: Ticket,
    pub employee_name: String,
    pub course: Course,
    pub session: Session,
}

/// チケット 1 枚の詳細を組み立てる。従業員・日程・コースの取得は関数で受け取る。
pub fn ticket_view<FE, FS>(
    ticket: &Ticket,
    find_employee: FE,
    find_session: FS,
) -> Option<TicketView>
where
    FE: Fn(&EmployeeId) -> Option<Employee>,
    FS: Fn(&SessionId) -> Option<(Session, Course)>,
{
    let employee = find_employee(&ticket.employee_id)?;
    let (session, course) = find_session(&ticket.session_id)?;
    Some(TicketView {
        ticket: ticket.clone(),
        employee_name: employee.display_name,
        course,
        session,
    })
}
