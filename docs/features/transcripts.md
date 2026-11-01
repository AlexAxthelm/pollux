---
type: feature
status: draft
priority: low
area: player
---
# Transcripts

* OPEN QUESTION: Are there any good public repos of podcast transcripts?
* OPEN QUESTION: Use on-device model to transcribe episodes?

## UI:

* in player view, there should be a line or two of text under the main art.
* clicking on the lines expands them to rougly the size of the main art view
(shrinking that)
    * maybe the art goes up into the corner, and there's a fade-out on the text.
* Text should scroll with timestamps, if available. 
* Text should highlight with timestamp, if available:
    * Current line: `Base01`
    * Past lines: `Base00`
    * Future lines: `Base1`

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
