//! チケットのデータの形（entities.md の Ticket）と、購入の決定で使う補助の型。

use std::fmt;

use chrono::{DateTime, Utc};
use course_catalog::{Course, Session, SessionId};
use eligibility::{EligibilityResult, EmployeeId};
use training_budget::BudgetEntry;

/// ID・番号の形式が正しくないことを表すエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    /// どの値か（例: `ticketId`）。
    pub kind: &'static str,
    /// 受け取った値。
    pub value: String,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} の形式が正しくありません: {:?}",
            self.kind, self.value
        )
    }
}

impl std::error::Error for FormatError {}

/// チケット ID（内部の識別子）。小文字の 16 進数をハイフンで区切った UUID の形
/// （`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TicketId(String);

impl TicketId {
    /// 形式を確かめてチケット ID を作る。
    pub fn parse(value: &str) -> Result<Self, FormatError> {
        if is_uuid_shape(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(FormatError {
                kind: "ticketId",
                value: value.to_owned(),
            })
        }
    }

    /// 文字列としての値。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TicketId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_uuid_shape(value: &str) -> bool {
    const GROUPS: [usize; 5] = [8, 4, 4, 4, 12];
    let parts: Vec<&str> = value.split('-').collect();
    parts.len() == GROUPS.len()
        && parts.iter().zip(GROUPS).all(|(part, len)| {
            part.len() == len
                && part
                    .chars()
                    .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        })
}

/// 利用者に見せるチケット番号（BR5.6 の「XXXX-XXXX」の形）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TicketNumber(String);

impl TicketNumber {
    /// 形式を確かめてチケット番号を作る。
    pub fn parse(value: &str) -> Result<Self, FormatError> {
        if crate::ticket_number::is_valid_ticket_number(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(FormatError {
                kind: "ticketNumber",
                value: value.to_owned(),
            })
        }
    }

    /// 文字列としての値。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// e チケット。1 名・1 日程につき 1 枚（FR5.4）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
    pub id: TicketId,
    pub number: TicketNumber,
    pub employee_id: EmployeeId,
    pub session_id: SessionId,
    /// 購入時点のコースの受講料（BR5.2）。
    pub paid_points: u32,
    /// Web API の時計で決めた購入日時（BR5.7）。
    pub purchased_at: DateTime<Utc>,
}

/// 購入の要求。受講料などの金額は含めない（BR5.2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchaseRequest {
    /// Cookie から得た操作中の従業員 ID。Cookie がない・形式不正なら `None`。
    pub employee_id: Option<EmployeeId>,
    pub session_id: SessionId,
}

/// 購入の決定関数の入力（ADR-004）。記録の直前に最新の状態から作る（BR5.4）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchaseSnapshot {
    pub employee_id: Option<EmployeeId>,
    pub eligibility: EligibilityResult,
    pub session_id: SessionId,
    /// 対象の日程とそのコース。日程がなければ `None`。
    pub session: Option<(Session, Course)>,
    /// その日程の発行済みチケット数。
    pub issued: u32,
    /// 従業員の残高（BR4.1）。
    pub balance: i64,
    /// その従業員がその日程を購入済みか（BR5.5）。
    pub already_purchased: bool,
    /// 現在時刻（Web API の時計。BR5.7）。
    pub now: DateTime<Utc>,
}

/// 承認したときに発行するチケットの識別子。決定関数の外で作って渡す（乱数を外に出すため）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketIdentity {
    pub ticket_id: TicketId,
    pub ticket_number: TicketNumber,
}

/// 拒否の理由（BR5.1 の順で 1 つだけ返す）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RejectionReason {
    EmployeeNotFound,
    CompanyNotActive,
    SessionNotFound,
    Closed,
    AlreadyPurchased,
    Full,
    InsufficientBalance,
}

impl RejectionReason {
    /// Web API の応答・ログで使う値。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmployeeNotFound => "employee-not-found",
            Self::CompanyNotActive => "company-not-active",
            Self::SessionNotFound => "session-not-found",
            Self::Closed => "closed",
            Self::AlreadyPurchased => "already-purchased",
            Self::Full => "full",
            Self::InsufficientBalance => "insufficient-balance",
        }
    }
}

/// 承認の中身: 発行するチケット 1 枚と消費の取引 1 件（BR5.3）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchaseApproval {
    pub ticket: Ticket,
    pub consumption: BudgetEntry,
    /// 購入後の残高（BR4.4）。
    pub balance_after: i64,
}

/// 購入の決定（承認か拒否の理由 1 つ）。承認の中身は大きいため箱に入れる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurchaseDecision {
    Approved(Box<PurchaseApproval>),
    Rejected(RejectionReason),
}

/// 表示用の日程の状態（BR2.4）: 日程の状態に、操作中の従業員の購入済みを重ねたもの。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionAvailability {
    Available,
    Full,
    Closed,
    Purchased,
}

impl SessionAvailability {
    /// Web API などで使う値。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Full => "full",
            Self::Closed => "closed",
            Self::Purchased => "purchased",
        }
    }
}
