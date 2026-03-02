use std::path::Path;

use axum::{response::Redirect, routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};

/// Scan `apps_dir` for immediate subdirectories; returns sorted list of names.
pub fn scan_apps(apps_dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut apps = Vec::new();
    for entry in std::fs::read_dir(apps_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            apps.push(name.to_string());
        }
    }
    apps.sort();
    Ok(apps)
}

/// Build the Axum router:
///   /                   → 301 to /rw_index/index.html
///   /<name>             → served by ServeDir at apps_dir/<name>
///   /<name>/<file>      → served directly
///   /<name>/<missing>   → SPA fallback to apps_dir/<name>/index.html
///   /<unknown>/...      → 404 (no route registered)
pub fn build_router(apps_dir: &Path, apps: &[String]) -> Router {
    let mut router = Router::new().route(
        "/",
        get(|| async { Redirect::permanent("/rw_index/index.html") }),
    );

    for app_name in apps {
        let app_path = apps_dir.join(app_name);
        let index_path = app_path.join("index.html");

        let serve_dir = ServeDir::new(&app_path)
            .fallback(ServeFile::new(&index_path));

        router = router.nest_service(&format!("/{app_name}"), serve_dir);
    }

    router
}
