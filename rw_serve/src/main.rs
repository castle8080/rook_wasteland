mod cli;
mod tls;
use rw_serve::router;

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response};
use clap::Parser;
use tower_http::trace::TraceLayer;
use tracing::{error, info, Span};
use tracing_subscriber::EnvFilter;

use cli::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    setup_logging(args.log_json);

    if !args.apps_dir.exists() {
        error!(path = %args.apps_dir.display(), "Directory not found");
        std::process::exit(1);
    }

    let app = router::build_router(&args.apps_dir).layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                let ip = extract_ip(request);
                let bytes_in: u64 = request
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
                let ua = request
                    .headers()
                    .get("user-agent")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("-")
                    .to_string();
                tracing::info_span!(
                    "http",
                    method = %request.method(),
                    path = %request.uri().path(),
                    ip = %ip,
                    bytes_in,
                    ua = %ua,
                    status = tracing::field::Empty,
                    latency_ms = tracing::field::Empty,
                    bytes_out = tracing::field::Empty,
                )
            })
            .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
                let bytes_out = response
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "?".to_string());
                span.record("status", response.status().as_u16());
                span.record(
                    "latency_ms",
                    format!("{:.2}", latency.as_secs_f64() * 1000.0).as_str(),
                );
                span.record("bytes_out", bytes_out.as_str());
                tracing::info!(parent: span, "");
            })
            .on_failure(()),
    );

    let port = args.port.unwrap_or(if args.https { 8443 } else { 8080 });
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    if args.https {
        tls::ensure_certs(&args.cert, &args.key)?;
        let config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&args.cert, &args.key)
            .await
            .context("Failed to load TLS config")?;
        info!(addr = %addr, mode = "https", "Server starting");
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    } else {
        info!(addr = %addr, mode = "http", "Server starting");
        axum_server::bind(addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    }

    Ok(())
}

fn setup_logging(json: bool) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    }
}

fn extract_ip<B>(request: &Request<B>) -> String {
    // Prefer proxy headers
    if let Some(forwarded) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        let ip = forwarded.split(',').next().unwrap_or("").trim();
        if !ip.is_empty() {
            return ip.to_string();
        }
    }
    if let Some(real_ip) = request
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
    {
        return real_ip.to_string();
    }
    // Fall back to TCP peer address (set by axum-server's ConnectInfo)
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
