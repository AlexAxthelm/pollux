# Player: Show Notes
---
priority: high
depends: player, episode, episode-chapters
---

A page in the player's paged content area (carousel) showing the episode's
description / show notes.

## MVP State

The page is built. It shows the episode's show notes rendered as rich text (HTML
from the feed, via the same renderer as the episode detail page) in a scrollable
area. If the episode has no show notes the page is not shown, and neither are the
dot indicators. Timestamp detection (below) is not built yet.

## Full Implementation

### Content

- Scrollable text area showing the episode description extracted from the feed
- Rendered as rich text (built); timestamps are not yet links
- If no show notes are available, the page is hidden (dot indicator removed)

### Timestamp Detection

Many episodes embed timestamps in their show notes as plain text
(e.g. `1:23:45 — Interview segment`, `[0:32] Topic intro`). When detected:

- Timestamps are rendered as tappable links
- Tapping jumps playback to that position in the episode
- Timestamps may be extracted as synthetic chapters (see `episode-chapters.md`),
  this happens automatically. the process should be generous in accepting
  formats for timestamps

This is the primary mechanism for shows that don't use embedded chapter formats.

### Interaction with Chapters

- If the episode has proper embedded chapters (see `episode-chapters.md`),
  show notes timestamps are secondary — embedded chapters take precedence
- If only show note timestamps are present, they serve as the chapter source
- If both exist, prefer embedded chapters but surface the conflict for review

## Future

- Chapter list as a separate carousel page
- Link detection and handling in show notes (e.g. tappable URLs)
