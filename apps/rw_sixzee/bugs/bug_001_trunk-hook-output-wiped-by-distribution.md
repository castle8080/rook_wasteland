# Bug 001 — Trunk post_build hook output wiped by distribution swap

## Status
Fixed

## Summary
When building rw_sixzee with `trunk build`, `trunk serve`, or `python make.py build`,
the Web Worker binary files (`grandma_worker_core.js` and `grandma_worker_core_bg.wasm`)
are never present in the final `dist/assets/` directory. The Trunk `post_build` hook
runs `python make.py worker` which writes those files to `dist/assets/`, but Trunk then
performs an atomic distribution swap ("applying new distribution") that replaces `dist/`
with a freshly staged directory — discarding everything the hook wrote. Additionally,
`make.py build()` was changed to rely solely on the hook rather than calling
`_build_worker()` explicitly, so there is no fallback path that survives the swap.

## Steps to Reproduce
1. Ensure a clean state: `Remove-Item -Recurse dist/` (or start fresh checkout).
2. Run `python make.py build` from `apps/rw_sixzee/`.
3. Observe terminal output: the hook prints `[grandma worker] built -> dist/assets/...`.
4. After build completes, inspect `dist/assets/`.
5. Only `grandma_quotes.json` and `grandma_worker.js` are present.
   `grandma_worker_core.js` and `grandma_worker_core_bg.wasm` are missing.
6. Open the app in Chrome with DevTools open.
7. Navigate to a game, roll dice until "Ask Grandma" is enabled, click it.
8. **Observed:** Console error: `Failed to load resource: the server responded with a
   status of 404 ()` for `grandma_worker_core.js`, followed by `Uncaught ReferenceError:
   wasm_bindgen is not defined` in `grandma_worker.js`.

## Expected Behavior
After any build command, `dist/assets/` contains all four worker-related files:
- `grandma_quotes.json`
- `grandma_worker.js`
- `grandma_worker_core.js`
- `grandma_worker_core_bg.wasm`

The "Ask Grandma" overlay opens and displays advice cards without errors.

## Actual Behavior
`grandma_worker_core.js` and `grandma_worker_core_bg.wasm` are absent from `dist/assets/`
after every build. The worker fails to initialize and the overlay either shows a spinner
indefinitely or displays an error state.

## Environment / Context
- **App:** rw_sixzee
- **Build mode:** debug (trunk build) and via `python make.py build`
- **Browser:** Chrome / Chromium
- **Recent changes:** M7 "Ask Grandma" implementation added a `[[hooks]]` entry in
  `Trunk.toml` (`stage = "post_build"`, command = `python make.py worker`) to auto-build
  the worker WASM binary. At the same time, the explicit `_build_worker()` call was
  removed from `make.py build()` in favour of relying on the hook.
- **Error output:**
  ```
  GET http://localhost:8080/rw_sixzee/assets/grandma_worker_core.js net::ERR_ABORTED 404
  Uncaught ReferenceError: wasm_bindgen is not defined
      at grandma_worker.js:7
  ```

## Initial Triage
The `post_build` hook stage in Trunk fires **before** the "applying new distribution"
step. Trunk stages the new distribution in a temporary directory and then atomically
replaces `dist/` — so any files the hook wrote directly to `dist/assets/` are lost.
This affects `Trunk.toml` (the `[[hooks]]` section) and `make.py` (the `_build_worker`
and `worker` functions). A likely fix is to output `wasm-bindgen` artifacts into the
**source** `assets/` directory (gitignored entries for `grandma_worker_core.*`) so that
Trunk's `[[copy-dir]]` picks them up during staging, or to restore the explicit
`_build_worker()` call in `make.py build()` and run it **after** `trunk build` completes
at the process level (not as an in-process hook). See also `doc/lessons.md` L16 which
documents the worker setup but does not yet capture this hook-ordering gotcha.

---

## Root Cause

`Trunk.toml` had a `[[hooks]] stage = "post_build"` entry that ran
`python make.py worker` after every build. Trunk's build pipeline works as
follows: (1) compile WASM, (2) stage all outputs (including `[[copy-dir]]`
contents) into a **temp directory**, (3) fire `post_build` hooks, (4) atomically
replace `dist/` with the temp directory ("applying new distribution"). The hook
wrote `grandma_worker_core.js` and `grandma_worker_core_bg.wasm` to `dist/assets/`
(the *old* distribution) in step 3, but step 4 then discarded that directory
entirely and replaced it with the staged temp dir — which never included the
worker files. Compounding the problem, `make.py build()` was also changed to rely
*solely* on the hook, so there was no other code path that would produce the
worker files. The files appeared in `dist/assets/` during development only because
they had been pre-built by an earlier manual `python make.py build` run (before
the hook/explicit-call change), and trunk serve was serving them from a stale
`dist/`.

Root files: `Trunk.toml` (`[[hooks]]` section), `make.py` (`build()`, `dist()`,
`_build_worker()`).

## Fix

**`make.py`** — `_build_worker()` now writes to the **source** `assets/`
directory instead of `dist/assets/`. Both `build()` and `dist()` call
`_build_worker()` *before* the `trunk build` command so that when Trunk's
`[[copy-dir]]` stages `assets/` into the temp distribution directory, the worker
core files are already there. A new `serve()` target does the same pre-build step
before calling `trunk serve --no-autoreload`.

**`Trunk.toml`** — the `[[hooks]]` section is removed entirely. Trunk's hook
mechanism is inappropriate for placing files into `dist/` because hooks execute
after staging but before distribution, making their output ephemeral.

**`.gitignore`** — Added `/assets/grandma_worker_core.js` and
`/assets/grandma_worker_core_bg.wasm` since these are now build artifacts living
in a source-tracked directory.

Before (broken):
```python
def build():
    # worker built by Trunk hook into dist/assets/ — gets wiped
    _run("trunk", "build")

def _build_worker(...):
    dist_dir = ROOT / "dist" / "assets"
    ...wasm-bindgen --out-dir dist_dir...
```

After (fixed):
```python
def build():
    _build_worker()          # write to assets/ BEFORE trunk stages
    _run("trunk", "build")   # [[copy-dir]] picks up assets/ including worker core

def _build_worker(...):
    assets_dir = ROOT / "assets"
    ...wasm-bindgen --out-dir assets_dir...
```

## Regression Test

A unit/cargo test cannot exercise a build system ordering issue. The regression
test is a shell verification procedure:

1. `Remove-Item -Recurse -Force dist` (ensure clean state)
2. `python make.py build`
3. `Get-ChildItem dist\assets\` — must list all four files:
   `grandma_quotes.json`, `grandma_worker.js`, `grandma_worker_core.js`,
   `grandma_worker_core_bg.wasm`

This procedure was failing before the fix (only 2 files present) and passes after.

## Post-Mortem / Lessons Learned

### Trunk post_build hooks cannot write to dist/ reliably

Trunk's build pipeline stages everything into a temp directory and then does an
atomic directory-rename onto `dist/`. The `post_build` hook fires *after* staging
but *before* the rename — which means the hook's working directory is the previous
`dist/`, not the soon-to-be `dist/`. Files written to `dist/` by a `post_build`
hook are silently discarded when the rename occurs.

The correct mental model for adding files to Trunk's output is: any file that must
appear in `dist/` must either be (a) produced by the main WASM compilation, or (b)
present in a directory referenced by a `[[copy-dir]]` or `[[copy-file]]` directive
**before** `trunk build` runs. Build artifacts from separate compilation steps
should be placed in a source-tracked (or gitignored) staging directory that
`[[copy-dir]]` references, and that staging step must happen before `trunk` is
invoked — not inside a Trunk hook.

This lesson has been added to `doc/lessons.md` as L17.
