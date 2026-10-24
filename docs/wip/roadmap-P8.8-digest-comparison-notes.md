# P8.8 — The digest's comparison: notes

What landed: `bruce report digest --from N | --since FILE|URL` and the default
(the finished crawl run before the digest's), `report::digest::compare` and
`Against`, the header's "Compared with" line, each figure's change, section 7
(anchors removed, `anchors_removed`'s rows over every kind) in the HTML and the
JSON's `compared_with` and `anchors_removed`, `cli::file` (moved up from
`cli::check_dir::file`, a download paced by the defaults where no config is
given), `Store::discard_import`, the manual. D73 says why the import is removed.
**Section 3 did not land**: it is 8.8.1, below.

## The split

The row held two pieces of different confidence. The comparison (a run chosen,
its figures, a section over `anchors_removed`'s existing join) is plumbing over
queries 8.7 tested; section 3 is six attributions, each a new join between two
runs, whose reading needs a pair of runs that differ. Landing both would have
asked one review to accept the plumbing with the attribution. 8.8.1 is the
second.

## On 8.14's en-US database against 8.1's

`report digest --db <8.14> --since <8.1>`, in scratch copies of both (the
digest imports into the database it reads): 8.1's file is one finished run, so
`--since` takes it as it is. The header names the run of a baseline file; the
figures' changes are pages 2,320 to 2,323 (+3), links 586,175 to 586,175 (no
change), links to a broken URL 263 to 832 (+569), to a missing fragment 610 to
712 (+102), errored 0 to 88 (+88): the widened allowlist and the double-slash
pages (8.14's notes), which section 3 is to tell from en-US drift.
**`anchors_removed` is empty**, as 8.7's notes found for the two databases:
no en-US anchor was lost in a day, so section 7 over a real pair is read on the
fixture alone. The digest took 38.8 s against 26.3 s without `--since` (a debug
build, 8,990 pages): the import, two `totals` and the join. The database grew
33.6 to 55.3 MB at the first import and 0.5 MB at the second, the freed blocks
reused.

## What 8.8.1 inherits

- **The comparison's rows are in the database only while `digest::build`
  runs.** An imported baseline is deleted after it (D73), so section 3's
  queries run inside `build`, before `Store::discard_import`, over
  `compared.run.id`; the JSON and HTML cannot name that run, `compared_with.id`
  being `null` for a file's.
- **`Digest.compared` and `Compared.run.id`** are the run to join; `Against`
  chooses it. A digest with none has no section 3 rows, and says so as section 7
  does.
- **`anchors_removed` is per page and anchor**; 8.7's notes say section 3's
  "anchor removed" cause is the same join per link, not grouped.
- **The figures' changes are computed from `Compared.totals`** in the HTML; the
  JSON holds both totals and no change.

## Negative results

- **A `--from` naming a `check-dir` run is refused** (`NotCrawl`), as a digest
  of one is; the default never picks one (`mode = 'crawl'`).
- **`compared_with.id` for a file's run is `null`**, not the number the import
  gave: the number is a run the database held for a moment, and the file's own
  is not read.
- **No `source_url` on `anchors_removed`'s rows**: the report has none, and the
  section shows the page's file and repo as `source` does where it has no URL.
  Reopens: a reader wanting the forge link there.
