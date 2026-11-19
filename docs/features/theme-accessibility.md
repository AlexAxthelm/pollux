---
type: feature
status: draft
priority: mid
area: theme
parent: "[[theme]]"
depends:
  - "[[theme]]"
  - "[[theme-semantic-mapping]]"
---
# Theme accessibility

Known contrast limitations of the built-in palettes, and the planned follow-up. The
decision about what is enforced is in [[ADR-0035 Contrast is enforced for text roles only]].

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
