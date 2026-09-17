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

### How surfaces are themed

The whole app follows the active theme, not just foreground text:

- **Screen background** — each screen sets `.background(themeColors.background)`;
  the app root does too, so pushed views inherit it.
- **Lists** — `.scrollContentBackground(.hidden)` removes the system list
  background so `background` shows through, and `.listRowBackground(...)` paints
  rows (`secondaryBackground` for the grouped Library, `background` for the plain
  episode list).
- **Navigation bars** are left transparent (no `toolbarBackground` override — it
  suppresses the large title), so the themed background shows behind them.
- **Primary text** carries an explicit `.foregroundStyle(themeColors.text)`;
  show-note HTML has its foreground color stripped (see `ShowNotes.swift`) so it
  inherits the same token.

## Accessibility (WCAG contrast) — known limitations

The built-in **Solarized** and **Nord** palettes are canonical developer color
schemes, not palettes designed to meet WCAG AA (4.5:1 for normal text) for app
UI. Current status:

- **`text` (base05)** clears AA (≥4.5:1) against the background on every built-in
  theme. A core test guards this
  (`built_in_text_tokens_stay_legible_on_the_background` in `shared/src/theme.rs`).
- **`secondaryText` (base04)** clears AA on Solarized dark and Nord, but is only
  ~4.1:1 on Solarized light — just under AA for small text, though a large jump
  from base03's ~2.9:1. It was moved off **base03** (base16's intentionally
  low-contrast "comments" color, ~1.7:1 on Nord) precisely for this reason. The
  test enforces at least the 3:1 large-text/UI bar so it can't regress toward
  base03.
- **`error` (base08)** and **`accent`/links (base0D)** are the palette's own red
  and blue. Several combinations fall below 4.5:1 (e.g. Solarized-light error
  ~4.3:1, Nord error ~3.1:1, Solarized links ~3.4–4.1:1). Forcing these to AA
  would recolor them away from the palettes' identity, so they are left as-is.

**Planned follow-up:** ship one or more curated, WCAG-AA-compliant built-in
themes alongside the canonical palettes (and extend the contrast test to the
colored roles once a palette claims full compliance). The default **System**
theme already inherits the OS's accessible semantic colors, so this affects only
users who opt into a canonical palette.

## The `System` theme reproduces today's appearance

The default selection is `System` / `FollowSystem`. For it, `ThemeColors.resolve`
returns the platform's own semantic colors (the right column above), so every
themed surface maps back to its native system color and the app looks unchanged
from before theming landed. `Light` / `Dark` still force the OS appearance via
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
