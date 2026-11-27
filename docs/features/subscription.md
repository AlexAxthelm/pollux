---
type: feature
status: accepted
priority: MVP
area: subscription
depends:
  - "[[episode]]"
  - "[[storage]]"
  - "[[settings]]"
---
# Feed Subscription & Library

A subscription is a mapping between the app and an RSS/Atom/JSONFeed URI. It is
the primary way users add content to Pollux.

## Terminology Note (Internal)

- **Subscription**: a podcast/show, represented by a feed URI that the app polls
  for updates. 1:1 with an RSS/Atom/JSONFeed file.
- **EpisodeSource**: the internal abstraction for "an ordered, potentially changing set of
  episodes", which both subscriptions and playlists satisfy; the rest of the app talks to it,
  not specifically to subscriptions or playlists ([[DATA_MODEL]]). "Feed" is reserved for the
  feed document ([[GLOSSARY]]).

These terms are internal. User-facing language should use "podcast" or "show"
for subscriptions, and "playlist" for user-created feeds.

There are no limits on the number of subscriptions (though in practice space
limitations may impose one). Internal note: the metadata database should have
higher priority in storage than content files


## Parts

- [[subscription-subscribe]]: subscribing, preview, entering a feed, failures
- [[subscription-refresh]]: how and when feeds are refreshed
- [[subscription-delisted-episodes]]: episodes that drop out of a feed
- [[subscription-transport-security]]: http and https feeds
- [[subscription-auth]]: authenticated feeds
- [[subscription-inactive]]: shows that have stopped publishing
- [[subscription-unsubscribe]] and [[subscription-soft-delete]]: removing a subscription

## Unsubscribing

see `subscription-soft-delete.md` for details

for MVP: if a subscription is removed, all assosciated content files are also
deleted.

## Subscription settings

Reached from a gear icon on the subscription's page; see [[settings-cascade]]. The cascading
default pattern applies to all subscription settings. Each setting
shows whether it is using the global default or a custom value, and can be
reset to default individually.

Known subscription settings:
- Refresh interval
- Download rules (see `downloads-auto.md`)
- *(more to be defined)*

*As built:* the page ("Podcast settings") has the three playback settings from [[settings]]:
skip forward, skip back and rewind on resume. A setting the subscription hasn't set follows the
global one and reads "(inherited from library)"; one it has set offers a reset to the global
value. Refresh interval and download rules are not subscription settings yet.

## UI

### Library View

- List or grid of subscriptions (feed artwork)
- "Refresh all" trigger
- Link to add new subscription

### Subscription Details Page

- Feed artwork, title, description
- Episode list (full, including de-listed episodes behind toggle)
- The Sort menu is the order of the show's implicit playlist: it decides what plays next when the show is the active context, and it does not apply when the show is a source of another playlist ([[ADR-0028 A subscription is an implicit single-feed playlist]]).
- Gear icon to the subscription's settings. *As built:* next to the Sort button.
- Unsubscribe (in menu, protected)

The subscription preview shown before subscribing is in [[subscription-subscribe]].

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
