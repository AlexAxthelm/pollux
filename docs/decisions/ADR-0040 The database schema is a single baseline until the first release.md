---
type: decision
status: accepted
feature: []
related: []
---
# The database schema is a single baseline until the first release

## Context

Until the first release the database is disposable. (Moved from [[DATA_MODEL]].)

## Decision

Until the first release the database is disposable, so the schema is a single
baseline (`v1_initial`, in `iOS/Pollux/DatabaseMigrations.swift`) that is edited in
place rather than grown by appending migrations. In debug builds the migrator
erases and recreates the database whenever that definition changes, so an existing
simulator database never needs wiping by hand. Before the first release, remove
that erase-on-change and freeze the baseline; after that every schema change is a
new append-only migration, and each one that alters a table that can already hold
rows should be tested against a database that has some.

## Consequences

- The baseline and the debug-only erase-on-change live in `iOS/Pollux/DatabaseMigrations.swift`.
- Freezing the baseline is a pre-release task; see the last two sentences of the Decision.
