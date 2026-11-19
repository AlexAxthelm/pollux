---
type: feature
status: draft
priority: mid
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription-subscribe]]"
  - "[[feed-parsing]]"
---
# Transport security for feeds

Some shows are still served over plain http (older or self-hosted shows). Pollux
should be able to subscribe to them without weakening security for everything else.

## Behavior (proposed)

- **Secure by default.** Only secure feeds are fetched by default, with a per-feed
  override so the user can opt in to an insecure feed.
- **Try https first.** When the user pastes an http link, retry over https, so the
  user is never asked to allow insecure access for a host that supports https anyway.
  Permanent redirects overlap with this and are tracked in [[feed-parsing]].
- **Informative prompt.** When the user is asked to accept an insecure feed, the
  prompt says in concrete terms what they are accepting, so it is a real choice
  rather than a click-through.
- **Scoped exceptions.** Any transport-security exception is scoped to the specific
  feeds that need it, not applied to every request the app makes.
- **Our wording.** Entering an `http://` URL currently surfaces Apple's raw ATS string
  verbatim ("The resource could not be loaded because the App Transport Security policy
  requires the use of a secure connection."). That is accurate but not actionable, and
  it is Apple's wording rather than ours; see the error-string story in
  [[subscription-subscribe]].

## Current state

`iOS/Pollux/Info.plist` declares no ATS exceptions, so iOS default behavior
(arbitrary loads disabled) applies.

## Unresolved

The per-feed override is in tension with how ATS works, and enforcing it probably
means moving the https-only default into our own code. The reasoning and the open
decision are in
[[ADR-0026 Feed transport security is enforced by the app, not scoped through ATS]].

## Decisions

- [[ADR-0026 Feed transport security is enforced by the app, not scoped through ATS]] (draft)

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
