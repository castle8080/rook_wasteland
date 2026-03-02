# rw_serve

A lightweight, fast static web app server written in Rust. Serves multiple single-page applications (SPAs) from a single directory, with optional HTTPS and automatic self-signed certificate generation.

## Features

- Serves each subdirectory under an apps root as `/<name>/`
- SPA fallback — unknown paths within a mounted app return `index.html` (supports React Router, Vue Router, etc.)
- HTTP and HTTPS modes (pure-Rust TLS via `rustls`, no OpenSSL dependency)
- Auto-generates a self-signed certificate when none is present
- Per-request structured logging: method, path, status, latency, IP, bytes in/out, user agent
- JSON log output mode for log aggregators
- Zero runtime dependencies beyond the binary itself

## Directory Layout

```
dist/                   # default apps directory
  rw_index/             # served at /rw_index/   ← root / redirects here
    index.html
  rw_chess/             # served at /rw_chess/
    index.html
    *.wasm
    *.js
  rw_defender/          # served at /rw_defender/
    index.html
  rw_poetry/            # served at /rw_poetry/
    index.html

certs/                  # created automatically in HTTPS mode
  cert.pem
  key.pem
```

## Building

```bash
# debug build
cargo build

# optimized release build
cargo build --release
```

The binary is placed at `target/release/rw_serve`.

## Running

### HTTP (default port 8080)

```bash
cargo run
# or after a release build:
./target/release/rw_serve
```

### HTTP on a custom port

```bash
./target/release/rw_serve --port 3000
```

### Custom apps directory

```bash
./target/release/rw_serve --apps-dir /var/www/apps
```

### HTTPS (default port 8443)

If `./certs/cert.pem` and `./certs/key.pem` do not exist, a self-signed certificate is generated automatically.

```bash
./target/release/rw_serve --https
```

### HTTPS with existing certificate

```bash
./target/release/rw_serve --https \
  --cert /etc/ssl/my-cert.pem \
  --key  /etc/ssl/my-key.pem \
  --port 443
```

### JSON log output (for log aggregators)

```bash
./target/release/rw_serve --log-json
```

## CLI Reference

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--apps-dir <PATH>` | `-d` | `./dist` | Directory containing app subdirectories |
| `--port <PORT>` | `-p` | `8080` / `8443` | Port to listen on |
| `--https` | | off | Enable HTTPS mode |
| `--cert <PATH>` | | `./certs/cert.pem` | TLS certificate (PEM) |
| `--key <PATH>` | | `./certs/key.pem` | TLS private key (PEM) |
| `--log-json` | | off | Emit logs as newline-delimited JSON |

## Log Level

Control log verbosity via the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug ./target/release/rw_serve
RUST_LOG=warn  ./target/release/rw_serve
```

Default level is `info`. Each request logs a single line:

```
2026-03-02T21:27:22Z  INFO http{method=GET path=/rw_chess/ ip=203.0.113.5 bytes_in=0 ua="Mozilla/5.0 ..." status=200 latency_ms="0.72" bytes_out="1188"}: rw_serve:
```

## URL Routing

| Request | Result |
|---------|--------|
| `/` | 308 redirect → `/rw_index/index.html` |
| `/<name>` | 308 redirect → `/<name>/` |
| `/<name>/` | Serves `<apps-dir>/<name>/index.html` |
| `/<name>/<file>` | Serves file directly (correct MIME type) |
| `/<name>/<missing>` | SPA fallback — serves `<apps-dir>/<name>/index.html` with `200` |
| `/<unknown>/...` | `404 Not Found` |

## Development

```bash
# run tests
cargo test

# lint
cargo clippy --all-targets -- -D warnings

# run with live apps dir and debug logging
RUST_LOG=debug cargo run -- --apps-dir ../dist --port 8080
```

## Notes

- Self-signed certificates are written to disk and reused on subsequent starts. They are **not suitable for production** — use a cert from a CA (e.g. Let's Encrypt) for public deployments.
- The app list is scanned once at startup. Restart the server to pick up new subdirectories.
- Requests to a reverse proxy that sets `X-Forwarded-For` or `X-Real-IP` are logged with the original client IP.
