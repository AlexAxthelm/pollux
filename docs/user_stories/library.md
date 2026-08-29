# User Stories: Library

As a curator
I want to view a show's backlog,
so that I can access previous episodes

As a curator
I want to view episode details of un-downloaded episodes
so that I can learn more before downloading

As listener
I want to view episode details
so that I can learn about what I'm listening to

As a user 
I want to mark an episode as played manually
so that I can exclude it from future listening

As a subscriber
I want to know if a show has stopped publishing
so that I can not expect new content

As a subscriber
I want to check for new episodes manually
so I don't have to wait for new content

As a curator
I want to access episodes that are no longer in the feed
so that I can listen to old episodes

As a curator
I want to see what episodes have been removed from the feed
so that I know what's missing

As a listener
I want to listen to episodes that have been removed from the feed
so that I can have a complete library

As a user
I want to search a catalog of shows
so that I can find something new

As a User
I want to easily access both playlists and subscriptions from my Library
so that I can easily navigate from one view

As a user
I want to manage playlists and subscriptions similarly
so that I can expect consistent behavior

As a user
I want a quick way to tell if a playlist is a default "subscription playlist"
so that I know if it's all coming from one source

As a user
I want to check for new episodes in a playlist manually
so that I don't have to refresh my whole library

As a user
I want to refresh my entire library manually (at once)
so that I don't have to individually update them or wait for a timed refresh

As a curator
I want to be able to flag/star an episode
so that I can come back to it or filter it

As a curator
I want to have multiple flag types
so that I can distinguish why I flagged

As a listener
I want to mark an episode as unplayed
so that I can listen again

As a user
I want common episode actions to be available as swipe gestures (where
available)
so that I can access them easily

As a user
I want common episode actions to be visible when I click on an episode
so that I can get to them without swiping

As a curator
I want to see suggested instructions when my library is empty
so that I know what to do next

## Ordering

As a subscriber whose shows aren't all in English
I want my library sorted the way my language sorts
so that a show starting with "Ä" or "É" appears where I expect it rather than
after everything else

As a user
I want the same ordering wherever the library is listed
so that a show doesn't move depending on which screen I came from

## Notes / context

- Current behavior is naive code-point ordering, not locale-aware collation.
  `Pollux::view` sorts with `sort_by_cached_key(|s| s.title.to_lowercase())`,
  which folds `Ä` to U+00E4 — past ASCII `z` at U+007A. Measured output for a
  mixed list is
  `["apple", "Banana", "eagle", "Zebra", "Ärger", "Ångström", "Émile"]`.
  That matches no real locale: German sorts `Ä` with `A`, Swedish places it after
  `Z` but ordered `Å < Ä < Ö`, and we emit `Ärger` before `Ångström`.
- The SQLite side had the same blind spot — `ORDER BY title COLLATE NOCASE` in
  `DatabaseManager.swift` folds ASCII only — so this was never correct on either
  side. Sorting now happens only in the core, which at least means one place to
  fix rather than two that can disagree.
- A real fix means locale-aware collation (e.g. the `icu_collator` crate) plus a
  decision about where the locale comes from. That is a dependency and
  architecture call, so per `docs/policies/CONTRIBUTING.md` it wants discussion
  before implementation rather than being folded into a feature PR.
- Deliberately **not** covered by a test. Asserting the current order would pin
  behavior no locale actually wants and would make a correct implementation later
  look like a regression.
- Related: `docs/features/library.md` sets alphabetical as the default ordering
  for both Playlists and Shows, with "by last updated" and custom order as later
  additions — whatever collation lands here should apply to those too.




