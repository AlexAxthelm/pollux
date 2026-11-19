# Pollux - A different sort of podcast app

A podcast player built around the idea that podcasts are libraries, not newspapers.

Most podcast apps are great at what they do — surface the latest episode, mark it played, move on. That's the right model for a lot of shows. But there's a whole category of podcasts that are evergreen: music mixes, narrative series, educational content, interview archives. For those, the "latest episode first" model actively gets in the way. Pollux is built for those.

The core idea is that a podcast's back catalog is as worth listening to as the latest episode, and the app should make it easy to work through both systematically. Not just "you can scroll back and find old episodes" — but genuinely good tooling for managing a queue that spans archives and new releases at the same time.

## Status

Early development. The design thinking lives in `docs/`, and implementation has started: feed subscribing and refresh, downloads, the library, and audio playback with a mini-player and lock-screen controls are built. Playlists and a Settings screen are next. See `docs/ROADMAP.md` for the plan and what is built.

## What makes it different

- **Playlists as queries**: a playlist is a set of sources (feeds, other playlists) run through filters and sorts. The same model that powers a simple "all unplayed episodes" list also powers "the 2 most recent unplayed episodes from each of these five feeds, new releases first, then archives in chronological order."
- **Archive binge**: first-class support for working through a back catalog without losing track of new episodes.
- **Library view**: subscriptions and playlists in one place, because a subscription is just a special case of a playlist.
- **User control**: defaults should be good, but the user should always be able to override them and the app should respond promptly.

## Docs

The docs are plain Markdown in `docs/`, which is also an Obsidian vault. `docs/README.md` explains how they are organized. In short:

- `docs/features/`: one spec per feature, with a `priority` (MVP, high, mid, low) and a `depends` property where relevant. Larger features are split into parts.
- `docs/user_stories/`: one note per user story, linked to the features it belongs to.
- `docs/decisions/`: one note per decision (ADR), recording why something is built the way it is.
- `docs/bugs/`: one note per known bug.
- `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, `docs/DESIGN_PRINCIPLES.md`, `docs/GLOSSARY.md` and `docs/ROADMAP.md`: the overview documents.

MVP features are the minimum needed to actually use the app for its intended purpose. High-priority features (especially archive playlists) are required before it's v1.0.

For setup and build commands, see `docs/HACKING.md`.

## Contributing

See `docs/policies/CONTRIBUTING.md`.
