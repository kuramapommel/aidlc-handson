//! 静的ファイルサーバーのテスト。テストごとに一時ディレクトリに配信用のファイルを作る。

use std::fs;
use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::*;

/// `<一時ディレクトリ>/<名前>/public` に index.html と app.js、その外に secret.txt を作る。
fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("static_server_{name}_{}", std::process::id()));
    let public = root.join("public");
    fs::create_dir_all(&public).unwrap();
    fs::write(
        public.join("index.html"),
        "<!doctype html><title>index</title>",
    )
    .unwrap();
    fs::write(public.join("app.js"), "console.log('app');").unwrap();
    fs::write(root.join("secret.txt"), "TOP-SECRET").unwrap();
    public
}

async fn get(dir: &std::path::Path, uri: &str) -> (StatusCode, String) {
    let response = router(dir)
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

#[tokio::test]
async fn existing_file_is_served() {
    let dir = fixture("existing");
    let (status, body) = get(&dir, "/app.js").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "console.log('app');");
    assert!(check_dir(&dir).is_ok());
}

#[tokio::test]
async fn in_app_urls_fall_back_to_index_html() {
    let dir = fixture("fallback");
    for uri in [
        "/",
        "/courses/C001",
        "/courses/C001/sessions/S0001/confirm",
        "/tickets/abc",
    ] {
        let (status, body) = get(&dir, uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert!(body.contains("<title>index</title>"), "{uri}: {body}");
    }
}

#[tokio::test]
async fn paths_outside_the_directory_are_not_served() {
    let dir = fixture("outside");
    for uri in [
        "/../secret.txt",
        "/%2e%2e/secret.txt",
        "/courses/../../secret.txt",
    ] {
        let (_, body) = get(&dir, uri).await;
        assert!(
            !body.contains("TOP-SECRET"),
            "{uri} でディレクトリの外のファイルが返った"
        );
    }
}

#[test]
fn config_defaults_and_missing_index_are_reported() {
    let config = StaticConfig::from_lookup(|_| None).unwrap();
    assert_eq!(config.port, DEFAULT_STATIC_PORT);
    assert_eq!(config.dir, PathBuf::from(DEFAULT_STATIC_DIR));
    assert!(StaticConfig::from_lookup(|k| (k == "STATIC_PORT").then(|| "0".into())).is_err());
    let empty = std::env::temp_dir().join(format!("static_server_empty_{}", std::process::id()));
    fs::create_dir_all(&empty).unwrap();
    assert!(check_dir(&empty).is_err());
}
