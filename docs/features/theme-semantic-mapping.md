# Theme — semantic mapping

Companion to [theme.md](theme.md). The theme spec mandates that UI code reference
**semantic roles**, never raw hex or ad-hoc system colors, and that the base16
token behind each role be documented here. This is that document.

## How it fits together

- The **core** owns palette data. `shared/src/theme.rs` defines the base16
  palettes (`Base16Palette`, base00–base0F) and the built-in themes; the active
  selection (`ThemeId` + `ThemeMode`) lives in the `Model` and is projected into
  `ViewModel.theme` as a `ThemeView`.
- The **shell** resolves tokens to colors. `iOS/Pollux/Theme.swift` maps base16
  tokens to a `ThemeColors` set, injected at the app root
  (`iOS/Pollux/Pollux.swift`) into `@Environment(\.themeColors)`. Views read the
  semantic role, never a token or a hex value.

## Semantic roles

| Role (`themeColors.…`) | base16 | Used for | `System` theme resolves to |
|---|---|---|---|
| `background` | base00 | Screen background (applied at the app root) | `.systemBackground` |
| `secondaryBackground` | base01 | Elevated/secondary surfaces (e.g. artwork placeholder fill + border) | `.secondarySystemBackground` |
| `text` | base05 | Default foreground / body text | `.primary` |
| `secondaryText` | base03 | Captions, metadata, inactive, badges | `.secondary` |
| `accent` | base0D | Tint, links | `.accentColor` |
| `error` | base08 | Error text | `.red` |
| `success` | base0B | Success state (reserved) | `.green` |
| `warning` | base0A | Warning state (reserved) | `.orange` |

`success` and `warning` are defined but not yet consumed: status badges are
currently `secondaryText` (unchanged from before theming). They exist so a later
pass can color those states without touching the infrastructure.

## The `System` theme reproduces today's appearance

The default selection is `System` / `FollowSystem`. For it, `ThemeColors.resolve`
returns the platform's own semantic colors (the right column above), so
**foreground/text colors and the primary background are identical** to the app
before theming landed. `Light` / `Dark` still force the OS appearance via
`preferredColorScheme`, so the System theme honors the mode selector too.

## Deliberate exceptions

- **`Color.debug`** (`DebugStyle.swift`) is intentionally *not* a theme color: it
  is a loud marker for not-yet-wired UI (`.stubbed()`) and must never be replaced
  by a semantic role — that would hide the "this does nothing" signal.

## base16 token reference

base00 background · base01 lighter background · base02 selection · base03 comments
/ inactive · base04 dark foreground · base05 default foreground · base06 light
foreground · base07 light background · base08 red · base09 orange · base0A yellow
· base0B green · base0C cyan · base0D blue · base0E magenta · base0F brown.
See <https://github.com/chriskempson/base16>.
