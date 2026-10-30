//! コース ID と日程 ID の値の型と、形式の検証。

use std::fmt;

/// ID の形式が正しくないことを表すエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdFormatError {
    /// どの ID か（例: `courseId`）。
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

/// `value` が「接頭の 1 文字 + 決まった桁数の数字」の形かどうかを返す。
pub fn has_prefix_and_digits(value: &str, prefix: char, digits: usize) -> bool {
    let mut chars = value.chars();
    value.len() == 1 + digits && chars.next() == Some(prefix) && chars.all(|c| c.is_ascii_digit())
}

/// コース ID（C と 3 桁の数字。例: `C001`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CourseId(String);

impl CourseId {
    /// 形式を確かめてコース ID を作る。
    pub fn parse(value: &str) -> Result<Self, IdFormatError> {
        if has_prefix_and_digits(value, 'C', 3) {
            Ok(Self(value.to_owned()))
        } else {
            Err(IdFormatError {
                kind: "courseId",
                value: value.to_owned(),
            })
        }
    }

    /// 文字列としての値。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CourseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 日程 ID（S と 4 桁の数字。例: `S0001`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(String);

impl SessionId {
    /// 形式を確かめて日程 ID を作る。
    pub fn parse(value: &str) -> Result<Self, IdFormatError> {
        if has_prefix_and_digits(value, 'S', 4) {
            Ok(Self(value.to_owned()))
        } else {
            Err(IdFormatError {
                kind: "sessionId",
                value: value.to_owned(),
            })
        }
    }

    /// 文字列としての値。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
