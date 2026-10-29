//! Web API の JSON の形（応答と要求の本文）。業務の型から変換する純粋関数を持つ。
//! 日時は日本時間のタイムゾーン付きの形式（RFC 3339）、ポイントは整数で返す。

use chrono::{DateTime, SecondsFormat, Utc};
use chrono_tz::Asia::Tokyo;
use course_catalog::{Course, CourseListRow, Session};
use eligibility::{EligibilityResult, SwitchableEmployee};
use purchasing::{SessionAvailability, TicketView};
use serde::{Deserialize, Serialize};

/// 日時を日本時間のタイムゾーン付きの文字列にする（例: `2026-11-09T13:00:00+09:00`）。
pub fn format_time(at: DateTime<Utc>) -> String {
    at.with_timezone(&Tokyo)
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseRowDto {
    pub course_id: String,
    pub title: String,
    pub category: &'static str,
    pub summary: String,
    pub fee_points: u32,
    pub delivery_format: &'static str,
    /// 直近の開催日。今後の日程がなければ `null`。
    pub next_session_starts_at: Option<String>,
    pub open_session_count: u32,
}

impl From<&CourseListRow> for CourseRowDto {
    fn from(row: &CourseListRow) -> Self {
        Self {
            course_id: row.course.id.as_str().to_owned(),
            title: row.course.title.clone(),
            category: row.course.category.as_str(),
            summary: row.course.summary.clone(),
            fee_points: row.course.fee_points,
            delivery_format: row.course.delivery_format.as_str(),
            next_session_starts_at: row.next_session_starts_at.map(format_time),
            open_session_count: row.open_session_count,
        }
    }
}

/// `GET /api/courses` の応答。U2 で探索条件を加えても形を保てるよう、件数を別に持つ。
#[derive(Debug, Serialize)]
pub struct CourseListDto {
    pub courses: Vec<CourseRowDto>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseDto {
    pub course_id: String,
    pub title: String,
    pub category: &'static str,
    pub summary: String,
    pub description: String,
    pub fee_points: u32,
    pub duration_minutes: u32,
    pub delivery_format: &'static str,
}

impl From<&Course> for CourseDto {
    fn from(course: &Course) -> Self {
        Self {
            course_id: course.id.as_str().to_owned(),
            title: course.title.clone(),
            category: course.category.as_str(),
            summary: course.summary.clone(),
            description: course.description.clone(),
            fee_points: course.fee_points,
            duration_minutes: course.duration_minutes,
            delivery_format: course.delivery_format.as_str(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    pub session_id: String,
    pub starts_at: String,
    pub ends_at: String,
    pub venue: Option<String>,
    pub capacity: u32,
    pub remaining_seats: u32,
    /// 表示用の状態（available・full・closed・purchased。BR2.4）。
    pub status: &'static str,
}

impl SessionDto {
    /// 日程と、計算済みの残席数・表示用の状態から作る。
    pub fn new(session: &Session, remaining_seats: u32, status: SessionAvailability) -> Self {
        Self {
            session_id: session.id.as_str().to_owned(),
            starts_at: format_time(session.starts_at),
            ends_at: format_time(session.ends_at),
            venue: session.venue.clone(),
            capacity: session.capacity,
            remaining_seats,
            status: status.as_str(),
        }
    }
}

/// `GET /api/courses/{courseId}` の応答。
#[derive(Debug, Serialize)]
pub struct CourseDetailDto {
    pub course: CourseDto,
    pub sessions: Vec<SessionDto>,
}

/// 操作中の従業員（`GET /api/me` と従業員の切り替えの応答）。
/// 従業員が見つからないときは、名前などを `null` にして利用資格を employee-not-found にする（BR6.2）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeDto {
    pub employee_id: Option<String>,
    pub display_name: Option<String>,
    pub company_name: Option<String>,
    pub contract_status: Option<&'static str>,
    pub eligibility: &'static str,
    pub balance: i64,
}

impl MeDto {
    /// 従業員が見つからないときの形（閲覧には応え、購入は BR1.1 で拒否される）。
    pub fn not_found(employee_id: Option<String>) -> Self {
        Self {
            employee_id,
            display_name: None,
            company_name: None,
            contract_status: None,
            eligibility: EligibilityResult::EmployeeNotFound.as_str(),
            balance: 0,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchableEmployeeDto {
    pub employee_id: String,
    pub display_name: String,
    pub company_name: String,
    pub contract_status: &'static str,
}

impl From<&SwitchableEmployee> for SwitchableEmployeeDto {
    fn from(e: &SwitchableEmployee) -> Self {
        Self {
            employee_id: e.employee_id.as_str().to_owned(),
            display_name: e.display_name.clone(),
            company_name: e.company_name.clone(),
            contract_status: e.contract_status.as_str(),
        }
    }
}

/// `GET /api/employees` の応答。
#[derive(Debug, Serialize)]
pub struct EmployeeListDto {
    pub employees: Vec<SwitchableEmployeeDto>,
}

/// e チケット（BR5.9 の項目）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketDto {
    pub ticket_id: String,
    pub ticket_number: String,
    pub employee_name: String,
    pub course_id: String,
    pub course_title: String,
    pub session_id: String,
    pub starts_at: String,
    pub ends_at: String,
    pub venue: Option<String>,
    pub delivery_format: &'static str,
    pub purchased_at: String,
    pub paid_points: u32,
}

impl From<&TicketView> for TicketDto {
    fn from(view: &TicketView) -> Self {
        Self {
            ticket_id: view.ticket.id.as_str().to_owned(),
            ticket_number: view.ticket.number.as_str().to_owned(),
            employee_name: view.employee_name.clone(),
            course_id: view.course.id.as_str().to_owned(),
            course_title: view.course.title.clone(),
            session_id: view.session.id.as_str().to_owned(),
            starts_at: format_time(view.session.starts_at),
            ends_at: format_time(view.session.ends_at),
            venue: view.session.venue.clone(),
            delivery_format: view.course.delivery_format.as_str(),
            purchased_at: format_time(view.ticket.purchased_at),
            paid_points: view.ticket.paid_points,
        }
    }
}

/// `POST /api/purchases` の成功の応答。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseResultDto {
    pub ticket: TicketDto,
    pub balance_after: i64,
}

/// `POST /api/purchases` の本文。日程 ID だけを読む。金額などほかの項目は読み取らない（BR5.2）。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseBody {
    pub session_id: String,
}

/// `POST /api/session/employee` の本文。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchEmployeeBody {
    pub employee_id: String,
}
