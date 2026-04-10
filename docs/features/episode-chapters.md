# Chapters
---
priority: high
depends: episode, player
---

Chapter support is a progressive enhancement. Episodes without chapters behave
normally; the UI adapts when chapter data is present. MVP includes chapter
detection and navigation controls — chapter art and embedded timestamp detection
in show notes are post-MVP.

## Data

Chapters may be embedded in the audio file or provided via the feed's RSS
namespace. Known standards:

- **ID3 chapters** (MP3 files) — `CHAP` frames in ID3v2 tags
- **QuickTime/AAC chapters** — embedded in AAC/M4A container
- **Podcasting 2.0 namespace** — `<podcast:chapters>` pointing to a JSON file

The app should attempt to parse all three. If multiple sources are present,
prefer the most complete. This is an implementation decision, but the product
behavior is the same regardless of source.

if Podcasting 2.0 JSON chapters are unavailable (cannot be fetched), then retry
(in background) but proceed as if they do not exist. If they can be fetched
later, update

## Episode Behavior

When chapters are present:

- Displayed as a navigable list in the episode detail view
- Chapter title and timestamp shown per entry
- Tapping a chapter in the list jumps to that position

## Player Behavior

When chapters are present:

- **Chapter back / chapter forward** controls appear in the player layout,
  replacing the plain title row (see `player.md`)
- Chapter navigation is preferred over time-skip when chapters are available
- Current chapter title shown in the title area

## Bookmarks

A bookmark created while within a chapter should record and display the chapter
title in the bookmark summary (see `bookmarks.md`).

## Chapter Art

*(Post-MVP)* Some chapter formats support per-chapter artwork. When present,
the artwork carousel page should display the chapter art rather than the episode
or feed art.

## Embedded Timestamps in Show Notes

*(Post-MVP)* Some episodes include timestamps in their show notes (e.g.
`1:23:45 - Interview with guest`) as a manual substitute for chapters. These
should be detected and rendered as tappable jump targets. When present, they may
also be extractable as synthetic chapters.

See `player-show-notes.md` for show notes implementation.

## Future

- On-device model to suggest chapter markers for episodes that have none
- Scrubber chapter markers (see `player.md`)
