//! アダプター InMemoryStore。
//!
//! 業務の部品が定める型と関数の形を、メモリ上の不変なデータで実装する（ADR-001、NFR7）。
//! 将来は同じ関数の形の PostgreSQL の実装に差し替える。Web API を再起動すると初期データに戻る。

pub mod seed;
pub mod state;
pub mod store;

pub use seed::seed_state;
pub use state::{StoreState, apply_purchase, purchase_snapshot};
pub use store::{Store, StoreError};

#[cfg(test)]
mod tests;
