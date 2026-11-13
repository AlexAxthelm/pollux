---
type: feature
status: draft
priority: MVP
area: storage
parent: "[[storage]]"
depends:
  - "[[settings]]"
  - "[[downloads-queue]]"
---
# Storage limits

The user may set a soft cap on how much local storage Pollux may use. This limit
is not reserved — other apps may use the same space freely. Pollux will simply
stop downloading new episodes once the limit is reached.

Display note: the UI should make clear that the limit is a cap, not a
reservation. Suggested copy: *"Pollux will not download new episodes once this
limit is reached, but this space is not reserved — other apps can use it
freely."*

## Default limit

No limit is set by default — Pollux may use all available device storage.

The app should nudge the user to set a limit at an appropriate moment (not first
run). Nudge intensity should increase when available device storage is low
(roughly under 4 GB free). See [[storage-notifications]] and `notifications.md`
for nudge implementation.

A suggested starting point when the user does set a limit: **max(4 GB, 20% of
device storage)**. At ~60 MB/hour (128kbps MP3), 4 GB ≈ 66 hours of content.

## Configuration

The limit is set as a number in GB or hours (user picks unit). Whichever unit is
used, the other representations update live, so the user can see a conversion
between time and space and understand the consequences of their decision:

- **GB**: direct storage size
- **Hours**: approximate playback time (~60 MB/hour at 128kbps; shown as an
  estimate, not a guarantee)
- **% of device**: read-only annotation, not an input mode. It shows the limit
  as a fraction of the device's total space, so the user can see how it will
  impact their system overall (see also [[storage-ui]])

The limit is per-device and does not sync for MVP (see [[storage-sync]]).

## Enforcement

### Check before downloading

The limit is consulted **before** starting a download. If current space used +
size of pending downloads + the file under consideration is bigger than the
limit, do not download.

Individual episodes that are bigger than the limit on their own (small limit, or
very big episode files) should surface a notification to the user (once
notifications are implemented). See [[storage-notifications]].

### Fail on full

When the download queue would push usage over the storage limit (or device
storage is exhausted), new downloads fail rather than evicting existing content.
The user is notified and must manually free space before downloads resume.

This is intentional — the app should never silently delete content the user
might care about.

*(Future)* Automatic eviction with user-configured priority rules; see
[[storage-eviction]]. Even then, deletion is surfaced to the user first by
default.

## Open questions

- Should the limit also be expressible per subscription or per playlist? (See
  [[storage-styles]], which proposes presets that playlists can override.)

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
