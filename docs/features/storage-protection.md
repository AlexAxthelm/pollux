---
type: feature
status: accepted
priority: mid
area: storage
parent: "[[storage]]"
depends:
  - "[[episode]]"
  - "[[subscription]]"
  - "[[bookmarks]]"
---
# Episode protection

Hard rules that keep certain episodes safe from automatic cleanup. These
interact with [[storage-eviction]] and retention rules, but since they protect
data, should generally be considered more important than other rules.

Protection comes from two places: the user can protect an episode explicitly, and
Pollux protects some episodes automatically so the user doesn't have to guard
against permanent data loss themselves.

## Flagged and starred (manual)

The user can flag / star an episode as safe from eviction, so it will always be
available. Flagged / starred episodes are never automatically deleted. (Flags
are described in `episode.md`.)

## Automatic protection

Episodes that meet these conditions are marked as protected without the user
doing anything:

### Irrecoverable files

Episodes that can't be fetched again, because the copy in local storage may be
the only one left, are protected so the user doesn't have to guard against
permanent data loss.

- **Removed from feed** — episodes removed from a feed's RSS but still in local
  storage are flagged as "Removed from feed" and protected from automatic
  cleanup.
- Other cases where the source goes away are covered in
  `subscription-soft-delete.md` and `playlists-archive.md`.

### Favorites and bookmarks

Episodes the user has favorited or bookmarked (see [[bookmarks]]) are
auto-marked as protected, so the user doesn't have to protect each one by hand to
guard against the file becoming unavailable.

## Open questions

- Can the app check server availability of an episode without downloading it?
  (Needed to distinguish "removed from feed but still on server" from "gone
  entirely", and "listed in feed, but missing file" — relevant for archive
  protection.)
- Does "favorite" mean the same as flag / star, or is it a separate marker? (See
  `episode.md`.)
- Should the user be able to un-protect an auto-protected episode, and if so, how
  is that shown?

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
