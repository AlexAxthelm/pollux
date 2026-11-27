---
type: reference
status: draft
---
# How a playlist works

*A plain-language draft, written for a listener and meant to become the source of the in-app help. It repeats nothing new: every rule here is specified in [[playlists]] and the notes and decisions it links. If this page and a spec disagree, the spec wins and this page is the bug. Parts marked (later) are not in the first version.*

## The idea

A playlist is not a list you fill by hand. It is a **question you ask your library**: "which episodes, from where, in what order?" Every time you open it or reach the end of an episode, Pollux asks the question again, so the list is always up to date.

A playlist has three parts:

1. **Sources**: where episodes come from. Shows, other playlists, or **All shows** (everything you subscribe to, including shows you add later).
2. **Rules**: what to keep and how to order it.
3. **Settings**: how the episodes play (for example skip time).

A playlist with no sources is empty. A playlist with sources and no rules plays every episode of those sources, one source after another.

## Rules are applied top to bottom

Each rule works on what the rules above it left behind. That one idea explains everything else.

**Example: "my 20 newest episodes that are not played".**

| Rule | What is left |
|---|---|
| Sort by age, newest first | everything, newest first |
| Keep only not played | the not-played ones, still newest first |
| First 20 episodes | the 20 newest not played |

Swap the last two and you get something different: the first 20 episodes, of which only the not-played ones are kept, which may be just a few. Each rule row shows how many episodes are left after it, so you can see the difference as you drag.

**Several sorts.** The last sort is the main order. The ones above it only decide ties. If you sort by show and then by age, the list is in age order, and episodes published at exactly the same time stay grouped by show.

**Duplicates.** If an episode comes in from two sources, **Deduplicate** keeps the first one. It starts at the top of a new playlist and you can move it or turn it off.

## What plays next

When an episode ends, Pollux rebuilds the list and plays the episode **after the one that just finished**. You can edit the playlist while it plays; the change shows from the next episode on. If your edit removes the episode that is playing, it finishes, and then playback stops.

Settings for an episode come from where you started playing it. *(In the first version only the playlist you started from, the show and your global settings are read; settings on playlists it includes come later.)*

The full order is: the playlist (and any playlists it includes), then the show, then your global settings, with an override on the episode itself winning over all of them. The playing episode keeps the route it started with. A change to a setting at a level on that route, such as the show's skip time, takes effect straight away; anything that would change the route, such as reordering the sources, waits for the next episode.

## Playlists inside playlists

A playlist can use other playlists as sources. If playlists would include each other in a circle, Pollux leaves out the playlist it is already inside, so resolution never runs forever, though an episode that arrives by two routes can appear twice if Deduplicate is off. The editor asks you before you make a loop (later).

## Sorting by hand (later)

You can send an episode to the top or the end of a playlist. The playlist then gets a **Manual ordering** row among its rules. Keep it at the bottom and your pins hold. Move it up, and the sorts below it will reorder them.

## Adding an episode by hand (later)

"Add to playlist" puts a single episode into that playlist. The playlist's rules still apply to it, so a rule can hide an episode you added; the editor tells you when that happens.

## Where to read more

- The full picture: [[playlists]], [[playlists-rules]], [[playlists-editor]], [[playlists-playback]].
- Why: [[ADR-0027 A playlist is a query over sources]], [[ADR-0047 A playlist's rules are one ordered stack of operations on the list]], [[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]].
