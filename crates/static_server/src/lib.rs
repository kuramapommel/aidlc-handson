//! 画面用の静的ファイルサーバー（TD-2）。
//!
//! MoonBit のビルド結果と HTML・CSS を置いたディレクトリを配信する。存在しないパスには
//! 画面の入り口（`index.html`）を返し、画面内の URL（`/courses/C001` など）を直接開いたり
//! 再読み込みしたりしても画面が動くようにする。ディレクトリの外のパス（`..`）は配信しない
//! （`tower-http` の `ServeDir` がパスの `..` を拒否する）。

use std::path::{Path, PathBuf};

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

/// 待ち受けのポートの既定値。
pub const DEFAULT_STATIC_PORT: u16 = 8080;
/// 配信するディレクトリの既定値（リポジトリのルートからの相対パス）。
pub const DEFAULT_STATIC_DIR: &str = "web/dist";

/// 静的ファイルサーバーの設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticConfig {
    /// 待ち受けのポート（環境変数 `STATIC_PORT`）。
    pub port: u16,
    /// 配信するディレクトリ（環境変数 `STATIC_DIR`）。
    pub dir: PathBuf,
}

impl StaticConfig {
    /// 値を引く関数から設定を作る（実行時は環境変数、テストでは代わりの関数を渡す）。
    pub fn from_lookup<F>(lookup: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let port = match lookup("STATIC_PORT") {
            None => DEFAULT_STATIC_PORT,
            Some(raw) => raw
                .trim()
                .parse::<u16>()
                .ok()
                .filter(|p| *p != 0)
                .ok_or_else(|| {
                    format!("STATIC_PORT は 1〜65535 の数で指定してください: {raw:?}")
                })?,
        };
        let dir =
            PathBuf::from(lookup("STATIC_DIR").unwrap_or_else(|| DEFAULT_STATIC_DIR.to_owned()));
        Ok(Self { port, dir })
    }

    /// 環境変数から読む。
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }
}

/// 配信の準備ができているか（`index.html` があるか）を確かめる。
pub fn check_dir(dir: &Path) -> Result<(), String> {
    if dir.join("index.html").is_file() {
        Ok(())
    } else {
        Err(format!(
            "{} に index.html がありません。先に画面をビルドしてください（bash scripts/build-web.sh）",
            dir.display()
        ))
    }
}

/// `dir` を配信する Router を返す。存在しないパスには `index.html` を返す。
pub fn router(dir: &Path) -> Router {
    let serve = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
    Router::new()
        .fallback_service(serve)
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests;
