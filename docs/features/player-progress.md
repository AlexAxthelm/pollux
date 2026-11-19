---
type: feature
status: accepted
priority: MVP
area: player
parent: "[[player]]"
depends:
  - "[[player]]"
  - "[[episode]]"
  - "[[settings]]"
---
# Playback progress and resume

How Pollux remembers where the listener is, decides when an episode is finished,
and brings playback back after the app closes.

## Resume position

If the user leaves and comes back, playback resumes approximately where they left
off. A small rewind on resume (e.g. 3-5s) is acceptable and common practice for
context overlap. The user can disable that behavior (see `settings.md`).

Today the rewind is 3s on any resume after a pause, on cold-start restore, and when
resuming a saved position (clamped at 0), and is hard-coded until Settings exists.

## Saving position

Position is written at most every 10s while playing, and on pause, seek,
backgrounding, switching episode, and end. Per-second ticks are transient.

## When an episode counts as played

Played is decided when the listener *leaves* an episode, never as a side effect of
saving progress: when the engine reaches the end, when they pause, or when they
start another episode, while within **15s** of the end (never more than a quarter
of a short episode). The episode becomes played, its position resets to 0, and the
player goes inactive. The 15s will become a user setting
([[US-0155 Set played threshold]]).

Saving progress and backgrounding never finish an episode, so listening on through
the last seconds with the screen locked is never cut short. A system pause (a call,
Siri, unplugged headphones) keeps the episode and its place.

## Restore on launch

The active playlist and current position are restored on cold start, so the
mini-player is visible immediately if something was playing when the app closed.

- An episode already marked played is not restored.
- An in-progress episode comes back paused at its place.
- Saving the play context is retried until storage confirms it, up to 5 attempts.
  If it never succeeds, a relaunch may not restore that episode.

## End of episode

There is no auto-advance yet. At the end of an episode it is marked played and the
player goes inactive (the mini-player disappears). "Next episode" arrives with
playlists.

## Unsubscribing from the playing show

The player has to stop and clear if its episode's show is removed. See "Effect on
playback" in [[subscription-unsubscribe]].

## Decisions

- [[ADR-0002 Played is decided when the listener leaves an episode]]
- [[ADR-0005 Position persistence cadence and resume rewind]]
- [[ADR-0006 No auto-advance until playlists exist]]
- [[ADR-0010 The engine's duration beats the feed's]]
- [[ADR-0012 The play context is restored on launch with capped save retries]]

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
