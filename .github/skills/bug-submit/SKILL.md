---
name: bug-submit
description: Submit and triage a bug through guided conversation. Creates a bug report document in bugs/. Use this when reporting a new bug or unexpected behavior.
---

You are running the bug submission workflow. Your job is **not** to fix the bug — it is to collect enough information to write a thorough, actionable bug report and save it to disk.

Work through the phases below in order. Never skip a phase.

---

## Phase 1 — Capture the Initial Report

Ask the user to describe the bug in their own words if they haven't already. One open question is enough:

> "Tell me what happened. What did you see, and what did you expect to see?"

Do not prompt for structured fields yet — let them describe it naturally first.

---

## Phase 2 — Lightweight Code Probe

Before asking any follow-up questions, do a targeted scan of the codebase to understand what area the bug touches. This informs smarter questions.

Scan steps (run in parallel where possible):
- **Determine the app root** using this logic (in priority order):
  1. If `cwd` is inside `apps/<app-name>/` (e.g. `apps/rw_sixzee/src`), the app root is the nearest ancestor that contains a `Cargo.toml` and a `src/` directory — typically `apps/<app-name>/`.
  2. If `cwd` is an app root itself (e.g. `apps/rw_sixzee/`), use it directly.
  3. If `cwd` is the monorepo root or elsewhere, and the user's description names a specific app, use `apps/<app-name>/`.
  4. If still ambiguous, ask the user which app the bug is in using `ask_user`.
  - Bug reports **always** live at `<app-root>/bugs/` — never at the monorepo root.
  - Create the `bugs/` directory if it doesn't exist yet.
- Grep for keywords from the description (function names, UI labels, route names, module names).
- Check `<app-root>/src/` for files related to the area the user described.
- If the bug involves a specific UI flow or feature, look for the relevant component or module.
- Skim `<app-root>/doc/lessons.md` (if present) for any prior known issues in this area.

Do **not** share your full code analysis with the user — use it internally to ask sharper questions.

---

## Phase 3 — Targeted Follow-up Questions

Based on what you found in Phase 2, ask **3–5 focused questions**. Use `ask_user` for each one.
Ask them one at a time — do not batch all questions into a single message.

Good questions to consider (pick the most relevant, not all):
- How do you reproduce it? (steps, exact sequence of actions)
- Is it consistent or intermittent? Does it happen every time?
- What was the last thing that changed before you noticed this?
- Are there any errors or warnings in the browser console / terminal?
- What browser / OS / build mode were you using? (debug vs release)
- Does it happen with a fresh build / fresh browser profile?
- Is there a specific data state or user input that triggers it vs. one that doesn't?
- Which part of the UI or feature is affected? (if multiple candidates exist)

If the user already answered a question in Phase 1, skip it.

---

## Phase 4 — Initial Triage

Based on the code probe and answers, form a brief hypothesis (2–4 sentences) about where in the code the bug likely originates. This is an educated guess — not a confirmed root cause.

Document:
- Likely module / file(s) involved
- Whether this looks like a logic bug, reactive signal issue, async timing issue, data problem, or something else
- Any similar past bugs you noticed in `doc/lessons.md`

---

## Phase 5 — Assign Bug ID and Slug

Determine the next available bug ID:
- List existing files in `<app-root>/bugs/` to find the highest-numbered ID.
- Increment by 1 for the new ID (zero-padded to 2 digits, e.g. `03`).
- Generate a short kebab-case slug from the title (3–6 words, no articles).

File name format: `bug_<id>_<slug>.md`

Example: `bug_03_score_not_resetting_on_new_game.md`

---

## Phase 6 — Write the Bug Report

Create the file at `<app-root>/bugs/<filename>`. Use this template:

```markdown
# Bug <id> — <Title>

## Status
Open

## Summary
<2–4 sentence plain-English description of what is broken.>

## Steps to Reproduce
1. <Step 1>
2. <Step 2>
3. <Observed result>

## Expected Behavior
<What should happen>

## Actual Behavior
<What actually happens>

## Environment / Context
- **App:** <app name>
- **Build mode:** debug / release / unknown
- **Browser:** <if relevant>
- **Recent changes:** <last known good state or recent code changes, if known>
- **Error output:** <paste of any console errors, panics, or log output>

## Initial Triage
<Your 2–4 sentence hypothesis about where the bug lives and why. Name specific
files and functions if identified. Flag any related lessons from doc/lessons.md.>

---
<!-- The sections below are filled in during the fix phase -->

## Root Cause
*To be determined*

## Fix
*To be determined*

## Regression Test
*To be determined*

## Post-Mortem / Lessons Learned
*To be determined*
```

---

## Phase 7 — Confirm and Close

After writing the file, tell the user:
- The full path of the bug document
- Your 1-sentence triage hypothesis
- What to do next: invoke the `bug-fix` skill when ready to fix it

Do not attempt to fix the bug in this workflow. If you find yourself writing code, stop.
