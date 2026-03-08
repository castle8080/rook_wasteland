---
name: bug-fix
description: Fix a bug following a structured TDD process with post-mortem documentation. Use this when working on an existing bug report in bugs/.
---

You are running the bug-fix workflow. Follow every phase in order. Do not skip phases, even if the fix seems obvious — the post-mortem and lessons are as important as the code change.

---

## Phase 1 — Read the Bug Report

Locate and read the bug document. If the user didn't provide a path, list `bugs/` and ask them which bug to fix using `ask_user`.

Extract and internalize:
- The summary and steps to reproduce
- The initial triage hypothesis (the suspected area)
- Any error output or environmental context
- Any related lessons from `doc/lessons.md` mentioned in the triage

---

## Phase 2 — Deep Code Analysis

Do a thorough investigation of the code area identified in the triage. Run searches in parallel.

Goals:
- Confirm (or revise) the triage hypothesis
- Trace the full data / execution path related to the bug
- Identify the exact file(s) and function(s) where the fault lies
- Note any related tests that already exist
- Check `doc/lessons.md` for similar past issues in this area

Write a 3–6 sentence internal summary of your findings before moving on. This becomes the Root Cause section.

---

## Phase 3 — Write a Failing Test

Before touching any production code, write a test that **fails** in the current state and **passes** after the fix.

Choose the right test tier for this project:

| Tier | When to use | Where |
|---|---|---|
| 1 — `#[cfg(test)]` | Pure logic, math, state transitions, string ops | Same file as the function |
| 2 — `#[wasm_bindgen_test]` | Isolated web-sys / DOM API calls | `tests/` directory |
| 3 — Integration | Signal→DOM wiring, full component reactive behavior | `tests/integration.rs` |

Guidelines:
- Prefer Tier 1 if any part of the logic can be extracted and tested natively.
- If the buggy code mixes pure logic with WASM-only calls (e.g. `js_sys::Math::random()`, `web_sys::...`), **refactor to extract the pure core first**, then test the extracted function.
- Name the test clearly: `test_<function>_<scenario>` or `<scenario>_returns_<expected>`.
- Confirm the test fails before moving on (run `cargo test <test_name>` for Tier 1).

If a test genuinely cannot be written (the bug is a visual glitch, a timing issue only reproducible in a live browser, etc.), document the reason explicitly and skip to Phase 4 with a manual reproduction checklist instead.

---

## Phase 4 — Fix the Bug

Make the minimal change that makes the failing test pass without breaking other tests.

Rules:
- Do not refactor unrelated code.
- Do not introduce new features.
- Preserve all existing behavior outside the bug's blast radius.
- Follow project conventions (no `.unwrap()` in non-test code, `///` doc comments on public items, etc.).

After making the fix, run the failing test to confirm it now passes.

---

## Phase 5 — Run the Full Test Suite

```bash
cargo test
cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

Both must pass with zero warnings. If browser tests exist:
```bash
python make.py test
```

Fix the code (not the tests) if anything fails. Re-run until clean.

---

## Phase 6 — Self-Review

Read the changed code as a maintainer seeing it for the first time in six months. Check:
- The fix is understandable without context
- Any non-obvious behavior has a comment explaining *why*
- No dead code, no leftover debug prints
- The test clearly describes what it's testing and why that matters

---

## Phase 7 — Code Review

Stage the changed files and run the `code-review` agent against the diff.

For each finding, either:
- **Fix** it before proceeding
- **Waive** it with a one-sentence justification

Do not proceed until all findings are resolved.

---

## Phase 8 — Update the Bug Report

Fill in the sections that were left as "To be determined":

### Root Cause
Write 3–8 sentences explaining:
- **Where** in the code the bug lived (file, function, line range)
- **Why** it caused the symptom (the mechanism, not just "there was a bug here")
- **Why it wasn't caught earlier** (if applicable — missing test, misleading API, silent failure mode)

### Fix
Describe the fix clearly:
- Which file(s) changed
- What changed and why that corrects the bug
- Include short before/after code snippets for the key change

### Regression Test
Describe the test(s) added:
- Test name and file
- What scenario it exercises
- Why this test would have caught the bug originally

### Post-Mortem / Lessons Learned
Write at least one lesson. Good lessons cover:
- A subtle API behavior that surprised you (reactive subscription semantics, WebGL buffer lifecycle, etc.)
- A class of mistake this bug represents (not just the specific instance)
- A rule or checklist item that would prevent recurrence

Format each lesson as a named section with a bolded title:

```markdown
### <Lesson Title>
<2–4 paragraphs>
```

---

## Phase 9 — Update doc/lessons.md

If any lesson from Phase 8 is **broadly applicable** (would be useful in other parts of the codebase, not just this specific bug location), add it to `doc/lessons.md`.

Broadly applicable means: another developer working in a different part of the project could benefit from knowing this.

Criteria for adding:
- It describes a non-obvious behavior of a crate, browser API, or the Leptos reactive system
- It documents a pattern that is easy to get wrong and hard to debug
- It has not already been captured in `doc/lessons.md`

If nothing meets the bar, explicitly note "No new broadly-applicable lessons" in the bug report.

---

## Phase 10 — Update Bug Status and Commit

Update the `## Status` field in the bug report to `Fixed`.

Commit in two steps:

**Commit 1 — the fix:**
```
bug_<id>: <imperative description of what was fixed>

<optional 1–3 line body explaining the root cause>

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

**Commit 2 — the doc update:**
```
bug_<id>: document root cause, fix, and lessons

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Commit only files changed by this bug fix. Do not bundle unrelated changes.

---

## Phase 11 — Smoke Test Suggestions

Finish by suggesting 3–5 manual smoke tests the developer should run to verify the fix in a live browser. These should cover:
- The exact scenario from the bug report
- The most common adjacent workflows (to check for regressions)
- Any edge case that the automated tests couldn't fully cover
