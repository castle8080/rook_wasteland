# RW Poetry — Building Guide

This document defines the process and standards for all development work on this project. Every contributor (human or AI) should follow this guide when implementing features, fixing bugs, or making any non-trivial change.

---

## 1. Core Principles

- **Think before you type.** Design before writing code. Verify before committing.
- **Small, correct steps.** Prefer a series of small, verifiable changes over large uncommitted diffs.
- **The compiler and tests are your ground truth.** If it doesn't compile and pass tests, it isn't done.
- **Leave things better than you found them.** Don't fix unrelated bugs, but do leave the code cleaner than you found the area you touched.

---

## 2. Before Starting Any Task

### 2.1 Understand the Task
- Re-read the relevant sections of `doc/poetry_spec.md` before writing a line of code.
- Re-read `doc/leptos_technical_design_principles_and_api_practices.md` if the task touches Leptos components, signals, resources, or async.
- Check the `tasks/` directory for any prior design documents related to this area.

### 2.2 Break the Task Down
Do not attempt to implement a large task in one pass. Decompose it:
1. Identify the distinct pieces (data model, fetch logic, component, styling, tests).
2. Order them by dependency — implement the foundation before building on top of it.
3. Aim for units of work that can be compiled and verified independently.

### 2.3 Write a Task Document First
Before writing any implementation code, create a task document in the `tasks/` directory.

**Filename convention:** `tasks/{YYYY-MM-DD}-{short-slug}.md`  
Example: `tasks/2026-03-02-poem-reader-component.md`

**Required sections:**
```markdown
# Task: <title>

## Status
In Progress  ← change to "Complete" when done

## Goal
One paragraph describing what this task accomplishes and why.

## Scope
- What is included
- What is explicitly NOT included (avoids scope creep)

## Design
Describe the approach. Include:
- Data flow (how data moves from source to screen)
- Component/module structure
- Key types or structs
- How this fits with existing code
- Any tradeoffs or alternatives considered

## Implementation Plan
Numbered steps in order. Check off as you go.
1. [ ] Step one
2. [ ] Step two
...

## Testing Plan
How will correctness be verified?
- Unit tests: what functions/logic needs tests
- Manual smoke tests: what to click/do to verify in the browser

## Notes / Decisions
Record any decisions made during implementation that aren't obvious from the code.
```

---

## 3. Implementation Standards

### 3.1 Code Quality
- Write clean, readable code. If something needs explanation, add a brief inline comment — but don't comment the obvious.
- Prefer explicitness over cleverness. Rust is verbose; lean into it.
- Follow the existing code style and module layout (see `doc/poetry_spec.md` section 12 for module boundaries).
- Keep functions small and focused. If a function is getting long, split it.

### 3.2 Error Handling
- Do not use `.unwrap()` in production code paths. Use `?`, `map_err`, or explicit match.
- Propagate errors to the UI where applicable — the user should see a graceful message, not a silent failure.
- Panics in WASM produce cryptic browser errors. Treat them as hard bugs.

### 3.3 Leptos-specific Standards
- Use `signal_local()`, `LocalResource`, `Action::new_local()` for any `!Send` types (DOM nodes, JS objects, `web-sys` handles).
- Always consult `doc/leptos_technical_design_principles_and_api_practices.md` for signal and resource patterns before inventing new ones.
- Never call async code directly in component body — use `spawn_local` or `LocalResource`.
- Clean up event listeners and object URLs on unmount to avoid memory leaks.

### 3.4 Unit Tests
Write unit tests for:
- All pure functions (data transformation, parsing, formatting)
- Business logic (e.g. recording state machine transitions, duration formatting)
- Serialization/deserialization of data types (round-trip tests for `PoemEntry`, `Recording`, etc.)

Tests go alongside the source in the same file using `#[cfg(test)]` modules:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration(91342), "1:31");
    }
}
```

For WASM-specific tests (e.g. IndexedDB, browser APIs), use `wasm-bindgen-test` and run with `wasm-pack test`.

---

## 4. Compiler, Tests, and Linter — Run Frequently

**The rule:** Run the compiler and tests after every meaningful code change, not just at the end.

### 4.1 Check the Project Compiles
```bash
cargo check
```
Run this constantly. It is fast and catches type errors immediately.

### 4.2 Run All Tests
```bash
cargo test
```
All tests must pass before a commit. A failing test is a blocking issue — do not commit with known failures.

For WASM-targeted tests:
```bash
wasm-pack test --headless --firefox
# or
wasm-pack test --headless --chrome
```

### 4.3 Run Clippy (Linter)
```bash
cargo clippy -- -D warnings
```
Treat all Clippy warnings as errors. Fix them before committing. Clippy catches real bugs in Rust, not just style nits.

### 4.4 Check Formatting
```bash
cargo fmt --check
```
Auto-format before committing:
```bash
cargo fmt
```

### 4.5 Build with Trunk (Integration Check)
```bash
trunk build
```
Run a full Trunk build periodically and always before a commit that touches `Cargo.toml`, `index.html`, or significant structural changes. This catches WASM-specific compilation issues that `cargo check` misses.

---

## 5. Code Review — Self-Critique Checklist

Before committing, run through this checklist yourself. Treat it as a mandatory self-review.

### 5.1 Correctness
- [ ] Does the code do what the task document says it should?
- [ ] Are all edge cases handled? (empty state, loading state, error state, empty corpus, no mic permission)
- [ ] Are there any off-by-one errors or incorrect type conversions?
- [ ] Does the code handle the `NaN` / `Infinity` cases that `web-sys` audio APIs can return?

### 5.2 Robustness
- [ ] Are there any `.unwrap()` or `.expect()` calls on paths that could realistically fail?
- [ ] Does error handling surface useful information to the user?
- [ ] Are async operations guarded against component unmount races?

### 5.3 Memory and Resource Safety
- [ ] Are event listeners removed on unmount?
- [ ] Are object URLs (`Url::create_object_url_with_blob`) revoked when no longer needed?
- [ ] Are `Closure::forget()` uses intentional, minimal, and documented?

### 5.4 Spec Compliance
- [ ] Does this implementation match the data contract in `poetry_spec.md` (field names, types, paths)?
- [ ] Does the visual output match the typography and color spec (sections 8.3–8.4)?
- [ ] Does the UX match the navigation and interaction spec (sections 8.5–8.10)?

### 5.5 Code Cleanliness
- [ ] Is there any dead code, commented-out code, or debug `console_log!` calls left in?
- [ ] Are imports organized (std, then external crates, then local)?
- [ ] Are type names, variable names, and function names clear and consistent?

### 5.6 Test Coverage
- [ ] Are there tests for the new logic added?
- [ ] Do existing tests still pass?
- [ ] Would a reviewer be able to understand what the tests are checking?

---

## 6. Completing a Task

When implementation is done and all checks pass:

### 6.1 Update the Task Document
Open the task document in `tasks/` and:
- Change `Status` from `In Progress` to `Complete`
- Check off all implementation steps
- Add a `## Completion Notes` section at the bottom:
  ```markdown
  ## Completion Notes
  Date completed: YYYY-MM-DD
  
  Brief summary of what was implemented. Note any deviations from the original design,
  decisions made during implementation, or follow-up items for future tasks.
  ```

### 6.2 Commit the Work
Each task should result in **one focused commit** covering:
- The implementation files
- The test files
- The updated task document

**Commit message format:**
```
<type>: <short imperative description>

<optional body — 1-3 sentences explaining WHY, not what>

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Types: `feat` (new feature), `fix` (bug fix), `refactor`, `test`, `docs`, `chore`

Examples:
```
feat: implement poem reader component with static JSON fetch

Loads poem list from /poems/poems_index.json via gloo-net, renders
selected poem using pre-wrap text layout per spec section 8.3.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

```
feat: add IndexedDB persistence for audio recordings

Implements rw_poetry_db v1 schema with recordings and audio_blobs
stores using the idb crate. Audio stored as ArrayBuffer.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

**Keep commits atomic and meaningful.** One task = one commit (or a small number of logical commits if the task is large). Avoid "WIP" commits on the main branch.

---

## 7. Tasks Directory

The `tasks/` directory is a living log of design decisions and implementation history.

- Every non-trivial task gets a markdown file **before work begins**.
- Files are never deleted — they are a record of what was built and why.
- When a bug is found later, check the relevant task doc — the original design decisions are often the explanation.

**What counts as "non-trivial"?** Any task that:
- Adds a new component, module, or data type
- Changes the data schema or file layout
- Involves async, WASM browser APIs, or IndexedDB
- Would take more than ~30 minutes to implement

Simple one-liner fixes, typos, or config tweaks do not need a task doc.

---

## 8. Quick Reference — Command Checklist

```bash
# While developing (run constantly)
cargo check

# Before committing
cargo fmt
cargo clippy -- -D warnings
cargo test
trunk build

# After adding poems to corpus
python3 scripts/build_poems_index.py

# WASM-specific tests (when you have browser-targeted tests)
wasm-pack test --headless --chrome
```
