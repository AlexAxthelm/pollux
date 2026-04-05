# TODO

These are things that come up that need to be more fully scoped before work
starts, but I don't want to forget.

- [ ] Parse episode details (timestamps, etc)
    - [ ] Maybe parse html in episode details (currently strip and discard)
- [ ] Set up Dev tools (linter, formatter, etc)
    - [ ] make them nicely accessible for Claude (invoke with make?)
- [ ] it seems like there's a bunch of duplication in time/data parsing utils.
  Shoudl these be calls to a common library (external/internal)? Whart is
  idiomatic here?

  Should parse episode inherit from the episode datamodel?

  Shoult fetching a feed/episode and parsing it be separate?

  Can we check if an episode is available on server without downloading?

    

