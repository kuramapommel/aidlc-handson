//! コース一覧の集計と既定の並び（BR3.1〜BR3.3）、コースの詳細。
//!
//! 日程ごとの発行済みチケット数は Purchasing が持つため、呼び出し側から
//! 「日程 ID → 発行済み数」の関数として受け取る（ADR-005。循環依存を避ける高階関数）。

use std::cmp::Ordering;

use chrono::{DateTime, Utc};

use crate::ids::{CourseId, SessionId};
use crate::model::{Course, Session};
use crate::status::{SessionStatus, is_upcoming, remaining_seats, session_status};

/// コース一覧の 1 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseListRow {
    pub course: Course,
    /// 直近の開催日（今後の日程のうち最も早い開催開始。満席も含む）。なければ `None`。
    pub next_session_starts_at: Option<DateTime<Utc>>,
    /// 今後の日程のうち残席が 1 以上の数。
    pub open_session_count: u32,
}

/// 詳細に表示する日程 1 件（残席数と状態を計算済み）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionView {
    pub session: Session,
    pub remaining_seats: u32,
    pub status: SessionStatus,
}

/// コースの詳細（コースの全項目と日程の一覧）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseDetail {
    pub course: Course,
    /// 開催開始の早い順（同じなら日程 ID 順）。
    pub sessions: Vec<SessionView>,
}

/// 1 つのコースの一覧の行を集計する（BR3.1、BR3.2）。探索（U2）でも同じ集計を使う。
pub(crate) fn summarize_course<F>(
    course: &Course,
    sessions: &[Session],
    issued_count: &F,
    now: DateTime<Utc>,
) -> CourseListRow
where
    F: Fn(&SessionId) -> u32,
{
    let upcoming: Vec<&Session> = sessions
        .iter()
        .filter(|s| s.course_id == course.id && is_upcoming(s, now))
        .collect();
    let next_session_starts_at = upcoming.iter().map(|s| s.starts_at).min();
    let open_session_count = upcoming
        .iter()
        .filter(|s| remaining_seats(s, issued_count(&s.id)) >= 1)
        .count();
    CourseListRow {
        course: course.clone(),
        next_session_starts_at,
        open_session_count: u32::try_from(open_session_count).unwrap_or(u32::MAX),
    }
}

/// 既定の並び（BR3.3）: 直近の開催日の昇順、同値はコース ID の昇順、予定なしは最後。
/// 探索の「開催日が近い順」（U2 の BR10.1）も同じ並び。
pub(crate) fn default_order(a: &CourseListRow, b: &CourseListRow) -> Ordering {
    match (a.next_session_starts_at, b.next_session_starts_at) {
        (Some(x), Some(y)) => x.cmp(&y).then_with(|| a.course.id.cmp(&b.course.id)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.course.id.cmp(&b.course.id),
    }
}

/// 探索条件なしのコース一覧を、既定の並びで返す。
///
/// `issued_count` は日程 ID から発行済みチケット数を返す関数。`now` は Web API の時計の現在時刻。
pub fn list_courses<F>(
    courses: &[Course],
    sessions: &[Session],
    issued_count: F,
    now: DateTime<Utc>,
) -> Vec<CourseListRow>
where
    F: Fn(&SessionId) -> u32,
{
    let mut rows: Vec<CourseListRow> = courses
        .iter()
        .map(|course| summarize_course(course, sessions, &issued_count, now))
        .collect();
    rows.sort_by(default_order);
    rows
}

/// コースの詳細を返す。コースがなければ `None`（Web API は 404 にする。BR7.9）。
pub fn course_detail<F>(
    courses: &[Course],
    sessions: &[Session],
    course_id: &CourseId,
    issued_count: F,
    now: DateTime<Utc>,
) -> Option<CourseDetail>
where
    F: Fn(&SessionId) -> u32,
{
    let course = courses.iter().find(|c| &c.id == course_id)?;
    let mut views: Vec<SessionView> = sessions
        .iter()
        .filter(|s| &s.course_id == course_id)
        .map(|s| {
            let issued = issued_count(&s.id);
            SessionView {
                session: s.clone(),
                remaining_seats: remaining_seats(s, issued),
                status: session_status(s, issued, now),
            }
        })
        .collect();
    views.sort_by(|a, b| {
        a.session
            .starts_at
            .cmp(&b.session.starts_at)
            .then_with(|| a.session.id.cmp(&b.session.id))
    });
    Some(CourseDetail {
        course: course.clone(),
        sessions: views,
    })
}

/// 日程 ID から日程とそのコースを引く（Purchasing の購入の写しづくりなどで使う）。
pub fn find_session_with_course<'a>(
    courses: &'a [Course],
    sessions: &'a [Session],
    session_id: &SessionId,
) -> Option<(&'a Session, &'a Course)> {
    let session = sessions.iter().find(|s| &s.id == session_id)?;
    let course = courses.iter().find(|c| c.id == session.course_id)?;
    Some((session, course))
}
