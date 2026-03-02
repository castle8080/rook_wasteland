use std::fs;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use rw_serve::{scan_apps, build_router};

fn make_test_dist() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // rw_index app
    fs::create_dir_all(base.join("rw_index")).unwrap();
    fs::write(base.join("rw_index/index.html"), "<h1>Index</h1>").unwrap();

    // rw_chess app with an asset and a deep path
    fs::create_dir_all(base.join("rw_chess/assets")).unwrap();
    fs::write(base.join("rw_chess/index.html"), "<h1>Chess</h1>").unwrap();
    fs::write(base.join("rw_chess/assets/main.js"), "console.log('chess')").unwrap();

    dir
}

#[tokio::test]
async fn root_redirects_to_rw_index() {
    let dir = make_test_dist();
    let apps = scan_apps(dir.path()).unwrap();
    let router = build_router(dir.path(), &apps);

    let req = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(resp.headers()["location"], "/rw_index/index.html");
}

#[tokio::test]
async fn serves_app_index() {
    let dir = make_test_dist();
    let apps = scan_apps(dir.path()).unwrap();
    let router = build_router(dir.path(), &apps);

    let req = Request::builder()
        .uri("/rw_chess/")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(body.starts_with(b"<h1>Chess</h1>"));
}

#[tokio::test]
async fn serves_deep_asset() {
    let dir = make_test_dist();
    let apps = scan_apps(dir.path()).unwrap();
    let router = build_router(dir.path(), &apps);

    let req = Request::builder()
        .uri("/rw_chess/assets/main.js")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"console.log('chess')");
}

#[tokio::test]
async fn spa_fallback_for_missing_file() {
    let dir = make_test_dist();
    let apps = scan_apps(dir.path()).unwrap();
    let router = build_router(dir.path(), &apps);

    let req = Request::builder()
        .uri("/rw_chess/some/deep/spa/route")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();

    // SPA fallback: returns index.html with 200
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(body.starts_with(b"<h1>Chess</h1>"));
}

#[tokio::test]
async fn unknown_app_returns_404() {
    let dir = make_test_dist();
    let apps = scan_apps(dir.path()).unwrap();
    let router = build_router(dir.path(), &apps);

    let req = Request::builder()
        .uri("/no_such_app/anything")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
