# User Stories: Subscribing

As a user
I want to subscribe with just a link to a show,
so that I can get new episodes

As a curator
I want to unsubscribe from a show
so that I stop listening to it 

As a user
I want to listen to an episode without subscribing
so that I can tell if it's worth it

As a user,
I want to preview the episodes without subscribing
so that I can decide if I want to subscribe

As a subscriber
I want to subscribe to a podcast without caring about the format
so that I can see all the episodes

As a subscriber
I want to subscribe to access-controlled feeds
so that I can listen to premium feeds

As a subscriber
I want to undo an unsubscribe
so that I can fix a mistake

As a subscriber
I want to remove all data related to a subscription
so that it's like I never subscribed

As a subscriber
I want to be warned if I try to subscribe to a show I'm already subscribed to
so that I can see the existing subscription instead

As a subscriber
I want to know if my credentials for an authenticated feed are expired/invalid
so that I can update them

## Feed errors

As a user
I want subscribe failures explained in plain language
so that I know whether to fix the link, wait, or give up

As a user
I want to tell apart "that link has a typo", "the feed is down right now", and
"that link isn't a podcast feed"
so that I know whether retrying is worth it

As a user
I want the URL I typed to still be there when a subscribe attempt fails
so that I can correct it instead of retyping it

As a maintainer
I want user-facing error strings to be produced by the core rather than passed
through from platform APIs
so that the wording is consistent across platforms and can be tested

## Insecure feeds

As a subscriber
I want to subscribe to shows whose feeds are still http-only
so that older or self-hosted shows aren't unreachable

As a subscriber
I want the app to try a secure connection first and fall back only if needed
so that my connection is protected whenever the host supports it

As a subscriber
I want to be told when a feed can only be loaded insecurely
so that I can decide whether I care

As a maintainer
I want any transport-security exception scoped to the specific feeds that need it
so that we aren't weakening security for every request the app makes

## Notes / context

- Both sections came out of manual testing of the `feat/parsing` subscribe flow.
  Entering an `http://` feed URL surfaced Apple's raw ATS string verbatim:
  "The resource could not be loaded because the App Transport Security policy
  requires the use of a secure connection." That is accurate but not actionable,
  and it is Apple's wording rather than ours.
- `iOS/Pollux/Info.plist` currently declares no ATS exceptions, so iOS default
  behavior (arbitrary loads disabled) applies. Supporting http-only feeds means a
  deliberate decision about `NSExceptionDomains` scope — hence the maintainer
  story above, rather than a blanket `NSAllowsArbitraryLoads`.
- The "URL still there" story is a concrete current gap: `ContentView.swift`
  clears the text field immediately on tapping Add, so a failed attempt loses
  what was typed.
- Related roadmap items: Phase 5 "Error states — invalid URI on subscribe".
  `docs/features/feed-parsing.md` separately covers recording permanent
  redirects, which overlaps with the http→https story but is tracked there.

## Import/Export

As a user
I want to import my subscriptions from a different app
so that I can keep listening

As a subscriber
I want to export my subscriptions
so that I can import them into another app

As a user
I want to export a playlist
so that I can share it without sharing my whole library

As a user
I want to export a subset of subscriptions
so that I can share parts of my library

## Cognitohazard

As a subscriber
I want to be warned before subscribing to a show that is detrimental to mental
health
so that I don't feel pressure from it

As a subscriber
I want to be warned before subscribing to a high cadence feed
so that I don't feel pressure from it

As a subscriber
I want to disable Cognitohazard warnings
so that I can subscribe easily to things I want
