# Playlists
---
priority: high
---

Playlists are fundamentally a collection of episodes, with some ordering.
We can define them by specifying particular epidodes, and the order in which we
want to play them (manually)
Or they can be generated automatically, based on criteria.

Episodes in a playlist may or may not be available in local storage, and should
be noted as so (as part of episode info)

It would be good to hae a mecahnism to include other playlists in playlists
(meta-playlists). For this I'm thinking a UI with a single (short, single line
of text) header, and then a 1-standard-row "quick view" with things like episode
counts / playtime. Conceptually, this is "podcast as feed", and I expect it will
be useful with rules-based lists.

## Behavior

In general, from a "new playlist" entrypoint, I should be able to easily:
* Add feed
* Add Rule
* Add episodes

All playlist types support:
- Sequential playback through episodes
- Current position tracking
- Pause and resume

### Add Feeds:

The simplest concept: A playlist can have a collection of feeds. All the
episodes from those feeds are added to the playlist (by each episode's default
behavior) and by default, sorted by time.

later complications here include "priority podcasts" 

### Add Episodes:

Adding an episode basically brings up a quick episode finder (search / scroll
through feeds), and when finding the episode you want, you can add to beginning
or end of playlist.

This has a natural followup of re-ordering the episodes in the playlist.

### Add Rule

This defines some transformation to the playlist.
Some example 

Adding a rule to an empty playlist implicity acts on all episodes from all feeds.
Adding a rule to a populated playlist (either from feed subscriptions or from
curated episodes) acts on the episodes in that set

Example rules; 
* Include most recent N episodes (filter)
* Sort by "number of times played" (find favorites)
* Include unplayed episodes only 
* include only downloaded (immediately available) episodes

Effectively, this acts in much the same way as a SQL select, and the "stacking"
of rules is something that the UI should make easy (re-ordering of rules, easy
field selection, etc), along with a live preview of what the list looks like if
saved now. Ideally, this would support an "undo" trigger.

One example "composite playlist" would have a set of podcasts, with the most
recent episode of each, if unplayed, and then after that, the available other
unplayed episodes in the podcasts in natural (chronological) order. This could
be accomplished by having the first set be one playlist, and then the other.

### Episodes

It is plausible for an episode to be in a playlist more than once. If a user has
explicitly added an episode to a playlist, this should be absolutely included.
Otherwise, this should be a toggle "allow duplicate episodes" or something like
that. If disallowed, each episode shows up at the first point in the playlist,
and subsequent deplicate entries are not included.

## UI

### Full view

Top row: Icon / title / stummary stats / (config menu)
Scrollable list of Episodes

### Compact (menu) view

For presentation in menus / selecting:
Icon (alternately, tiled album art).
optionally: number of episodes, playtime
alternately: immediately available starts
Playlist type
