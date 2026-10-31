//! 設定値（環境変数で変えられる。既定値はコードに持つ。NFR3.9）。秘密情報は扱わない。

use axum::http::HeaderValue;

/// Web API の待ち受けのポートの既定値。
pub const DEFAULT_API_PORT: u16 = 3000;
/// 画面のオリジンの既定値（静的ファイルサーバーの既定のポート 8080）。
pub const DEFAULT_UI_ORIGIN: &str = "http://localhost:8080";

/// Web API の設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiConfig {
    /// 待ち受けのポート（環境変数 `API_PORT`）。
    pub port: u16,
    /// 資格情報付きの通信と書き込みの要求を許可する画面のオリジン（環境変数 `UI_ORIGIN`）。
    pub ui_origin: String,
}

impl ApiConfig {
    /// 環境変数から読む。値が不正なら起動を止めるための説明を返す。
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// 値を引く関数から設定を作る（テストでは環境変数の代わりの関数を渡す）。
    pub fn from_lookup<F>(lookup: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let port = match lookup("API_PORT") {
            None => DEFAULT_API_PORT,
            Some(raw) => raw
                .trim()
                .parse::<u16>()
                .ok()
                .filter(|p| *p != 0)
                .ok_or_else(|| format!("API_PORT は 1〜65535 の数で指定してください: {raw:?}"))?,
        };
        let ui_origin = lookup("UI_ORIGIN").unwrap_or_else(|| DEFAULT_UI_ORIGIN.to_owned());
        validate_origin(&ui_origin)?;
        Ok(Self { port, ui_origin })
    }

    /// 画面のオリジンをヘッダーの値として返す。
    pub fn ui_origin_header(&self) -> HeaderValue {
        HeaderValue::from_str(&self.ui_origin).expect("from_lookup で検証済み")
    }
}

/// オリジンは `http://` か `https://` で始まり、末尾の `/` やワイルドカードを含まない（NFR3.6）。
fn validate_origin(origin: &str) -> Result<(), String> {
    let has_scheme = origin.starts_with("http://") || origin.starts_with("https://");
    let ok = has_scheme
        && !origin.ends_with('/')
        && !origin.contains('*')
        && HeaderValue::from_str(origin).is_ok();
    if ok {
        Ok(())
    } else {
        Err(format!(
            "UI_ORIGIN は http://host:port の形で指定してください（ワイルドカード不可）: {origin:?}"
        ))
    }
}
