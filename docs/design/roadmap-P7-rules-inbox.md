# P7 inbox

Facts addressed to the declarative per-root rules phase before it is grilled.
Drained when it is, and then deleted (`../process.md`, "Inboxes: facts filed
by destination").

## A finding's severity belongs to the profile

- **Fact.** The digest marks each finding with a level read off its outcome
  and reason: red for `broken_url` and `error`, amber for `broken_anchor` and
  a redirect that is not pedantic, a grey ring for `pedantic_trailing_slash`
  (`roadmap-P8-history.md`, "Severity is marked"). The maintainer
  ruled these the default values and the classification of each kind the
  profile's, presumably through P7's rules.
- **Why P7 cares.** It decides what a TOML rule may set beside the facts
  extraction writes (D33, D37): whether a root, or a profile compiled in,
  can reclassify an outcome or reason, and whether the levels themselves are
  fixed or the profile's too.
- **Origin.** 8.6's review of 8.14's digest, 2026-10-24 (`roadmap-P8-history-notes.md`,
  "8.6's Finding 8"); the defaults land in 8.19.1. Contingent on P7 keeping its
  sketch's scope, per-root rules a site writes without Rust.

## A doubled slash in a path is the linking page's fault

- **Fact.** P8 makes a link to a URL whose path doubles a slash
  `broken_url`, reason `doubled_slash`, and charges the breakage on the page
  at that URL to the linking page (`roadmap-P8-history.md`, "A
  trailing slash is pedantic"). The maintainer ruled it true of Fedora and
  likely of most sites, not of every one.
- **Why P7 cares.** Whether a root's rules can turn the reason off, for a
  site whose URLs carry a meaningful `//`, is a rule a site would write
  without Rust. So is how far the digest's "broken through it" reaches:
  `full`, every doubled page the linking page's link reaches by relative
  links (554 on 8.14's crawl, P8's only mode); `immediate`, the landing page
  alone (183); `off`, the link red and the doubled pages' broken relative
  links counted nowhere. The depth costs nothing measurable, so the choice is
  meaning, not speed.
- **Origin.** 8.6's review of 8.14's digest, 2026-10-24 (`roadmap-P8-history-notes.md`,
  "8.6's Finding 10"); it lands in 8.18; the modes from the review of 8.18's calls
  (2026-10-24 history entry, "8.18's calls reviewed; 8.18.1 earned"), held
  for P7 rather than made a config key. Contingent on P7 keeping its
  sketch's scope.
