# P8 inbox

Facts addressed to the run-history reports phase before it is grilled.
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
- **Why P8 cares.** Its `changed` report would list every page after a scope
  change. P8 decides whether to record the scope per run, or how else the
  report tells the two apart, and whether it flags the pages a root missed.
- **Origin.** P2's grilling, and the review of P2.5's fallback, 2026-10-17;
  moved from P4's inbox at P4's grilling, 2026-10-19, which deferred
  `changed`. Contingent on the hash staying scoped by config.

## An allowlisted page's hash moves between runs hours apart

- **Fact.** Between P4.4's and P4.10's en-US crawls (3 hours apart), 138 of
  the pages both reached have a different `content_hash`: 137 are forge pages
  (`generic`, allowlisted, so hashed whole), 98 of them at the same byte
  size, and one is a docs.fp.o page. Nothing on the forge was looked at to
  say what moved.
- **Why P8 cares.** `changed` would list the forge's pages at every run
  unless it restricts itself to pages a root claimed (`pages.root` not NULL)
  or an allowlisted page's hash is scoped; that is P8's choice.
- **Origin.** P4.10, 2026-10-19 (`runs/p4.10-en-us-20261019-2125/`, runs 1
  and 2 of `combined.duckdb`). Contingent on the forge's markup carrying
  something volatile; unconfirmed.

## D20's "did you mean" hint waits for Resolve

- **Fact.** P4 places a link with `xref_refspec` set as `broken_url`,
  reason `unresolved_xref` (D20), in its `link_verdicts` view, and builds
  no hint: D20's fuzzy match over resource IDs is left for the Resolve hook
  (D19, D38), which `antora` builds for CI as an exact match (D55).
- **Why P8 cares.** P5's grilling left the hint out of CI mode, as it
  changes no CI verdict, so it is the next reports phase's: whether
  Resolve's lookup, which P5 builds, gives the live report's hint, and where
  the hint lives, a column of the view or the report's rendering.
- **Origin.** P4's grilling, 2026-10-19, moved from P5's inbox at P5's
  grilling, 2026-10-20
  (`roadmap-P4-reports.md`, "Where the fragment cannot be checked").
