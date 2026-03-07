---
name: wasm-test
description: Set up and run tests for a Rust/WASM Leptos project. Use this when asked to write tests, run the test suite, or debug test failures.
---

## Two Test Targets

This project compiles as both `cdylib` (WASM) and `rlib` (native host). This gives you two separate test surfaces.

### 1. Native unit tests — `cargo test`

Use for: pure Rust logic with no browser dependencies.

✅ Good candidates: math helpers, state machines, routing logic, data transformations, signal-independent business logic.

```rust
// In the same file as the logic being tested
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_function_returns_expected_value() {
        assert_eq!(my_function(input), expected);
    }
}
```

Run:
```bash
cargo test                          # all tests
cargo test my_function              # single test by name
cargo test -- --nocapture           # show println! output
```

`.unwrap()` is fine in test code — a panic is a test failure.

### 2. Browser WASM tests — `wasm-pack test`

Use for: code that requires a real browser (Web Audio API, Canvas, DOM APIs, `web-sys` types).

```rust
// In tests/my_test.rs
use wasm_bindgen_test::*;

// This line must appear in lib.rs (crate root) under #[cfg(test)]:
// wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn audio_node_creates_successfully() {
    let ctx = web_sys::AudioContext::new().unwrap();
    let gain = ctx.create_gain().unwrap();
    assert!(gain.gain().value() >= 0.0);
}
```

Run:
```bash
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome   # alternative
```

### Decision tree

```
Does the test require a browser API (AudioContext, Canvas, DOM)?
  Yes → wasm_bindgen_test in tests/, run with wasm-pack
  No  → #[test] in the same file, run with cargo test
```

### Cargo.toml setup for WASM tests

```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
```

### lib.rs setup

```rust
// Configure ALL wasm_bindgen_test tests in this crate to run in a browser.
// Placed at crate root so it covers every test module.
#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
```

### Excluding the WASM entry point during tests

The `#[wasm_bindgen(start)]` entry point must be excluded during `wasm-pack test` to avoid a duplicate start symbol linker error:

```rust
#[cfg(not(test))]
#[wasm_bindgen(start)]
fn main() { ... }
```

### Clippy — always target wasm32

```bash
cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

`cargo clippy` without the target flag runs against the native host rlib and can miss WASM-specific issues. Always specify the target.
