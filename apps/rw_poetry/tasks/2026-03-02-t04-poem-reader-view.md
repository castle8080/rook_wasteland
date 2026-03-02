# Task: T04 · Poem Reader View

## Status
In Progress

## Goal
Implement the full poem reader home view — load a random poem on arrival, display it with correct typography, handle loading/error states, and provide the New Poem button.

## Scope
- `ReaderView` component with `LocalResource` for index + poem fetches
- Render: title (h1), author + date (secondary), poem body (pre-wrap serif)
- Loading state: "Loading poem…" placeholder
- Error state: error message + "Try again" button
- `New Poem` button picks a new random poem excluding current
- Placeholder div for recording controls (filled in T08)

**Out of scope:** Recording controls (T08).

## Design
Two-step fetch: first load index via LocalResource keyed on a `refresh` counter signal.
After index loads, use `pick_random` to select a poem, then fetch its detail.
`New Poem` increments `refresh` counter and updates `exclude_id` signal.

Signals:
- `refresh: RwSignal<u32>` — incrementing triggers re-fetch of index+poem
- `current_poem_id: RwSignal<Option<String>>` — current poem id for exclude logic

## Implementation Plan
1. [x] Replace placeholder ReaderView with full implementation
2. [x] LocalResource for two-step fetch (index → poem)
3. [x] Render poem content with correct CSS classes
4. [x] Loading, error states, New Poem button
5. [x] Remove #![allow(dead_code)] from poem_repository now it's consumed
6. [x] cargo clippy + trunk build pass

## Testing Plan
- `trunk build` passes
- Browser: poem loads on open, New Poem loads a different poem, loading state visible

## Notes / Decisions
- Using LocalResource (not Resource) since fetch is !Send in WASM
- New Poem increments a counter signal to re-trigger the resource
