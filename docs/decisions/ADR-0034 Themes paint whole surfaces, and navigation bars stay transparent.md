---
type: decision
status: accepted
feature:
  - "[[theme-semantic-mapping]]"
related:
  - "[[ADR-0031 Views paint with semantic roles, not base16 tokens or hex]]"
---
# Themes paint whole surfaces, and navigation bars stay transparent

## Context

A theme that only changed foreground text would leave system backgrounds showing and look half-applied.

## Decision

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

## Consequences

- The roles each surface uses are in [[theme-semantic-mapping]].
