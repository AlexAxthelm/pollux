# Pollux - A different sort of podcast app

A podcast player built around the idea that podcasts are libraries, not newspapers.

Most podcast apps are great at what they do — surface the latest episode, mark it played, move on. That's the right model for a lot of shows. But there's a whole category of podcasts that are evergreen: music mixes, narrative series, educational content, interview archives. For those, the "latest episode first" model actively gets in the way. Pollux is built for those.

The core idea is that a podcast's back catalog is as worth listening to as the latest episode, and the app should make it easy to work through both systematically. Not just "you can scroll back and find old episodes" — but genuinely good tooling for managing a queue that spans archives and new releases at the same time.

## Status

Early development. The feature specs are in `docs/features/` — that's where the design thinking lives right now. Implementation is just getting started.

## What makes it different

- **Playlists as queries**: a playlist is a set of sources (feeds, other playlists) run through filters and sorts. The same model that powers a simple "all unplayed episodes" list also powers "the 2 most recent unplayed episodes from each of these five feeds, new releases first, then archives in chronological order."
- **Archive binge**: first-class support for working through a back catalog without losing track of new episodes.
- **Library view**: subscriptions and playlists in one place, because a subscription is just a special case of a playlist.
- **User control**: defaults should be good, but the user should always be able to override them and the app should respond promptly.

## Feature docs

All feature specs live in `docs/features/`. Each file has a `priority` tag (MVP, high, mid, low) and a `depends` tag where relevant.

MVP features are the minimum needed to actually use the app for its intended purpose. High-priority features (especially archive playlists) are required before it's v1.0.

## Contributing

See `docs/policies/CONTRIBUTING.md`.
