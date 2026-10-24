# P8.17.1 — The landing toggled: notes

A target whose redirect lands on a broken 4xx has the landing's URL toggled
and asked (`crawl::slash::ask` is called with the hop's end in `Shared::walk`,
no longer only where `form == item.form`), and a 2xx sets
`wrong_trailing_slash` on the redirecting target's row and the landing's, as
every outcome is shared by both. The digest joins the landing for a
`wrong_trailing_slash` link as for a `doubled_slash` one
(`sections::landing_join`), `fix` is the landing's toggle where there is one
(`sections::add_fix`), and the drill-down and fix-once cell show "→ lands on …"
with the landing's final `/` marked (`html::final_slash`), then "→ try …". D72
holds the decision; the manual's `crawl`, `report` and `digest` say what a
reader sees. `fixtures/antora/digest.html` changes only in the pages section's
note, the fixture site holding no redirect onto a 404.

## What the next slices inherit

- **Each URL that lands on one 404 asks its toggle itself.** Two redirecting
  URLs with one landing make two probes (one HEAD, and on a 4xx one GET, under
  the host's pace). They cannot disagree in a run that is not flaky, and the
  landing's row takes the answer of whichever claimed it. A per-landing memo
  would be state shared across the walk for 3 of 34 redirected-then-4xx targets
  on 8.14's database (D72's evidence).
- **The yield is still unmeasured**; 8.17's notes say where the next full
  crawl counts it (`reason = 'wrong_trailing_slash'`), which now includes the
  redirected targets.
- **A landing without a final `/` has nothing marked**: the fix adds one, and
  "→ try …" shows it. Only a stray `/` is marked, as the maintainer's three
  examples (`pdc.fedoraproject.org/` and `//`, `taskotron.fedoraproject.org/`)
  are.

## Negative results

- **No change to `link_verdicts`.** The redirecting target already carries the
  landing's outcome and reason, so the view passes the reason through; only
  the digest's `landing` column widened.
- **A toggled landing that redirects is no answer**, as for an unredirected
  target (D72, Rejected).
