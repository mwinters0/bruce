# P8.8.1 — The digest's section 3, "since the last run": notes

What landed: `report::digest::cause` (`new_breakage`, `fixed_breakage`, `CAUSES`),
the `Digest` fields `new_breakage` and `fixed_breakage` and the JSON's two lists
(no `format_version` change: a field added), the HTML section `since` between
the figures and the pages (`html::compared::since_last_run`, one closed
`<details>` per cause, then the fixed links), the manual's section 3 and its
"Why a link is newly broken", the later sections renumbered there. D74 says why
the causes are tried in that order.

## On 8.14's en-US database against 8.1's

`report digest --db <8.14> --since <8.1>` in scratch copies (the digest imports
into the database it reads): **671 new breaks, every one `first_judged`**, and
**none fixed**. 569 are `broken_url` and 102 `broken_anchor`, the figures' changes
8.8's notes list (+569, +102); the 88 `error` links are not breakage and so in
no row. So no en-US link broke between the two crawls: the widened allowlist,
the other locales asked once (8.11) and the doubled-slash and `bad_scheme` rules
(8.18, 8.10) made `unchecked` or `error` links into judged ones, 592 of them with no
reason (a 404 or a missing fragment), 41 `doubled_slash`, 36 `nxdomain`, 2
`bad_scheme`. The other six causes and `fixed_breakage` are read on the fixture
alone (`cause::tests`), as `changed` and `anchors-removed` were: a pair of real
runs that differ is the first reading of `target_removed`, `target_moved`,
`anchor_removed` and `unexplained`. The digest took 51.8 s against 38.8 s with 8.8's
sections (a debug build, 8,990 pages); the HTML is 4.0 MB, section 3's 0.47 MB.

## What 8.9 and 8.20 inherit

- **Section 3's table is wide with 671 rows in one fold**: 8.20's review reads
  it, and a reader wanting the new breaks grouped by repo or page is the first
  thing it may ask for. Reopens: `first_judged` folded to a count where a run
  holds thousands (an every-locale digest, P9).
- **`fixed_breakage` has no cause**: a fixed link's `became` says what it is
  now, not why (its page edited, the target restored); the spec asks for a cause
  of the new alone.
- **The golden file moved**: `fixtures/antora/digest.html` holds the empty
  section 3 and its nav link; it has no compared run, so the fixture alone does
  not show a cause in the browser. 8.9's renders of it read the empty section.

## Negative results

- **A key held broken in the compared run is no new break even where the break
  has a different cause now**: a link that was `broken_url` and is `broken_anchor`
  is in neither list, as in `diff`.
- **`report_broken` is counted links only (D71)**, so a link on a doubled-slash
  page that gains a twin is a fixed link whose `became` is `uncounted` at most;
  none is on 8.14's database.
- **`unexplained` is not a reason column**: a doubled-slash rule newly judging
  a link `ok` before is `unexplained` and its `reason` says `doubled_slash`;
  a cause for each reason was refused (D74).
