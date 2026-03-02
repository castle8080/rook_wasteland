# Task: T13 · Accessibility Pass

## Status
In Progress

## Goal
Audit the full app for accessibility: keyboard navigability, aria-labels, semantic HTML, focus styles, color contrast, aria-live for timer.

## Audit Checklist

| Item | Component | Current State | Action |
|---|---|---|---|
| aria-label on icon buttons | recordings_list ▶ ↓ | ✅ aria-label set | None |
| aria-label on audio player play/pause | audio_player | ✅ dynamic aria-label | None |
| aria-label on seek bar | audio_player | ✅ aria-label="seek" | None |
| aria-label on Record/Stop buttons | recording_controls | ❌ text-only buttons, no aria-label | Add aria-label |
| aria-live on elapsed timer | recording_controls | ✅ aria-live="polite" | None |
| Semantic <main> | reader, recordings_list, recording_detail | ✅ | None |
| <nav> for back links | recording_detail | ✅ | None |
| role="list" on recordings ul | recordings_list | ✅ | None |
| lang="en" on poem content | reader | ✅ main has lang="en" | None |
| Focus styles visible | CSS | Need to check | Verify :focus-visible in CSS |
| TopBar nav semantics | components/mod.rs | Check if <nav> wraps links | Add <nav> |
| h1 hierarchy | all views | Check | Verify |
| WCAG AA contrast | CSS | Already designed with 4.5:1+ | Note in doc |

## Changes
1. Add `<nav>` wrapper to TopBar links
2. Add `<html lang="en">` to index.html  
3. Verify focus-visible CSS exists; add if missing
