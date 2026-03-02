# rw_serve — Specification

## Overview

`rw_serve` is a lightweight Rust binary that serves multiple static web applications from a single configurable directory. Each subdirectory under the apps root is automatically mounted at `/<subdirectory-name>`. The server supports both HTTP and HTTPS, with automatic self-signed certificate generation when no certificate is present.

---

## Framework Recommendation: Axum

After evaluating the major Rust web frameworks, **Axum** is recommended for this project.

### Candidates Considered

| Framework   | Backing         | Notes |
|-------------|-----------------|-------|
| **Axum**    | Tokio team      | Built on `hyper` + `tower`; ergonomic, modular, very actively maintained |
| Actix-web   | Community       | Extremely fast; heavier actor model; slightly more complex for purely static use |
| Rocket      | Community       | Developer-friendly; historically slower to adopt stable Rust features |
| Warp        | Community       | Filter-based DSL; less ergonomic for routing; lower activity |
| Poem/Salvo  | Community       | Younger projects; less proven at scale |

### Why Axum

- **Performance**: Built directly on `hyper` (one of the fastest HTTP implementations in any language) and the `tokio` async runtime. For static file serving, performance is largely I/O bound — Axum's zero-overhead abstraction layer adds negligible overhead.
- **Static file serving**: `tower-http`'s `ServeDir` service handles static file serving natively, with support for correct MIME types, `ETag`/`Last-Modified` caching headers, `Range` requests, and `index.html` fallback — all with no manual implementation required.
- **Stability**: Backed by the Tokio project (the de-facto async runtime for Rust). Axum follows semver strictly and has a large, active community.
- **TLS**: `axum-server` (community crate, used widely) provides drop-in HTTPS via `rustls` — pure-Rust TLS with no OpenSSL dependency.
- **Extensibility**: The `tower` middleware ecosystem means adding future API routes, rate limiting, auth, compression, or CORS requires no framework change.
- **Familiarity**: The project author has prior Axum experience, reducing ramp-up.

---

## Functional Requirements

### 1. Static App Serving

The apps directory (default: `./dist`) is scanned for immediate subdirectories on startup. Each subdirectory is mounted as a static file tree.

**URL mapping:**

| Request path | Behavior |
|---|---|
| `/` | 301 redirect to `/rw_index/index.html` |
| `/<name>` | 301 redirect to `/<name>/` |
| `/<name>/` | Serve `<apps-dir>/<name>/index.html` |
| `/<name>/<file>` | Serve `<apps-dir>/<name>/<file>` directly |
| `/<name>/<deep>/<path>/<file>` | Serve the corresponding file under `<apps-dir>/<name>/` |
| `/<name>/` (file not found) | Serve `<apps-dir>/<name>/index.html` (SPA fallback) |
| `/<unknown-name>/...` | 404 |

**SPA fallback policy:** When a request path resolves to a mounted app (`/<name>/...`) but the specific file does not exist on disk, the server responds with `<apps-dir>/<name>/index.html` and status `200`. This enables client-side routers (React Router, Vue Router, etc.) to handle the path. Only paths under a known app name trigger this fallback; unknown top-level names return a true 404.

**Startup behavior:**
- If the apps directory does not exist or is not readable, the server logs a fatal error and exits.
- If the apps directory exists but contains no subdirectories, the server starts and logs a warning; all requests (except `/`) return 404.
- The app list is determined once at startup. A future hot-reload feature may rescan on SIGHUP (see Future Considerations).

### 2. Command-Line Interface

The binary accepts the following flags:

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--apps-dir <PATH>` | `-d` | `./dist` | Path to the directory containing static app subdirectories |
| `--port <PORT>` | `-p` | `8080` (HTTP) / `8443` (HTTPS) | Port to listen on |
| `--https` | | off | Enable HTTPS mode |
| `--cert <PATH>` | | `./certs/cert.pem` | Path to TLS certificate (PEM format) |
| `--key <PATH>` | | `./certs/key.pem` | Path to TLS private key (PEM format) |
| `--log-json` | | off | Emit logs as newline-delimited JSON instead of human-readable text |

Parsing will use the `clap` crate (derive API).

### 3. HTTPS / TLS

- When `--https` is passed, the server binds with TLS using `rustls` (via `axum-server` with the `tls-rustls` feature).
- If the certificate and key files specified by `--cert` / `--key` do not exist:
  - The server automatically generates a self-signed certificate using the `rcgen` crate.
  - The generated certificate and key are written to the paths specified by `--cert` / `--key` (parent directories are created if needed).
  - A warning is printed to stderr noting that a self-signed certificate was generated and is not suitable for production.
- If the certificate/key files exist, they are loaded directly without regeneration.

### 4. Request Logging

Every request is logged as a single line at `INFO` level containing:

| Field | Description |
|-------|-------------|
| `method` | HTTP method (`GET`, `POST`, etc.) |
| `path` | Request URI path |
| `status` | Response HTTP status code |
| `latency` | Time from request received to response sent (e.g. `3.2ms`) |
| `remote_ip` | Client IP address; if behind a proxy, prefer `X-Forwarded-For` / `X-Real-IP` headers |
| `bytes_in` | Size of the incoming request body in bytes (`0` for bodyless requests) |
| `bytes_out` | Size of the response body in bytes |
| `user_agent` | Value of the `User-Agent` header (optional, omit if absent) |

Example log line (human-readable format):
```
2026-03-02T21:10:03Z INFO  GET /rw_chess/assets/main.js 200 1.4ms ip=203.0.113.42 in=0B out=84321B ua="Mozilla/5.0 ..."
```

Implementation notes:
- `tower-http`'s `TraceLayer` provides hooks for `on_request`, `on_response`, and `on_failure` that can populate all of the above fields.
- `bytes_out` comes from the `Content-Length` response header when available; for streaming responses it may read as `unknown`.
- Client IP extraction checks `X-Forwarded-For` → `X-Real-IP` → TCP peer address in that priority order.
- Log level is configurable via the `RUST_LOG` environment variable (default: `info`).
- Log output format defaults to human-readable; a `--log-json` flag switches to newline-delimited JSON for consumption by log aggregators (e.g. Loki, Datadog).
- Uses `tracing` + `tracing-subscriber` for structured output.

---

## Non-Functional Requirements

- **Resource usage**: Minimal memory footprint; async I/O ensures threads are not blocked on file reads.
- **Correctness**: Serve correct `Content-Type` headers for all common web asset types (HTML, JS, CSS, images, fonts, WASM, etc.) — handled automatically by `tower-http`.
- **Security**: No dynamic code execution; attack surface is limited to file path traversal (mitigated by `ServeDir`'s safe path resolution) and TLS configuration.
- **Portability**: Pure Rust, no C dependencies (rustls over OpenSSL). Builds on Linux, macOS, and Windows.

---

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP routing and request handling |
| `tokio` | Async runtime (features: `full`) |
| `tower-http` | Static file serving (`ServeDir`), tracing middleware |
| `axum-server` | Hyper server binding; TLS support via rustls |
| `rustls` | Pure-Rust TLS implementation |
| `rcgen` | Self-signed X.509 certificate generation |
| `clap` | Command-line argument parsing (derive feature) |
| `tracing` | Structured logging instrumentation |
| `tracing-subscriber` | Log output formatting and filtering |

---

## Directory Layout (Runtime)

```
<apps-dir>/          # default: ./dist
  rw_chess/          # served at /rw_chess/
    index.html
    assets/
  rw_defender/       # served at /rw_defender/
    index.html
  rw_poetry/         # served at /rw_poetry/
    index.html

<certs-dir>/         # default: ./certs (auto-created in HTTPS mode)
  cert.pem
  key.pem
```

---

## Future Considerations

- API route support: Axum's router makes it straightforward to add `GET /api/...` handlers alongside static routes.
- HTTP → HTTPS redirect: A companion HTTP listener that issues 301 redirects when running in HTTPS mode.
- Hot reload: Re-scanning the apps directory on SIGHUP without restarting.
- Configuration file: TOML/YAML config as an alternative to CLI flags, parsed with `config` or `figment`.
- Compression: `tower-http`'s `CompressionLayer` for gzip/brotli responses.
