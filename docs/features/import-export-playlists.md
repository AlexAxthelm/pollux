---
type: feature
status: accepted
priority: mid
area: subscription
parent: "[[import-export]]"
depends:
  - "[[playlists]]"
  - "[[import-export-opml]]"
---
# Detailed export and sharing

> **Not built yet.** There is no export code in the app.

We would also look to have a more detailed export functionality that includes OPML plus some other information around actual application-specific settings — so that I would be able to share like a playlist, some smart playlist where it's like "all of the podcasts from these three feeds that have been released since 2024" or whatever, but that doesn't necessarily need to be OPML

- #needs-decision **What format is the detailed export?** The text above says it
  "doesn't necessarily need to be OPML", and nothing else decides it.

## What a shared playlist carries

A playlist is sources plus one ordered rule stack ([[ADR-0027 A playlist is a query over sources]], [[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]), so that is what an export has to hold, along with its settings and episode limit. The hand-picked list and pins refer to episodes the receiver may not have, so whether they travel is open. An import can also bring a loop, or a dense cluster of playlists, past the editor's confirmation. Building import is the point at which to decide the resolution work budget and to mark loops ([[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]).

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
