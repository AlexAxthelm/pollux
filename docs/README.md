# Pollux docs

These docs are plain Markdown in the repo, and `docs/` is also an Obsidian
vault. Everything here must stay readable as ordinary text (GitHub, an editor,
or an agent reading the files); Obsidian only adds navigation on top.

## Layout

```
docs/
  README.md            this file
  _templates/          note templates (Obsidian Templates plugin points here)
  features/            one note per feature
  user_stories/        one note per user story
  decisions/           one note per decision (ADR)
  bugs/                one note per bug
  policies/            CODE_OF_CONDUCT, CONTRIBUTING
  *.base               saved queries over the notes (see Bases)
  (root)               ARCHITECTURE, DATA_MODEL, DESIGN_PRINCIPLES, GLOSSARY,
                       HACKING, ROADMAP
```

Folders are for browsing only. Nothing depends on which folder a note is in,
and sub-folders may be added under `features/` or `user_stories/` when a
directory gets crowded. Grouping that matters lives in properties (below), not
in the path.

## Naming

- **Filenames must be unique across all of `docs/`.** Links resolve by
  filename, so two notes named `bookmarks.md` make `[[bookmarks]]` ambiguous.
- **Features:** a lowercase slug, e.g. `bookmarks.md`. The slug is the feature's
  handle, so don't rename it casually. A sub-feature is named `<parent>-<part>`,
  e.g. `storage-limits` under `storage`.
- **User stories:** `US-NNNN short title.md`, e.g.
  `US-0004 jump back to a bookmark.md`. `NNNN` is sequential and never reused.
  The filename prefix is the ID; there is no separate `id` property. Cite just
  the ID (`US-0004`) in commit messages, tests and code comments, not the full
  filename.
- **Decisions:** `ADR-NNNN short title.md`, numbered the same way. Write the
  title as the decision itself ("Played is decided when the listener leaves an
  episode"), not the topic. For a choice that is still open, title it as the
  proposed decision and mark it `draft`. A decision is superseded, not edited
  away: mark the old one `superseded` and link the new one from `related`.
- **Bugs:** `BUG-NNNN short title.md`, numbered the same way, in `bugs/`. Cite
  just the ID (`BUG-0001`) in commits, tests and code comments.

## Properties (frontmatter)

Frontmatter must be the first thing in the file, before the `# Title`.
Properties that hold links must be quoted: `feature: ["[[bookmarks]]"]`.

| Note type | Properties |
|---|---|
| all | `type`: `feature`, `user-story`, `decision`, `bug`, `policy` or `reference` |
| feature | `status`, `priority` (low/mid/high), `area`, `depends` (list of links), `parent` (link, for sub-features) |
| user-story | `status`, `feature` (list of links), `related` (list of links) |
| decision | `status` (`draft` → `accepted` → `superseded`), `feature` (list of links), `related` (list of links); body is Context, Decision, Consequences |
| bug | `status`, `feature` (list of links to the feature or features affected), `related` (list of links); body is a title, what happens, and steps to reproduce |
| policy, reference | `type` only |

**`status`** is the same for features and stories: `draft` → `accepted` →
`done`. "Accepted" means agreed, not built. `closed` marks a note set aside
without being built, for example a story that duplicates another. Decisions have
their own sequence (see the table). Bugs use the same `status` values as stories.

**`area`** is for features only. It is a single value such as `player`, `library`, `subscription`,
`playlists`, `storage`, `sync`, `settings` or `theme`. Use it to filter across
folders instead of tagging. A story's area is the area of its feature(s).

**`parent`** marks a sub-feature, e.g. `storage-eviction` has
`parent: "[[storage]]"`. A parent feature lists its children under a `## Parts`
heading, so the split is visible from the parent.

Example story:

```markdown
---
type: user-story
status: draft
feature:
  - "[[bookmarks]]"
related: ["[[transcripts]]"]
---

As a curator
I want to jump quickly back to a bookmark
so that I can relisten to what I saved
```

## What goes where

Keep each fact in one place and link to it from the others.

- **User story:** who wants what, and why. A few lines. Short notes on the current
  state are fine; findings and open questions are not (see decisions).
- **Feature:** what the product does, as a spec. Where something is built, say so
  briefly (an "As built" or "Not yet" section). Long implementation reasoning goes
  in a decision, not here.
- **Decision (ADR):** why it is built this way, and what follows from it. This
  includes an open choice that needs discussion before anyone implements it.

**Unresolved decisions get a draft ADR.** The feature and the story state the
requirement and link to the draft; the options, evidence and open questions live in
the ADR, under a Decision section that says "Not yet decided". When it is decided,
fill in the Decision section and set the status to `accepted`.

Decisions whose reasoning was never written down are marked as such in their
Context section. Add the real reason when you know it.

## Relationships

- A story names its feature(s) with `feature:`, a list, so a story can belong to
  more than one feature. A feature never has to be edited
  to gain a story.
- Each feature note ends with a `## User stories` section holding a base: a
  query that lists every story linking to that feature (see Bases). The query is
  stored as plain text in the note, but the list itself only appears in Obsidian,
  so use the `grep` recipes below to find stories without it. Decisions and bugs
  tie to features through their own `feature:` property and are found the same way.
- Use `[[links]]` in the body for anything else. Obsidian's Backlinks pane
  shows what points at a note.
- Embeds (`![[note]]`) render only in Obsidian. Use them sparingly, and make
  sure the note still reads fine without them.

## Bases

A base is an Obsidian query over the notes' properties, shown as a table. Two
kinds are used here:

- **The stories table in each feature note.** It is a fenced `base` block copied
  from `_templates/feature.md`. It filters on `file.hasLink(this.file)` and
  `type == "user-story"`, so a story shows up in a feature's table as soon as its
  `feature:` property links to it. Nothing in the feature needs editing.
- **Saved queries as `.base` files at the root of `docs/`.** For example,
  `clean_up_query.base` lists every note that is not yet `accepted` or `closed`,
  grouped by status and ordered by priority (MVP first), then parent, then name.
  Use it to see what still needs review.

Bases read the properties described above, so they only work on notes that have
them (see "Status of the conversion").

With Obsidian open, the same queries run from a terminal through the Obsidian CLI:

```bash
obsidian bases                                              # list the .base files
obsidian base:query path=clean_up_query.base format=md      # run one
```

On macOS the binary is `/Applications/Obsidian.app/Contents/MacOS/obsidian`; it is
not on `PATH` by default. Formats are `json` (the default), `csv`, `tsv`, `md` and
`paths`.

## Tags

Tags are only for cross-cutting flags that don't fit a property, e.g.
`#needs-decision` or `#blocked`. Use properties for everything else.

## Finding things without Obsidian

```bash
grep -rl '"\[\[bookmarks\]\]"' docs/user_stories   # stories for a feature
grep -rl 'status: draft' docs                                # everything still draft
grep -rl 'status: draft' docs/decisions                      # open decisions
grep -rl '"\[\[player-streaming\]\]"' docs/decisions         # decisions for a feature
ls docs/user_stories/*/ | grep US-0004                       # a story by ID
```

## Status of the conversion

Only notes converted to this format have the properties above. Older notes
(including most feature notes and the multi-story files in `user_stories/`)
have none, or have frontmatter under the title that Obsidian ignores. Convert
a feature and its stories together, and move frontmatter to the top of the file
when you do.

## Obsidian settings

Open `docs/` as the vault. Only two files in `docs/.obsidian/` are in git, the
ones the notes depend on; everything else (appearance, fonts, graph, layout,
plugin choices) is yours and ignored.

Shared (committed):
- `templates.json`: the Templates plugin folder, `_templates`.
- `types.json`: property types, e.g. `feature` and `related` are lists.

Set these yourself if they aren't already (not committed):
- Core plugins: Templates, Properties and Backlinks on. They are on by default.
- Settings → Files & Links: wikilinks on, and "Automatically update internal
  links" on so renames rewrite links.
- Core plugin Bases on, so the stories tables and `.base` files render. The
  Obsidian CLI is optional; enable it in settings to run bases from a terminal.

Optional, to see git status inside Obsidian: the community plugin Obsidian Git. The
vault is a subfolder of the repository, so leave its "Repository folder within the
vault" setting **empty** (it finds the parent repository itself; `..` is rejected).
Leave "Limit file operations to the vault" on so only `docs/` shows as changed.
Open its panel with the command "Obsidian Git: Open source control view".
