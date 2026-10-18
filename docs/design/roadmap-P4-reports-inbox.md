# P4 inbox

Facts addressed to the verdicts-and-reports phase before it is grilled.
Drained when it is, and then deleted (`../process.md`, "Inboxes: facts filed
by destination").

## A changed hash scope makes every page read as changed once

- **Fact.** A page's content hash covers a per-root scope set by `hash_root`
  and `hash_exclude` (D15; `roadmap-P2-extractor.md`, "The content
  hash"). Changing either can change every page's hash in that root at the
  next run. Nothing records the scope per run: `runs.config_hash` changes on any
  config edit, so it cannot tell a scope change from a rate change.
  Nor does a hash record whether the root matched: a page a set `hash_root`
  misses is hashed whole (D15), its chrome included, and reads as changed at
  every build with nothing saying why.
- **Why P4 cares.** A report of changed pages between two runs would list
  every page after a scope change. P4 decides whether to record the scope per
  run, or how else the report tells the two apart, and whether it flags the
  pages a root missed.
- **Origin.** P2's grilling, and the review of P2.5's fallback, 2026-10-17.
  Contingent on the hash staying scoped by config.

## Links through a redirect: both uses, no setting

- **Fact.** The maintainer settled that some users care only about broken
  links, while others track every link through a redirect as a task to point
  it at its target, and that both are supported **without configuration**.
  P3 records each target's redirect chain, its final URL and `redirected`
  on its links (D7), a meta refresh of delay 0 counting as a
  hop; a target page's own `rel=canonical` is a `canonical` link.
- **Why P4 cares.** `bruce report` serves both as separate views: the broken
  report never lists a working redirected link, and a redirects report (D27
  drafted one) lists them as tasks. P4 decides whether a link to a page whose
  canonical names another URL belongs in that second view.
- **Origin.** P3's grilling, 2026-10-17 (`../wip/roadmap-P3-crawler.md`,
  "Redirects").

## An interrupted crawl leaves an unfinished run

- **Fact.** A Ctrl-C'd `bruce crawl` flushes what it fetched and leaves
  `runs.finished_at` NULL; no run resumes, and its rows stay (D2).
- **Why P4 cares.** A report's default run, and a diff's pair, should be the
  latest *finished* ones, and `prune` decides what becomes of unfinished
  runs.
- **Origin.** P3's grilling, 2026-10-18 (`../wip/roadmap-P3-crawler.md`,
  "An interrupted run").
