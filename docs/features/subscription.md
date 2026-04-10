# Feed Subscription & Library
---
priority: MVP
---

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

## Subscribing

The user provides a URI. The app fetches the feed file and presents a
**subscription preview**:

- Feed title, artwork, description
- Episode list (most recent first)
- Prominent "Subscribe" button

If the URI is already subscribed, show a warning and offer a button to navigate
to the existing subscription's details page. Optionally surface archive playlist
suggestions for re-listening.

On subscribe, the app stores feed metadata and begins the subscription machinery
(refresh schedule, download rules).

MVP is to support RSS files, for future enhancements, see Feed parsing spec

After hitting the "subscribe button" the user is taken to the subscription
details page, which they shouldn't notice, since the only difference between the
preview and the details page is the presence of the "subscribe" button, which
disappears when subscribing

### Authenticated Feeds

Feeds requiring authentication (e.g. Patreon, supporting-cast) are out of scope
for MVP. See `subscription-auth.md`.

## Feed Refresh

The app periodically fetches the feed URI to discover new episodes. Refresh
is a metadata-only operation — it does not trigger downloads directly. Download
decisions are evaluated separately against the feed's download rules after a
refresh, but new metadata should trigger re-evaluation of those rules

### Refresh Schedule

A cascading default pattern applies: each subscription may set its own refresh
interval, falling back to the global default if unset.

Display pattern:
- Global default unset: `Refresh every 12 hours (default)`
- Global default changed: `Refresh every 24 hours (default)`
- Per-feed override: `Refresh every 1 hour (custom)`

Refresh can also be triggered manually from the subscription details page or
the library view (refresh all).

Feed refresh interval: 12 is the default, managed by settings (cascading)

Refresh should be informed by server-side HTTP conditional GETs, and in general
play well with 200/304/429 HTTP codes

If the OS/platform supports background operations, then
polling/refresh/update/downloads
should happen in background. If not, then there should be a signal to user
(panel somewhere?) that the app only updates when it's foregrounded.

## De-listed Episodes

When an episode is no longer present in a feed's RSS/Atom/JSONFeed:

- Episode metadata is retained permanently in the local database
- Episode is hidden from the default subscription view
- A "Show unavailable episodes" toggle reveals them
- If the episode audio file is still on device, it remains playable
- Episode is marked with "Removed from feed" status (see `episode.md`)

Permanent metadata retention is intentional — it supports the library model
and keeps the archive complete even as publishers rotate content.

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
- Download rules (see `storage.md`)
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

### Subscription Preview (pre-subscribe)

- Feed artwork, title, description
- Episode list preview
- Prominent "Subscribe" button
- If already subscribed: warning + "Go to subscription" button
