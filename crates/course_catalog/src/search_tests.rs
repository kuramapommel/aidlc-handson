//! 探索（U2）のテスト。データストアを使わず、固定のコース・日程と、固定の発行済み数を返す
//! クロージャ、固定の時刻を渡して動かす（NFR7.2、NFR8.3）。
//! 実行: `cargo test -p course_catalog search_tests`

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};

use crate::*;

/// 2026-11-02T10:00:00+09:00
fn fixed_now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 2, 1, 0, 0).unwrap()
}

/// 日本時間の 2026-11-02 から `days` 日後の 13 時。
fn jst_day(days: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 2, 4, 0, 0).unwrap() + Duration::days(days)
}

fn date(text: &str) -> NaiveDate {
    parse_calendar_date(text).unwrap()
}

fn course(
    id: &str,
    title: &str,
    reading: &str,
    category: Category,
    summary: &str,
    fee: u32,
) -> Course {
    Course {
        id: CourseId::parse(id).unwrap(),
        title: title.into(),
        title_reading: reading.into(),
        category,
        summary: summary.into(),
        description: "説明".into(),
        fee_points: fee,
        duration_minutes: 120,
        delivery_format: DeliveryFormat::Online,
    }
}

fn session(id: &str, course_id: &str, starts_at: DateTime<Utc>, capacity: u32) -> Session {
    Session {
        id: SessionId::parse(id).unwrap(),
        course_id: CourseId::parse(course_id).unwrap(),
        starts_at,
        ends_at: starts_at + Duration::hours(2),
        venue: None,
        capacity,
    }
}

/// 境目を持つ小さなテスト用のデータ。
/// - C001 と C002 は直近の開催日時が同じ（11/5）。C001 と C003 は受講料が同じ（25,000）
/// - C003 は 11/9 が満席、11/22 に空席。C005 は今後の日程が満席だけ
/// - C004 は受付終了の日程にだけ空席。C006 は日程なしで、よみがなが C001 と同じ
fn fixture() -> (Vec<Course>, Vec<Session>) {
    let courses = vec![
        course(
            "C001",
            "チームマネジメント入門",
            "ちーむまねじめんとにゅうもん",
            Category::Management,
            "初めてリーダーになる人のための基本。",
            25_000,
        ),
        course(
            "C002",
            "生成 AI 活用",
            "せいせいえーあいかつよう",
            Category::Technology,
            "業務で AI を安全に使う。",
            35_000,
        ),
        course(
            "C003",
            "データ分析入門",
            "でーたぶんせきにゅうもん",
            Category::Technology,
            "集計と可視化の基本。",
            25_000,
        ),
        course(
            "C004",
            "情報セキュリティ",
            "じょうほうせきゅりてぃ",
            Category::Compliance,
            "守るべき基本。",
            10_000,
        ),
        course(
            "C005",
            "交渉力",
            "こうしょうりょく",
            Category::BusinessSkills,
            "合意をつくる。",
            28_000,
        ),
        course(
            "C006",
            "チーム運営",
            "ちーむまねじめんとにゅうもん",
            Category::Management,
            "チームを回す。",
            22_000,
        ),
    ];
    let sessions = vec![
        session("S0001", "C001", jst_day(3), 3),
        session("S0002", "C001", jst_day(10), 2),
        session("S0003", "C002", jst_day(3), 5),
        session("S0004", "C003", jst_day(7), 1),
        session("S0005", "C003", jst_day(20), 4),
        session("S0006", "C004", jst_day(-1), 30),
        session("S0007", "C005", jst_day(5), 2),
    ];
    (courses, sessions)
}

/// 満席の日程（S0002・S0004・S0007）の発行済み数を返す関数（データストアの代わり）。
fn issued(id: &SessionId) -> u32 {
    match id.as_str() {
        "S0002" | "S0007" => 2,
        "S0004" => 1,
        _ => 0,
    }
}

fn condition(raw: RawSearchCondition<'_>) -> SearchCondition {
    parse_search_condition(&raw).expect("正しい条件のはず")
}

/// 条件で探索したコース ID の並び。
fn search_ids(raw: RawSearchCondition<'_>) -> Vec<String> {
    let (courses, sessions) = fixture();
    search_courses(&courses, &sessions, &condition(raw), issued, fixed_now())
        .iter()
        .map(|r| r.course.id.as_str().to_owned())
        .collect()
}

fn terms(keyword: &str) -> Vec<String> {
    normalize_keyword(keyword).terms
}

// ---------- 正規化（BR8.1） ----------

#[test]
fn keyword_normalizes_width_case_and_kana() {
    assert_eq!(terms("ＡＩ"), ["ai"]);
    assert_eq!(terms("ai"), terms("AI"));
    // カタカナ・半角カタカナはひらがなにそろえる（長音符はそのまま）
    assert_eq!(terms("マネジメント"), ["まねじめんと"]);
    assert_eq!(terms("ﾏﾈｼﾞﾒﾝﾄ"), ["まねじめんと"]);
    assert_eq!(normalize_text("データ"), "でーた");
    assert_eq!(katakana_to_hiragana('ー'), 'ー');
    // 全角スペースでも区切る
    assert_eq!(terms("AI　入門"), ["ai", "入門"]);
}

#[test]
fn keyword_of_only_spaces_is_no_condition_and_long_input_uses_first_200_chars() {
    assert!(terms(" 　\t ").is_empty());
    assert!(terms("").is_empty());
    // 200 文字目が空白なので、それより後の語は捨てられる
    let long = format!("{} {}", "あ".repeat(199), "い".repeat(50));
    assert_eq!(terms(&long), ["あ".repeat(199)]);
    // 数万文字でも拒否しない
    assert_eq!(terms(&"x".repeat(50_000)), ["x".repeat(KEYWORD_MAX_CHARS)]);
    // 記号・HTML の記号もそのまま検索語として扱う
    assert_eq!(terms("<script>"), ["<script>"]);
}

// ---------- 検証（BR8.2〜BR8.6） ----------

fn error_fields(raw: RawSearchCondition<'_>) -> Vec<&'static str> {
    parse_search_condition(&raw)
        .expect_err("不正なはず")
        .iter()
        .map(|e| e.field.as_str())
        .collect()
}

#[test]
fn valid_raw_condition_is_parsed_and_omitted_fields_are_defaults() {
    let parsed = condition(RawSearchCondition {
        keyword: Some("AI 入門"),
        category: Some("business-skills"),
        available: Some("true"),
        from: Some("2026-11-05"),
        to: Some("2026-11-05"),
        sort: Some("fee-desc"),
    });
    assert_eq!(parsed.keyword.terms, ["ai", "入門"]);
    assert_eq!(parsed.category, Some(Category::BusinessSkills));
    assert!(parsed.available_only);
    assert_eq!(parsed.period_from, Some(date("2026-11-05")));
    assert_eq!(parsed.sort, SortOrder::FeeDesc);
    // 省略と空の値は「条件なし」
    let empty = condition(RawSearchCondition {
        category: Some(""),
        sort: Some(""),
        ..RawSearchCondition::default()
    });
    assert_eq!(empty, SearchCondition::default());
    assert_eq!(empty.sort, SortOrder::Date);
}

#[test]
fn each_invalid_shaped_condition_is_reported() {
    let cases: [(RawSearchCondition<'_>, &str); 7] = [
        (
            RawSearchCondition {
                category: Some("cooking"),
                ..Default::default()
            },
            "category",
        ),
        (
            RawSearchCondition {
                available: Some("yes"),
                ..Default::default()
            },
            "availableOnly",
        ),
        (
            RawSearchCondition {
                from: Some("2026-1-05"),
                ..Default::default()
            },
            "periodFrom",
        ),
        (
            RawSearchCondition {
                to: Some("2026-02-30"),
                ..Default::default()
            },
            "periodTo",
        ),
        (
            RawSearchCondition {
                from: Some("2026-11-10"),
                to: Some("2026-11-09"),
                ..Default::default()
            },
            "period",
        ),
        (
            RawSearchCondition {
                sort: Some("price"),
                ..Default::default()
            },
            "sort",
        ),
        (
            RawSearchCondition {
                category: Some("Management"),
                ..Default::default()
            },
            "category",
        ),
    ];
    for (raw, field) in cases {
        assert_eq!(error_fields(raw), [field]);
    }
}

#[test]
fn all_invalid_conditions_are_listed_and_keyword_never_fails() {
    let fields = error_fields(RawSearchCondition {
        keyword: Some("<script>alert(1)</script>"),
        category: Some("x"),
        available: Some("1"),
        from: Some("tomorrow"),
        to: Some("2026-13-01"),
        sort: Some("random"),
    });
    assert_eq!(
        fields,
        [
            "category",
            "availableOnly",
            "periodFrom",
            "periodTo",
            "sort"
        ]
    );
    // 理由は値そのものを含まない（NFR3.6）
    let errors = parse_search_condition(&RawSearchCondition {
        category: Some("<b>x</b>"),
        ..Default::default()
    })
    .unwrap_err();
    assert!(!errors[0].reason.contains("<b>"));
    // キーワードだけなら、どんな文字列でも不正にならない
    assert!(
        parse_search_condition(&RawSearchCondition {
            keyword: Some(&"漢".repeat(30_000)),
            ..Default::default()
        })
        .is_ok()
    );
}

// ---------- 絞り込み（BR9.1〜BR9.5） ----------

#[test]
fn keyword_matches_title_or_summary_with_all_terms() {
    let by_keyword = |q| {
        search_ids(RawSearchCondition {
            keyword: Some(q),
            ..Default::default()
        })
    };
    // AC1.2.1: 「入門」を含むコースだけ
    assert_eq!(by_keyword("入門"), ["C001", "C003"]);
    // 概要に含まれていても一致する
    assert_eq!(by_keyword("リーダー"), ["C001"]);
    // AC1.2.3: 全角・小文字・ひらがなでも同じ結果
    assert_eq!(by_keyword("ＡＩ"), by_keyword("AI"));
    assert_eq!(by_keyword("ai"), ["C002"]);
    assert_eq!(by_keyword("まねじめんと"), by_keyword("マネジメント"));
    // AC1.2.2: 複数の語は AND。どちらにも合うコースがなければ 0 件
    assert_eq!(by_keyword("AI 入門"), Vec::<String>::new());
    assert_eq!(by_keyword("データ 入門"), ["C003"]);
    // AC1.2.5: 空白だけは条件なし
    assert_eq!(by_keyword("　").len(), 6);
}

#[test]
fn category_and_available_only_filters() {
    let tech = search_ids(RawSearchCondition {
        category: Some("technology"),
        ..Default::default()
    });
    assert_eq!(tech, ["C002", "C003"]);
    // 空席のある今後の日程があるコースだけ。受付終了の日程にだけ空席がある C004、
    // 満席だけの C005、日程がない C006 は出ない（AC1.4.1、AC1.4.2）
    let available = search_ids(RawSearchCondition {
        available: Some("true"),
        ..Default::default()
    });
    assert_eq!(available, ["C001", "C002", "C003"]);
    // false は指定なしと同じ（AC1.4.3）
    let all = search_ids(RawSearchCondition {
        available: Some("false"),
        ..Default::default()
    });
    assert_eq!(all.len(), 6);
}

#[test]
fn period_includes_both_ends_and_requires_an_open_upcoming_session() {
    let by_period = |from: Option<&'static str>, to: Option<&'static str>| {
        search_ids(RawSearchCondition {
            from,
            to,
            ..Default::default()
        })
    };
    // 開始日 = 終了日: その日に開催開始する空席のある日程（AC1.5.2）
    assert_eq!(
        by_period(Some("2026-11-05"), Some("2026-11-05")),
        ["C001", "C002"]
    );
    // 11/9 の日程は満席なので出ない。11/7 の C005 も満席
    assert_eq!(
        by_period(Some("2026-11-07"), Some("2026-11-09")),
        Vec::<String>::new()
    );
    // 開始日だけ: 開始日以降（AC1.5.4）
    assert_eq!(by_period(Some("2026-11-12"), None), ["C003"]);
    // 終了日だけ: 終了日以前の今後の日程（過去の日程の C004 は出ない）
    assert_eq!(by_period(None, Some("2026-11-04")), Vec::<String>::new());
    // 条件の組み合わせは AND（BR9.5）
    let combined = search_ids(RawSearchCondition {
        keyword: Some("入門"),
        category: Some("technology"),
        from: Some("2026-11-01"),
        ..Default::default()
    });
    assert_eq!(combined, ["C003"]);
}

// ---------- 並べ替えと件数（BR10.1〜BR10.5） ----------

#[test]
fn four_sort_orders_break_ties_by_course_id() {
    let by_sort = |sort| {
        search_ids(RawSearchCondition {
            sort: Some(sort),
            ..Default::default()
        })
    };
    // 開催日が近い順: 同じ日時の C001・C002 は ID 順、予定なしの C004・C006 は最後
    assert_eq!(
        by_sort("date"),
        ["C001", "C002", "C005", "C003", "C004", "C006"]
    );
    // 受講料: 同じ 25,000 の C001・C003 は ID 順（AC1.7.1、AC1.7.2）
    assert_eq!(
        by_sort("fee-asc"),
        ["C004", "C006", "C001", "C003", "C005", "C002"]
    );
    assert_eq!(
        by_sort("fee-desc"),
        ["C002", "C005", "C001", "C003", "C006", "C004"]
    );
    // コース名順はよみがなの五十音順。同じよみがなの C001・C006 は ID 順（AC1.7.3）
    assert_eq!(
        by_sort("name"),
        ["C005", "C004", "C002", "C001", "C006", "C003"]
    );
}

#[test]
fn date_order_uses_nearest_upcoming_session_even_outside_the_period() {
    let (courses, sessions) = fixture();
    let rows = search_courses(
        &courses,
        &sessions,
        &condition(RawSearchCondition {
            from: Some("2026-11-12"),
            ..Default::default()
        }),
        issued,
        fixed_now(),
    );
    assert_eq!(rows.len(), 1);
    // 期間（11/12 以降）で絞り込んでも、直近の開催日は期間の外の 11/9（満席）を数える（BR10.1）
    assert_eq!(rows[0].next_session_starts_at, Some(jst_day(7)));
}

#[test]
fn default_condition_returns_the_same_as_the_u1_list() {
    let (courses, sessions) = fixture();
    let searched = search_courses(
        &courses,
        &sessions,
        &SearchCondition::default(),
        issued,
        fixed_now(),
    );
    assert_eq!(
        searched,
        list_courses(&courses, &sessions, issued, fixed_now())
    );
}
