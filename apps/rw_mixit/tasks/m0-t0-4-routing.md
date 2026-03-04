# Task T0.4: src/routing.rs

**Milestone:** M0 — Project Scaffold
**Status:** ✅ Done

---

## Restatement

Implement the `Route` enum with three variants (`Main`, `Settings`, `About`) and two methods: `from_hash(hash: &str) -> Route` (parses URL hash to a variant, unknown hashes fall back to `Main`) and `to_hash(&self) -> &'static str` (returns the canonical hash string). Include unit tests verifying all hash strings round-trip correctly. Out of scope: any DOM interaction — this is pure Rust with no web-sys calls.

---

## Design

### Data flow
URL hash string → `Route::from_hash` → `Route` variant → `Route::to_hash` → hash string.

### Function / type signatures
```rust
pub enum Route { Main, Settings, About }

impl Route {
    pub fn from_hash(hash: &str) -> Self
    pub fn to_hash(&self) -> &'static str
}
```

### Edge cases
- Empty string, `"#/"`, and any unrecognised hash → `Route::Main` (safe default).
- `to_hash` returns `"#/"` for `Main`, which `from_hash` maps back to `Main`. ✓

### Integration points
- Used by `App` (T0.5) to initialise the route signal and update it on `hashchange`.
- Used by `Header` (T0.6) to set the hash on nav link click.

---

## Design Critique

| Dimension   | Issue | Resolution |
|---|---|---|
| Correctness | `from_hash("#/")` must return `Main`, not fall through to the wildcard with a different result. | Wildcard `_` catches `"#/"` and returns `Main`. ✓ |
| Simplicity  | Could use `str::starts_with` matching. | Exact match is safer and clearer for 3 variants. |
| Coupling    | None — pure data type. | — |
| Performance | N/A (called only on user navigation, not in hot path). | — |
| Testability | 100% unit-testable on native host. | 4 unit tests cover all variants and edge cases. |

---

## Implementation Notes

`Route` derives `Clone`, `PartialEq`, `Debug` — all needed by Leptos signals and `Show` comparisons.

---

## Test Results

**Automated:**
```
test routing::tests::from_hash_defaults_to_main ... ok
test routing::tests::from_hash_settings        ... ok
test routing::tests::from_hash_about           ... ok
test routing::tests::round_trip_all_routes     ... ok

test result: ok. 4 passed; 0 failed
```

**Manual steps performed:** N/A — fully automated.

---

## Review Notes

No issues found.

---

## Callouts / Gotchas

- If new routes are added later, update `from_hash`, `to_hash`, and the `App` `<Show>` blocks.
