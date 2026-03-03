# Task T0.1: Init Cargo Workspace

**Milestone:** M0 — Project Scaffold
**Status:** ✅ Done

---

## Restatement

Create `Cargo.toml` for the `rw_mixit` crate with all v1 dependencies: `leptos` (csr), `web-sys` (all required Web Audio/DOM/event features), `wasm-bindgen`, `gloo-events`, `console_error_panic_hook`, `rustfft`, and `js-sys`. Include a `[lib]` section with `crate-type = ["cdylib", "rlib"]` (cdylib for WASM output, rlib for host-side `cargo test`). Set release profile to `opt-level = "z"`, `lto = true`, `codegen-units = 1` for minimal WASM binary size. Out of scope: adding any source files.

---

## Design

### Data flow
Static configuration only. No runtime data flow.

### Function / type signatures
None — this is a `Cargo.toml` change only.

### Edge cases
- `web-sys` requires every Web API type to be explicitly listed as a crate feature; missing features cause compile errors later. All types referenced in later milestones are included now to avoid surprise breakage.
- `OscillatorNode` and `PointerEvent` added proactively (needed in M9).

### Integration points
All subsequent tasks depend on this.

---

## Design Critique

| Dimension   | Issue | Resolution |
|---|---|---|
| Correctness | `[lib] crate-type = ["cdylib", "rlib"]` means `fn main()` in lib.rs is not automatically the WASM entry. | Added `#[wasm_bindgen(start)]` in T0.3. |
| Simplicity  | A binary-only setup (no `[lib]`) would be simpler. | Spec requires `cdylib` for wasm-bindgen; `rlib` enables `cargo test` on the host. |
| Coupling    | All features listed upfront. | Intentional — avoids mid-milestone feature additions breaking builds. |
| Performance | N/A (config file) | — |
| Testability | N/A | — |

---

## Implementation Notes

Followed spec verbatim. Added `OscillatorNode` and `PointerEvent` web-sys features not listed in the tech spec but clearly needed by M9 (flanger LFO and scratch/pointer events).

---

## Test Results

**Automated:**
```
cargo check --target wasm32-unknown-unknown → Finished in ~55s, 0 errors
```

**Manual steps performed:**
- [x] Confirmed Cargo.lock generated with all expected crates

---

## Review Notes

No issues found.

---

## Callouts / Gotchas

- `web-sys` feature list is exhaustive for v1. If a new Web API type is needed, add its feature string here.
- `opt-level = "z"` is size-optimised, not speed-optimised. Performance-sensitive DSP code is handled by the Web Audio API nodes (which run natively), not WASM, so this is acceptable.
