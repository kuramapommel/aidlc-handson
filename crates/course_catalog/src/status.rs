//! 日程の残席数と状態の判定（BR2.1〜BR2.3）。

use chrono::{DateTime, Utc};

use crate::model::Session;

/// 日程の状態（計算で求める。保存しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionStatus {
    /// 購入可。
    Available,
    /// 満席。
    Full,
    /// 受付終了。
    Closed,
}

impl SessionStatus {
    /// Web API などで使う値。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Full => "full",
            Self::Closed => "closed",
        }
    }
}

/// 残席数 = max(0, 定員 − 発行済み数)（BR2.1）。
pub fn remaining_seats(session: &Session, issued: u32) -> u32 {
    session.capacity.saturating_sub(issued)
}

/// 現在時刻が開催開始の時刻以上なら受付終了（BR2.2）。開始ちょうども受付終了。
pub fn is_closed(session: &Session, now: DateTime<Utc>) -> bool {
    now >= session.starts_at
}

/// 今後の日程（現在時刻 < 開催開始）かどうか（BR3.1）。
pub fn is_upcoming(session: &Session, now: DateTime<Utc>) -> bool {
    !is_closed(session, now)
}

/// 日程の状態を「受付終了 → 満席 → 購入可」の順に判定する（BR2.3）。
/// 過去の満席の日程は受付終了になる。
pub fn session_status(session: &Session, issued: u32, now: DateTime<Utc>) -> SessionStatus {
    if is_closed(session, now) {
        SessionStatus::Closed
    } else if remaining_seats(session, issued) == 0 {
        SessionStatus::Full
    } else {
        SessionStatus::Available
    }
}
