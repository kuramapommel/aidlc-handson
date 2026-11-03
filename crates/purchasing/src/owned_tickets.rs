//! 購入済みチケットの一覧（U3。BR12.1〜BR12.3）。
//!
//! 操作中の従業員のチケットだけを選び、Web API の時計で受講済みかを決め、
//! 「これから」を開催日時の近い順、そのあとに「受講済み」を開催日時の新しい順に並べる。
//! データストアには依存しない。日程とコース・従業員は引く関数として、現在時刻は値として受け取る（NFR7.1）。

use std::cmp::Ordering;
use std::fmt;

use chrono::{DateTime, Utc};
use course_catalog::{Course, Session, SessionId};
use eligibility::{Employee, EmployeeId};

use crate::model::{Ticket, TicketId};
use crate::tickets::TicketView;

/// チケットの受講の状態（導出属性。保存しない。BR12.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Attendance {
    /// これから（開始日時が現在時刻より後）。
    Upcoming,
    /// 受講済み（開始日時が現在時刻以前）。
    Attended,
}

impl Attendance {
    /// Web API の応答で使う値。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upcoming => "upcoming",
            Self::Attended => "attended",
        }
    }
}

/// 開始日時と現在時刻から受講の状態を決める。開始日時ちょうどは受講済み（BR12.3）。
///
/// U1 の日程の受付終了（`now >= starts_at`）と同じ時点で切り替わる。
pub fn attendance_of(starts_at: DateTime<Utc>, now: DateTime<Utc>) -> Attendance {
    if starts_at <= now {
        Attendance::Attended
    } else {
        Attendance::Upcoming
    }
}

/// 購入済みチケットの一覧の 1 件（entities.md の OwnedTicket）。
///
/// 表示の項目（AC4.1.1 の 7 項目と、終了日時・形式）は U1 のチケット 1 枚の詳細と同じ
/// [`TicketView`] で持ち、受講の状態を足す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedTicket {
    pub view: TicketView,
    pub attendance: Attendance,
}

/// 一覧を作れないデータの不整合（チケットの日程・コースが見つからない）。
///
/// どのチケットで起きたかをログに残すため、該当するチケット ID をすべて持つ（NFR10.3）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedTicketsInconsistency {
    pub ticket_ids: Vec<TicketId>,
}

impl fmt::Display for OwnedTicketsInconsistency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ids: Vec<&str> = self.ticket_ids.iter().map(TicketId::as_str).collect();
        write!(
            f,
            "日程またはコースが見つからないチケットがあります: {}",
            ids.join(",")
        )
    }
}

impl std::error::Error for OwnedTicketsInconsistency {}

/// 操作中の従業員の購入済みチケットの一覧を作る（BR12.1〜BR12.3）。
///
/// - 操作中の従業員 ID が `None`（Cookie がない・形式不正）か、存在しない従業員を指すときは 0 件。
/// - 他人のチケットは含めない。
/// - 日程またはコースが見つからないチケットがあれば、そのチケット ID を持つ不整合を返す。
///
/// PostgreSQL に移すときの注意（NFR7.2）: ここでは全チケットを受け取って従業員 ID で絞るが、
/// データベースでは `tickets.employee_id` の索引を使った「従業員 ID でチケットを引く検索」に
/// 置き換え、絞った結果をこの関数に渡せばよい（絞り込みが二重になっても結果は変わらない）。
/// 日程・コースの取得も、日程 ID の集まりでまとめて引く検索にできる。
pub fn list_owned_tickets<FS, FE>(
    tickets: &[Ticket],
    employee_id: Option<&EmployeeId>,
    now: DateTime<Utc>,
    find_session: FS,
    find_employee: FE,
) -> Result<Vec<OwnedTicket>, OwnedTicketsInconsistency>
where
    FS: Fn(&SessionId) -> Option<(Session, Course)>,
    FE: Fn(&EmployeeId) -> Option<Employee>,
{
    let Some(employee) = employee_id.and_then(&find_employee) else {
        return Ok(Vec::new());
    };
    let own: Vec<&Ticket> = tickets
        .iter()
        .filter(|t| t.employee_id == employee.id)
        .collect();
    let missing: Vec<TicketId> = own
        .iter()
        .filter(|t| find_session(&t.session_id).is_none())
        .map(|t| t.id.clone())
        .collect();
    if !missing.is_empty() {
        return Err(OwnedTicketsInconsistency {
            ticket_ids: missing,
        });
    }
    let mut owned: Vec<OwnedTicket> = own
        .into_iter()
        .filter_map(|t| owned_ticket(t, &employee, now, &find_session))
        .collect();
    owned.sort_by(compare_owned_tickets);
    Ok(owned)
}

fn owned_ticket<FS>(
    ticket: &Ticket,
    employee: &Employee,
    now: DateTime<Utc>,
    find_session: &FS,
) -> Option<OwnedTicket>
where
    FS: Fn(&SessionId) -> Option<(Session, Course)>,
{
    let (session, course) = find_session(&ticket.session_id)?;
    let attendance = attendance_of(session.starts_at, now);
    Some(OwnedTicket {
        view: TicketView {
            ticket: ticket.clone(),
            employee_name: employee.display_name.clone(),
            course,
            session,
        },
        attendance,
    })
}

/// 一覧の並びの比較（BR12.2。安定した全順序。NFR6.3）。
///
/// これからを先、受講済みを後に置く。これからは開始日時の早い順、受講済みは開始日時の遅い順。
/// 開始日時が同じなら、どちらのグループでも購入日時の早い順、さらにチケット番号の昇順。
pub fn compare_owned_tickets(a: &OwnedTicket, b: &OwnedTicket) -> Ordering {
    let group = |t: &OwnedTicket| match t.attendance {
        Attendance::Upcoming => 0,
        Attendance::Attended => 1,
    };
    let starts = a.view.session.starts_at.cmp(&b.view.session.starts_at);
    let by_start = match a.attendance {
        Attendance::Upcoming => starts,
        Attendance::Attended => starts.reverse(),
    };
    group(a)
        .cmp(&group(b))
        .then(by_start)
        .then(a.view.ticket.purchased_at.cmp(&b.view.ticket.purchased_at))
        .then_with(|| {
            a.view
                .ticket
                .number
                .as_str()
                .cmp(b.view.ticket.number.as_str())
        })
}

#[cfg(test)]
#[path = "owned_tickets_tests.rs"]
mod owned_tickets_tests;
