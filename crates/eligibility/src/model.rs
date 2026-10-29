//! 企業と従業員のデータの形（entities.md の Company・Employee）と ID の値の型。

use std::fmt;

/// ID の形式が正しくないことを表すエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdFormatError {
    /// どの ID か（例: `employeeId`）。
    pub kind: &'static str,
    /// 受け取った値。
    pub value: String,
}

impl fmt::Display for IdFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} の形式が正しくありません: {:?}",
            self.kind, self.value
        )
    }
}

impl std::error::Error for IdFormatError {}

fn has_prefix_and_digits(value: &str, prefix: char, digits: usize) -> bool {
    let mut chars = value.chars();
    value.len() == 1 + digits && chars.next() == Some(prefix) && chars.all(|c| c.is_ascii_digit())
}

fn parse_prefixed(
    value: &str,
    prefix: char,
    digits: usize,
    kind: &'static str,
) -> Result<String, IdFormatError> {
    if has_prefix_and_digits(value, prefix, digits) {
        Ok(value.to_owned())
    } else {
        Err(IdFormatError {
            kind,
            value: value.to_owned(),
        })
    }
}

/// 企業 ID（K と 3 桁の数字。例: `K001`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompanyId(String);

impl CompanyId {
    /// 形式を確かめて企業 ID を作る。
    pub fn parse(value: &str) -> Result<Self, IdFormatError> {
        parse_prefixed(value, 'K', 3, "companyId").map(Self)
    }

    /// 文字列としての値。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 従業員 ID（E と 3 桁の数字。例: `E001`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EmployeeId(String);

impl EmployeeId {
    /// 形式を確かめて従業員 ID を作る。
    pub fn parse(value: &str) -> Result<Self, IdFormatError> {
        parse_prefixed(value, 'E', 3, "employeeId").map(Self)
    }

    /// 文字列としての値。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmployeeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 契約状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContractStatus {
    Active,
    Ended,
}

impl ContractStatus {
    /// Web API などで使う値。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Ended => "ended",
        }
    }
}

/// 契約企業。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Company {
    pub id: CompanyId,
    pub name: String,
    pub contract_status: ContractStatus,
}

/// 従業員。社内ポータルで認証済みとみなす（ADR-008）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Employee {
    pub id: EmployeeId,
    pub display_name: String,
    pub company_id: CompanyId,
}
