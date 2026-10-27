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
  policies/            CODE_OF_CONDUCT, CONTRIBUTING
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
  handle, so don't rename it casually.
- **User stories:** `US-NNNN short title.md`, e.g.
  `US-0004 jump back to a bookmark.md`. `NNNN` is sequential and never reused.
  The filename prefix is the ID; there is no separate `id` property. Cite just
  the ID (`US-0004`) in commit messages, tests and code comments, not the full
  filename.

## Properties (frontmatter)

Frontmatter must be the first thing in the file, before the `# Title`.
Properties that hold links must be quoted: `feature: ["[[bookmarks]]"]`.

| Note type | Properties |
|---|---|
| all | `type`: `feature`, `user-story`, `policy` or `reference` |
| feature | `status`, `priority` (low/mid/high), `area`, `depends` (list of links), `parent` (link, for sub-features) |
| user-story | `status`, `feature` (list of links), `related` (list of links) |
| policy, reference | `type` only |

**`status`** is the same for features and stories: `draft` → `accepted` →
`done`.

**`area`** is for features only. It is a single value such as `player`, `library`, `subscription`,
`playlists`, `storage`, `sync`, `settings` or `theme`. Use it to filter across
folders instead of tagging. A story's area is the area of its feature(s).

**`parent`** marks a sub-feature, e.g. `storage-eviction` has
`parent: "[[storage]]"`.

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

## Relationships

- A story names its feature(s) with `feature:`, a list, so a story can belong to
  more than one feature. A feature never has to be edited
  to gain a story.
- Each feature note ends with a `## User stories` list of plain links to its
  stories. Keep this list as plain text so it reads without Obsidian.
- Use `[[links]]` in the body for anything else. Obsidian's Backlinks pane
  shows what points at a note.
- Embeds (`![[note]]`) render only in Obsidian. Use them sparingly, and make
  sure the note still reads fine without them.

## Tags

Tags are only for cross-cutting flags that don't fit a property, e.g.
`#needs-decision` or `#blocked`. Use properties for everything else.

## Finding things without Obsidian

```bash
grep -rl '"\[\[bookmarks\]\]"' docs/user_stories   # stories for a feature
grep -rl 'status: draft' docs                                # everything still draft
ls docs/user_stories/*/ | grep US-0004                       # a story by ID
```

## Status of the conversion

Only notes converted to this format have the properties above. Older notes
(including most feature notes and the multi-story files in `user_stories/`)
have none, or have frontmatter under the title that Obsidian ignores. Convert
a feature and its stories together, and move frontmatter to the top of the file
when you do.

## Obsidian settings

- Vault root is `docs/`.
- Templates plugin folder: `_templates`.
- Keep wikilinks on (Settings → Files & Links), and keep "Automatically update
  internal links" on so renames rewrite links.
- Commit shared settings only; leave workspace files (`.obsidian/workspace*.json`)
  out of git.
