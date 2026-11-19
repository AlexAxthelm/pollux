# Pollux: instructions for Claude

Build, run and test commands live in `docs/HACKING.md` and the `Makefile`. Architecture is in
`docs/ARCHITECTURE.md`. This file covers only what's easy to get wrong when working with an agent.

## The docs are living docs

`docs/` is an Obsidian vault and the user's primary way of working on this project: they
read, plan and decide there, and code follows. Treat it as the source of intent, and keep it
true. `docs/README.md` has the conventions (naming, properties, templates, what goes where).
Read it before adding or restructuring notes.

- **Before changing behavior, read the docs.** Open the feature note, its user stories
  (`feature:` links; the note's stories table lists them) and the ADRs that link to it.
- **Keep feature notes current with everything.** When you build, change or discover
  something, update the feature note in the same change: the spec, plus a short "As built" or
  "Not built yet" note. A feature note that no longer matches the app is a bug in the docs.
- **User stories are cheap and welcome.** Adding them should be frequent. When you notice a
  need the stories don't cover, suggest one (or write it) as `status: draft`. Don't mark a
  story or feature `accepted` or `done`; the user decides that. `done` means built and
  checked in the app, so you can say what you built and what you checked, and leave the
  status to them.
- **ADRs are not to be relitigated lightly, and are not set in stone.** Follow an accepted
  ADR. Don't quietly work against one. If there is a compelling reason to change it (new
  evidence, a changed constraint), say so and propose a new ADR that supersedes it: mark the
  old one `superseded` and link the new one from `related`. Record a new decision, or an open
  choice that needs discussion, as an ADR (a `draft` if unresolved) instead of burying it in a
  feature note or a code comment.
- **Flag contradictions, don't resolve them silently.** When two notes disagree, mark it
  `#needs-decision` where it appears and keep both statements as written.
- **Cite IDs** (`US-0004`, `ADR-0014`, `BUG-0001`) in commits and code comments, not full
  filenames. Keep files in the vault's formats: frontmatter first, quoted `[[links]]`.

## Driving the iOS Simulator

To tap, type, swipe or inspect the app in the simulator, use tools in this order:

1. **Native simulator tools, if they're in your tool list.** That means the Claude desktop app's
   iOS Simulator tools, or an MCP server with simulator UI automation (for example
   MobileBuildMCP's `ui-automation` workflow: `snapshot_ui`, `tap`, `type_text`, `swipe`,
   `screenshot`). Check with ToolSearch before assuming they're missing. An MCP server added
   mid-session doesn't load until the session restarts.
2. **AXe**, if (1) isn't available: `axe describe-ui`, `axe tap --label`, `axe type`,
   `axe screenshot`, always with `--udid <sim>`.
3. If neither is available, say so and ask. Don't script the Simulator window with AppleScript or
   System Events (that needs Accessibility permission the terminal usually lacks), and don't try
   to work around that.

`xcrun simctl` alone can boot, install, launch, screenshot and open URLs, but it can't tap or type.

### Rules

- **Never install anything without asking first.** That covers `brew install`, `npm install -g`,
  `claude mcp add`, and any other tool or MCP server. Say what it is and what it changes on the
  machine, and wait for a yes. A yes covers that one install only.
- **Ask before destructive simulator actions**, such as uninstalling the app or erasing the
  simulator. They wipe the user's subscriptions and playback state.
- **If you use MobileBuildMCP, keep its Sentry telemetry off** (`MOBILEBUILDMCP_SENTRY_DISABLED=true`).
  It's on by default. Enable only the workflows you need, which is `ui-automation` for this.

### Verifying UI actions

- A tool reporting "success" means input was sent, not that the intended element received it.
  Take a fresh snapshot or screenshot after each step and check the result.
- Element refs (MobileBuildMCP) are snapshot-specific. Re-snapshot after any UI change before
  reusing one.
- Taps go to whatever is on top at that position. The mini-player bar can sit over the Library
  screen's Feed URL row (see the issue tracker), so a tap on the field may open the full player.

### Practical notes

- **Build:** `make ios-xcodebuild`. A fresh worktree has no `iOS/generated`, so the first build runs
  the full Rust → Swift → xcodegen → xcodebuild chain and takes a while. The app lands in
  `build/DerivedData/Build/Products/Debug-iphonesimulator/Pollux.app`. Install and launch it
  yourself with `xcrun simctl install` and `xcrun simctl launch`. `make ios-sim` blocks on
  `launch --console`. Get the bundle id with
  `/usr/libexec/PlistBuddy -c "Print CFBundleIdentifier" <app>/Info.plist`.
- **The simulator may be shut down** between turns or sessions. If a UI tool says the device isn't
  booted, run `xcrun simctl boot <id>`, `open -a Simulator`, `xcrun simctl bootstatus <id> -b`,
  then relaunch the app. The installed app and its data survive.
- **Simulator choice:** the Makefile picks `SIM_DEVICE_NAME`. Use `xcrun simctl list devices` to
  find its id.
- **Test feeds:** use real, reachable feed URLs and check them first with `curl -sIL <url>`.
- **AXe output:** the raw `describe-ui` JSON is huge. Filter it, for example
  `jq -r '.. | objects | select(.AXLabel != null) | "\(.type)\t\(.AXLabel)\t\(.AXFrame)"'`.
- **zsh:** don't store a multi-word command in a variable and run it unquoted. zsh won't split it
  and the command silently doesn't run. Use a shell function.
