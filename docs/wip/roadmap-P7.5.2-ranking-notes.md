# P7.5.2 — The map ranks: notes

What the next slice inherits. The decision is D82, amended.

## What landed

- **`report::digest::html::standing`** is the HTML's reading of the tables the
  JSON also holds: `pages` builds each page's `Counts` (kind by level) from
  `broken`'s and `errors`' rows at their mapped level and the `through` of its
  row at a doubled slash's, drops a page with none, and ranks by red then amber,
  a tie by URL; `repos` sums them per repo. `tally` words a `Counts`, one part
  to a kind and level, the red parts first. `html.rs` no longer reads `pages`'
  or `by_repo`'s own counts, only their rows' source, title, `new` and `repo`.
- **A kind at none leaves** the drill-downs, the unverified section, the
  link-target list (its "of the N broken links" less the links left out), the
  Housekeeping redirects and their count, and sections 3 and 4 (a section it
  empties goes, with its nav link). `Severity::is_finding` is the test.
- **The legend** names the kinds at none; the pedantic guide entry says the
  digest lists no such redirect where the map puts it at none, and the
  redirects fold's ring sentence goes with it.
- The figures, the JSON and its ordering, `report`'s other commands and the
  gate are untouched.

## For P7.6

- `--config` hands the digest rows, as 7.5 noted; nothing here reads the
  file. A trial map now changes the page list, the counts and the order, so it
  is how a site sees what a `none` costs it.

## Negative results

- Nothing measured: the HTML builds its counts from tables already in the
  document, one pass over `broken` and `errors`.
- A kind split over two levels (`doubled_slash = "attention"` with the other
  broken URLs red) prints two counts that word alike, "3 broken links" red and
  "2 broken links" amber; the dots tell them apart. A count by level alone
  ("5 red") was not built: it loses which kind a level holds.
- The `new` marks on a page, a repo and a repo's `new_links` come from SQL over
  all findings: a page whose only finding before was at none reads as not new.
  The links counted "new" in a summary are those at a level above none.
