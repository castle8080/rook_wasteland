# Task: T12 · Error Handling Completeness

## Status
In Progress

## Goal
Audit and fill all gaps in user-facing error handling per spec section 11.

## Gap Analysis

| Error Path | Current State | Action |
|---|---|---|
| Index fetch fails | ✅ "Unable to load poems. Check your connection." + Retry | None |
| Poem JSON 404/malformed | ❌ Shows error state, requires manual retry | Retry up to 3x with different poem; inline warning |
| Mic NotAllowedError | ✅ "Microphone access was denied. Open browser settings to allow access." | None |
| Mic NotFoundError | ✅ "No microphone found on this device." | None |
| Mic NotReadableError | ✅ "Could not access the microphone. Another app may be using it." | None |
| MediaRecorder NotSupportedError | ✅ "This browser doesn't support audio recording." | None |
| IDB QuotaExceeded | ✅ "Storage full. Delete some recordings to free space." | None |
| Audio blob missing on playback | ❌ Silently logs warning | Show "Recording data unavailable." in row |
| IDB completely unavailable | ⚠️ Shows error but no explanation | Improve message clarity |

## Changes Planned
1. reader.rs: retry up to 3 different random poems if fetch_poem fails; show inline warning
2. recordings_list.rs: show "Recording data unavailable." when get_audio_blob fails for play
