---
name: feature-implement
description: Implement a feature following a structured TDD process with spec updates and post-implementation documentation. Use this when working on an existing feature document in features/.
---

You are running the feature implementation workflow. Follow every phase in order. Do not skip phases — the spec updates, decisions, and lessons are as important as the code.

---

## Phase 1 — Read the Feature Document

Locate and read the feature document. If the user didn't provide a path, list `features/` and ask them which feature to implement using `ask_user`.

Extract and internalize:
- The summary, goals, and non-goals
- The functional requirements (these drive test cases)
- The UI / UX notes
- The architecture fit section (existing modules to touch, new modules to introduce)
- Any open questions (flag any unresolved ones to the user before proceeding)
- Any existing lessons from `doc/lessons.md` relevant to the areas being touched

If there are unresolved Open Questions that would block implementation, use `ask_user` to resolve them now. Update the feature document with the answers before continuing.

---

## Phase 2 — Read Supporting Docs

Read the following in parallel (skip any that don't exist):
- `doc/prd.md` — confirm the feature aligns with product direction
- `doc/tech_spec.md` — understand the existing architecture constraints
- `doc/wireframes.md` — understand existing layout patterns and where the feature fits
- `doc/project_plan.md` — understand which milestone this work belongs to
- `doc/lessons.md` — surface any non-obvious traps in the areas this feature touches

Also scan the source files listed in the "Architecture Fit" section of the feature document.

---

## Phase 3 — Create the Task Document

Create `tasks/<milestone>-<feature-slug>.md` following the standard task-workflow template:

```markdown
# Feature <id>: <Title>

**Feature Doc:** features/<filename>
**Milestone:** <milestone from project plan, or "standalone feature">
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
## Coverage Audit
| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| | | | |
## Test Results
## Review Notes
## Decisions Made
## Lessons / Highlights
## Callouts / Gotchas
```

---

## Phase 4 — Restate and Design

### Restatement

Write 4–7 sentences under `## Restatement` covering:
- What is being built and why
- Where it lives in the codebase
- What is explicitly out of scope (refer to Non-Goals in the feature doc)
- Which milestone / feature doc this implements

Stop and confirm the restatement with the user before proceeding.

### Design

Under `## Design`, write:
- **Data flow**: trace the user action → signals/state → output path, naming specific signals and functions
- **Function / type signatures**: new public functions, structs, trait impls (signatures + doc comments, not implementations)
- **Edge cases**: concrete cases that must be handled (None signals, empty state, rapid user input, etc.)
- **Integration points**: exact files and functions this feature touches

---

## Phase 5 — Design Critique

Fill in the Design Critique table. Challenge each dimension honestly. Write a one-sentence Resolution for each issue. If the critique reveals a better design, update Phase 4 first.

---

## Phase 6 — Implement + Tests

Follow the existing patterns and conventions in the codebase. Every feature must include tests.

Choose the right test tier:

| Tier | When to use | Where |
|---|---|---|
| 1 — `#[cfg(test)]` | Pure logic, math, state transitions, string ops | Same file as the function |
| 2 — `#[wasm_bindgen_test]` | Isolated web-sys / DOM API calls | `tests/` directory |
| 3 — Integration | Signal→DOM wiring, full component reactive behavior | `tests/integration.rs` |

Guidelines:
- Write failing tests before writing the implementation code (TDD).
- Prefer Tier 1 if any part of the logic can be extracted and tested natively.
- For each functional requirement in the feature doc, there should be at least one test.
- Name tests clearly: `test_<function>_<scenario>` or `<scenario>_returns_<expected>`.

**When to add an integration test in `tests/integration.rs`:**
- A new signal gates a DOM element's visibility or content
- A user action triggers a signal change that causes a redraw or layout change
- A new pipeline is wired through multiple components
- The full `App` gains a new DOM landmark that should always be present
- An error path should show UI feedback in the DOM

---

## Phase 6.5 — Coverage Audit

**This step is mandatory. Do not skip it.**

Before moving to Phase 7, fill in the `## Coverage Audit` table in the task doc. For every functional requirement and meaningful behaviour:

1. **Happy path** — is there a test?
2. **Each edge case from Phase 4** — is each one covered?
3. **Error / `None` / empty-input paths** — tested, or explicitly waived with reason?
4. **Signal → DOM reactive wiring** — is there an integration test (Tier 3)?

An undocumented gap is a defect in the test suite. Fill it or document it — never silently skip it.

---

## Phase 7 — Run Tests + Clippy

```bash
cargo test
cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings
trunk build
```

All three must pass with zero warnings. If browser tests exist:
```bash
python make.py test
```

Fix the code (not the tests) if anything fails. Re-run until clean.

---

## Phase 8 — Self-Review

Read the changed code as a maintainer seeing it for the first time in six months. Check:
- Every public `fn`/`struct`/`trait` has a `///` doc comment
- Magic numbers have named constants with comments
- `.unwrap()` / `.expect()` calls are justified
- Names read like prose — no unexplained abbreviations
- No dead code, no leftover debug prints

Record findings (even "No issues found") under `## Review Notes` in the task doc.

---

## Phase 9 — Code Review

Stage the changed files and run the `code-review` agent against the diff.

For each finding, either:
- **Fix** it before proceeding
- **Waive** it with a one-sentence justification in `## Review Notes`

Do not proceed until all findings are resolved or waived.

---

## Phase 10 — Update Existing Specs

Review whether any of the following documents need updating to reflect the new feature. Only update a doc if the feature genuinely changes what the doc describes — do not pad or restate things already covered by the feature doc.

- **`doc/prd.md`** — add user stories or functional requirements if the feature expands the product scope
- **`doc/tech_spec.md`** — add new modules, signals, types, or architectural patterns introduced by this feature
- **`doc/wireframes.md`** — add or update wireframe sections for any new or changed screens / layouts
- **`doc/project_plan.md`** — add the feature to the relevant milestone, or add a new milestone entry

For each doc updated, briefly note what changed in `## Spec Changes` in the task doc.

If a doc does not need updating, note "No changes needed" for that doc in `## Spec Changes`. Do not leave the section blank.

---

## Phase 11 — Update the Feature Document

Fill in the sections that were left as "To be determined":

### Implementation Plan
Describe the final implementation approach:
- Which files were added or modified and why
- Key architectural decisions made
- Any deviations from the original Architecture Fit section and why

### Spec Changes
List which `doc/` files were updated and what changed.

### Test Strategy
Describe the tests added:
- Test names and files
- What scenarios they exercise
- Coverage gaps explicitly noted

### Decisions Made
Record every significant technical or product decision made during implementation:
- The decision (what was chosen)
- The alternatives considered
- Why this option was selected

Format as a list of named entries:

```markdown
### Decision: <Short Title>
**Chosen:** <what was decided>
**Alternatives considered:** <what else was evaluated>
**Rationale:** <why this was chosen>
```

### Lessons / Highlights
Write at least one lesson. Good lessons cover:
- A subtle API behavior that surprised you
- A crate or browser quirk discovered during implementation
- A class of mistake that could recur in similar work
- A highlight — an elegant solution or pattern worth repeating

Format each entry with a bolded title:

```markdown
### <Lesson or Highlight Title>
<2–4 sentences>
```

---

## Phase 12 — Update doc/lessons.md

If any lesson from Phase 11 is **broadly applicable** — useful in other parts of the codebase, not just this feature — add it to `doc/lessons.md`.

Criteria for adding:
- It describes a non-obvious behavior of a crate, browser API, or the Leptos reactive system
- It documents a pattern that is easy to get wrong and hard to debug
- It has not already been captured in `doc/lessons.md`

If nothing meets the bar, explicitly note "No new broadly-applicable lessons" in the feature document.

---

## Phase 13 — Update Feature Status and Commit

Update the `## Status` field in the feature document to `Implemented`.

Commit in two steps:

**Commit 1 — the implementation:**
```
feature_<id>: <imperative description of what was added>

<optional 2–4 line body summarizing the approach>

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

**Commit 2 — the doc updates:**
```
feature_<id>: update specs, feature doc, and lessons

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Commit only files changed by this feature. Do not bundle unrelated changes.

---

## Phase 14 — Smoke Test Suggestions

Finish by suggesting 3–5 manual smoke tests the developer should run to verify the feature in a live browser. These should cover:
- The primary happy path from the user stories
- Key edge cases that automated tests couldn't fully cover
- Integration with adjacent existing features (to check for regressions)
