---
type: feature
status: accepted
priority: mid
area: downloads
parent: "[[downloads]]"
depends:
  - "[[downloads-queue]]"
---
# Background downloads

Downloads should keep going when the user leaves the app or locks their phone, so
they don't have to babysit a long download.

If the platform OS supports background app refresh / tasks, downloading should
proceed using that. If not, the user should be warned that downloads only
proceed while the app is open.

> **Not built:** downloads currently stop when the app is suspended.
> `DownloadManager` uses a foreground `URLSession` (default configuration, one
> session per download). The ~30s `BGAppRefreshTask` budget used by feed refresh
> is the wrong tool; iOS background downloads need a **background `URLSession`**
> (`URLSessionConfiguration.background(withIdentifier:)`), where the system
> transfers out of process and relaunches the app to deliver completion. That
> means a single long-lived session, delegate-based completion, completion
> arriving as a core event rather than the one-shot `Download` request, and
> reconciling with `getAllTasks()` at launch. Sketch only; see the user story
> [[US-0017 Background downloads]].

## Open questions

- How does the download queue interact with background app refresh and OS
  download managers? Likely varies significantly by platform.

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
