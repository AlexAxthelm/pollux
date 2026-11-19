---
type: feature
status: accepted
priority: MVP
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription]]"
  - "[[feed-parsing]]"
---
# Subscribing

The user provides a URI. The app fetches the feed file and presents a
**subscription preview**:

- Feed title, artwork, description
- Episode list (most recent first)
- Prominent "Subscribe" button

If the URI is already subscribed, show a warning and offer a button to navigate
to the existing subscription's details page. Optionally surface archive playlist
suggestions for re-listening.

On subscribe, the app stores feed metadata and begins the subscription machinery
(refresh schedule, download rules).

MVP is to support RSS files, for future enhancements, see [[feed-parsing]]

After hitting the "subscribe button" the user is taken to the subscription
details page, which they shouldn't notice, since the only difference between the
preview and the details page is the presence of the "subscribe" button, which
disappears when subscribing


### Authenticated Feeds

Feeds requiring authentication (e.g. Patreon, supporting-cast) are out of scope
for MVP. See `subscription-auth.md`.

## Preview (pre-subscribe)

The preview is shown before the user subscribes:

- Feed artwork, title, description
- Episode list preview
- Prominent "Subscribe" button
- If already subscribed: warning + "Go to subscription" button

## Entering a feed

- The user can subscribe with just a link to a show.
- A URL entered by hand should be completed where it can be, e.g. `example.com/feed`
  becomes `https://www.example.com/feed`.
- The feed format (XML, JSON, Atom) should not matter to the user. MVP supports RSS;
  see [[feed-parsing]].
- If a subscribe attempt fails, the URL the user typed stays in the field so it can
  be corrected instead of retyped. *(A current gap: the text field is cleared as soon
  as Add is tapped; related to ROADMAP phase 5, invalid URI on subscribe.)*
- Fetching over plain http and the insecure-feed prompt are in
  [[subscription-transport-security]].

## Failures

- Subscribe failures are explained in plain language, so the user knows whether to
  fix the link, wait, or give up.
- The user can tell apart "that link has a typo", "the feed is down right now", and
  "that link isn't a podcast feed". Some of these are transient errors rather than
  subscription failures, and should be retried later.
- User-facing error strings are produced by the core rather than passed through from
  platform APIs, so the wording is consistent across platforms and can be tested.

*This section is drawn from the stories below; it has not been specified further.*

## Open questions

- What exact categories of error does the core distinguish, and what does each say?

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
