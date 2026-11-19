---
type: feature
status: draft
priority: low
area: player
depends:
  - "[[player]]"
  - "[[bookmarks]]"
---
# Transcripts

> **Not built yet.** The app has no transcript code.

* OPEN QUESTION: Are there any good public repos of podcast transcripts?
* OPEN QUESTION: Use on-device model to transcribe episodes?

## UI:

* in player view, there should be a line or two of text under the main art.
* clicking on the lines expands them to rougly the size of the main art view
(shrinking that)
    * maybe the art goes up into the corner, and there's a fade-out on the text.
* Text should scroll with timestamps, if available. 
* Text should highlight with timestamp, if available:
    * Current line: `Base01`
    * Past lines: `Base00`
    * Future lines: `Base1`

## Open questions

- #needs-decision **Where do the transcript lines sit in the player?** The UI above puts
  them "under the main art" and expands them over the art. [[player]] describes the top of
  the player as a paged carousel (Artwork, then Show notes), with further pages as a
  future item. Both are kept as written.
- #needs-decision **How are the highlight colors named?** The UI above names base16 tokens
  for the current, past and future lines (`Base01`, `Base00`, `Base1`). `Base1` is not a
  base16 token (they run base00 to base0F), and
  [[ADR-0031 Views paint with semantic roles, not base16 tokens or hex]] says views use
  semantic roles, of which none exists for a current line. Kept as written.

## User stories

```base
views:
  - type: table
    name: User Stories
    filters:
      and:
        - file.hasLink(this.file)
        - type == "user-story"
    order:
      - file.name
      - status
    sort:
      - property: file.name
        direction: ASC
```
