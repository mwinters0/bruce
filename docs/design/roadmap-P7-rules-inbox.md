# P7 inbox

Facts addressed to the declarative per-root rules phase before it is grilled.
Drained when it is, and then deleted (`../process.md`, "Inboxes: facts filed
by destination").

## A finding's severity belongs to the profile

- **Fact.** The digest marks each finding with a level read off its outcome
  and reason: red for `broken_url` and `error`, amber for `broken_anchor` and
  a redirect that is not pedantic, a grey ring for `pedantic_trailing_slash`
  (`../wip/roadmap-P8-history.md`, "Severity is marked"). The maintainer
  ruled these the default values and the classification of each kind the
  profile's, presumably through P7's rules.
- **Why P7 cares.** It decides what a TOML rule may set beside the facts
  extraction writes (D33, D37): whether a root, or a profile compiled in,
  can reclassify an outcome or reason, and whether the levels themselves are
  fixed or the profile's too.
- **Origin.** 8.6's review of 8.14's digest, 2026-10-24 (8.6's notes,
  "Finding 8"); the defaults land in 8.19.1. Contingent on P7 keeping its
  sketch's scope, per-root rules a site writes without Rust.
