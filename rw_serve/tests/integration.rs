use std::fs;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use rw_serve::build_router;

fn make_test_dist() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    fs::write(base.join("index.html"), "<h1>Index</h1>").unwrap();
    fs::create_dir_all(base.join("rw_chess")).unwrap();
    fs::write(base.join("rw_chess/index.html"), "<h1>Chess</h1>").unwrap();

    dir
}

#[tokio::test]
async fn root_serves_index() {
    let dir = make_test_dist();
    let router = build_router(dir.path());

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(body.starts_with(b"<h1>Index</h1>"));
}

#[tokio::test]
async fn subdir_serves_index() {
    let dir = make_test_dist();
    let router = build_router(dir.path());

    let req = Request::builder().uri("/rw_chess/").body(Body::empty()).unwrap();
    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(body.starts_with(b"<h1>Chess</h1>"));
}

#[tokio::test]
async fn missing_file_returns_404() {
    let dir = make_test_dist();
    let router = build_router(dir.path());

    let req = Request::builder().uri("/does_not_exist.js").body(Body::empty()).unwrap();
    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
