---
type: decision
status: accepted
feature:
  - "[[theme-semantic-mapping]]"
related: []
---
# The System theme resolves to the platform's own colors

## Context

Theming should not change how the app looks for anyone who has not chosen a theme.

## Decision

The default selection is `System` / `FollowSystem`. For it, `ThemeColors.resolve` returns the platform's own semantic colors (the right-hand column of the roles table in [[theme-semantic-mapping]]), so every themed surface maps back to its native system color and the app looks unchanged from before theming landed.

`Light` and `Dark` still force the OS appearance via `preferredColorScheme`, so the System theme honors the mode selector too.

## Consequences

- The default theme inherits the OS's accessible semantic colors, so contrast work only affects users who opt into a canonical palette; see [[theme-accessibility]].
