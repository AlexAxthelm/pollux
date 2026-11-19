---
type: feature
status: draft
priority: MVP
area: theme
parent: "[[theme]]"
depends:
  - "[[theme]]"
  - "[[settings]]"
  - "[[mini-player]]"
---
# Theme UI

The Settings screens where the user chooses a theme. The palettes and how they reach
views are in [[theme-semantic-mapping]].

> **Not built yet.** There is no Appearance section; the active selection is set only
> through `Event::SetTheme`.

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
