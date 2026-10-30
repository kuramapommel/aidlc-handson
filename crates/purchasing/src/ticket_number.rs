//! チケット番号の生成（BR5.6）。
//!
//! 使う文字は、紛らわしい文字（0・O・1・I・L）を除いた英大文字と数字。8 文字を選んで
//! 「XXXX-XXXX」の形にする。乱数は呼び出し側から「上限 n を受け取り 0 以上 n 未満の
//! 添字を返す関数」として受け取る（本番では暗号論的に安全な乱数を渡す）。

use std::fmt;

use crate::model::TicketNumber;

/// チケット番号に使う文字（31 文字）。
pub const TICKET_NUMBER_ALPHABET: &str = "ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// 重複したときに作り直す回数の上限。31^8 通りあるため、通常は 1 回目で決まる。
pub const MAX_TICKET_NUMBER_ATTEMPTS: usize = 32;

/// 作り直しの上限まで重複が続いたことを表すエラー（乱数の関数が壊れている可能性が高い）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TicketNumberExhausted;

impl fmt::Display for TicketNumberExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "チケット番号を {MAX_TICKET_NUMBER_ATTEMPTS} 回作り直しても重複しました"
        )
    }
}

impl std::error::Error for TicketNumberExhausted {}

/// 文字列が「XXXX-XXXX」の形で、決められた文字だけを使っているか。
pub fn is_valid_ticket_number(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 9
        && bytes[4] == b'-'
        && bytes
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 4)
            .all(|(_, b)| TICKET_NUMBER_ALPHABET.as_bytes().contains(b))
}

/// 乱数の関数から番号の候補を 1 つ作る。
pub fn ticket_number_candidate<R>(random_index: &mut R) -> TicketNumber
where
    R: FnMut(usize) -> usize,
{
    let alphabet = TICKET_NUMBER_ALPHABET.as_bytes();
    let chars: String = (0..8)
        .map(|_| char::from(alphabet[random_index(alphabet.len()) % alphabet.len()]))
        .collect();
    let formatted = format!("{}-{}", &chars[..4], &chars[4..]);
    TicketNumber::parse(&formatted).expect("決められた文字だけで組み立てているため常に正しい形")
}

/// 発行済みの番号と重ならないチケット番号を作る。重なったら作り直す。
///
/// `is_taken` は番号がすでに使われているかを返す関数。
pub fn generate_ticket_number<R, T>(
    mut random_index: R,
    is_taken: T,
) -> Result<TicketNumber, TicketNumberExhausted>
where
    R: FnMut(usize) -> usize,
    T: Fn(&TicketNumber) -> bool,
{
    (0..MAX_TICKET_NUMBER_ATTEMPTS)
        .map(|_| ticket_number_candidate(&mut random_index))
        .find(|candidate| !is_taken(candidate))
        .ok_or(TicketNumberExhausted)
}
