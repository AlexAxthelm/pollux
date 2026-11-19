---
type: decision
status: accepted
feature:
  - "[[theme-accessibility]]"
related:
  - "[[ADR-0033 The System theme resolves to the platform's own colors]]"
---
# Contrast is enforced for text roles only

## Context

The built-in Solarized and Nord palettes are canonical developer color schemes, not palettes designed to meet WCAG AA (4.5:1 for normal text) for app UI. Forcing every role to AA would recolor them away from the identity that makes them those palettes.

## Decision

- `text` (base05) must clear AA against the background on every built-in theme. A core test guards this (`built_in_text_tokens_stay_legible_on_the_background` in `shared/src/theme.rs`).
- `secondaryText` is base04, not base03. Base03 is base16's intentionally low-contrast "comments" color. The same test enforces at least the 3:1 large-text/UI bar, so it cannot regress toward base03.
- `error` (base08) and `accent`/links (base0D) are left as the palette's own red and blue, even where they fall below 4.5:1.

## Consequences

- The measured ratios and the planned follow-up (curated AA-compliant themes, and extending the test to the colored roles) are in [[theme-accessibility]].
