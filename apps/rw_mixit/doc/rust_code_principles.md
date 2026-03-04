# rw_mixit — Rust Code Principles

> Coding standards for production-quality Rust in this codebase.  
> These apply equally to native and WebAssembly targets — WASM does not change
> what constitutes good Rust.

---

## 1. Error Handling

This is the most important section. Rust's type system distinguishes two kinds
of failures. Treat them differently.

### 1.1 Unrecoverable programming errors — `expect` / `panic`

Use `.expect("reason")` only when a failure is a **bug in this program** —
something that should never happen given correct code, and for which there is no
sensible recovery path.

```rust
// OK: logic invariant — if this is None, the code above is wrong
let peaks = state.waveform_peaks.get();
let first = peaks.first().expect("peaks must be non-empty after load");
```

**Rules for `.expect()`:**
- The message must explain **why this cannot fail**, not just what failed.
  `"create_gain pre_gain"` is not a reason; `"AudioContext guarantees gain node
  creation succeeds"` is.
- Never use `.expect()` on a failure that can arise from user input, a missing
  file, a network call, or a browser permission denial.
- Prefer `.expect()` over `.unwrap()` everywhere — the message is free and
  invaluable when debugging a panic in production.
- `.unwrap()` is acceptable only inside `#[test]` functions.

### 1.2 Recoverable runtime errors — `Result` and `?`

Any operation that can fail for reasons outside the program's control must
return `Result<T, E>` and be propagated or handled explicitly.

```rust
// Bad — silently panics if the user's file is unreadable
let data = std::fs::read(path).unwrap();

// Good — caller decides how to handle the failure
fn load_bytes(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let data = std::fs::read(path)?;
    Ok(data)
}
```

**Use the `?` operator** to propagate errors up the call stack. Reserve
explicit `match` or `.map_err` for cases where you need to translate or enrich
the error before returning it.

### 1.3 Web Audio API nodes and `JsValue` errors

Most `web-sys` audio node creation methods return `Result<Node, JsValue>`.
Per the Web Audio API specification, node creation on a live `AudioContext`
does not fail in any supported browser. `.expect()` is acceptable here —
but the message must cite the spec:

```rust
// Acceptable — spec guarantees this succeeds on a live AudioContext
let gain = ctx.create_gain()
    .expect("AudioContext.createGain() is infallible per Web Audio spec");
```

For operations that **can** fail (user file loading, `decodeAudioData`,
`AudioContext.resume()`), handle the `JsValue` error explicitly and
surface it to the user or log it — never `.expect()`.

### 1.4 `Option` handling

| Pattern | Use when |
|---|---|
| `.expect("reason")` | `None` is a programming error |
| `if let Some(x) = opt { ... }` | `None` is an expected case |
| `.unwrap_or(default)` / `.unwrap_or_else(\|\| ...)` | `None` has a sensible fallback |
| `?` (in functions returning `Option`) | Propagate absence up the call stack |

Never use `.unwrap()` on `Option` in non-test code. It reads as
"I didn't think about this."

### 1.5 Error logging in WASM

When you catch a recoverable error and cannot propagate it further (e.g., inside
a `spawn_local` callback), log it to the browser console instead of panicking:

```rust
use web_sys::console;

match do_fallible_thing() {
    Ok(val) => handle(val),
    Err(e) => console::error_1(&format!("Failed to do thing: {:?}", e).into()),
}
```

A panic in WASM terminates the module. The `console_error_panic_hook` crate
will print the panic message, but the app is dead. Prefer surfacing errors
gracefully wherever possible.

---

## 2. Types and Data Modelling

### 2.1 Make illegal states unrepresentable

Use enums and newtype wrappers to encode invariants in the type system rather
than validating them at runtime.

```rust
// Bad — any f32 is accepted; callers must remember the range
fn set_volume(v: f32) { ... }

// Better — the type carries the constraint
struct Volume(f32); // enforced in the constructor, not callers
impl Volume {
    pub fn new(v: f32) -> Option<Volume> {
        if (0.0..=1.0).contains(&v) { Some(Volume(v)) } else { None }
    }
    pub fn get(self) -> f32 { self.0 }
}
```

For this codebase this applies particularly to BPM ranges, loop points
(`loop_in < loop_out`), and playback rates.

### 2.2 Prefer owned types in public APIs

Pass `String` rather than `&str` when ownership will be taken anyway. Avoid
gratuitous `clone()` by designing APIs that take ownership up front.

### 2.3 Newtype for units

Avoid bare `f64` for time and `f32` for gain when they mean different things.
A `Seconds(f64)` and a `NormalisedGain(f32)` are not interchangeable and the
compiler will tell you so.

---

## 3. Panics and Unsafe

- **No `unsafe` blocks** unless interfacing with a C FFI that has no safe
  wrapper. WASM FFI goes through `wasm-bindgen`; that boundary is already safe.
- **No `todo!()` or `unimplemented!()` in committed code.** Use a `//TODO:`
  comment and return an `Option::None` or an early `Err` instead.
- **`panic!()` is for invariants, not control flow.** If you find yourself
  writing `panic!("should not reach here")`, model the type so it's
  structurally unreachable.

---

## 4. Closures and Lifetimes

### 4.1 Capture minimally

Capture only what a closure needs. If a closure needs one field from a struct,
clone or copy that field before the closure rather than capturing `self`.

```rust
// Bad — closure captures the whole state to use one field
let f = move || state.is_playing.get();

// Good — capture only the signal
let is_playing = state.is_playing;
let f = move || is_playing.get();
```

### 4.2 `Rc<RefCell<T>>` for shared mutable state in single-threaded WASM

WASM runs on a single thread. `Arc<Mutex<T>>` adds unnecessary overhead.
Use `Rc<RefCell<T>>` for shared mutable objects (audio nodes, tap-BPM buffers).
Document the sharing site.

### 4.3 Leptos signals are the reactive layer

Do not put `Rc<RefCell<T>>` inside a `RwSignal`. Signals are the source of
truth for UI-visible state. Audio node handles belong outside the signal system
in `Rc<RefCell<Option<AudioDeck>>>`.

---

## 5. Naming and Style

- Follow Rust naming conventions: `snake_case` functions/variables,
  `PascalCase` types/traits, `SCREAMING_SNAKE_CASE` constants.
- Prefer clarity over brevity in identifiers. `channel_gain_node` is better
  than `cgn`.
- One concept per function. If a function needs a paragraph of prose to
  describe what it does, split it.
- Keep functions under ~40 lines. Beyond that, look for sub-functions to
  extract.

---

## 6. Testing

- **Pure functions must have unit tests.** BPM math, crossfader gain formulas,
  pitch-to-rate conversion, loop point calculations — none of these need a
  browser. Test them with `cargo test` (the `rlib` target).
- **WASM integration tests** (node construction, AudioParam updates) go in
  `tests/` and run with `wasm-pack test --headless --chrome`.
- Do not test implementation details. Test observable behaviour:
  inputs → outputs, signals → side effects.
- `.unwrap()` is fine inside `#[test]` functions — a panic is a test failure.

---

## 7. Clippy and Warnings

The project compiles with `deny(warnings)`. Treat every Clippy lint as a
design signal, not a style nit:

- `clippy::unwrap_used` — turn it on; it surfaces every unreviewed `.unwrap()`.
- `clippy::expect_used` — consider turning it on per-module to audit `.expect()` density.
- `clippy::too_many_arguments` — if suppressed, refactor arguments into a
  struct instead of ignoring the lint indefinitely.
- Do not suppress lints with `#[allow(...)]` without a comment explaining why
  the suppression is correct.

---

## Summary — Decision Tree for Failures

```
Can this failure happen because of a bug in our code?
  └─ Yes → .expect("why the invariant holds") or restructure types
  └─ No (user input, external API, browser state)
       └─ Is there a sensible fallback?
            └─ Yes → .unwrap_or / .unwrap_or_else / if let
            └─ No  → return Err(...) and propagate with ?
                     or log to console and degrade gracefully in callbacks
```

The goal: a panic in production means **we made a mistake**. A gracefully
handled error means **something external went wrong and we dealt with it**.
