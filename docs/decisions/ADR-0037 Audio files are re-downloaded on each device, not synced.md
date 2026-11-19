---
type: decision
status: draft
feature:
  - "[[sync]]"
related: []
---
# Audio files are re-downloaded on each device, not synced

## Context

Episode audio is large and the same files are available from the feed. Per-device storage
limits and download rules already differ between devices; see [[sync-downloads]] and
[[storage-sync]]. (Inferred: the sync note says only that episodes "should be re-downloaded
if possible", with no rationale and no decision recorded. Sync is not built.)

## Decision

Sync moves metadata (settings, playback positions, event logs, playlists, bookmarks), not
audio. A device that needs an episode downloads it from the feed again, if that is still
possible.

## Consequences

- An episode that is no longer in the feed cannot be re-downloaded on another device; see
  [[subscription-delisted-episodes]] and [[storage-protection]].
- Which episodes a device downloads is a per-device decision; see [[sync-downloads]].
