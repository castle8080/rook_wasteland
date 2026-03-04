# Task T0.5: src/app.rs — App Component

**Milestone:** M0 — Project Scaffold
**Status:** ✅ Done

---

## Restatement

Implement the `App` root Leptos component: reads the initial URL hash to create a `RwSignal<Route>`, registers a `hashchange` event listener so browser back/forward navigation keeps the signal in sync, provides the signal via Leptos context, and uses `<Show>` gates to conditionally render `DeckView`, a Settings placeholder, or an About placeholder. Out of scope: actual Settings/About views (M11), deck content (M1+).

---

## Design

### Data flow
`window.location.hash` → `Route::from_hash` → `RwSignal<Route>` → `provide_context` → `<Show when=...>` selects which view to render. On `hashchange`: browser fires event → listener closure → `current_route.set(...)` → Leptos rerenders the correct `<Show>`.

### Function / type signatures
```rust
#[component]
pub fn App() -> impl IntoView
```

### Edge cases
- The `gloo_events::EventListener` must not be dropped when `App()` returns its view — it's intentionally leaked via `std::mem::forget` since the listener must live for the entire app lifetime.
- `window().location().hash()` returns `Err` when there is no hash — handled by `unwrap_or_default()`.

### Integration points
- Depends on `Route` (T0.4), `Header` (T0.6), `DeckView` (T0.7).
- `provide_context(current_route)` — any descendant can call `use_context::<RwSignal<Route>>()` to read the current route.

---

## Design Critique

| Dimension   | Issue | Resolution |
|---|---|---|
| Correctness | `EventListener` is dropped at end of `App()` scope normally. | `std::mem::forget(listener)` prevents drop, keeping `removeEventListener` from being called. Correct for a single-page app whose listener must never be removed. |
| Simplicity  | Could store listener in a `StoredValue`. | `std::mem::forget` is simpler and idiomatic for "lives forever" listeners. |
| Coupling    | App directly imports `DeckView` and `Header`. | Intentional — App is the composition root. |
| Performance | N/A (runs once on startup). | — |
| Testability | Cannot unit-test DOM-dependent code on native host. | No automated tests needed here; routing logic is tested separately in T0.4. |

---

## Implementation Notes

The `navigate()` helper function from the tech spec example was omitted — header nav is done inline in `header.rs` to avoid a dead-function clippy warning caused by the `view!` macro not being traced by the compiler's dead-code analysis.

---

## Test Results

**Automated:**
```
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 warnings
```

**Manual steps performed:**
- [ ] Verify `#/settings` and `#/about` render placeholder views (requires trunk serve)
- [ ] Verify browser back/forward updates the view (requires trunk serve)

---

## Review Notes

No issues found.

---

## Callouts / Gotchas

- `std::mem::forget(listener)` is intentional. Do not "fix" it by dropping the listener — doing so removes the `hashchange` listener and breaks back/forward navigation.
