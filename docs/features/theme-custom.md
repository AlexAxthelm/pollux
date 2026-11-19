---
type: feature
status: draft
priority: MVP
area: theme
parent: "[[theme]]"
depends:
  - "[[theme]]"
  - "[[theme-ui]]"
---
# Custom theme

> **Not built yet.** `Custom` is a future `ThemeId`; user-defined palettes would be
> persisted through storage ([[ADR-0032 The core owns palette data and the active theme, and the shell resolves colors]]).

Selecting Custom expands (or navigates to) a table with three columns:

| Name & notes | Light (hex) | Dark (hex) |
|---|---|---|
| base00 — Background | #002b36 | #fdf6e3 |
| base01 — Lighter background | #073642 | #eee8d5 |
| base02 — Selection background | #586e75 | #93a1a1 |
| base03 — Comments, inactive | #657b83 | #839496 |
| base04 — Dark foreground | #839496 | #657b83 |
| base05 — Default foreground / text | #93a1a1 | #586e75 |
| base06 — Light foreground | #eee8d5 | #073642 |
| base07 — Light background | #fdf6e3 | #002b36 |
| base08 — Red / errors | #dc322f | #dc322f |
| base09 — Orange | #cb4b16 | #cb4b16 |
| base0A — Yellow / warnings | #b58900 | #b58900 |
| base0B — Green / success | #859900 | #859900 |
| base0C — Cyan | #2aa198 | #2aa198 |
| base0D — Blue / links | #268bd2 | #268bd2 |
| base0E — Magenta | #6c71c4 | #6c71c4 |
| base0F — Brown / deprecated | #d33682 | #d33682 |

- Default values are populated from the currently active theme
- The Dark column is greyed out (inherited from Light) until the user edits it
- Once a dark value is edited, it becomes independent of the light value
- A "Reset to light" option per row restores the inherited state

The example values above are Solarized for illustration — actual defaults will
reflect whatever theme was active when Custom was selected.

Only the base16 colors that are actually used in app should be raised to the
user in the table, to avoid confusion

For first draft, just one custom theme, but explore multiple custom themes/import
in the future

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
