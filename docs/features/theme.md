# Theme (colorscheme)
---
priority: MVP
depends: settings
---

The app's color scheme is configurable via a theme selector in Settings. All
colors are defined using the base16 nomenclature internally, giving access to
a broad library of existing palettes and a clear framework for custom themes.

## Settings Entry Point

In the Settings screen, under an **Appearance** section:

```
Theme: Solarized (Follow System) >
```

Tapping opens the Theme Selector page.

## Theme Selector Page

```
[< Cancel]  Theme  [Save]

( Light )  ( Dark )  ( Follow System )

○ System          ☀️/🌙
○ Solarized       ☀️/🌙
○ Nord
○ Custom
  ...
```

### Light / Dark / Follow System

A radio selector at the top. Applies globally:
- **Light**: always use the light variant of the selected theme
- **Dark**: always use the dark variant
- **Follow System**: use light or dark based on OS setting

Themes that have both light and dark variants are indicated with ☀️/🌙. Themes
with only one variant (e.g. Nord, which is dark-only) have no indicator and are
unaffected by the Light/Dark/Follow selection.

### Behavior

- Selecting a theme instantly applies it to the app — the user can see the
  effect immediately without saving
- A small inline preview (episode list row + mini-player) is shown on this
  page so the user can evaluate the theme in context. If the mini-player is
  actually active, this is the "real" miniplayer, otherwise just a
  non-functional preview. The previews should not have actual behavior (visual
  only)
- **Save**: applies the selection and returns to Settings
- **Cancel**: restores the previous theme and returns to Settings

### Built-in Themes

At minimum:
- System (OS default light/dark) ☀️/🌙
- Solarized ☀️/🌙
- Nord
- Custom

More palettes can be added incrementally. The base16 framework makes this
straightforward.

## Custom Theme

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

For first draft, just one custom theme, but explor multiple cutom themes/import
in the future

## Internal Notes

- All color references in UI code should use base16 token names, never hardcoded
  hex values
- A semantic mapping document should be maintained alongside this spec, defining
  which base16 values are used for which UI elements
- The base16 spec: https://github.com/chriskempson/base16
