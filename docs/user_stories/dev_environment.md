# User Stories: Developer Environment

Dev-facing stories about setting up and building the project. Unlike the other
files here (which are end-user stories), these describe the contributor
experience.

As a developer setting up a new machine
I want `make` to check that the required build tools are installed before it runs
so that I get a clear "install X" hint instead of an opaque failure like
`error: no such command: bin` (missing cargo-run-bin) or `xcbeautify: command not found`

As a developer
I want the tool check to cover every build prerequisite at once
(cargo-run-bin, xcbeautify, xcodegen, and the pinned toolchain/targets)
so that I learn about all missing tools up front, not one failing command at a time

As a maintainer
I want the tool check to live in one place (e.g. a `check-tools` target the iOS
targets depend on) rather than scattered guards on individual recipe lines
so that adding or removing a tool dependency is a single edit and stays consistent

## Notes / context

- Prompted by PR #9 review: `make package` now depends on `cargo-run-bin`, and a
  missing install surfaces as `error: no such command: bin`, which does not name
  the tool. `xcbeautify` has the same class of problem on `make ios-sim`/`ios-test`.
- Current mitigation is documentation only — these tools are listed under
  Prerequisites in `docs/HACKING.md`, and the opaque errors have Troubleshooting
  entries. This story tracks the deferred idea of a preflight `check-tools` target
  so the failure is caught early with a clear hint.
