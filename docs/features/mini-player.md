# Mini Player
---
priority: important
depends: player
---

The mini-player is a persistent, compact playback control visible whenever an
episode is active (playing or paused) and the full player view is not open.

## Visibility

- Shown whenever there is an active episode (playing or paused)
- Hidden entirely when nothing is active — no placeholder state
- Not shown when the full player is open

## Layout

Full-width bar, anchored to the bottom of the screen (above any system nav):

```
[ Art ] [ Episode title / Feed title ]  [ From: source ] [ ▶ / ⏸ ]
[────────────────── progress strip ──────────────────────────────]
```

- **Episode art**: small thumbnail
- **Episode title / Feed title**: truncated as needed
- **From: source**: display-only, shows active playlist or subscription name
- **Play/Pause button**: primary action
- **Progress strip**: thin strip at the bottom edge, not a touch target —
  visual only

## Touch Targets

Two zones only:
- **Play/Pause button**: toggles playback
- **Everything else**: opens the full player view

No other tappable elements. Keeps accidental navigation to a minimum.

## Transitions

*(Nice-to-have, not MVP)* When expanding to full player or collapsing back,
a transition showing the mini-player as a contained version of the full player
would be a polished touch. Similarly, when a new episode starts from a list row,
the row could animate into the player as an "open" affordance.

## Open Questions

- On tablet/desktop layouts, should the mini-player be a sidebar or bottom bar?

## Large Screen Layout

For MVP, large screen layouts are a scaled-up version of the mobile layout
(bottom bar, full width). Later, this will likely become a width-constrained
component, but the interaction model remains the same.
