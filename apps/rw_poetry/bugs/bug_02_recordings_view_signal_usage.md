# Bug 02 — Signal `.get()` Outside Reactive Context in Audio Player

## Status
Fixed

---

## Summary

Opening the recordings page and playing a recording produces console warnings about signal reads outside a reactive tracking context. The warnings come from `src/ui/audio_player.rs` at the lines where the `NodeRef<html::Audio>` is read inside DOM event handler closures.

---

## Console Warnings Observed

```
At src/ui/audio_player.rs:86:44, you access a
reactive_graph::signal::rw::RwSignal<Option<SendWrapper<HtmlAudioElement>>>
outside a reactive tracking context. This might mean your app is not
responding to changes in signal values in the way you expect.

At src/ui/audio_player.rs:76:44, you access a
reactive_graph::signal::rw::RwSignal<Option<SendWrapper<HtmlAudioElement>>>
outside a reactive tracking context.
```

Both warnings point to `audio_ref.get()` calls inside `wasm_bindgen::Closure::<dyn FnMut()>::new(...)` callbacks registered as `ontimeupdate` and `onloadedmetadata` DOM event handlers.

---

## Root Cause

In Leptos, calling `.get()` on a signal **subscribes** to it — it registers the current reactive owner as a dependent so the owner re-runs when the signal changes. This mechanism only makes sense inside a reactive context: a `move ||` closure in a `view!`, an `Effect::new`, a `Memo::new`, or similar.

Plain Rust closures — DOM event handler callbacks, `wasm_bindgen::Closure::new`, `spawn_local` bodies, and `on:click`/`on:input` handlers — run outside any reactive tracking context. When `.get()` is called there, Leptos has no reactive owner to register the subscription against. It emits the warning to signal that the call is meaningless (no subscription is made) and that the developer may have confused this with a tracked read.

The affected call sites in `audio_player.rs`:

| Line | Context | Signal |
|------|---------|--------|
| 76 | `ontimeupdate` handler (`wasm_bindgen::Closure`) | `audio_ref` |
| 86 | `onloadedmetadata` handler (`wasm_bindgen::Closure`) | `audio_ref` |
| 104 | `on_play_pause` (`on:click` event handler) | `audio_ref` |
| 119 | `on_seek` (`on:input` event handler) | `audio_ref` |

Lines 76 and 86 were the ones reported. Lines 104 and 119 carry the same class of error and were fixed proactively.

In all four cases, the intent is simply to read the current DOM node to call a method on it. No reactive subscription is needed or desired. The correct call is `.get_untracked()`.

---

## Fix

All four `.get()` calls on `audio_ref` inside non-reactive closures were changed to `.get_untracked()`. A comment was added at each site explaining why.

**`src/ui/audio_player.rs`:**

```rust
// Before (all four sites):
if let Some(a) = audio_ref.get() { ... }

// After:
if let Some(a) = audio_ref.get_untracked() { ... }
```

No behaviour change — the value read is identical. The warnings are eliminated and the intent is now explicit.

---

## Similar Issues Checked

A codebase-wide scan for `.get()` calls in non-reactive contexts found no other instances.
All remaining `.get()` calls are either inside `move ||` reactive closures in `view!`, or inside `LocalResource`/`Effect` closures where tracking is intentional and correct.

---

## On Testing: Can These Warnings Be Caught Automatically?

### Why `cargo test` Cannot Catch This

The reactive-context warning is emitted at runtime by the Leptos signal graph, inside the browser's WASM runtime. The native `cargo test` target has no reactive graph and no DOM — these calls never execute there.

### `wasm-pack test --headless`

Running tests in a headless browser via `wasm-pack test --headless --chrome` (or `--firefox`) would execute WASM tests in a real browser context with a real Leptos reactive graph. It would be possible to write a test that:

1. Mounts an `AudioPlayer` component in a test harness
2. Simulates the DOM events (`timeupdate`, `loadedmetadata`, etc.)
3. Asserts that no Leptos warnings were emitted (by intercepting `console.warn`)

However, this has significant setup cost: a `wasm-pack` test harness for Leptos components requires a headless browser, a Trunk-compatible build pipeline, and careful management of async DOM timing. For a small personal app, the overhead outweighs the benefit.

### Practical Detection Strategy

The most reliable approach for this codebase is a **code-review rule**, stated clearly in the Leptos technical reference doc and enforced during every review:

> In any closure that is not a reactive Leptos context — `on:click`, `on:input`, `wasm_bindgen::Closure::new`, `spawn_local` body, plain `FnMut` — always use `.get_untracked()` when reading a signal value. Reserve `.get()` for reactive closures (`move ||` in `view!`, `Effect::new`, `Memo::new`, `LocalResource::new`).

---

## Lessons Learned

### 1. `.get()` vs `.get_untracked()` is about context, not intent

Both calls return the same value. The difference is whether the current reactive owner is notified. In non-reactive contexts there is no owner — `.get()` still works but Leptos warns, because the subscription is silently dropped. Using `.get_untracked()` in non-reactive contexts is not a workaround; it is the semantically correct call.

### 2. DOM event handler closures are never reactive contexts

This is easy to overlook because DOM event handlers look structurally similar to reactive closures — both are `move || ...` or `move |ev| ...`. The distinction: reactive closures are called *by the Leptos runtime* inside a tracking scope; DOM event handlers are called *by the browser* with no Leptos tracking scope active. Any signal read inside a DOM event handler should use `.get_untracked()`.

This applies to:
- `wasm_bindgen::Closure::new(move || ...)`
- `on:click=move |_| ...` and all other `on:*` event handlers
- `spawn_local(async move { ... })`
- `on_cleanup(move || ...)`

### 3. The pattern creates silent misbehaviour, not loud failures

The warning is diagnostic output, not a panic. If ignored or unnoticed (e.g. browser console not open during development), the app continues to function — the signal is read without registering a subscription. This makes the class of bug easy to ship undetected.

### 4. This is the same class of issue as Bug 01

Bug 01 was `.get()` where it *should not* track (inside a `LocalResource` closure that also wrote the signal, causing a feedback loop). This bug is `.get()` where it *cannot* track (outside any reactive context, causing a silent non-subscription). Both stem from not distinguishing "reading a signal value" from "subscribing to a signal". The rule of thumb: *always ask whether you want this closure to re-run when this signal changes. If no — use `get_untracked()`.*
