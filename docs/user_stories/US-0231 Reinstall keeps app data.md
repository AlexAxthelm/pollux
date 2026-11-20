---
type: user-story
status: accepted
feature:
  - "[[dev-environment]]"
related:
  - "[[US-0230 Reinstall device builds easily]]"
---
As a developer
I want reinstalling a development build to replace the app in place
so that my subscriptions, downloads and playback state survive and I can keep testing with real data

Notes:
* Installing over the existing app keeps its data. Uninstalling first erases it, so the
  reinstall target must not uninstall.
* This only holds while the bundle id and signing team stay the same.
