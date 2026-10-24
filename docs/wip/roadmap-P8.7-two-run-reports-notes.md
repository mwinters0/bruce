# P8.7 — `changed` and `anchors-removed`: notes

What landed: `Report::Changed` and `Report::AnchorsRemoved`, `report/changed.rs`
and `report/anchors_removed.rs`, each a `report` child as `diff` is, choosing
`--from` and `--to` as it does (`Report::compares`, which `report/run.rs` and
`cli::report::check` read), and the manual's `report.md`. No `D<k>` entry:
the scope is D15's and 8.1's, the rest is the spec's.

## On the real databases

8.1's and 8.14's en-US databases, one copied in as run 2 of the other
(`runs/p8.1-en-us-20261023-2004` and `runs/p8.14-en-us-20261024-1322`, the
join done in a scratch copy and deleted): **both reports are empty.** Of the
2,320 pages a root covers, none has a different hash a day apart (the 21 whose
hash moved are forge pages, `root IS NULL`, which `changed` leaves out), none
is `whole` under a `hash_root` (the config sets none), and no anchor of run 1
is missing from a page of run 2. So the SQL is shown on a fixture only; 8.8's
section 3 and 7 over a pair that does differ are its first real reading.

## What 8.8 inherits

- **`changed` and `anchors_removed` are `pub(super)` fns** taking `(conn,
  from, to, request)` and returning `(Table, notes)`, which the digest, a
  descendant of `report`, can call. Section 7 is `anchors_removed`'s rows;
  section 3's "anchor removed" cause (a link `ok` in the comparison and
  `broken_anchor` now, its target page without the anchor) is the same join,
  `report_broken` to the landing page and `from`'s `anchors`, per link and not
  grouped.
- **`anchors_removed` joins on the link's target URL text** (`targets.url`),
  as `diff` does, then on the landing page by `final_url`; the landing page
  need not be a page a root covers, so an allowlisted page's lost anchor is a
  row, its `source_repo` null.
- **A link counts if it is `report_broken`'s**, so `counted` (D71) and the
  `kinds` filter apply; a link with the anchor in `from` and no `from` link at
  all (a new link) is still a row, the report saying the anchor is lost and
  named, not that the link was `ok` before.
- **`changed`'s `scope_changed` also covers a root whose `hash_root` or
  `hash_exclude` differ between the runs**, not only a differing
  `hash_scope`: a `hash_exclude` edit moves every hash of a `region` page
  without moving its scope, and 8.1's notes say a hash is comparable within
  one root's keys. A page whose root is not in `from` as a covered page is
  not compared. The comparison is of `runs.hash_scopes` lists by value, so a
  reordered `hash_exclude` reads as changed.
- **`root_missed` is the `to` run's fact alone**, so a page new in `to` can
  be one, and it is a column beside `change`: a missed page that also moved
  is `changed` and flagged, the D15 symptom.

## Negative results

- **`changed` lists no page in only one run**: a page gained or lost is not
  a changed page, and `diff` and `sitemap` tell it.
- **`anchors-removed` has no row for an anchor no link names**, as the spec's
  "while links still name them" has it; a page that lost forty unlinked
  anchors is not shown. Reopens: a consumer wanting the lost anchors whether
  or not linked.
