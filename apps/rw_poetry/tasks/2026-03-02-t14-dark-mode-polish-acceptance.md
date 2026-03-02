# Task: T14 · Dark Mode Toggle and Final Polish

## Status
In Progress

## Goal
Wire dark mode toggle in TopBar, final accessibility/visual polish, acceptance criteria sign-off.

## Scope
- Dark mode toggle button in TopBar (☀/🌙)
- Read initial preference from LocalStorage key "theme"
- On toggle: update `[data-theme]` on `<html>` element + persist to LocalStorage
- Clean up any debug/placeholder comments in src/
- Run `cargo fmt` and `trunk build --release`

## Notes
- CSS already uses `[data-theme="dark"]` selectors (T05)
- Use `gloo_storage::LocalStorage` for persistence
- Use `web_sys::window().document().document_element()` to set attribute
