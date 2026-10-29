//! コースと日程のデータの形（entities.md の Course・Session）。すべて不変の値として扱う。

use chrono::{DateTime, Utc};

use crate::ids::{CourseId, SessionId};

/// コースのカテゴリ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Management,
    Technology,
    BusinessSkills,
    Compliance,
}

impl Category {
    /// Web API などで使う値（entities.md の allowed_values）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Management => "management",
            Self::Technology => "technology",
            Self::BusinessSkills => "business-skills",
            Self::Compliance => "compliance",
        }
    }
}

/// 受講の形式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliveryFormat {
    Online,
    Venue,
}

impl DeliveryFormat {
    /// Web API などで使う値（entities.md の allowed_values）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Venue => "venue",
        }
    }
}

/// 研修コース。受講料は購入時に差し引く金額の唯一の根拠（FR5.5、BR5.2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Course {
    pub id: CourseId,
    pub title: String,
    pub category: Category,
    pub summary: String,
    pub description: String,
    pub fee_points: u32,
    pub duration_minutes: u32,
    pub delivery_format: DeliveryFormat,
}

/// コースの開催日程。定員だけを持ち、残席数は持たない（ADR-003）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: SessionId,
    pub course_id: CourseId,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    /// 会場名。形式が会場のときだけ持つ。
    pub venue: Option<String>,
    pub capacity: u32,
}

/// 文字数（Unicode のスカラー値の数）が範囲内かどうか。
fn char_count_within(text: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&text.chars().count())
}

/// コースが entities.md の制約を満たすかを確かめる。満たさない項目の説明を返す。
pub fn validate_course(course: &Course) -> Result<(), String> {
    let checks = [
        (
            char_count_within(&course.title, 1, 80),
            "title は 1〜80 文字",
        ),
        (
            char_count_within(&course.summary, 1, 120),
            "summary は 1〜120 文字",
        ),
        (
            char_count_within(&course.description, 1, 2000),
            "description は 1〜2000 文字",
        ),
        (course.fee_points >= 1, "feePoints は 1 以上"),
        (course.duration_minutes >= 1, "durationMinutes は 1 以上"),
    ];
    first_failure(course.id.as_str(), &checks)
}

/// 日程がコースに対して entities.md の制約を満たすかを確かめる。
pub fn validate_session(session: &Session, course: &Course) -> Result<(), String> {
    let venue_matches_format = match course.delivery_format {
        DeliveryFormat::Venue => session.venue.as_deref().is_some_and(|v| !v.is_empty()),
        DeliveryFormat::Online => session.venue.is_none(),
    };
    let checks = [
        (
            session.course_id == course.id,
            "courseId がコースと一致する",
        ),
        (
            session.ends_at > session.starts_at,
            "endsAt は startsAt より後",
        ),
        (session.capacity >= 1, "capacity は 1 以上"),
        (venue_matches_format, "venue は会場の形式のときだけ持つ"),
    ];
    first_failure(session.id.as_str(), &checks)
}

fn first_failure(id: &str, checks: &[(bool, &str)]) -> Result<(), String> {
    checks
        .iter()
        .find(|(ok, _)| !ok)
        .map_or(Ok(()), |(_, message)| Err(format!("{id}: {message}")))
}
