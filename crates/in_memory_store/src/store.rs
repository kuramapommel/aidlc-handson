//! 排他ロックで守った「現在の状態」を置き換えるデータストア（TD-1）。
//!
//! 可変な共有状態はこの 1 か所（`Mutex<Arc<StoreState>>`）だけ（NFR6.2）。
//! 読み出しはロックの中で現在の状態の写し（`Arc`）を取り出し、計算はロックの外で行う。
//! 更新は「現在の状態 → 新しい状態＋結果」を返す関数をロックの中で 1 件ずつ適用する。

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use chrono::{DateTime, Utc};
use purchasing::{
    PurchaseDecision, PurchaseRequest, TicketId, TicketIdentity, TicketNumberExhausted,
    generate_ticket_number, is_number_taken,
};
use thiserror::Error;

use crate::state::{StoreState, apply_purchase};

/// データストアの予期しない失敗。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// チケット番号が作り直しの上限まで重複した。
    #[error("チケット番号を発行できませんでした: {0}")]
    TicketNumber(#[from] TicketNumberExhausted),
}

/// メモリ上のデータストア。複製しても同じ状態を共有する。
#[derive(Debug, Clone)]
pub struct Store {
    current: Arc<Mutex<Arc<StoreState>>>,
}

impl Store {
    /// 初期の状態からデータストアを作る。
    pub fn new(initial: StoreState) -> Self {
        Self {
            current: Arc::new(Mutex::new(Arc::new(initial))),
        }
    }

    /// ロックを取る。
    ///
    /// ロックを持ったまま処理が異常終了しても、状態の置き換えは新しい状態を作り終えた
    /// 後の 1 回の代入だけなので、中の状態は常に壊れていない。そのためロックの毒化
    /// （poison）からは中の状態をそのまま取り出して処理を続ける（NFR2.5）。
    fn lock(&self) -> MutexGuard<'_, Arc<StoreState>> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 現在の状態の写しを取り出す（読み出し用。計算はロックの外で行う）。
    pub fn snapshot(&self) -> Arc<StoreState> {
        Arc::clone(&self.lock())
    }

    /// 状態の更新を 1 件適用する。`update` が新しい状態を返したときだけ置き換える。
    pub fn apply<T, E, F>(&self, update: F) -> Result<T, E>
    where
        F: FnOnce(&StoreState) -> Result<(Option<StoreState>, T), E>,
    {
        let mut guard = self.lock();
        let (next, result) = update(&guard)?;
        if let Some(next) = next {
            *guard = Arc::new(next);
        }
        Ok(result)
    }

    /// 購入を記録する（BR5.3、BR5.4）。
    ///
    /// ロックの中で最新の状態から写しを作り、決定関数を評価し直す。承認ならチケットと
    /// 消費の取引を同時に追加した新しい状態に置き換え、拒否なら状態を変えない。
    /// `random_index` はチケット番号の乱数（BR5.6）、`new_ticket_id` はチケット ID を作る関数。
    ///
    /// PostgreSQL に移すときに必要な保証（NFR7.3、ADR-004）:
    /// - 写しの読み出し・決定・チケットと消費の取引の挿入を 1 つのトランザクションで行う。
    /// - 定員の超過は、対象の日程の行を `SELECT ... FOR UPDATE` でロックしてから発行済み数を
    ///   数えることで防ぐ（このメモリ実装では全体の排他ロックがその役目）。
    /// - 同じ従業員の同じ日程の重複は、チケットの `(employee_id, session_id)` の一意制約で防ぐ。
    /// - チケット番号の重複は `ticket_number` の一意制約で防ぎ、違反したら作り直す。
    /// - 残高が負にならないことは、同じトランザクションの中で従業員の取引を数え直して確かめる
    ///   （従業員の行のロックで、同じ従業員の同時の購入を直列にする）。
    pub fn record_purchase<R, U>(
        &self,
        request: &PurchaseRequest,
        now: DateTime<Utc>,
        random_index: R,
        new_ticket_id: U,
    ) -> Result<PurchaseDecision, StoreError>
    where
        R: FnMut(usize) -> usize,
        U: FnOnce() -> TicketId,
    {
        self.apply(|state| {
            let ticket_number =
                generate_ticket_number(random_index, |n| is_number_taken(&state.tickets, n))?;
            let identity = TicketIdentity {
                ticket_id: new_ticket_id(),
                ticket_number,
            };
            Ok(apply_purchase(state, request, now, identity))
        })
    }
}
