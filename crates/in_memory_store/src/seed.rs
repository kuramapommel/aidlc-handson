//! 初期データ（functional-spec.md 6 節、FR7）。
//!
//! 起動するたびに、渡された現在時刻の日本時間の日付を基準にして日程を生成する（FR7.3）。
//! 時刻は引数で受け取るため、テストでは固定の時刻で毎回同じデータになる（NFR8.4）。
//! 満席の日程は、初期データとして発行済みのチケット（と対応する消費の取引）で作る。
//! そのチケットも購入の決定関数（BR5.1〜BR5.6）を通して作る。

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Asia::Tokyo;
use course_catalog::{Category, Course, CourseId, DeliveryFormat, Session, SessionId};
use eligibility::{Company, CompanyId, ContractStatus, Employee, EmployeeId};
use purchasing::{
    PurchaseDecision, PurchaseRequest, TicketId, TicketIdentity, generate_ticket_number,
    is_number_taken,
};
use training_budget::{BudgetEntry, EmployeeRef, EntryId, EntryKind};

use crate::state::{StoreState, apply_purchase};

/// 企業: (ID, 名前, 契約状態)
const COMPANIES: &[(&str, &str, ContractStatus)] = &[
    ("K001", "株式会社サンプル商事", ContractStatus::Active),
    ("K002", "みらい技研株式会社", ContractStatus::Active),
    ("K003", "旧契約産業株式会社", ContractStatus::Ended),
];

/// 従業員: (ID, 表示名, 企業 ID, 付与するポイント)。先頭は高橋 遥（BR6.2）。
const EMPLOYEES: &[(&str, &str, &str, u32)] = &[
    ("E001", "高橋 遥", "K001", 60_000),
    ("E002", "佐藤 健太", "K001", 25_000), // 残高ちょうど 25,000 pt の確認用
    ("E003", "鈴木 美咲", "K002", 20_000), // 残高不足の確認用
    ("E004", "田中 一郎", "K003", 50_000), // 契約終了の企業の従業員
    ("E005", "伊藤 さくら", "K002", 100_000), // 初期データの満席の日程を購入済み
    ("E006", "渡辺 翔", "K002", 100_000),  // 初期データの満席の日程を購入済み
];

/// コースの文章: (ID, コース名, カテゴリ, 概要, 説明, 受講料, 所要時間（分）, 形式)
type CourseRow = (
    &'static str,
    &'static str,
    Category,
    &'static str,
    &'static str,
    u32,
    u32,
    DeliveryFormat,
);

const COURSES: &[CourseRow] = &[
    (
        "C001",
        "チームマネジメント入門",
        Category::Management,
        "初めてチームを率いる人のための、目標設定と役割分担の基本。",
        "チームの目標の立て方、役割分担、進み具合の確かめ方を、演習を通して学びます。初めてリーダーになった方や、これからなる方に向けた入門の講座です。",
        25_000,
        180,
        DeliveryFormat::Venue,
    ),
    (
        "C002",
        "1on1 ミーティング実践",
        Category::Management,
        "部下の成長を支える 1on1 の進め方を、ロールプレイで身につける。",
        "傾聴と質問の技法、話題の選び方、記録の残し方を、ロールプレイで練習します。",
        18_000,
        120,
        DeliveryFormat::Online,
    ),
    (
        "C003",
        "評価面談の進め方",
        Category::Management,
        "納得感のある評価面談のための準備と伝え方。",
        "評価の根拠の整理、伝え方、面談後のフォローまでを、事例をもとに学びます。",
        22_000,
        150,
        DeliveryFormat::Venue,
    ),
    (
        "C004",
        "Rust による Web API 開発",
        Category::Technology,
        "Rust と axum で、安全で速い Web API を作る実践講座。",
        "所有権と型を生かした設計、エラー処理、テストの書き方を、小さな Web API を作りながら学びます。",
        40_000,
        360,
        DeliveryFormat::Venue,
    ),
    (
        "C005",
        "クラウド設計の基礎",
        Category::Technology,
        "可用性とコストを考えたクラウドの構成の基本。",
        "代表的な構成の型と、可用性・性能・コストの考え方を、図を描きながら学びます。",
        30_000,
        240,
        DeliveryFormat::Online,
    ),
    (
        "C006",
        "データ分析入門",
        Category::Technology,
        "表計算から一歩進んだ、データの集計と可視化の基本。",
        "データの整え方、集計、グラフでの見せ方を、業務のデータに近い例で練習します。",
        25_000,
        180,
        DeliveryFormat::Online,
    ),
    (
        "C007",
        "生成 AI 活用ワークショップ",
        Category::Technology,
        "業務で生成 AI を安全に使うための考え方と実践。",
        "指示の書き方、結果の確かめ方、情報の扱いの注意点を、ワークショップ形式で学びます。",
        35_000,
        240,
        DeliveryFormat::Venue,
    ),
    (
        "C008",
        "ロジカルシンキング",
        Category::BusinessSkills,
        "筋道を立てて考え、分かりやすく伝えるための基本の型。",
        "主張と根拠の組み立て、問題の分け方、結論から話す練習をします。",
        15_000,
        120,
        DeliveryFormat::Online,
    ),
    (
        "C009",
        "プレゼンテーション技法",
        Category::BusinessSkills,
        "伝わる資料と話し方を、実際の発表で身につける。",
        "構成の作り方、資料の見せ方、話し方を学び、短い発表を行って講評を受けます。",
        20_000,
        180,
        DeliveryFormat::Venue,
    ),
    (
        "C010",
        "交渉力強化講座",
        Category::BusinessSkills,
        "双方が納得する合意をつくるための交渉の準備と進め方。",
        "交渉の準備、条件の整理、合意の作り方を、ケーススタディで学びます。",
        28_000,
        240,
        DeliveryFormat::Venue,
    ),
    (
        "C011",
        "情報セキュリティ基礎",
        Category::Compliance,
        "日々の業務で守るべき情報セキュリティの基本。",
        "パスワードの扱い、不審なメールの見分け方、情報の持ち出しの注意点を学びます。",
        10_000,
        90,
        DeliveryFormat::Online,
    ),
    (
        "C012",
        "ハラスメント防止研修",
        Category::Compliance,
        "職場のハラスメントを防ぐための知識と対応。",
        "ハラスメントの種類、起きたときの対応、相談の受け方を学びます。",
        12_000,
        90,
        DeliveryFormat::Online,
    ),
    (
        "C013",
        "個人情報保護の実務",
        Category::Compliance,
        "個人情報を扱う業務で守るべきルールと手順。",
        "個人情報の取得・保管・廃棄のルールと、漏えいを防ぐための手順を学びます。",
        16_000,
        120,
        DeliveryFormat::Venue,
    ),
];

/// 日程: (ID, コース ID, 起動日からの日数, 開始の時（日本時間）, 定員, 会場)
///
/// - S0002・S0011 は定員 2 で、初期データのチケット 2 枚により満席
/// - S0003・S0014・S0019 は過去の日程（受付終了の確認用）。C012 は過去の日程だけ（今後の開催予定なし）
/// - S0005 は残席 1、S0001 は残席 3（受講料 25,000 pt）
/// - C013 は日程なし
type SessionRow = (
    &'static str,
    &'static str,
    i64,
    u32,
    u32,
    Option<&'static str>,
);

const SESSIONS: &[SessionRow] = &[
    ("S0001", "C001", 7, 13, 3, Some("東京本社 研修室A")),
    ("S0002", "C001", 14, 13, 2, Some("東京本社 研修室A")),
    ("S0003", "C001", -3, 13, 10, Some("東京本社 研修室A")),
    ("S0004", "C002", 5, 10, 20, None),
    ("S0005", "C002", 21, 10, 1, None),
    ("S0006", "C003", 10, 14, 8, Some("大阪支社 セミナールーム")),
    ("S0007", "C004", 30, 10, 12, Some("東京本社 大会議室")),
    ("S0008", "C004", 60, 10, 12, Some("東京本社 大会議室")),
    ("S0009", "C005", 3, 13, 30, None),
    ("S0010", "C006", 9, 13, 15, None),
    ("S0011", "C006", 40, 13, 2, None),
    ("S0012", "C007", 12, 13, 6, Some("大阪支社 セミナールーム")),
    ("S0013", "C008", 2, 10, 50, None),
    ("S0014", "C008", -10, 10, 50, None),
    ("S0015", "C009", 18, 13, 10, Some("名古屋支社 研修室")),
    ("S0016", "C010", 45, 13, 8, Some("東京本社 研修室B")),
    ("S0017", "C011", 4, 10, 100, None),
    ("S0018", "C011", 90, 10, 100, None),
    ("S0019", "C012", -20, 15, 30, None),
];

/// 初期データで購入済みにする (従業員 ID, 日程 ID)。満席の日程を作るため。
const SEED_PURCHASES: &[(&str, &str)] = &[
    ("E005", "S0002"),
    ("E006", "S0002"),
    ("E005", "S0011"),
    ("E006", "S0011"),
];

fn id<T, E: std::fmt::Debug>(parsed: Result<T, E>) -> T {
    parsed.expect("初期データの ID は正しい形式で書いている")
}

/// 日本時間の「基準日 + days 日の hour 時」を UTC の時刻にする。
fn jst(base: NaiveDate, days: i64, hour: u32) -> DateTime<Utc> {
    let local = (base + Duration::days(days))
        .and_hms_opt(hour, 0, 0)
        .expect("0〜23 時の正しい時刻");
    Tokyo
        .from_local_datetime(&local)
        .single()
        .expect("日本時間には夏時間がないため時刻は 1 つに決まる")
        .with_timezone(&Utc)
}

fn companies() -> Vec<Company> {
    COMPANIES
        .iter()
        .map(|(cid, name, status)| Company {
            id: id(CompanyId::parse(cid)),
            name: (*name).to_owned(),
            contract_status: *status,
        })
        .collect()
}

fn employees() -> Vec<Employee> {
    EMPLOYEES
        .iter()
        .map(|(eid, name, cid, _)| Employee {
            id: id(EmployeeId::parse(eid)),
            display_name: (*name).to_owned(),
            company_id: id(CompanyId::parse(cid)),
        })
        .collect()
}

fn grants(granted_at: DateTime<Utc>) -> Vec<BudgetEntry> {
    EMPLOYEES
        .iter()
        .map(|(eid, _, _, points)| BudgetEntry {
            id: EntryId(format!("grant-{eid}-1")),
            employee: EmployeeRef((*eid).to_owned()),
            kind: EntryKind::Grant,
            points: *points,
            occurred_at: granted_at,
        })
        .collect()
}

fn courses() -> Vec<Course> {
    COURSES
        .iter()
        .map(
            |(cid, title, category, summary, description, fee, minutes, format)| Course {
                id: id(CourseId::parse(cid)),
                title: (*title).to_owned(),
                category: *category,
                summary: (*summary).to_owned(),
                description: (*description).to_owned(),
                fee_points: *fee,
                duration_minutes: *minutes,
                delivery_format: *format,
            },
        )
        .collect()
}

fn sessions(base: NaiveDate, courses: &[Course]) -> Vec<Session> {
    SESSIONS
        .iter()
        .map(|(sid, cid, days, hour, capacity, venue)| {
            let course_id = id(CourseId::parse(cid));
            let minutes = courses
                .iter()
                .find(|c| c.id == course_id)
                .map_or(60, |c| c.duration_minutes);
            let starts_at = jst(base, *days, *hour);
            Session {
                id: id(SessionId::parse(sid)),
                course_id,
                starts_at,
                ends_at: starts_at + Duration::minutes(i64::from(minutes)),
                venue: venue.map(str::to_owned),
                capacity: *capacity,
            }
        })
        .collect()
}

/// 初期データのチケットの識別子（決まった値。番号は決まった並びの「乱数」で作る）。
fn seed_identity(state: &StoreState, ordinal: usize) -> TicketIdentity {
    let mut counter = ordinal * 8;
    let ticket_number = generate_ticket_number(
        move |n| {
            counter += 7;
            counter % n
        },
        |candidate| is_number_taken(&state.tickets, candidate),
    )
    .expect("決まった並びでも作り直しの上限までに重複しない番号が見つかる");
    TicketIdentity {
        ticket_id: id(TicketId::parse(&format!(
            "00000000-0000-4000-8000-{:012x}",
            ordinal + 1
        ))),
        ticket_number,
    }
}

/// 初期データの購入を、購入の状態遷移（決定関数）を通して反映する。
fn apply_seed_purchases(base_state: StoreState, purchased_at: DateTime<Utc>) -> StoreState {
    SEED_PURCHASES
        .iter()
        .enumerate()
        .fold(base_state, |state, (ordinal, (eid, sid))| {
            let request = PurchaseRequest {
                employee_id: Some(id(EmployeeId::parse(eid))),
                session_id: id(SessionId::parse(sid)),
            };
            let identity = seed_identity(&state, ordinal);
            match apply_purchase(&state, &request, purchased_at, identity) {
                (Some(next), PurchaseDecision::Approved(_)) => next,
                (_, decision) => {
                    panic!("初期データの購入が承認されませんでした: {eid} {sid} {decision:?}")
                }
            }
        })
}

/// 現在時刻を基準に初期データを作る。
pub fn seed_state(now: DateTime<Utc>) -> StoreState {
    let base = now.with_timezone(&Tokyo).date_naive();
    let courses = courses();
    let sessions = sessions(base, &courses);
    let base_state = StoreState {
        companies: companies(),
        employees: employees(),
        budget_entries: grants(jst(base, -30, 9)),
        courses,
        sessions,
        tickets: Vec::new(),
    };
    apply_seed_purchases(base_state, now - Duration::days(1))
}
