---
type: user-story
status: accepted
feature:
  - "[[dev-environment]]"
related:
  - "[[US-0191 Make targets for physical devices]]"
  - "[[US-0231 Reinstall keeps app data]]"
  - "[[US-0232 See when a device build expires]]"
---
As a developer
I want to rebuild and reinstall the development build on my phone with one command
so that I can keep using testing builds without working out the signing, install and launch steps each time

Notes:
* A build signed with a free Personal Team expires after 7 days. Once it does, the app
  won't launch and has to be reinstalled. Today that means remembering the raw
  `xcodebuild` invocation described in [[US-0191 Make targets for physical devices]].
* This story is the day-to-day use of that target: build, install on the connected
  phone, launch, with no manual steps in Xcode.
