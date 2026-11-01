//! 業務の部品 CourseCatalog。
//!
//! 研修コースと開催日程のデータの形、日程の残席数と状態の判定、コース一覧の集計と
//! 既定の並び、コースの詳細、コース一覧の探索（U2: 条件の検証・キーワードの正規化・絞り込み・
//! 並べ替え）を純粋関数で提供する。HTTP・データ保存・画面には依存しない
//! （六角形アーキテクチャ。ADR-001）。現在時刻と発行済み数は引数で受け取る。

pub mod ids;
pub mod listing;
pub mod model;
pub mod search;
pub mod status;

pub use ids::{CourseId, IdFormatError, SessionId, has_prefix_and_digits};
pub use listing::{
    CourseDetail, CourseListRow, SessionView, course_detail, find_session_with_course, list_courses,
};
pub use model::{
    Category, Course, DeliveryFormat, Session, is_valid_reading, validate_course, validate_session,
};
pub use search::{
    ConditionField, KEYWORD_MAX_CHARS, NormalizedKeyword, RawSearchCondition, SearchCondition,
    SearchConditionError, SortOrder, katakana_to_hiragana, normalize_keyword, normalize_text,
    parse_calendar_date, parse_search_condition, search_courses,
};
pub use status::{SessionStatus, is_closed, is_upcoming, remaining_seats, session_status};

#[cfg(test)]
mod search_tests;
#[cfg(test)]
mod tests;
