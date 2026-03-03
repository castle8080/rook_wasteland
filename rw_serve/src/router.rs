use std::path::Path;

use axum::Router;
use tower_http::services::ServeDir;

/// Build the Axum router: serves `dir` as static files, with `index.html`
/// returned automatically for directory requests.
pub fn build_router(dir: &Path) -> Router {
    Router::new().fallback_service(ServeDir::new(dir))
}
