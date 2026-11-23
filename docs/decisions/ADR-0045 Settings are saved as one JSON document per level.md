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
- Missing keys are read as unset and unknown keys are ignored when reading, so adding a setting needs no migration and an older or newer build can still read the document. Unknown keys are kept and written back unchanged when the document is saved, so a build that doesn't know a setting doesn't erase it.
- A subscription or playlist level is a new `scope_kind`, not a new table or column, as with `play_context`'s source columns. A subscription's document is `subscription` with the subscription's id, and it is removed in the same transaction as the subscription.
- All documents are read in one query at launch and kept in the core, because the player needs a subscription's settings the moment an episode starts and a load is asynchronous. There are few of them. A document for a level this build doesn't know (written by a newer build) is left in the table and not read, so it isn't lost.

## Consequences

- Adding a setting means a field on the settings type and a case in the change event, with no schema change. That matters most after the first release, when a schema change is a real migration ([[ADR-0040 The database schema is a single baseline until the first release]]).
- A user who never changed a setting follows a new app default automatically, and the stored document shows which values were set on purpose. A key is present because the user set it, even when its value equals the default, and it stays until they reset it ([[settings-cascade]]). Both help [[US-0181 Notifications for changes in default]].
- Values can't be queried or filtered in SQL, for example "every subscription with a custom skip". If that is ever needed it is done in the core after loading, and the number of documents is small enough for that.
- Each key is read on its own: a known key that is missing or has a bad value (not a whole number, or outside what the setting accepts: a skip of 1 to 3600 s, a rewind of 0 to 600 s) is unset and the others are kept (a change made in the app outside the same range is ignored: not applied, queued or saved, so a live value and a saved one follow one rule), so one bad setting doesn't cost the rest. Nothing tries to repair a document. One that isn't a JSON object at all (for example a truncated one) is treated as unset, so the settings fall back to defaults. It is left in place until the user changes a setting, and that change replaces it, and any bad known keys with it. Unknown keys stay. A read that fails (a database error) also falls back to defaults, but saving is held for the run, so the saved document, which may be fine, is not overwritten. A change made while the read is still pending is applied at once and kept; when the read answers, it is replayed over what was saved (resets included) and each level it touched is saved, so neither the saved document nor the change is lost. After a failed read it stays in memory only, and the read is tried again each time the app comes to the foreground, without holding playback; if that succeeds, the changes made meanwhile are replayed and saved as at launch. Neither case is yet reported to the user ([[US-0235 Be told when saved settings can't be read]]), and nor is a failed write ([[US-0236 Be told when a setting couldn't be saved]]).
- Whether settings sync between devices is open ([[sync-settings]]). One document per level is a natural unit to sync, but nothing here decides it.
- Revisit if a setting needs a type that JSON can't carry cleanly, or if settings need to be queried across levels.
