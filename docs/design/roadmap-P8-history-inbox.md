# P8 inbox

Facts addressed to the run-history reports phase before it is grilled.
Drained when it is, and then deleted (`../process.md`, "Inboxes: facts filed
by destination").

## A changed hash scope makes every page read as changed once

- **Fact.** A page's content hash covers a per-root scope set by `hash_root`
  and `hash_exclude` (D15). Changing either can change every page's hash in that root at the
  next run. Nothing records the scope per run: `runs.config_hash` changes on any
  config edit, so it cannot tell a scope change from a rate change.
  Nor does a hash record whether the root matched: a page a set `hash_root`
  misses is hashed whole (D15), its chrome included, and reads as changed at
  every build with nothing saying why.
- **Why P8 cares.** Its `changed` report would list every page after a scope
  change. P8 decides whether to record the scope per run, or how else the
  report tells the two apart, and whether it flags the pages a root missed.
- **Origin.** The extractor's grilling and the review of its hash fallback,
  2026-10-17; moved from the reports phase's inbox at its grilling,
  2026-10-19, which deferred `changed`. Contingent on the hash staying scoped by config.

## An allowlisted page's hash moves between runs hours apart

- **Fact.** Between two en-US crawls 3 hours apart, 138 of
  the pages both reached have a different `content_hash`: 137 are forge pages
  (`generic`, allowlisted, so hashed whole), 98 of them at the same byte
  size, and one is a docs.fp.o page. Nothing on the forge was looked at to
  say what moved.
- **Why P8 cares.** `changed` would list the forge's pages at every run
  unless it restricts itself to pages a root claimed (`pages.root` not NULL)
  or an allowlisted page's hash is scoped; that is P8's choice.
- **Origin.** 2026-10-19 (`runs/p4.10-en-us-20261019-2125/combined.duckdb`:
  the earlier crawl as run 1, the later as run 2, each DB's rows copied with
  `attach` and `insert`; schema 3, so read with the `duckdb` CLI). Contingent on the forge's markup carrying
  something volatile; unconfirmed.

## D20's "did you mean" hint waits for Resolve

- **Fact.** `link_verdicts` places a link with `xref_refspec` set as
  `broken_url`, reason `unresolved_xref` (D20), and builds no hint: D20's fuzzy match over resource IDs is left for the Resolve hook
  (D19, D38), which `antora` builds for CI as an exact match (D55).
- **Why P8 cares.** CI mode's grilling left the hint out of it, as it
  changes no CI verdict, so it is the next reports phase's: whether
  Resolve's lookup gives the live report's hint, and where the hint lives, a
  column of the view or the report's rendering.
- **Origin.** The reports phase's grilling, 2026-10-19, moved from CI mode's
  inbox at its grilling, 2026-10-20.

## The deployment guide's roots hash the profile's region

- **Fact.** P6's grilling settled that the deployment guide's roots, now
  P9's, set neither `hash_root` nor `hash_exclude`, so a docs.fp.o page
  hashes `article.doc` (D15): a whole-page scope takes in the nav, which
  lists a component's pages, so one added page would move every page of its
  component.
- **Why P8 cares.** `changed` reads that scope on a deployment's runs; a
  report proposing another scope reopens P9's spec, not just the guide.
- **Origin.** P6's grilling, 2026-10-21 (`../wip/roadmap-P9-deployment.md`,
  "The guide's hash scope is the profile's region"). Contingent on the nav
  being inside no hashed region.
