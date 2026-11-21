---
type: user-story
status: draft
feature:
  - "[[dev-environment]]"
related: []
---
As a developer setting up a new machine
I want the build tasks to check that the required build tools are installed before they run
so that I get a clear "install X" hint instead of an opaque failure like
`error: no such command: bin` (missing cargo-run-bin) or `xcbeautify: command not found`

Notes:
* Partly addressed by [[ADR-0041 Dev tools and tasks are managed with mise]]: `mise install`
  provides xcodegen, xcbeautify, swiftlint and swiftformat at pinned versions, so a
  missing `xcbeautify` is now one setup step instead of a separate brew install. Still open:
  `cargo-run-bin`, Xcode and the simulator runtime are not covered. Mise's `auto_install`
  setting is on by default, so `mise run` may install missing pinned tools itself; that has
  not been checked end to end. #needs-decision whether this story still needs a preflight
  task once it has been.
* Prompted by PR #9 review: `package` (then a make target, now a mise task) depends on
  `cargo-run-bin`, and a missing install surfaces as `error: no such command: bin`, which
  does not name the tool. `xcbeautify` had the same class of problem on `ios-sim` and
  `ios-test`.
- Current mitigation: the tools are listed under Prerequisites in `docs/HACKING.md`, and
  the opaque errors have Troubleshooting entries. This story tracks the deferred idea of a
  preflight `check-tools` task so the failure is caught early with a clear hint.