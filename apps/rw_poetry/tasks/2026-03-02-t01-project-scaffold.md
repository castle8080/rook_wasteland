# Task: T01 · Project Scaffold

## Status
Complete

## Goal
Stand up the Rust/Leptos/Trunk project with correct structure, dependencies, and a "hello world" build. This establishes the compilable foundation all subsequent tasks depend on.

## Scope
- Create `Cargo.toml` with all crates from spec section 4.2
- Create `index.html` with Trunk directives including `copy-dir` for poems
- Create `src/main.rs` that mounts the Leptos app
- Set up the full `src/` module layout from spec section 12
- Verify `cargo check` and `trunk build` pass

**Out of scope:** Any real UI or logic beyond a blank app shell.

## Design
Single-crate CSR app. No workspace — standalone `Cargo.toml` at the app root, mirroring the sibling `rw_chess` app pattern. Module stubs are created now so subsequent tasks can fill them in without touching module declarations.

The `index.html` uses `<link data-trunk rel="copy-dir" href="./public/poems"/>` to serve poem assets at `/poems/...` per spec section 8 Trunk integration note.

## Implementation Plan
1. [x] Create `Cargo.toml`
2. [x] Create `index.html`
3. [x] Create `style/main.css` (empty baseline)
4. [x] Create `src/main.rs`
5. [x] Create `src/app.rs`
6. [x] Create module stubs: `poem_repository`, `audio_capture`, `recording_store`, `ui`
7. [x] Verify `cargo check` passes
8. [x] Verify `trunk build` passes

## Testing Plan
- `cargo check` — zero errors
- `trunk build` — succeeds, dist/ produced

## Notes / Decisions
- Using `edition = "2024"` to match sibling apps
- `console_error_panic_hook` included for better WASM panic messages during development
