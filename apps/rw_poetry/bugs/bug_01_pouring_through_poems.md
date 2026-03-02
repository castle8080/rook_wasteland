# Bug 01 — Pouring Through Poems

## Status
Fixed

## Summary
On load, the app cycles through poems rapidly in what appears to be a tight loop. Poems flash one after another without any user interaction.

## Expected Behavior
When the app loads, a single random poem is selected and displayed. The poem remains on screen until the user explicitly requests a new one (e.g. clicks the "New Poem" button).

## Actual Behavior
On load, the reader view cycles through multiple poems in rapid succession — each one briefly visible before being replaced by the next. This continues without any user action, giving the appearance of an infinite or near-infinite fetch/render loop.

## Root Cause

**File:** `src/ui/reader.rs` — `ReaderView`, `LocalResource` closure

The `LocalResource` closure read `current_poem_id` using `.get()`:

```rust
let exclude = current_poem_id.get();  // BUG: registers reactive dependency
```

In Leptos, calling `.get()` on a signal inside a reactive closure subscribes that signal as a dependency. Any subsequent write to that signal re-triggers the closure.

The fetch loop then called `current_poem_id.set(Some(poem.id.clone()))` inside the async block upon a successful load (both in the query-param path and the random-pick path). This write notified the reactive system, which saw that the resource depended on `current_poem_id`, and re-ran the resource — picking a new poem, writing `current_poem_id` again, and cycling indefinitely.

The reactive graph was:

```
LocalResource depends on: [query, current_poem_id, refresh]
                                        ↑
                          current_poem_id.set() called on fetch success
                                        |
                          → resource re-runs → new poem fetched → .set() again → ∞
```

The `refresh` signal was correctly used as the only intended re-trigger; `current_poem_id` was only meant to be read for its current value (to pass as the exclude argument to `pick_random`), not tracked as a dependency.

## Fix

**One line change in `src/ui/reader.rs`:**

```rust
// Before (bug):
let exclude = current_poem_id.get();

// After (fix):
let exclude = current_poem_id.get_untracked();
```

`get_untracked()` reads the current signal value without registering it as a reactive dependency. The resource now only re-runs when `refresh` increments (New Poem / Try again) or when the URL query params change — both of which are explicit user actions.

A comment was added in the code explaining the reasoning, so the same mistake is not repeated.

## Tests Added

Five new unit tests were added to `src/poem_repository/mod.rs`. To enable these tests to run on the native target (plain `cargo test`, no WASM required), `pick_random` was refactored to extract its deterministic selection step into a private helper:

```rust
fn pick_at_index<'a>(candidates: &[&'a PoemIndexEntry], idx: usize) -> Option<&'a PoemIndexEntry>
```

`pick_random` now calls `pick_at_index` after obtaining a random index from `js_sys::Math::random()`. The helper is fully testable with fixed index values.

| Test | What it verifies |
|------|-----------------|
| `pick_at_index_empty_returns_none` | Empty candidate list yields `None` |
| `pick_at_index_selects_correct_entry` | Index 0/1/2 selects the expected entry |
| `pick_at_index_clamps_out_of_bounds` | An out-of-range index clamps to the last entry |
| `pick_at_index_exclude_filters_correctly` | Excluding "a" from [a,b,c] produces candidates [b,c]; index 0→b, 1→c |
| `pick_at_index_single_candidate_always_returns_it` | The common "New Poem" case: one poem left after exclusion, always returned |

All 18 tests pass (`cargo test`).

## Affected Component
`src/ui/reader.rs` — `ReaderView`

## Steps to Reproduce
1. Open the app in a browser.
2. Observe the reader view on initial load.

## Acceptance Criteria for Fix
- On load, exactly one poem is fetched and displayed.
- No further poem fetches occur until the user clicks "New Poem".
- "New Poem" still works correctly: fetches one new poem, excludes the current one, then stops.
- No reactive loops or excessive resource re-triggers appear in the browser console or network tab.

---

## Post-Mortem: Lessons Learned

### 1. In Leptos, `.get()` is a subscription, not just a read

In Leptos's reactive system, every call to `signal.get()` inside a reactive context (a closure passed to `LocalResource::new`, `create_memo`, `Effect::new`, etc.) **subscribes** to that signal. The closure will re-run whenever the signal changes. This is the fundamental mechanism of Leptos reactivity — but it means you must be conscious of every `.get()` call you make in a resource closure.

Reading a signal just to snapshot its current value for use as a function argument is a common pattern that looks harmless. It is not — it creates an invisible, unintended reactive dependency.

**Rule:** Inside a `LocalResource` (or any reactive closure), only call `.get()` on signals you explicitly want to trigger a re-run. For all other reads, use `.get_untracked()`.

### 2. Signals that are both read and written inside the same resource are almost always a bug

If a `LocalResource` reads a signal AND the async task that the resource runs also writes that signal, you have created a feedback loop. The pattern:

```rust
let resource = LocalResource::new(move || {
    let val = some_signal.get();        // subscribes
    async move {
        // ...
        some_signal.set(new_val);       // triggers re-run → ∞
    }
});
```

is almost never correct. The write should either use `.update()` inside an `Effect` that is separate from the resource, or the read should use `.get_untracked()` if the signal is only needed as an input snapshot.

### 3. Reactive loops are silent and fast

The bug produced no Rust panics, no console errors, and no warnings. The only symptom was the visual cycling of poems. Without knowing to look for reactive re-triggers in the browser devtools, this could be diagnosed as a bug in `pick_random` or a network issue, when the actual cause was a single `.get()` vs `.get_untracked()` distinction.

Reactive feedback loops should be added to the mental checklist when reviewing any component that writes to a signal inside an async resource task.

### 4. Make selection logic testable by separating randomness from selection

`pick_random` originally called `js_sys::Math::random()` inline, making the entire function WASM-only and non-deterministically testable. By extracting `pick_at_index` (deterministic: takes a pre-computed index), the filtering and selection logic becomes fully unit-testable on the native target with `cargo test`. The WASM-specific call is isolated to one line in the public function.

This is a general pattern for any function that mixes pure logic with browser-API randomness or time: extract the pure core, test it deterministically, keep the impure wrapper thin.
