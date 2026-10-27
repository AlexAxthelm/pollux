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

As a developer
I want `make check` to run the same forbidden-patterns scan CI runs
so that I find out about a banned pattern (a force unwrap or force cast, a lint
suppression, unsafe code; the full list is in `docs/HACKING.md`) before I push, not from
a failed PR check

As a developer
I want `make` targets to build and run on a physical device, and to build the Rust core in
release mode
so that I can verify audio, lock-screen and performance behavior on hardware, with numbers
that represent what ships

## Notes / context

- The Makefile has no device target and no release option: the simulator builds use a
  debug Rust core (`cargo swift package` with no `--release`), which overstated the
  player's render cost by more than 30 times. The device run used raw `xcodebuild` with
  `-xcconfig` and `cargo bin cargo-swift package --release`, with signing (a free Personal
  Team, which expires after 7 days) in a git-excluded `iOS/Local.xcconfig`. Signing settings
  must not be committed.

- The forbidden-patterns scan (`.github/workflows/admin-forbidden-patterns.yml`) runs only
  in CI, so PR #23 failed on three forced unwraps in tests that `make check` had passed.
  Until a local equivalent exists (for example a `git grep` with the same patterns wired
  into `make check`), run the search in `docs/HACKING.md` before pushing.

- Prompted by PR #9 review: `make package` now depends on `cargo-run-bin`, and a
  missing install surfaces as `error: no such command: bin`, which does not name
  the tool. `xcbeautify` has the same class of problem on `make ios-sim`/`ios-test`.
- Current mitigation is documentation only — these tools are listed under
  Prerequisites in `docs/HACKING.md`, and the opaque errors have Troubleshooting
  entries. This story tracks the deferred idea of a preflight `check-tools` target
  so the failure is caught early with a clear hint.
