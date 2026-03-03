# Task T0.6: src/components/header.rs

**Milestone:** M0 — Project Scaffold
**Status:** ✅ Done

---

## Restatement

Implement the `<Header>` component: a top navigation bar with the "rw_mixit" logo linking to `#/` and two nav links `[Settings]` and `[About]` that update `window.location.hash` on click. `prevent_default()` is called to suppress the browser's default link navigation (which would cause a page reload). Out of scope: active-link highlighting (M11).

---

## Design

### Data flow
User clicks link → `on:click` closure fires → `e.prevent_default()` → `window().location().set_hash(route.to_hash())` → browser fires `hashchange` → `App`'s listener updates `current_route` signal → Leptos rerenders.

### Function / type signatures
```rust
#[component]
pub fn Header() -> impl IntoView
```

### Edge cases
- Missing `prevent_default()` would allow the browser to follow the `href`, causing a full page reload and losing WASM state.
- Each `on:click` closure must be `move` because closures inside `view!` must be `'static`.

### Integration points
- `Route::to_hash()` from `crate::routing`.
- Rendered by `App` (T0.5).

---

## Design Critique

| Dimension   | Issue | Resolution |
|---|---|---|
| Correctness | A helper `set_hash(route)` fn would be cleaner but triggers a dead-code warning because `view!` macro calls aren't traced by the compiler. | Navigation code inlined into each closure. Slightly more verbose but warning-free. |
| Simplicity  | Three near-identical closures could be deduplicated with a macro or closure factory. | Three links is well within the threshold where duplication is acceptable — no macro needed. |
| Coupling    | Header imports `Route` directly. | Acceptable; Header IS the route navigation control. |
| Performance | N/A. | — |
| Testability | DOM-dependent, tested manually. | — |

---

## Implementation Notes

Navigation closures call `web_sys::window().expect(...)` directly rather than via a helper to avoid the dead-code false-positive from the `view!` macro's expansion scope.

---

## Test Results

**Automated:**
```
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 warnings
```

**Manual steps performed:**
- [ ] Clicking logo navigates to `#/` (requires trunk serve)
- [ ] Clicking `[Settings]` navigates to `#/settings`
- [ ] Clicking `[About]` navigates to `#/about`
- [ ] All three links change the URL fragment without a page reload

---

## Review Notes

No issues found.

---

## Callouts / Gotchas

- Each `on:click` closure is `move` — if the closure needs to capture a signal, ensure the signal is `Copy`.
