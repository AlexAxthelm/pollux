---
type: user-story
status: draft
feature:
  - "[[dev-environment]]"
related: []
---
As a developer setting up a new machine
I want `make` to check that the required build tools are installed before it runs
so that I get a clear "install X" hint instead of an opaque failure like
`error: no such command: bin` (missing cargo-run-bin) or `xcbeautify: command not found`

Notes:
* Prompted by PR #9 review: `make package` now depends on `cargo-run-bin`, and a
  missing install surfaces as `error: no such command: bin`, which does not name
  the tool. `xcbeautify` has the same class of problem on `make ios-sim`/`ios-test`.
- Current mitigation is documentation only — these tools are listed under
  Prerequisites in `docs/HACKING.md`, and the opaque errors have Troubleshooting
  entries. This story tracks the deferred idea of a preflight `check-tools` target
  so the failure is caught early with a clear hint.