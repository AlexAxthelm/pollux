---
type: decision
status: accepted
feature:
  - "[[theme]]"
related:
  - "[[ADR-0031 Views paint with semantic roles, not base16 tokens or hex]]"
---
# The core owns palette data and the active theme, and the shell resolves colors

## Context

Palette data should have one source of truth that every platform shell can share, while the mapping from a token to a platform color is inherently platform code.

## Decision

- The **core** owns palette data. `shared/src/theme.rs` defines the base16 palettes (`Base16Palette`, base00 to base0F) and the built-in themes. The active selection (`ThemeId` + `ThemeMode`) lives in the `Model` and is projected into `ViewModel.theme` as a `ThemeView`.
- The **shell** resolves tokens to colors. `iOS/Pollux/Theme.swift` maps base16 tokens to a `ThemeColors` set, injected at the app root (`iOS/Pollux/Pollux.swift`) into `@Environment(\.themeColors)`. Views read the semantic role, never a token or a hex value.

## Consequences

- Adding a palette is a new `ThemeId` variant, its data in `theme_view`, and an entry in `ThemeId::ALL`. A compile-time assertion fails the build if `ALL` and the enum disagree.
- The shell tolerates invalid hex strings, since the core does not validate them.
- User-defined palettes (the Custom theme) are expected to be persisted through storage later; see [[theme-custom]].
