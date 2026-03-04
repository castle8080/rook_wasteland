# Task T0.3: src/lib.rs Entry Point

**Milestone:** M0 — Project Scaffold
**Status:** ✅ Done

---

## Restatement

Create `src/lib.rs` as the WASM entry point: calls `console_error_panic_hook::set_once()` to route Rust panics to the browser console, then `leptos::mount::mount_to_body(App)` to mount the root Leptos component. Declares all top-level modules (`app`, `routing`, `state`, `audio`, `components`, `canvas`, `utils`). The spec refers to this as "src/main.rs entry" — in a `[lib] crate-type = ["cdylib"]` project, the library (`lib.rs`) is the WASM compilation unit, so the entry lives there, not in a binary `main.rs`. Out of scope: any component or business logic.

---

## Design

### Data flow
`#[wasm_bindgen(start)] fn main()` → `console_error_panic_hook::set_once()` → `mount_to_body(App)` → browser renders the app.

### Function / type signatures
```rust
#[wasm_bindgen(start)]
fn main()
```

### Edge cases
- Without `#[wasm_bindgen(start)]`, `fn main()` in a lib crate is a dead function and the WASM module has no entry — the app never starts.
- Without `console_error_panic_hook`, Rust panics produce an unhelpful "unreachable" error in the browser console.

### Integration points
- Depends on `app::App` (T0.5).
- All module declarations here must match existing `src/<module>/mod.rs` files or the compiler errors.

---

## Design Critique

| Dimension   | Issue | Resolution |
|---|---|---|
| Correctness | Spec says `src/main.rs` is the entry but we use `src/lib.rs`. | With `[lib] crate-type = ["cdylib"]`, lib.rs IS the WASM entry. The spec's intent is satisfied. |
| Simplicity  | Could use a binary-only setup (no lib). | Would lose `rlib` target needed for `cargo test` on host. |
| Coupling    | All modules declared here. | Unavoidable; this is the crate root. |
| Performance | N/A | — |
| Testability | The entry function itself cannot be unit-tested (browser only). | Routing and pure-Rust logic is tested via `#[cfg(test)]` modules in their own files. |

---

## Implementation Notes

`#[wasm_bindgen(start)]` is required to suppress the dead_code warning and to correctly mark the WASM start function. Without it, `cargo clippy -- -D warnings` fails.

---

## Test Results

**Automated:**
```
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 warnings, 0 errors
cargo test → 4 passed
```

**Manual steps performed:**
- [x] Confirmed module declaration list matches directory structure

---

## Review Notes

No issues found.

---

## Callouts / Gotchas

- Every new top-level module added in later milestones (e.g. `bpm`) must be declared here with `mod bpm;`.
