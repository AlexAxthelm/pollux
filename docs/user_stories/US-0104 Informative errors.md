---
type: user-story
status: draft
feature:
  - "[[subscription]]"
related:
  - "[[US-0103 Subscription failure explanations]]"
---
As a user
I want to tell apart "that link has a typo", "the feed is down right now", and
"that link isn't a podcast feed"
so that I know whether retrying is worth it

Note: related, but distinct from [[US-0103 Subscription failure explanations]] in that some of these aren't subscription failures, but transient errors (which should be retried later)