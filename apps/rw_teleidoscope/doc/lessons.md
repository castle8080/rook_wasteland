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

## L1: Module-only stub files require inner doc comments

**Milestone:** M1  
**Area:** Build  
**Symptom:** Compiler error "expected item after doc comment" in stub `.rs` files
that contained only `///` outer doc comments and no Rust items.  
**Cause:** `///` outer doc comments must precede an item (fn, struct, etc.). A
file with only outer doc comments has nothing for them to document.  
**Fix / Workaround:** Use `//!` inner doc comments (module-level) instead of `///`
in stub files that contain no items.  
**Watch out for:** Any future stub module or placeholder file — always use `//!`
until real items are added.

---

## L2: `wasm_bindgen_test_configure!` must be repeated in each integration test file

**Milestone:** M1  
**Area:** Build / Testing  
**Symptom:** `wasm-pack test --headless --firefox` reports "no tests to run" and
prints a message saying the suite is "only configured to run in node.js" — even
though `wasm_bindgen_test_configure!(run_in_browser)` exists in `src/lib.rs`.  
**Cause:** `tests/*.rs` integration tests are compiled as separate crates, so the
configure call in `src/lib.rs` does not apply to them.  
**Fix / Workaround:** Add `wasm_bindgen_test_configure!(run_in_browser);` at the
top of every file under `tests/` that contains `#[wasm_bindgen_test]` tests.  
**Watch out for:** Any new integration test file added to `tests/` — always add
the configure line, otherwise tests silently do nothing in browser mode.

---

## L3: `glow::Context::from_webgl2_context` is not `unsafe` in glow 0.13

**Milestone:** M2  
**Area:** WebGL / Build  
**Symptom:** Compiler warns "unnecessary `unsafe` block" when wrapping the
`from_webgl2_context` call in an `unsafe {}` block.  
**Cause:** In glow 0.13, `Context::from_webgl2_context` is a safe function —
the function signature does not include `unsafe`.  
**Fix / Workaround:** Call it directly without an `unsafe` block.  Remove any
SAFETY doc comment that refers to this call.  
**Watch out for:** Any future upgrade of the `glow` crate may change the
signature; re-check after bumping the version.

---

## L4: Use `inspect_err` instead of `map_err` for pure side-effect logging

**Milestone:** M2  
**Area:** Build  
**Symptom:** `cargo clippy -- -D warnings` fails with `manual_inspect` lint
when `map_err(|e| { side_effect(e); e })` is used solely to log the error
without transforming it.  
**Cause:** Clippy's `manual_inspect` lint detects `map_err` where the closure
returns the argument unchanged.  
**Fix / Workaround:** Replace `.map_err(|e| { log(e); e })` with
`.inspect_err(|e| { log(e); })`.  
**Watch out for:** Any `map_err` used purely for logging — always prefer
`inspect_err` for zero-transformation side effects.

---
