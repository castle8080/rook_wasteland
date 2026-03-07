
# Bug - Console Error On Mouse Move

When I first load the app and move the mouse over rendered page. I see errors. As the mouse moves, the errors keep going every time I move the mouse on the screen.


Unable to preventDefault inside passive event listener invocation.

__wbg_preventDefault_25a229bfe5c510f8	@	rw_teleidoscope-ccdbf19ad4f7c244.js:854
$web_sys::features::gen_Event::Event::prevent_default::__wbg_preventDefault_25a229bfe5c510f8::h75346413f71a608b externref shim	@	rw_teleidoscope-ccdb…44_bg.wasm:0x154218
$web_sys::features::gen_Event::Event::prevent_default::h6bcfef4c738269f2	@	rw_teleidoscope-ccdb…44_bg.wasm:0x1273e8
$rw_teleidoscope::components::canvas_view::__component_canvas_view::{{closure}}::{{closure}}::hbb47b2f8d3eff9f1	@	rw_teleidoscope-ccdb…244_bg.wasm:0xac99f
$wasm_bindgen::convert::closures::_::_::invoke::{{closure}}::h0055f192d8e0711e	@	rw_teleidoscope-ccdb…44_bg.wasm:0x112b4d
$<core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once::h2e231a21fff0cc66	@	rw_teleidoscope-ccdb…44_bg.wasm:0x119b29
$wasm_bindgen::__rt::maybe_catch_unwind::h92392eaa1ad449bf	@	rw_teleidoscope-ccdb…44_bg.wasm:0x153e2f
$wasm_bindgen::convert::closures::_::_::invoke::h87289fbd0667652f	@	rw_teleidoscope-ccdb…244_bg.wasm:0xc9c00
$wasm_bindgen::convert::closures::_::_::invoke::h87289fbd0667652f externref shim	@	rw_teleidoscope-ccdb…44_bg.wasm:0x13c11c
wasm_bindgen__convert__closures________invoke__h87289fbd0667652f	@	rw_teleidoscope-ccdbf19ad4f7c244.js:1190
real	@	rw_teleidoscope-ccdbf19ad4f7c244.js:1435


## Instructions

* Assess the bug quickly with potential causes
* Research the code to determine likely cause
* If requried you may ask questions during initial investigation as well
* After the cuase is likely determined, determine if a test can be written first to demonstrate the tests fails
* Fix the bug in code and continue tests which should then pass
* Run a code review of tests
* Update the bug report with root cause and a description of the first
* Consider and write learned lessons in the bug report and if the lessons is generally applicable update the doc/lessons.md

---

## Root Cause

`gloo_events::EventListenerOptions` defaults to `passive: true`. Every listener
created with `EventListener::new()` is therefore passive, and calling
`ev.prevent_default()` inside a passive listener is forbidden by the browser —
it logs a warning and silently ignores the call.

The `pointermove` handler called `ev.prevent_default()` **unconditionally** (before
the `buttons() & 1 != 0` drag check), so the warning fired on every mouse movement
over the canvas, even with no button held.

The `dragover`, `drop`, and `pointerdown` handlers were also passive, which would
have broken drag-and-drop and text-selection prevention as well (just less visibly).

## Fix

`src/components/canvas_view.rs`:

1. Import `EventListenerOptions` from `gloo_events`.
2. Create `let opts = EventListenerOptions::enable_prevent_default();` once (sets
   `passive: false`).
3. Switch all four affected listeners (`dragover`, `drop`, `pointerdown`,
   `pointermove`) from `EventListener::new(...)` to
   `EventListener::new_with_options(..., opts, ...)`.
4. In the `pointermove` handler, move `ev.prevent_default()` **inside** the
   `if ptr.buttons() & 1 != 0` block — only suppress touchscreen scroll during an
   active drag, not on every mouse move.

## Lessons

### `gloo_events` defaults to `passive: true` — always use `enable_prevent_default()` when calling `prevent_default()`

Any `EventListener::new(...)` call in this codebase creates a **passive** listener.
Calling `event.prevent_default()` inside a passive listener silently fails and logs
a browser console warning. For any listener that needs to call `prevent_default()`,
use:

```rust
let opts = EventListenerOptions::enable_prevent_default();
EventListener::new_with_options(target, event_type, opts, |ev| { ... });
```

Affected events in this app: `dragover`, `drop`, `pointerdown`, `pointermove`.

### Only call `prevent_default()` when the action actually needs it

`pointermove.prevent_default()` is only meaningful during an active drag (when the
primary mouse button is held). Calling it unconditionally fires on every cursor
movement and may interfere with normal browser scroll/selection behaviour. Gate it
behind the condition that requires it.
