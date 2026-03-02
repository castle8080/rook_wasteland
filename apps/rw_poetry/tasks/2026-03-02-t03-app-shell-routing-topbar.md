# Task: T03 · App Shell, Routing, and Top Bar

## Status
In Progress

## Goal
Wire up the Leptos router with two routes (reader `/` and readings list `/readings`), render the navigation top bar, and apply the base CSS custom properties.

## Scope
- `App` component with `leptos_router::Router` and two `Route` entries
- `TopBar` component: app name left, `Recordings` link right
- Base CSS: custom properties for light mode palette, base typography, body layout
- Navigation top bar present on both routes

**Out of scope:** Dark mode toggle interaction (T14), actual page content in either view (T04, T09).

## Design
Router wraps everything. Routes render placeholder views for now; T04 and T09 will replace them. `TopBar` lives in `ui/components/` and is rendered above the `<Routes>` outlet. Light mode CSS custom properties are set on `:root`; dark mode overrides added in T05.

## Implementation Plan
1. [x] Implement `TopBar` component in `ui/components/mod.rs`
2. [x] Wire `App` with `Router`, `Routes`, and two `Route` entries
3. [x] Add placeholder `ReaderView` and `RecordingsListView` stubs in ui/
4. [x] Add CSS custom properties for light mode + base layout
5. [x] `trunk build` passes

## Testing Plan
- `trunk build` succeeds
- Browser: top bar renders, clicking Recordings navigates to /readings

## Notes / Decisions
- TopBar uses `<A>` from leptos_router for client-side navigation
- CSS custom properties on :root cover light mode; dark mode overrides added fully in T05
