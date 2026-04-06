# Storage & Downloads
---
priority: mvp
---

Download manager:
* queue of files to Download
* (optionally) may download multiple episodes in parallel (configurable)
* App sandboxed Storage fine for now, but may want user managable later.
* would be good if there were tools for limiting space used, auto cleanup
  (deletion) of old episodes

Download episodes from feeds for offline playback. Nice-to-have: storage management view, per-feed breakdown. Big thing: identify episodes no longer available on server and protect them from accidental deletion.

## UI

### Download Screen
Shows download queue, and entries there can be altered (move to top, cancel)
If currently downloading, show progress bar (also elsewhere in episode summary
if downloading)

For failed downloads, show error messages, and offer a "retry" button


