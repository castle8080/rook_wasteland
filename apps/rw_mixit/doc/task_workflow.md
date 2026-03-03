# Task Workflow — rw_mixit

This document defines the standard process for implementing a task in this project. It applies to human developers and AI agents alike. Follow every phase in order; do not skip phases even for small tasks.

---

## Overview

```
[1. Initialize task doc]
        ↓
[2. Restate + status]
        ↓
[3. Design + 1 level deep]
        ↓
[4. Design critique]
        ↓
[5. Implement + tests]
        ↓
[6. Run tests + clippy]
        ↓
[7. Maintainer review]
        ↓
[8. Edit + re-test]
        ↓
[9. Commit]
        ↓
[10. Update task doc]
```

---

## Phase 1 — Initialize Task Document

Before writing any code, create a task document under `tasks/` if one does not already exist.

**File naming:** `tasks/<milestone>-<id>-<short-slug>.md`
Examples: `tasks/m2-t2-11-waveform-zoom.md`, `tasks/m9-t9-5-scratch-simulation.md`

Use the template below. Fill in what you know; leave unknowns blank for now — they are filled in during later phases.

```markdown
# Task <ID>: <Title>

**Milestone:** M<N> — <Milestone Name>
**Status:** 🔄 In Progress

---

## Restatement

<!-- Phase 2 -->

## Design

<!-- Phase 3 -->

## Design Critique

<!-- Phase 4 -->

## Implementation Notes

<!-- Filled in during / after Phase 5 -->

## Test Results

<!-- Filled in during Phase 6 -->

## Review Notes

<!-- Filled in during Phase 7 -->

## Callouts / Gotchas

<!-- Anything a future developer should know -->
```

> **Rule:** Never start writing code before this file exists and Phase 2 is complete.

---

## Phase 2 — Restate the Task

Write a brief (3–6 sentence) restatement of the task in your own words under the `## Restatement` heading. This confirms you understand the scope before touching any code.

Cover:
- **What** is being built or changed
- **Where** in the codebase it lives (file paths if known)
- **Why** it matters — what the user or system gets from it
- **What is explicitly out of scope** for this task

Set the status field to `🔄 In Progress`.

> **Sanity check:** If you cannot write the restatement in plain language without referring back to the spec every sentence, stop and re-read the relevant spec sections before proceeding.

---

## Phase 3 — Design (One Level Deeper)

The implementation plan gives task-level guidance. Here you go **one level deeper** — just for this task, just for what you are about to build.

Write this under `## Design`. It should cover:

### 3a. Data flow
Trace the path a user action takes from the UI event through signals/state changes to the audio graph or canvas draw. Be specific: name the signals, functions, and nodes involved.

### 3b. Function / type signatures
Sketch any new public functions, structs, or trait impls you plan to introduce. You do not need full implementations, just signatures and brief doc comments.

```rust
/// Computes zoom-adjusted peak slice for waveform draw.
/// `zoom` is 1–8 (powers of 2). Returns `num_cols` peaks.
fn zoomed_peaks(peaks: &[f32], zoom: u8, center: f64, num_cols: usize) -> Vec<f32>
```

### 3c. Edge cases
List the concrete edge cases this task must handle. Examples:
- What happens before a file is loaded?
- What if a signal is `None`?
- What if the user drags outside the canvas boundary?
- What if two events fire in rapid succession?

### 3d. Integration points
List which existing code this task touches or depends on. Name the exact files/functions. This surfaces potential conflicts early.

---

## Phase 4 — Design Critique

Before writing code, argue against your own design. Treat this like a short rubber-duck or peer review.

Write this under `## Design Critique`. Challenge at least these dimensions:

| Dimension | Question to answer |
|---|---|
| **Correctness** | Are there any logical errors in the design as stated? |
| **Simplicity** | Is there a simpler approach that meets the same spec? |
| **Coupling** | Does this design create tight dependencies that will make future tasks harder? |
| **Performance** | Is anything in the hot path (rAF loop, audio callback) that should not be? |
| **Testability** | Can the core logic be tested without spinning up the full WASM environment? |

After the critique, write a one-sentence **Resolution** for each issue raised. If the critique reveals a better design, update Phase 3 before proceeding.

> **Note:** The goal is not to find a perfect design — it is to catch obvious mistakes before they are committed to code.

---

## Phase 5 — Implement + Tests

### Implementation

Write the code. Follow the existing patterns in the codebase:

- Module layout and file placement from the tech spec §6 (Component Tree) and §7 (State Architecture)
- Leptos signal usage: use `.get()` in reactive contexts, `.get_untracked()` in rAF loop closures
- Web Audio node construction and connection in `src/audio/deck_audio.rs`
- Canvas draw passes in `src/canvas/`
- CSS in `static/style.css` — CSS custom properties for all colors

Keep changes surgical. Do not refactor unrelated code in the same task.

### Tests

Every task must include at least one of the following:

| Test type | When to use | Location |
|---|---|---|
| **Unit test** | Pure functions (DSP math, BPM calc, peak extraction, routing logic) | `#[cfg(test)]` module in the same file |
| **Integration test** | Multi-module interactions (load → peaks → draw) | `tests/` directory |
| **Manual test checklist** | UI behaviour that cannot be automated in WASM | Noted under `## Test Results` in the task doc |

For pure Rust logic (signal math, peak extraction, BPM, filters), write `#[test]` functions. These run on the host without WASM and are fast.

For browser-only code (Web Audio nodes, canvas), document the manual steps to verify correctness under `## Test Results`.

Example unit test pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoomed_peaks_returns_correct_count() {
        let peaks: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();
        let result = zoomed_peaks(&peaks, 2, 0.5, 100);
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn zoomed_peaks_center_sample_is_midpoint() {
        let peaks: Vec<f32> = vec![0.0; 512].into_iter()
            .chain(vec![1.0; 512])
            .collect();
        let result = zoomed_peaks(&peaks, 1, 0.75, 10);
        // right half should be all 1.0 when centered at 0.75
        assert!(result.iter().all(|&v| v == 1.0));
    }
}
```

---

## Phase 6 — Run Tests + Clippy

Run both in order. **Do not proceed if either fails.**

```bash
# Run all unit and integration tests (on the host, not WASM)
cargo test

# Run clippy — zero warnings policy
cargo clippy --target wasm32-unknown-unknown -- -D warnings

# Confirm the WASM build still compiles
trunk build
```

**Zero warnings policy:** Every clippy warning must be fixed or explicitly suppressed with `#[allow(...)]` and a comment explaining why. Do not suppress warnings without a reason.

If a test fails:
1. Fix the code, not the test (unless the test itself has a bug)
2. Re-run until green
3. Record the failure and fix under `## Test Results` in the task doc

---

## Phase 7 — Maintainer Review

Put the code down for a moment. Then read it as if you are a different developer who has never seen this code before and will have to maintain it in six months.

Check for:

| Concern | What to look for |
|---|---|
| **Clarity** | Can you tell what each function does from its name and signature alone? |
| **Documentation** | Every public `fn`, `struct`, and `trait` in `src/` must have a `///` doc comment. Private helpers need one if their purpose isn't obvious from the name. |
| **Magic numbers** | Any literal number that isn't `0`, `1`, or `-1` needs a named constant with a comment. |
| **Error handling** | Are `.unwrap()` / `.expect()` calls justified? Replace with proper handling or at minimum `expect("message explaining what should never happen")`. |
| **Naming** | Names should read like prose. Avoid abbreviations except for established domain terms (`bpm`, `lfo`, `eq`, `vu`). |
| **Consistency** | Does the style match the surrounding code? |

Write the findings (even if "looks good") under `## Review Notes` in the task doc. If nothing needed changing, write "No issues found." — do not leave it blank.

---

## Phase 8 — Edit + Re-test

Apply any fixes identified in Phase 7. Then re-run the full test and lint suite (same commands as Phase 6). Even a one-line doc comment addition warrants a re-run to confirm nothing was broken.

If the review surfaced a design issue (not just a style issue), go back to Phase 3/4 and update those sections before changing the code — keep the task doc as a living record of the reasoning.

---

## Phase 9 — Commit

Commit **only the files changed by this task**. Do not bundle unrelated changes.

### Commit message format

```
<task-id>: <short imperative summary> (~50 chars max)

<optional 1–3 line body if the summary alone isn't enough context>

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Examples:

```
T2.11: add waveform zoom controls (1×–8×)

Zoom adjusts the visible time window; peaks are re-sampled on each
zoom level change. Zoom state lives in DeckState as RwSignal<u8>.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

```
T8.4: wire vu_level signal to AnalyserNode in rAF loop

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

**Rules:**
- Use the imperative mood ("add", "fix", "wire", "implement") — not "added" or "adding"
- Reference the task ID at the start
- The subject line must stand alone as a meaningful sentence
- Do not include `[skip ci]` or other meta-flags unless you have a specific reason

---

## Phase 10 — Update Task Document

After the commit, update the task document one final time:

1. Set **Status** to `✅ Done`
2. Fill in `## Test Results` with the actual test output (a brief excerpt is fine) and any manual steps performed
3. Fill in `## Callouts / Gotchas` with anything a future developer should know — surprising browser behaviour, performance caveats, known limitations, follow-up tasks that were deferred

Commit the task doc update as a separate micro-commit:

```
T2.11: mark task done, add callouts
```

---

## Phase 11 — Milestone Lessons & Notes (after last task in a milestone)

After completing the final task of a milestone — or after any major standalone task that surfaced non-obvious discoveries — append a new section to `doc/implementation_lessons_and_notes.md`.

### What belongs here

- Patterns discovered during implementation that will recur in later milestones
- Gotchas not obvious from the spec (e.g. macro expansion quirks, browser API surprises, ownership / lifetime edge cases)
- Decisions where the spec and reality diverged, and why
- Performance or correctness caveats a future developer should know before touching related code

### Format

Add a new top-level section to the file:

```markdown
## Lessons from M<N> — <Milestone Name>

### <Short title>

<2–5 sentences explaining the lesson and the correct pattern going forward.>
```

One sub-section per lesson. Keep entries concise — this is a reference document, not a design doc.

### Commit

Commit the notes file as a separate micro-commit after the milestone commit:

```
M<N>: add implementation lessons and notes
```

> **Rule:** Do not skip this phase. Even "no new lessons" is worth a one-liner confirming it. The document compounds in value over time.

---

## Status Values

| Symbol | Meaning |
|---|---|
| `⬜ Not Started` | Task has not been picked up |
| `🔄 In Progress` | Task is actively being worked |
| `✅ Done` | Task complete, committed, task doc updated |
| `🚫 Blocked` | Cannot proceed; reason documented in task doc |
| `⏸ Deferred` | Descoped from current milestone; moved to backlog |

---

## Quick Reference — Task Document Template

Copy this for each new task:

```markdown
# Task <ID>: <Title>

**Milestone:** M<N> — <Milestone Name>
**Status:** 🔄 In Progress

---

## Restatement

_What is being built, where it lives, why it matters, and what is out of scope._

---

## Design

### Data flow

### Function / type signatures

### Edge cases

### Integration points

---

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | | |
| Simplicity | | |
| Coupling | | |
| Performance | | |
| Testability | | |

---

## Implementation Notes

---

## Test Results

**Automated:**
```
cargo test output excerpt
```

**Manual steps performed:**
- [ ] ...

---

## Review Notes

---

## Callouts / Gotchas
```

---

*This workflow applies to all tasks in `implementation_plan.md`. When in doubt, err on the side of doing more phases, not fewer.*
