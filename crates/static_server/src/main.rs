//! 画面用の静的ファイルサーバーの実行ファイル（ローカル検証専用）。
//!
//! 環境変数: `STATIC_PORT`（既定 8080）、`STATIC_DIR`（既定 `web/dist`）、`RUST_LOG`（既定 `info`）。
//! 待ち受けるのは 127.0.0.1 だけ。

use std::net::SocketAddr;
use std::process::ExitCode;

use static_server::{StaticConfig, check_dir, router};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .init();

    let config = match StaticConfig::from_env() {
        Ok(config) => config,
        Err(message) => {
            tracing::error!("設定が正しくありません: {message}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(message) = check_dir(&config.dir) {
        tracing::error!("{message}");
        return ExitCode::FAILURE;
    }
    let address = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!("ポート {} で待ち受けられません: {error}", config.port);
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(
        "画面を http://localhost:{} で配信します（ディレクトリ: {}）",
        config.port,
        config.dir.display()
    );
    match axum::serve(listener, router(&config.dir)).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!("静的ファイルサーバーが停止しました: {error}");
            ExitCode::FAILURE
        }
    }
}
