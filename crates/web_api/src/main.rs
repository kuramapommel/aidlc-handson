//! Web API の実行ファイル（ローカル検証専用。公開された環境に置いてはならない）。
//!
//! 環境変数: `API_PORT`（既定 3000）、`UI_ORIGIN`（既定 `http://localhost:8080`）、
//! `RUST_LOG`（ログの詳しさ。既定 `info`。開発中は `debug` にできる）。
//! 待ち受けるのは 127.0.0.1 だけ（同じ PC の外からはつながらない）。

use std::net::SocketAddr;
use std::process::ExitCode;

use chrono::Utc;
use in_memory_store::{Store, seed_state};
use tracing_subscriber::EnvFilter;
use web_api::{ApiConfig, Ports, purchase_recorder, router, state_reader};

/// 人が読める 1 行の形式のログを標準出力に出す（NFR10.4）。端末でなければ色を付けない。
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .init();
}

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    let config = match ApiConfig::from_env() {
        Ok(config) => config,
        Err(message) => {
            tracing::error!("設定が正しくありません: {message}");
            return ExitCode::FAILURE;
        }
    };
    // 起動した時刻を基準に初期データを作る（FR7.3）。再起動すると初期データに戻る。
    let store = Store::new(seed_state(Utc::now()));
    // データストアの関数を部分適用して、受け口が使う関数の組を作る
    let ports = Ports {
        read_state: state_reader(store.clone()),
        record_purchase: purchase_recorder(store),
        now: Utc::now,
    };
    let app = router(ports, &config);

    let address = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!("ポート {} で待ち受けられません: {error}", config.port);
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(
        "Web API を http://localhost:{} で待ち受けます（許可した画面のオリジン: {}）",
        config.port,
        config.ui_origin
    );
    match axum::serve(listener, app).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!("Web API が停止しました: {error}");
            ExitCode::FAILURE
        }
    }
}
