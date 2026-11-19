---
type: feature
status: draft
priority: low
area: subscription
depends:
  - "[[directory]]"
---
# Cognitohazard warning

Since the goal of the application isn't necessarily to keep up to date with the
latest episodes, offer a warning if the user is subscribing to a news or
politics podcast, that they may want to consider not doing that.

Possibly other categories, but another strong signal is daily(ish) update cadence

Can be disabled. In fact, can be disabled from the warning screen
(subscribe/no/don't warn again)

"Hey, this feed updates very frequently. Are you sure you want to subscribe? You
might want to use a different app for this feed."

"This feed is tagged `politics`. Is it good for your mental health to be listening to this?"    

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
