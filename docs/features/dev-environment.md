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
* Make targets for physical devices and release builds
  ([[US-0191 Make targets for physical devices]]).
* Easy reinstall of development builds on a phone, which expire after 7 days on a free
  Personal Team ([[US-0230 Reinstall device builds easily]]).

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
