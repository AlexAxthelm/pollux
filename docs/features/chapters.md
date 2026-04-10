# Chapters
---
priority: high
---

## Episodes

Some episodes include chapter markers embedded in the audio file. When present,
chapters should be:

- **Displayed** in the detail view as a navigable list
- **Navigable** from the player via chapter back / chapter forward controls
  (preferred over time-skip when chapters are available)
- **Visible in bookmarks** — a bookmark created within a chapter should display
  the chapter title in the bookmark summary (see `bookmarks.md`)

Chapter support should be treated as a progressive enhancement: episodes without
chapters behave normally, and the UI adapts when chapter data is present.

*(Future)* Consider using an on-device model to suggest chapter markers for
episodes that do not have them natively.


## Player

When an episode has chapters, chapter back / chapter forward controls are shown
in line with the title area. These are preferred over time-skip when chapters
are available.
