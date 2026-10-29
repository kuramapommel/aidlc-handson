//! 操作中の従業員 ID の Cookie の読み書き（BR6.1、BR6.2、NFR3.4）。
//!
//! **ローカル検証専用**: Cookie に入った従業員 ID を、そのまま認証済みの従業員として扱う
//! （ADR-008、NFR3.8）。Cookie の値を書き換えれば他人として操作できるため、このサーバーを
//! 公開された環境に置いてはならない。社内ポータルの認証につなぐときは、この読み取りを
//! ポータルが発行するセッションから従業員 ID を取り出す処理に差し替える。

use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use eligibility::EmployeeId;

/// 操作中の従業員 ID を入れる Cookie の名前。
pub const EMPLOYEE_COOKIE: &str = "aidlc_employee_id";

/// Cookie から読み取った操作中の従業員。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CookieEmployee {
    /// Cookie がない。
    Missing,
    /// Cookie がある。形式が正しければその従業員 ID、不正なら `None`（存在しない従業員として扱う）。
    Present(Option<EmployeeId>),
}

impl CookieEmployee {
    /// 従業員 ID（Cookie がない・形式不正なら `None`）。
    pub fn employee_id(&self) -> Option<&EmployeeId> {
        match self {
            Self::Missing | Self::Present(None) => None,
            Self::Present(Some(id)) => Some(id),
        }
    }
}

/// 要求の Cookie から操作中の従業員を読み取る（ローカル検証専用。上の注意を参照）。
pub fn read_employee(jar: &CookieJar) -> CookieEmployee {
    jar.get(EMPLOYEE_COOKIE)
        .map_or(CookieEmployee::Missing, |c| {
            CookieEmployee::Present(EmployeeId::parse(c.value()).ok())
        })
}

/// 従業員 ID を入れた Cookie を作る。HttpOnly・SameSite=Lax・Path=/。
/// ローカルの http で動かすため Secure は付けない（NFR3.4。公開しない前提）。
pub fn employee_cookie(employee_id: &EmployeeId) -> Cookie<'static> {
    Cookie::build((EMPLOYEE_COOKIE, employee_id.as_str().to_owned()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}
