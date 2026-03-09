---
name: feature-submit
description: Submit and triage a feature request through guided conversation. Creates a feature document in features/. Use this when proposing a new addition to an existing codebase and spec.
---

You are running the feature submission workflow. Your job is **not** to implement the feature — it is to gather enough information to produce a thorough, actionable feature document and save it to disk.

Work through the phases below in order. Never skip a phase.

---

## Phase 1 — Capture the Initial Idea

Ask the user for a short description of the feature. One open question is enough:

> "Describe the feature you have in mind. What should it do, and why do you want it?"

Do not prompt for structured fields yet — let them describe it naturally first.

---

## Phase 2 — Determine App Root and Feature Directory

Before asking follow-up questions, determine where this feature lives:

- **Determine the app root** using this logic (in priority order):
  1. If `cwd` is inside `apps/<app-name>/` (e.g. `apps/rw_sixzee/src`), the app root is the nearest ancestor that contains a `Cargo.toml` and a `src/` directory — typically `apps/<app-name>/`.
  2. If `cwd` is an app root itself (e.g. `apps/rw_sixzee/`), use it directly.
  3. If `cwd` is the monorepo root or elsewhere, and the user's description names a specific app, use `apps/<app-name>/`.
  4. If still ambiguous, ask the user which app this feature belongs to using `ask_user`.
- Feature documents **always** live at `<app-root>/features/` — never at the monorepo root.
- Create the `features/` directory if it doesn't exist yet.

---

## Phase 3 — Read the Existing Specs

Read all of the following in parallel (skip any that don't exist):

- `<app-root>/doc/prd.md` — product requirements and user stories
- `<app-root>/doc/tech_spec.md` — stack, module layout, state architecture
- `<app-root>/doc/wireframes.md` — screen layouts, component hierarchy, navigation
- `<app-root>/doc/project_plan.md` — milestones, current status
- `<app-root>/doc/lessons.md` — past non-obvious issues and hard-won insights
- `<app-root>/features/` — any existing feature documents to avoid overlap

Also do a targeted scan of `<app-root>/src/` for files likely to be touched by this feature (based on the description from Phase 1).

Do **not** share your full analysis with the user — use it to ask sharper questions and to determine how the feature fits into the existing codebase.

---

## Phase 4 — Base Clarifying Questions

Ask **3–5 focused questions** to understand the feature, one at a time using `ask_user`. Choose the most relevant from:

- Who is the primary user and what problem does this solve for them?
- Is this a new screen / view, an enhancement to an existing one, or a background capability?
- What is the minimum viable version of this feature? What is optional / v2?
- Are there any known constraints (performance, existing data model, accessibility)?
- Does this feature conflict with or extend anything already in the PRD or wireframes?
- Are there any existing UI patterns in the app that this feature should match?

If the user already answered a question in Phase 1, skip it.

---

## Phase 5 — Formulate a High-Level Plan

Based on the specs you read and the answers from Phase 4, formulate a high-level plan for the feature:

- Where does it fit in the existing architecture?
- Which existing modules, components, or signals would it touch or extend?
- What new modules, components, or state would need to be introduced?
- Does it require new data persistence, background workers, or external APIs?
- Are there any milestone gaps — places where the existing project plan doesn't account for this work?

Present the high-level plan to the user in 3–6 sentences. Then ask **2–4 refinement questions** to narrow it down, one at a time using `ask_user`. Examples:

- "My plan assumes X — is that correct, or did you have something different in mind?"
- "Should this be implemented as part of an existing milestone or as a new milestone?"
- "The wireframes show Y layout — should this feature extend that pattern or introduce a new one?"
- Technical questions are welcome here: "Should the state for this feature be persisted to localStorage, or is session-only acceptable?"

---

## Phase 6 — Assign Feature ID and Slug

Determine the next available feature ID:
- List existing files in `<app-root>/features/` to find the highest-numbered ID.
- Increment by 1 for the new ID (zero-padded to 3 digits, e.g. `003`).
- Derive a short kebab-case slug from the feature name (3–6 words, no articles).

File name format: `feature_<id>_<slug>.md`

Example: `feature_003_export_game_history_csv.md`

---

## Phase 7 — Write the Feature Document

Create the file at `<app-root>/features/<filename>`. Use this template:

```markdown
# Feature <id> — <Title>

## Status
Proposed

## Summary
<2–4 sentence plain-English description of what this feature adds and why it matters.>

## Problem Statement
<One paragraph. What user problem does this solve? Why does it matter now?>

## Goals
<Bullet list. 3–5 measurable outcomes that define success.>

## Non-Goals
<Explicit list of what this feature will NOT do. This is as important as the goals.>

## User Stories
<Format: "As a [user], I want [action] so that [benefit]."
Cover the primary happy path and important edge cases.>

## Functional Requirements
<Numbered list. Each requirement must be testable — "the feature shall X when Y."
Avoid vague language like "fast"; specify thresholds where needed.>

## UI / UX Notes
<Describe any screen layout changes, new controls, or interaction patterns.
Reference existing wireframes sections where applicable.
Note any accessibility requirements.>

## Architecture Fit
<How this feature fits into the existing codebase:
- Which existing modules/components/signals it extends or touches
- What new modules, components, or types need to be introduced
- Any changes to existing state shape or persistence>

## Open Questions
<Unresolved decisions. Each entry should note what needs to be decided before implementation.>

## Out of Scope / Future Work
<Things discussed but deliberately deferred.>

---
<!-- The sections below are filled in during the implementation phase -->

## Implementation Plan
*To be determined*

## Spec Changes
*To be determined (list any doc/*.md files that will need updating)*

## Test Strategy
*To be determined*

## Decisions Made
*To be determined*

## Lessons / Highlights
*To be determined*
```

---

## Phase 8 — Confirm and Close

After writing the file, tell the user:
- The full path of the feature document
- A 2-sentence summary of what the feature does and how it fits the existing app
- What to do next: review and edit the feature doc, then invoke the `feature-implement` skill when ready to build it

Do **not** change any existing `doc/` files (prd.md, tech_spec.md, wireframes.md, project_plan.md) during this workflow. Those are updated during implementation.

Do not attempt to implement the feature in this workflow. If you find yourself writing code, stop.
