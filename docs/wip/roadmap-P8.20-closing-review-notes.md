# P8.20 — The maintainer's closing review of the finished digest: notes

The review runs in rounds, each finding filed below with the slices it
spawned, and ends in the verdict at the foot. The findings so far were read
off the digest of run 2 of `runs/en-us-20261025-1301/en-us.duckdb` (M21's
crawl) against its run 1, a copy of 8.14's.

## Finding 1: "Broken since" held the fixed links too

Maintainer, 2026-10-25: section 3, "Broken since the compared run", confused
by holding subsections "New" and "Fixed". It becomes two sections, "Broken
since the previous run" and "Fixed since the previous run". **It spawns
8.8.2.**

## Finding 2: no section for the anchors removed

Maintainer, 2026-10-25: the "Anchors removed" section is not needed. The
HTML drops it; the JSON keeps `anchors_removed`, the HTML's choice not being
a loader's, and its links are section 3's `anchor_removed` rows. **8.8.2.**

## Finding 3: the compared run is a sub-title

Maintainer, 2026-10-25: "Compared with" moves from the header's list to a
sub-title under the title, trimmed to the day the run started, with its time
where another run started that day, and "(run #N)". **8.8.2.** Placed by this
session: "another run" is another crawl run of the database, a `check-dir`
run having no digest; a baseline file's run reads "(baseline file)", its
number being meaningless (D73); "never finished" is kept for a compared run
that did not, its figures being partial.

## Finding 4: the missing fragments' figure is amber

Maintainer, 2026-10-25: the figure counting links to a missing fragment is
amber, as their dots are. **It spawns 8.19.2.** Raised by this session and
unanswered: the errored links' figure is amber while their dots are red.

## Finding 5: what is new is marked in every section

Maintainer, 2026-10-25: mark the new entries in all sections, taking as
little room as possible, and none on a digest with nothing to compare with.
Asked, the maintainer chose a bar at the left edge, a drill-down already
broken counting its new links in its summary, and `new` in the JSON. **It
spawns 8.21** (D75).

## Finding 6: an empty two-run section is left out

Maintainer, 2026-10-25: with nothing broken or fixed since the compared run,
that section is hidden. Placed by this session: hidden with its link in the
header, and with no compared run too, the sub-title saying there was none.
**It spawns 8.8.3.**

## Verdict

Maintainer, 2026-10-25, on the digest with findings 1 to 6 landed (8.8.2,
8.8.3, 8.19.2, 8.21): reviewed and approved. Left open: the errored links'
figure is amber while their dots are red (8.19.2's notes).
