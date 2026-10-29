//! 業務の部品 CourseCatalog。
//!
//! 研修コースと開催日程のデータの形、日程の残席数と状態の判定、コース一覧の集計と
//! 既定の並び、コースの詳細を純粋関数で提供する。HTTP・データ保存・画面には依存しない
//! （六角形アーキテクチャ。ADR-001）。現在時刻と発行済み数は引数で受け取る。

pub mod ids;
pub mod listing;
pub mod model;
pub mod status;

pub use ids::{CourseId, IdFormatError, SessionId, has_prefix_and_digits};
pub use listing::{
    CourseDetail, CourseListRow, SessionView, course_detail, find_session_with_course, list_courses,
};
pub use model::{Category, Course, DeliveryFormat, Session, validate_course, validate_session};
pub use status::{SessionStatus, is_closed, is_upcoming, remaining_seats, session_status};

#[cfg(test)]
mod tests;
