---
type: decision
status: accepted
feature:
  - "[[settings]]"
  - "[[settings-cascade]]"
related:
  - "[[ADR-0044 An episode's settings come from the context it is played from]]"
  - "[[ADR-0040 The database schema is a single baseline until the first release]]"
---
# Settings are saved as one JSON document per level

## Context

Settings exist at several levels (global now; subscription, playlist and episode later, see [[ADR-0044 An episode's settings come from the context it is played from]]), and new settings will keep being added. Every level has the same shape, and a value that isn't set at a level is inherited. [[DATA_MODEL]] already says settings are structured config objects, not key-value rows, and leaves the storage open.

## Decision

Settings are stored in one table, keyed by level (`scope_kind`, `scope_id`; the global level is `global` with an empty id), with one JSON document per level.

- The document holds **only the values the user has overridden**. A value that is not in it is unset and follows the next level up, or the app default. Resetting a setting removes its key.
- The **core** writes and reads the JSON, and the shell stores it as an opaque string. The format lives in Rust, with its tests, and not in each shell.
- Missing keys are read as unset and unknown keys are ignored, so adding a setting needs no migration and an older or newer build can still read the document.
- A subscription or playlist level is a new `scope_kind`, not a new table or column, as with `play_context`'s source columns.

## Consequences

- Adding a setting means a field on the settings type and a case in the change event, with no schema change. That matters most after the first release, when a schema change is a real migration ([[ADR-0040 The database schema is a single baseline until the first release]]).
- A user who never changed a setting follows a new app default automatically, and the stored document shows which values were set on purpose. Both help [[US-0181 Notifications for changes in default]].
- Values can't be queried or filtered in SQL, for example "every subscription with a custom skip". If that is ever needed it is done in the core after loading, and the number of documents is small enough for that.
- A document that can't be parsed is treated as unset, so the settings fall back to defaults. It is left in place until the user changes a setting, and that change replaces it. A read that fails (a database error) also falls back to defaults, but saving is held for the run, so the saved document, which may be fine, is not overwritten. Neither case is yet reported to the user ([[US-0235 Be told when saved settings can't be read]]), and nor is a failed write ([[US-0236 Be told when a setting couldn't be saved]]).
- Whether settings sync between devices is open ([[sync-settings]]). One document per level is a natural unit to sync, but nothing here decides it.
- Revisit if a setting needs a type that JSON can't carry cleanly, or if settings need to be queried across levels.
