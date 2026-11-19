---
type: decision
status: accepted
feature:
  - "[[theme-semantic-mapping]]"
related: []
---
# Views paint with semantic roles, not base16 tokens or hex

## Context

Palettes are defined with the base16 nomenclature (base00 to base0F). Views need a stable vocabulary that does not change when a palette does, so that adding a theme never means touching a view.

## Decision

UI code references **semantic roles** (`themeColors.background`, `text`, `accent`, and so on), never a base16 token, a raw hex value or an ad-hoc system color. The token behind each role is documented in [[theme-semantic-mapping]].

**The one exception is `Color.debug`** (`DebugStyle.swift`). It is intentionally *not* a theme color: it is a loud marker for not-yet-wired UI (`.stubbed()`) and must never be replaced by a semantic role, because that would hide the "this does nothing" signal.

## Consequences

- A color a view needs that has no role gets a new role, not a hex value.
- `success` and `warning` are defined but not yet consumed (status badges are `secondaryText`), so a later pass can color those states without touching the infrastructure.
- `theme.md` has a line saying UI code uses base16 *token names*. That conflicts with this decision; see the open question in [[theme]].
