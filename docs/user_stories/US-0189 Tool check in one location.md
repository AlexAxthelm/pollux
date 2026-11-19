---
type: user-story
status: draft
feature:
  - "[[dev-environment]]"
related:
  - "[[US-0187 Check for build tools]]"
  - "[[US-0188 Tool check checks all]]"
---
As a maintainer
I want the tool check to live in one place (e.g. a `check-tools` target the iOS
targets depend on) rather than scattered guards on individual recipe lines
so that adding or removing a tool dependency is a single edit and stays consistent