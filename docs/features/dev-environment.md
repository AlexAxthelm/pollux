---
type: feature
status: draft
priority: low
area: dev-environment
depends: []
---
# Dev environment

How a developer builds, checks and runs Pollux locally. Setup and commands are in
[[HACKING]]; this note holds the requirements for improving them.

## Not yet

* Build tool preflight check ([[US-0187 Check for build tools]]).
* A local forbidden-patterns scan ([[US-0190 Admin checks run locally]]).
* A release-mode Rust core for device builds, and a device-run target that does not need
  the one-time signing setup ([[US-0191 Make targets for physical devices]]).
* Showing when a device build expires ([[US-0232 See when a device build expires]]).

## As built

* `make ios-device` builds, installs over the existing app and launches it on the first
  paired iPhone ([[US-0230 Reinstall device builds easily]],
  [[US-0231 Reinstall keeps app data]]). Verified on a physical iPhone 14 Pro Max: the
  reinstall kept subscriptions once `PRODUCT_BUNDLE_IDENTIFIER` in `iOS/Local.xcconfig`
  matched the installed app. Setup is in [[HACKING]].

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
