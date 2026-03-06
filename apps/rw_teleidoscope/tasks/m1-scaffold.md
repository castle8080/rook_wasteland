# Task M1: Project Scaffold

**Milestone:** M1 — Project Scaffold  
**Status:** 🔄 In Progress

## Restatement

This task creates the minimum project skeleton for `rw_teleidoscope`, a Leptos 0.8
CSR/WASM app. It establishes the `Cargo.toml`, `Trunk.toml`, `index.html`,
`make.py`, and all source module stubs so that `python make.py build` and
`python make.py lint` both exit 0. No kaleidoscope logic is implemented — the
app renders a blank page with a title. This is strictly scaffolding; all rendering,
controls, and image-processing work is deferred to later milestones.

## Design

### Data flow

No runtime data flow at this stage — the app mounts, provides `KaleidoscopeParams`
and `AppState` contexts (all signals at default values), and renders a placeholder div.

### Function / type signatures

See `tech_spec.md` Section 5 for full signal definitions. At scaffold stage all
structs are implemented with their full signal fields but no logic.

### Edge cases

- `wasm_bindgen(start)` must be excluded under `cfg(test)` to avoid duplicate
  start symbol linker errors with `wasm-pack test`.

### Integration points

All new files; no existing files modified.

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Stub modules must compile on `wasm32-unknown-unknown` | Use `#[allow(dead_code)]` via lib.rs cfg_attr; ensure no native-only types used |
| Simplicity | Many files to create in one task | Accept — it's a scaffold; all files are minimal |
| Coupling | State structs provide contexts from App | Matches tech spec; no change needed |
| Performance | N/A at scaffold stage | N/A |
| Testability | routing.rs must have `#[cfg(test)]` unit tests | Include tests matching rw_mixit pattern |

## Implementation Notes

- Follow rw_mixit `lib.rs` pattern exactly for `cfg(test)` / `cfg(not(test))` guards.
- `KaleidoscopeParams` and `AppState` must be `#[derive(Clone, Copy)]` and use
  `RwSignal` fields, provided via `provide_context` from `App`.
- Shader files need only be valid GLSL (passthrough colour output).

## Test Results

## Review Notes

## Callouts / Gotchas
