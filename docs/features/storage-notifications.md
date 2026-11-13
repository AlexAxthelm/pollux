---
type: feature
status: accepted
priority: mid
area: storage
parent: "[[storage]]"
depends:
  - "[[notifications]]"
  - "[[storage-limits]]"
  - "[[storage-eviction]]"
---
# Storage notifications

How Pollux tells the user about storage events. Delivery mechanics (platforms,
preferences) are in `notifications.md`; this file defines which storage events
produce a message and what each says.

## Notification types

- **Nearing the limit** — warn the user when they're getting close to filling the
  storage limit, so they can act before downloads stop.
- **Limit not set (nudge)** — prompt the user to set a limit when none is
  configured, with increasing urgency as device free space drops (roughly under
  4 GB free). Not shown on first run. See [[storage-limits]].
- **Downloads stopped** — tell the user when downloads are stopped because of the
  storage limit, so they understand why new content isn't available. See "Fail on
  full" in [[storage-limits]].
- **Episode larger than the limit** — an individual episode that can never fit
  (small limit, very large file) is reported rather than failing quietly.
- **This file could be deleted** — when [[storage-eviction]] rules match
  episodes in *suggest* mode, they are surfaced to the user for confirmation
  instead of being deleted silently.

Download-specific failures (network errors, auth) are covered in
[[downloads-failures]].

## Open questions

- What is the right escalation model for the storage nudge and the
  nearing-the-limit warning?
- At what threshold does "nearing the limit" fire?
- How are many "could be deleted" suggestions presented without being noisy
  (single digest vs. per-episode)?

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
