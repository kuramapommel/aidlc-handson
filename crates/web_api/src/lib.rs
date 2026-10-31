//! アダプター WebApi（ローカル検証専用）。
//!
//! HTTP で画面からの要求を受け、業務の部品を組み合わせて JSON を返す。起動時に
//! データストアの関数を業務の関数に部分適用して組み立てる（[`app::router`]）。
//!
//! **公開された環境に置いてはならない**: Cookie の従業員 ID を認証済みとして扱う
//! （[`session_cookie`] の注意を参照。NFR3.8）。

pub mod app;
pub mod config;
pub mod csrf;
pub mod dto;
pub mod errors;
pub mod handlers;
pub mod session_cookie;

pub use app::{Ports, purchase_recorder, router, state_reader};
pub use config::ApiConfig;
