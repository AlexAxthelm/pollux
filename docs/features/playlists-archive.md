# Archive Binge Playlists
---
priority: high
depends: playlists, playlist-rules
---

Archive binge is a primary use case that distinguishes Pollux from conventional podcast clients. It is not a separate feature, but a specific configuration of playlist rules that the app should make easy to discover and set up.

## The Problem

Many podcasts have deep, evergreen archives. Conventional clients are optimized for "latest episode only" consumption. Users who want to systematically work through a back catalog — while also keeping up with new episodes — have no good tooling for this.

Additionally, when binging multiple feeds simultaneously, episodes from the oldest-progress feed tend to clump at the front of the queue (because their archive dates are earlier). This makes multi-feed binge listening feel unbalanced.

## The Solution

A composite playlist built from two sub-playlists:

**"Latest" sub-playlist** (per feed):
- Source: selected feed(s)
- Filter: most recent 1 episode, unplayed
- Sort: by age (desc)

**"Archive" sub-playlist** (per feed):
- Source: selected feed(s)
- Filter: unplayed, oldest-first
- Sort: by age (asc)

**Composite playlist:**
- Sources: Latest playlist, then Archive playlist
- Sort: by source (sequential) — Latest episodes always precede archive episodes

This ensures new episodes surface immediately, and archive listening resumes naturally after.

## Clumping Problem (Later)

When multiple feeds are included and their archives are at different points in history, earlier-archive feeds dominate the front of the queue. Future sort strategies to address this:

- **Round-robin**: alternate episodes between feeds
- **Proportional**: feeds with more unplayed episodes get proportionally more slots
- **Date-normalized**: treat each feed's archive as unit length [0,1] and interleave by normalized position

For MVP, sequential sort by source is acceptable. Document this as a known limitation.

## UI Suggestion

Archive binge should be surfaced as a suggested playlist from a "create playlist" flow. The underlying rules should be visible and editable, but the template should make setup a one-tap action.

## Example

User subscribes to three music podcasts. Two have archives from 2016, one from 2018. They want to hear new episodes weekly and work through all archives.

Desired behavior:
1. Any new episode this week plays first
2. Archive episodes fill the rest of the queue
3. Queue does not clump all 2016-era episodes together (later)

This is achievable with MVP rules for steps 1–2. Step 3 requires an interleaved sort strategy.
