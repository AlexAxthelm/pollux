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
- **Feed**: the general abstraction for "an ordered, potentially changing set of
  episodes." Both subscriptions and playlists satisfy this interface. The rest of
  the app talks to feeds, not specifically to subscriptions or playlists. 

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

## Per-Feed Settings

The cascading default pattern applies to all per-feed settings. Each setting
shows whether it is using the global default or a custom value, and can be
reset to default individually.

Known per-feed settings:
- Refresh interval
- Download rules (see `downloads-auto.md`)
- *(more to be defined)*

## UI

### Library View

- List or grid of subscriptions (feed artwork)
- "Refresh all" trigger
- Link to add new subscription

### Subscription Details Page

- Feed artwork, title, description
- Episode list (full, including de-listed episodes behind toggle)
- Per-feed settings
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
