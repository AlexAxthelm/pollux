---
type: feature
status: draft
priority: MVP
area: theme
depends:
  - "[[settings]]"
---
# Theme (colorscheme)

The app's color scheme is configurable via a theme selector in Settings. All
colors are defined using the base16 nomenclature internally, giving access to
a broad library of existing palettes and a clear framework for custom themes.

> **Status:** the base16 infrastructure has landed. The active selection
> (`ThemeId` + `ThemeMode`) lives in the core (`shared/src/theme.rs`) and is
> projected through `ViewModel.theme`; the shell resolves it into semantic colors
> (`iOS/Pollux/Theme.swift`, injected at the app root). Built-in palettes: System
> (default), Solarized, Nord. Views paint with the semantic roles documented in
> [[theme-semantic-mapping]]. Still to build: the
> Settings **Appearance** selector UI (which drives `Event::SetTheme`),
> persistence, the Custom theme, and the live preview described below.

## Parts

- [[theme-ui]]: the Settings entry point and the theme selector
- [[theme-custom]]: user-defined colors
- [[theme-accessibility]]: contrast limits and the planned accessible themes
- [[theme-semantic-mapping]]: the semantic roles views paint with, and the base16 token behind each

## Internal Notes

- All color references in UI code should use base16 token names, never hardcoded
  hex values
- A semantic mapping document should be maintained alongside this spec, defining
  which base16 values are used for which UI elements
- The base16 spec: https://github.com/chriskempson/base16

## Decisions

- [[ADR-0031 Views paint with semantic roles, not base16 tokens or hex]]
- [[ADR-0032 The core owns palette data and the active theme, and the shell resolves colors]]
- [[ADR-0033 The System theme resolves to the platform's own colors]]
- [[ADR-0034 Themes paint whole surfaces, and navigation bars stay transparent]]
- [[ADR-0035 Contrast is enforced for text roles only]]

## Open questions

- #needs-decision **How do views refer to colors?** The Internal Notes above say "All color
  references in UI code should use base16 token names, never hardcoded hex values". The
  semantic mapping says views read the *semantic role* and never a token
  ([[ADR-0031 Views paint with semantic roles, not base16 tokens or hex]]). The code follows the semantic mapping. Both statements are kept as written.
- #needs-decision **UI size.** [[US-0013 UI size]] and [[US-0014 UI size defaults]] are tagged
  `theme`, but they are not about color, and no feature spec covers them. They are left as
  they are until it is decided where they belong (perhaps the Appearance settings section,
  which is still "to be defined" in [[settings]]).

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
