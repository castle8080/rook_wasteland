# Bug: Export Empty

I loaded images 3 times and changed them. I exported 2 as png and 1 as jpeg. All images were empty. The pngs showed up as empty and clear (like see through alpha channel). The jpeg was all black.


I listed the files and looked at their checksums. You can see the pngs for 2 different sources exported the same.

bryan@corvid MINGW64 ~/Downloads
$ ls -l te*.png te*.jpeg
-rw-r--r-- 1 bryan 197609  4508 Mar  6 17:02 teleidoscope-6m-20260306.jpeg
-rw-r--r-- 1 bryan 197609 15755 Mar  6 16:58 teleidoscope-6m-20260306.png
-rw-r--r-- 1 bryan 197609 15755 Mar  6 16:46 teleidoscope-8m-20260306.png

bryan@corvid MINGW64 ~/Downloads
$ md5sum te*.png te*.jpeg
8c94c8656b4dae2aa194c15620d4fd8c *teleidoscope-6m-20260306.png
8c94c8656b4dae2aa194c15620d4fd8c *teleidoscope-8m-20260306.png
07df076c06ee39a9256977938fceab22 *teleidoscope-6m-20260306.jpeg


I loaded a png and converted with gimp to a bitmap and you can see the hex output how it is empty.


bryan@corvid MINGW64 ~/Downloads
$ xxd teleidoscope-6m-20260306.bmp | head -20
00000000: 424d 8a10 2700 0000 0000 8a00 0000 7c00  BM..'.........|.
00000010: 0000 2003 0000 2003 0000 0100 2000 0300  .. ... ..... ...
00000020: 0000 0010 2700 130b 0000 130b 0000 0000  ....'...........
00000030: 0000 0000 0000 0000 ff00 00ff 0000 ff00  ................
00000040: 0000 0000 00ff 4247 5273 0000 0000 0000  ......BGRs......
00000050: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000060: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000070: 0000 0000 0000 0000 0000 0200 0000 0000  ................
00000080: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000090: 0000 0000 0000 0000 0000 0000 0000 0000  ................
000000a0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
000000b0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
000000c0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
000000d0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
000000e0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
000000f0: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000100: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000110: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000120: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000130: 0000 0000 0000 0000 0000 0000 0000 0000  ................





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

`renderer/context.rs` called `canvas.get_context("webgl2")` with no options.
By default the WebGL spec sets `preserveDrawingBuffer: false`, meaning the
browser is free to clear the drawing buffer immediately after compositing each
frame.

`canvas.toBlob()` is asynchronous — it fires in a future event-loop tick (after
the current JS task has completed and the browser has had the opportunity to
composite and clear the buffer).  By the time the callback runs the buffer is
already empty:

- **PNG**: all pixels transparent (RGBA 0,0,0,0) → see-through image
- **JPEG**: JPEG has no alpha channel, transparent maps to black → solid black
- **Same MD5 across different sources**: both exports captured the same cleared
  state, so they're byte-for-byte identical regardless of what was on-screen

## Fix

`src/renderer/context.rs` — replace `canvas.get_context("webgl2")` with
`canvas.get_context_with_context_options("webgl2", opts)` where `opts` is a JS
object `{ preserveDrawingBuffer: true }` built with `js_sys::Object` +
`js_sys::Reflect::set`.

Added `"WebGlContextAttributes"` to the web-sys feature list in `Cargo.toml` to
enable reading back context attributes in the test.

## Regression Test

`src/renderer/context::tests::context_has_preserve_drawing_buffer` — creates a
canvas element, calls `get_context()`, re-fetches the same context from the
canvas, reads the `preserveDrawingBuffer` attribute via `js_sys::Reflect::get`,
and asserts it is `true`.

## Lessons

**Performance note**: `preserveDrawingBuffer: true` has a small GPU cost (the
driver must keep the buffer intact between frames and may need an extra copy on
some hardware).  For an 800×800 interactive app this is negligible, but avoid
it in high-framerate game-style renderers.

**web-sys 0.3.91 `WebGlContextAttributes` API quirk**: the setter is
`preserve_drawing_buffer(val: bool)` (builder-style); there is no direct Rust
getter.  Read the property back from the returned object via
`js_sys::Reflect::get(attrs.as_ref(), &"preserveDrawingBuffer".into())` and
call `.as_bool()` on the result.

