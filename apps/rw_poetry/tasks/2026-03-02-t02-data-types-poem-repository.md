# Task: T02 · Core Data Types and Poem Repository

## Status
Complete

## Completion Notes
Date completed: 2026-03-02

Implemented PoemIndexEntry, PoemIndex, PoemDetail types with Deserialize.
fetch_index and fetch_poem use gloo_net. pick_random uses js_sys::Math::random()
(WASM-native, no rand crate). Five unit tests pass on the native target; the
one test that calls js_sys is gated with #[cfg(target_arch = "wasm32")].
#![allow(dead_code)] added at module level — removed in T04 when consumed.

## Goal
Define all shared Rust data types and implement the two-step poem fetch flow (index → poem JSON), plus `pick_random` for selecting poems from the index.

## Scope
- `PoemIndexEntry`, `PoemIndex`, `PoemDetail` structs with `serde::Deserialize`
- `poem_repository` module: `fetch_index`, `fetch_poem`, `pick_random`
- Unit tests: `pick_random` distribution/exclude, serde round-trips

**Out of scope:** Any UI — purely data and logic.

## Design
Types are defined in `poem_repository/mod.rs`. Fetch functions use `gloo_net::http::Request`. `pick_random` uses `js_sys::Math::random()` for WASM-compatible randomness (no `rand` crate needed — the corpus is small and security is irrelevant here).

The index is fetched at `/poems/poems_index.json` (site-root absolute, served by Trunk copy-dir). Individual poems are fetched at their `path` value from the index.

## Implementation Plan
1. [x] Define `PoemIndexEntry`, `PoemIndex`, `PoemDetail` in `poem_repository/mod.rs`
2. [x] Implement `fetch_index()`
3. [x] Implement `fetch_poem(path)`
4. [x] Implement `pick_random(index, exclude_id)`
5. [x] Write unit tests
6. [x] `cargo test` passes
7. [x] `cargo clippy -- -D warnings` clean

## Testing Plan
- Unit test: `pick_random` on a 3-entry index, run many times, check uniform distribution and exclude logic
- Unit test: deserialize a sample `PoemIndex` JSON string
- Unit test: deserialize a sample `PoemDetail` JSON string

## Notes / Decisions
- Using `js_sys::Math::random()` instead of the `rand` crate to keep WASM binary smaller
- `fetch_index` and `fetch_poem` return `Result<_, String>` per spec task description
