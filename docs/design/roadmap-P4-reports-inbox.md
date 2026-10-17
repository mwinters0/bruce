# P4 inbox

Facts addressed to the verdicts-and-reports phase before it is grilled.
Drained when it is, and then deleted (`../process.md`, "Inboxes: facts filed
by destination").

## A changed hash scope makes every page read as changed once

- **Fact.** A page's content hash covers a per-root scope set by `hash_root`
  and `hash_exclude` (D15; `../wip/roadmap-P2-extractor.md`, "The content
  hash"). Changing either changes every page's hash in that root at the next
  run. Nothing records the scope per run: `runs.config_hash` changes on any
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
