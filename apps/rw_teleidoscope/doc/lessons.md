# rw_teleidoscope — Lessons Learned

## Purpose

This document is a living record of non-obvious problems, surprises, and hard-won
insights discovered during the build-out of rw_teleidoscope. It is **not** a task
list or a design doc — it is a memory aid.

When you hit a bug that took time to diagnose, encounter a browser or crate quirk
that isn't obvious from the docs, or find that an assumption in the tech spec was
wrong, **add a lesson here**. Future development (and future AI-assisted sessions)
should check this file before starting work in a relevant area to avoid repeating
the same mistakes.

### What belongs here

- WebGL / GLSL gotchas specific to this codebase
- Leptos 0.8 / web-sys API surprises
- Browser compatibility issues discovered during manual testing
- Crate version incompatibilities or missing feature flags
- Performance findings (what was fast / slow in practice vs theory)
- Shader algorithm corrections (e.g. the fold formula needed adjusting)
- Any decision reversed from the tech spec, and why

### What does NOT belong here

- Tasks or status tracking → use `doc/milestones/`
- Design decisions → use `doc/tech_spec.md` or `doc/prd.md`
- General Rust/WASM knowledge that applies everywhere → use `.github/skills/`

### Format for each lesson

```
## L<N>: <Short title>

**Milestone:** M<N>  
**Area:** <e.g. WebGL / Leptos / Shader / Camera / Export / Build>  
**Symptom:** What went wrong or what was surprising.  
**Cause:** Why it happened.  
**Fix / Workaround:** What was done to resolve it.  
**Watch out for:** Any follow-on risks or related areas to check.
```

---

## Lessons

*No lessons recorded yet. The first lesson should be added during M1 or M2 when
the first non-trivial build or runtime issue is encountered.*
