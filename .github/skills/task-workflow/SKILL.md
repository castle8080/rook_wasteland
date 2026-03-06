---
name: task-workflow
description: Follow the structured task workflow for implementing a feature. Use this when starting any non-trivial implementation task.
---

Before writing any code, complete phases 1–4. Never skip them.

## Phase 1 — Create Task Document

Create `tasks/<milestone>-<id>-<slug>.md`. Use this template:

```markdown
# Task <ID>: <Title>

**Milestone:** M<N> — <Name>
**Status:** 🔄 In Progress

## Restatement
## Design
### Data flow
### Function / type signatures
### Edge cases
### Integration points
## Design Critique
| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | | |
| Simplicity | | |
| Coupling | | |
| Performance | | |
| Testability | | |
## Implementation Notes
## Test Results
## Review Notes
## Callouts / Gotchas
```

## Phase 2 — Restate the Task

Write 3–6 sentences under `## Restatement` covering: what is being built, where in the codebase it lives, why it matters, and what is explicitly out of scope.

Stop and confirm the restatement with the user before proceeding.

## Phase 3 — Design

Under `## Design`, write:
- **Data flow**: trace the user action → signals/state → output path, naming specific signals and functions
- **Function / type signatures**: new public functions, structs, trait impls (signatures + doc comments, not implementations)
- **Edge cases**: concrete cases that must be handled (None signals, empty state, rapid user input, etc.)
- **Integration points**: exact files and functions this task touches

## Phase 4 — Design Critique

Fill in the Design Critique table. Challenge each dimension honestly. Write a one-sentence Resolution for each issue. If the critique reveals a better design, update Phase 3 first.

## Phase 5 — Implement + Tests

Follow existing patterns. Every task must include at least one test:
- Pure Rust logic → `#[cfg(test)]` in the same file, runs with `cargo test`
- Browser-only WebGL / web-sys calls → `#[wasm_bindgen_test]` in `tests/mN_*.rs`, runs with `wasm-pack test --headless --firefox`
- Component wiring / reactive signal → DOM behaviour → `#[wasm_bindgen_test]` in `tests/integration.rs` (see guidance below)

**When to add an integration test in `tests/integration.rs`:**
- A new signal gates a DOM element's visibility or content
- A user action triggers a signal change that causes a redraw or layout change
- A new pipeline (e.g. file → GPU → signal) is wired through multiple components
- The full `App` gains a new DOM landmark that should always be present
- An error path should show UI feedback in the DOM

Integration tests mount a real Leptos component into a headless Firefox browser.
Follow these patterns (see existing tests for working examples):
```rust
use leptos::mount::mount_to;   // NOT leptos::mount_to

// Isolate each test with a fresh container.
let container = fresh_container();
let _handle = mount_to(container.clone(), App);

// Flush Leptos effects before asserting.
tick().await;

// Query within the container, not the whole document.
container.query_selector(".foo").unwrap()
```

## Phase 6 — Run Tests + Clippy

```bash
cargo test
cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings
trunk build
```

All three must pass. Zero warnings. Fix the code, not the test, if a test fails.

> **Note:** `cargo test` runs only native (Tier 1) tests. Browser tests
> (Tier 2 & 3) require `python make.py test` which also runs
> `wasm-pack test --headless --firefox`.

## Phase 7 — Self-Review

Read the code as a maintainer seeing it for the first time in six months. Check:
- Every public `fn`/`struct`/`trait` has a `///` doc comment
- Magic numbers have named constants with comments
- `.unwrap()` / `.expect()` calls are justified
- Names read like prose — no unexplained abbreviations

Record findings (even "No issues found") under `## Review Notes`.

## Phase 8 — Edit + Re-test

Apply Phase 7 fixes. Re-run the Phase 6 commands even for one-line changes.

## Phase 9 — Commit

```
<task-id>: <imperative summary (~50 chars)>

<optional 1–3 line body>

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Commit only files changed by this task. Imperative mood: "add", "fix", "wire" — not "added" or "adding".

## Phase 10 — Close Task Doc

Set Status to `✅ Done`. Fill in `## Test Results` and `## Callouts / Gotchas`. Commit the doc update as a separate micro-commit: `<task-id>: mark task done`.
