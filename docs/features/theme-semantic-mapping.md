---
type: feature
status: draft
priority: MVP
area: theme
parent: "[[theme]]"
---
# Theme — semantic mapping

Companion to [[theme]]. The theme spec mandates that UI code reference
**semantic roles**, never raw hex or ad-hoc system colors, and that the base16
token behind each role be documented here. This is that document.

## How it fits together

The core owns palette data and the active theme; the shell resolves tokens to colors and
injects them at the app root. Views read the semantic role, never a token or a hex value.
See [[ADR-0032 The core owns palette data and the active theme, and the shell resolves colors]].

## Semantic roles

| Role (`themeColors.…`) | base16 | Used for | `System` theme resolves to |
|---|---|---|---|
| `background` | base00 | Screen background (app root + each screen; list/scroll backgrounds hidden so it shows through) | `.systemBackground` |
| `secondaryBackground` | base01 | Elevated surfaces: grouped list rows/cards, artwork placeholder fill + border | `.secondarySystemBackground` |
| `text` | base05 | Default foreground / primary text (titles, body, show notes) | `.primary` |
| `secondaryText` | base04 | Captions, metadata, inactive, badges | `.secondary` |
| `accent` | base0D | Tint, links | `.accentColor` |
| `error` | base08 | Error text | `.red` |
| `success` | base0B | Success state (reserved) | `.green` |
| `warning` | base0A | Warning state (reserved) | `.orange` |

`success` and `warning` are defined but not yet consumed: status badges are
currently `secondaryText` (unchanged from before theming). They exist so a later
pass can color those states without touching the infrastructure.

## How surfaces are themed

The whole app follows the active theme, not just foreground text. The details (screen
backgrounds, lists, navigation bars, primary text) are in
[[ADR-0034 Themes paint whole surfaces, and navigation bars stay transparent]].

## Accessibility

Contrast limits and the planned follow-up are in [[theme-accessibility]].

## The `System` theme

The default theme reproduces the platform's own colors; see [[ADR-0033 The System theme resolves to the platform's own colors]].

## Deliberate exceptions

`Color.debug` is intentionally not a theme color; see [[ADR-0031 Views paint with semantic roles, not base16 tokens or hex]].

## base16 token reference

base00 background · base01 lighter background · base02 selection · base03 comments
/ inactive · base04 dark foreground · base05 default foreground · base06 light
foreground · base07 light background · base08 red · base09 orange · base0A yellow
· base0B green · base0C cyan · base0D blue · base0E magenta · base0F brown.
See <https://github.com/chriskempson/base16>.

## Decisions

- [[ADR-0031 Views paint with semantic roles, not base16 tokens or hex]]
- [[ADR-0032 The core owns palette data and the active theme, and the shell resolves colors]]
- [[ADR-0033 The System theme resolves to the platform's own colors]]
- [[ADR-0034 Themes paint whole surfaces, and navigation bars stay transparent]]
- [[ADR-0035 Contrast is enforced for text roles only]]

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
