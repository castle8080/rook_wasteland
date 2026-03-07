---
name: prd
description: Write a Product Requirements Document for a new feature or app. Use this when asked to write a PRD, spec, or requirements document.
---

Write a PRD as a Markdown file at `doc/prd-<feature-slug>.md`. Structure it as follows:

## 1. Problem Statement
One paragraph. What user problem does this solve? Why does it matter?

## 2. Goals
Bullet list. What does success look like? Keep to 3–5 measurable outcomes.

## 3. Non-Goals
Explicit list of what this feature will NOT do. This is as important as the goals.

## 4. User Stories
Format: "As a [user], I want [action] so that [benefit]."
Cover the primary happy path and the most important edge cases.

## 5. Functional Requirements
Numbered list. Each requirement must be testable — "the app shall X when Y."
Avoid vague language like "fast" or "easy to use"; specify thresholds.

## 6. Out of Scope / Future Work
Things that came up during requirements gathering but are deliberately deferred.

## 7. Open Questions
Unresolved decisions. Each entry should name who needs to answer it.

---

### Guidelines

- Keep the document short. A PRD that cannot be read in 10 minutes is too long for this project.
- Do not describe implementation details (which Rust types, which signals, etc.) — that belongs in the task design phase.
- The PRD defines *what* and *why*. The task doc defines *how*.
- After writing the PRD, ask the user to confirm the Non-Goals and Open Questions sections before any implementation begins.
