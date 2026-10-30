//! コース一覧の探索（U2）。探索条件の検証・キーワードの正規化・絞り込み・並べ替えを
//! 純粋関数で提供する（BR8.1〜BR8.5、BR9.1〜BR9.5、BR10.1〜BR10.5、NFR6.1）。
//!
//! HTTP・データ保存・画面には依存しない（ADR-001・ADR-006）。現在時刻と、日程 ID から
//! 発行済み数を返す関数は呼び出し側から受け取る（ADR-005）。
//!
//! PostgreSQL に移すときの注意（NFR7.1、NFR7.3）:
//! - 同じ [`SearchCondition`] を受け取って SQL で絞り込む別の実装に差し替える。関数の形
//!   （コース・日程・条件・発行済み数・現在時刻 → 一覧の行）は変えない。
//! - キーワードの照合は、コース名と概要を [`normalize_text`] と同じ規則で正規化した
//!   検索用の列を持たせ、その列に対して部分一致（`LIKE '%語%'`。語の `%`・`_` はエスケープする）で行う。
//!   正規化の規則を SQL 側で再実装せず、書き込み時にアプリケーションで列を作る。
//! - コース名順のために、よみがなを正規化した列に索引を張る。
//! - 期間は、日程の開催開始の日付（日本時間）の索引で範囲を絞ってから、空席（定員 − 発行済み数）を確かめる。

use std::cmp::Ordering;

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use unicode_normalization::UnicodeNormalization;

use crate::ids::SessionId;
use crate::listing::{CourseListRow, default_order, summarize_course};
use crate::model::{Category, Course, Session};
use crate::status::{is_upcoming, remaining_seats};

/// キーワードのうち探索に使う先頭の文字数（BR8.1、NFR3.2）。それより後は捨てる。
pub const KEYWORD_MAX_CHARS: usize = 200;

/// 日本時間（UTC+9。夏時間はない）。期間の暦日の判定に使う。
const JST_OFFSET_SECONDS: i32 = 9 * 3600;

/// 並べ替え（BR8.5、BR10.1〜BR10.4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortOrder {
    /// 開催日が近い順（既定）。
    #[default]
    Date,
    /// 受講料の安い順。
    FeeAsc,
    /// 受講料の高い順。
    FeeDesc,
    /// コース名順（よみがなの五十音順）。
    Name,
}

impl SortOrder {
    /// Web API・URL で使う値。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::FeeAsc => "fee-asc",
            Self::FeeDesc => "fee-desc",
            Self::Name => "name",
        }
    }

    /// 値から並べ方を得る。決められた 4 つ以外なら `None`。
    pub fn from_value(value: &str) -> Option<Self> {
        [Self::Date, Self::FeeAsc, Self::FeeDesc, Self::Name]
            .into_iter()
            .find(|s| s.as_str() == value)
    }
}

/// 照合のために正規化したキーワード（entities.md の NormalizedKeyword）。語が 0 個なら条件なし。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NormalizedKeyword {
    pub terms: Vec<String>,
}

/// 検証済みの探索条件（entities.md の SearchCondition）。項目どうしは AND。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchCondition {
    pub keyword: NormalizedKeyword,
    pub category: Option<Category>,
    pub available_only: bool,
    pub period_from: Option<NaiveDate>,
    pub period_to: Option<NaiveDate>,
    pub sort: SortOrder,
}

/// 不正な条件の項目（entities.md の SearchConditionError.field）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionField {
    Category,
    AvailableOnly,
    PeriodFrom,
    PeriodTo,
    Period,
    Sort,
}

impl ConditionField {
    /// 応答に出す項目名。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Category => "category",
            Self::AvailableOnly => "availableOnly",
            Self::PeriodFrom => "periodFrom",
            Self::PeriodTo => "periodTo",
            Self::Period => "period",
            Self::Sort => "sort",
        }
    }
}

/// 形の決まった条件が不正なときの理由（BR8.6）。受け取った値そのものは持たない（NFR3.6）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchConditionError {
    pub field: ConditionField,
    pub reason: &'static str,
}

/// 検証前の生の探索条件（Web API のクエリの値。省略は `None`）。
#[derive(Debug, Clone, Copy, Default)]
pub struct RawSearchCondition<'a> {
    pub keyword: Option<&'a str>,
    pub category: Option<&'a str>,
    pub available: Option<&'a str>,
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
    pub sort: Option<&'a str>,
}

/// 空文字は省略と同じに扱う（`?category=` など）。
fn present(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.is_empty())
}

/// `YYYY-MM-DD` の形で実在する暦日なら日付を返す（BR8.4）。
pub fn parse_calendar_date(value: &str) -> Option<NaiveDate> {
    let bytes = value.as_bytes();
    let shaped = bytes.len() == 10
        && bytes.iter().enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            _ => b.is_ascii_digit(),
        });
    if !shaped {
        return None;
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

/// 一つの項目を検証する。省略なら `Ok(None)`、正しければ `Ok(Some(値))`、不正なら理由。
fn check<T>(
    value: Option<&str>,
    parse: impl Fn(&str) -> Option<T>,
    error: SearchConditionError,
) -> Result<Option<T>, SearchConditionError> {
    match present(value) {
        None => Ok(None),
        Some(v) => parse(v).map(Some).ok_or(error),
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// 生の条件を検証して探索条件にする（BR8.1〜BR8.5）。
///
/// 不正な条件があれば、不正な項目と理由をすべて返す（BR8.6）。キーワードは自由入力として
/// 拒否せず、正規化するだけ（BR8.1）。
pub fn parse_search_condition(
    raw: &RawSearchCondition<'_>,
) -> Result<SearchCondition, Vec<SearchConditionError>> {
    let error = |field, reason| SearchConditionError { field, reason };
    let category = check(
        raw.category,
        Category::from_value,
        error(ConditionField::Category, "存在しないカテゴリです"),
    );
    let available = check(
        raw.available,
        parse_bool,
        error(
            ConditionField::AvailableOnly,
            "空席ありのみは true か false で指定してください",
        ),
    );
    let from = check(
        raw.from,
        parse_calendar_date,
        error(
            ConditionField::PeriodFrom,
            "開始日は YYYY-MM-DD の正しい日付で指定してください",
        ),
    );
    let to = check(
        raw.to,
        parse_calendar_date,
        error(
            ConditionField::PeriodTo,
            "終了日は YYYY-MM-DD の正しい日付で指定してください",
        ),
    );
    let sort = check(
        raw.sort,
        SortOrder::from_value,
        error(ConditionField::Sort, "存在しない並べ方です"),
    );
    let period = match (&from, &to) {
        (Ok(Some(f)), Ok(Some(t))) if t < f => {
            Err(error(ConditionField::Period, "終了日が開始日より前です"))
        }
        _ => Ok(()),
    };
    let errors: Vec<SearchConditionError> = [
        category.as_ref().err(),
        available.as_ref().err(),
        from.as_ref().err(),
        to.as_ref().err(),
        period.as_ref().err(),
        sort.as_ref().err(),
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();
    if !errors.is_empty() {
        return Err(errors);
    }
    // ここでは errors が空なので、どの項目も Ok
    Ok(SearchCondition {
        keyword: normalize_keyword(raw.keyword.unwrap_or("")),
        category: category.unwrap_or_default(),
        available_only: available.unwrap_or_default().unwrap_or(false),
        period_from: from.unwrap_or_default(),
        period_to: to.unwrap_or_default(),
        sort: sort.unwrap_or_default().unwrap_or_default(),
    })
}

/// カタカナをひらがなにする（TD-U2-1。カタカナの範囲をひらがなの範囲へずらす）。
/// 「ァ」〜「ヶ」と「ヽ」「ヾ」だけを変え、長音符「ー」などほかの文字はそのまま。
pub fn katakana_to_hiragana(c: char) -> char {
    const SHIFT: u32 = 0x60;
    match c {
        'ァ'..='ヶ' | 'ヽ' | 'ヾ' => char::from_u32(c as u32 - SHIFT).unwrap_or(c),
        _ => c,
    }
}

/// 照合用に文字列を正規化する（BR8.1）: NFKC（全角英数字・記号を半角に、半角カタカナを全角に）、
/// 英字を小文字に、カタカナをひらがなに。長さは切り詰めない。
pub fn normalize_text(text: &str) -> String {
    text.nfkc()
        .flat_map(char::to_lowercase)
        .map(katakana_to_hiragana)
        .collect()
}

/// キーワードを正規化して検索語に分ける（BR8.1）。
///
/// 先頭 [`KEYWORD_MAX_CHARS`] 文字だけを使い（拒否しない）、正規化してから空白（半角・全角など）で
/// 区切り、空の語を捨てる。どれだけ長い入力でも処理は先頭の 200 文字分だけで済む（NFR3.2）。
pub fn normalize_keyword(keyword: &str) -> NormalizedKeyword {
    let head: String = keyword.chars().take(KEYWORD_MAX_CHARS).collect();
    NormalizedKeyword {
        terms: normalize_text(&head)
            .split(char::is_whitespace)
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
            .collect(),
    }
}

/// キーワードの条件（BR9.1）: すべての語が、正規化したコース名か概要のどちらかに含まれる。
/// 照合は文字列の部分一致だけで、正規表現などとしては解釈しない（NFR3.3）。
fn matches_keyword(course: &Course, keyword: &NormalizedKeyword) -> bool {
    if keyword.terms.is_empty() {
        return true;
    }
    let title = normalize_text(&course.title);
    let summary = normalize_text(&course.summary);
    keyword
        .terms
        .iter()
        .all(|term| title.contains(term.as_str()) || summary.contains(term.as_str()))
}

/// 日程の開催開始の日本時間の暦日。
fn start_date_jst(session: &Session) -> NaiveDate {
    let jst = FixedOffset::east_opt(JST_OFFSET_SECONDS).expect("UTC+9 は正しいオフセット");
    session.starts_at.with_timezone(&jst).date_naive()
}

/// 空席のある今後の日程か（BR9.3。受付終了の日程の空席は数えない）。
fn is_open_upcoming<F>(session: &Session, issued_count: &F, now: DateTime<Utc>) -> bool
where
    F: Fn(&SessionId) -> u32,
{
    is_upcoming(session, now) && remaining_seats(session, issued_count(&session.id)) >= 1
}

/// 期間の条件（BR9.4）: 開催開始日が期間内（両端を含む）にある、空席のある今後の日程。
fn within_period(session: &Session, condition: &SearchCondition) -> bool {
    let date = start_date_jst(session);
    condition.period_from.is_none_or(|from| date >= from)
        && condition.period_to.is_none_or(|to| date <= to)
}

/// 日程に関する条件（空席ありのみ・期間）をコースが満たすか（BR9.3、BR9.4）。
fn matches_sessions<F>(
    course: &Course,
    sessions: &[Session],
    condition: &SearchCondition,
    issued_count: &F,
    now: DateTime<Utc>,
) -> bool
where
    F: Fn(&SessionId) -> u32,
{
    let has_period = condition.period_from.is_some() || condition.period_to.is_some();
    if !condition.available_only && !has_period {
        return true;
    }
    sessions.iter().any(|s| {
        s.course_id == course.id
            && is_open_upcoming(s, issued_count, now)
            && (!has_period || within_period(s, condition))
    })
}

/// 指定した並べ方の比較（BR10.1〜BR10.4）。どれも最後はコース ID で比べ、全順序になる（NFR6.3）。
fn compare_rows(order: SortOrder, a: &CourseListRow, b: &CourseListRow) -> Ordering {
    let by_id = || a.course.id.cmp(&b.course.id);
    match order {
        // 直近の開催日は期間の条件に関係なく、満席も含む今後の日程の最も早い日（BR10.1）
        SortOrder::Date => default_order(a, b),
        SortOrder::FeeAsc => a
            .course
            .fee_points
            .cmp(&b.course.fee_points)
            .then_with(by_id),
        SortOrder::FeeDesc => b
            .course
            .fee_points
            .cmp(&a.course.fee_points)
            .then_with(by_id),
        SortOrder::Name => normalize_text(&a.course.title_reading)
            .cmp(&normalize_text(&b.course.title_reading))
            .then_with(by_id),
    }
}

/// 探索条件でコース一覧を絞り込み、並べ替えて返す（BR9.5、BR10）。件数は返した行の数（BR10.5）。
///
/// `issued_count` は日程 ID から発行済みチケット数を返す関数、`now` は Web API の時計の現在時刻。
/// 条件が既定（条件なし）なら、U1 の [`crate::list_courses`] と同じ結果になる。
pub fn search_courses<F>(
    courses: &[Course],
    sessions: &[Session],
    condition: &SearchCondition,
    issued_count: F,
    now: DateTime<Utc>,
) -> Vec<CourseListRow>
where
    F: Fn(&SessionId) -> u32,
{
    let mut rows: Vec<CourseListRow> = courses
        .iter()
        .filter(|c| {
            condition
                .category
                .is_none_or(|category| c.category == category)
        })
        .filter(|c| matches_keyword(c, &condition.keyword))
        .filter(|c| matches_sessions(c, sessions, condition, &issued_count, now))
        .map(|c| summarize_course(c, sessions, &issued_count, now))
        .collect();
    rows.sort_by(|a, b| compare_rows(condition.sort, a, b));
    rows
}
