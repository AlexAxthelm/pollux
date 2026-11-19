---
type: feature
status: accepted
priority: MVP
area: settings
parent: "[[settings]]"
depends:
  - "[[settings]]"
  - "[[subscription]]"
  - "[[playlists]]"
  - "[[episode]]"
---
# Settings cascade

How a setting's value is inherited from a broader level and overridden at a more
specific one. The catalog of global settings is in [[settings]].

## Cascading defaults pattern

Many settings follow a cascading default pattern: a global default applies
unless overridden at a more specific level (episode → feed → global).

The UI should always make clear whether a value is using the global default or
a custom override, and offer a way to reset to default:

- `Refresh every 12 hours (default)` — using global default
- `Refresh every 24 hours (default)` — global default has been changed
- `Refresh every 1 hour` — per-feed override is set; subtle reset button shown

This pattern applies to: refresh interval, download rules, skip increments, and
any other setting that makes sense to configure globally but override locally.

## Seeing what is customised

The user should be able to tell where a value comes from:

- A setting that differs from the app's own default is shown as adjusted.
- A setting that differs from the user's own global value is shown as overridden.
- A global setting the user has changed from the app default is shown as changed.

Changing a global setting is respected by everything that has not overridden it.
Returning a setting to its default should be easy, so the user doesn't need to know
what a sensible default is.

## Per-feed settings

Per-feed settings are accessible from the subscription details page, not from
the main settings screen. They mirror global settings with override capability.
See [[subscription]].

Per feed settings should be a separate page, accessible either from the main
settings page, or by clicking the "gear/settings" button from the feed details
page.

### Feed settings

sections inherit from global page, and use defaults (with a "(default)" note
unless changed. if not default, there should be a listtle "reset to default"
button next to it.
- Playback
- Library (display options)
- Storage (how many episodes to keep locally)

### Subscription settings

Same as feed settings, but also refresh interval

## Playlist and episode overrides (not specified yet)

The cascade above is episode → feed → global, but this file only specifies feeds.
The stories add:

- Playlists have their own settings (for example skip interval and autoplay), and
  can use a show's default settings.
- An episode can override the defaults, and otherwise follows its show's defaults.
- Playlists and subscriptions should be managed similarly, so behavior is consistent.

List filtering and sorting for playlists is in [[playlists]], not here.

*This section is drawn from the stories below; it has not been specified further.*

## Open questions

- #needs-decision **Where are per-feed settings reached from?** The text above says
  per-feed settings are accessible from the subscription details page, "not from the
  main settings screen", and later that the page is accessible "either from the main
  settings page, or by clicking the gear/settings button from the feed details page".
  These contradict each other. Both are kept as originally written.
- When a playlist and a show both set a value, which wins? Where do playlists sit in
  the cascade between a show and global?

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
