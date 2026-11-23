---
type: feature
status: accepted
priority: MVP
area: settings
parent: "[[settings]]"
depends:
  - "[[settings]]"
  - "[[notifications]]"
---
# Settings UI

How the user finds, understands and changes settings. The catalog of settings is in
[[settings]]; inheritance and overrides are in [[settings-cascade]].

## Finding settings

- All settings are in one central place, so the user doesn't have to hunt for
  control surfaces.
- The user can search for a settings option.

## Advanced settings

More powerful controls are hidden but accessible, so the user opts in to things that
may cause problems. The user can also choose to show them all the time, as a
permanent opt-in. Which settings count as advanced is in the Advanced section of
[[settings]].

Which settings the Advanced filter reveals depends on what is being edited (library/global,
a subscription, a playlist). See [[US-0233 Advanced settings depend on what is being edited]].

## Changing settings

- A setting the user sets is persisted, and generally followed.
- The user can easily undo a settings change.
- Settings can be exported, so they can be included in bug reports. See
  [[US-0186 Settings export]].

## New settings and changed defaults

- New settings are opt-in, so the player doesn't change unexpectedly.
- The user is told when new settings become available, and when a default setting
  changes, so they can decide whether they like the old behavior.

*This file is drawn from the stories below; it has not been specified further.*

## Not built yet

- **VoiceOver and other accessibility on the settings screens.** The global Settings screen
  was built without a VoiceOver pass. Each row reads as one control ("Skip forward, 60 s")
  with a separate reset button, but the order, hints and reset wording haven't been tried with
  VoiceOver, and Dynamic Type and larger text haven't been looked at. To come back to, with
  the app-wide accessibility doc once it exists ([[accessibility]], not written yet). Contrast
  is covered in [[theme-accessibility]].

## Open questions

- How are the new-setting and changed-default notices delivered? See
  [[notifications]].

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
