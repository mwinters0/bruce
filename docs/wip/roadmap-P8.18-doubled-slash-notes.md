# P8.18 — A doubled slash is the linking page's fault: notes

What landed, for 8.6's Finding 10
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 10") and P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"A trailing slash is pedantic"), decided in D71: `link_verdicts` gives a link
to a URL whose path doubles a slash, or that redirected to one, `broken_url`,
reason `doubled_slash`; it gained `relative` and `counted` columns, and joins a
`doubled_slash` view of the run's such targets (`store::views`); `report_broken`,
`report_errors`, `report_stats` and `report_redirects` read only `counted`
links; the digest's `pages`, `by_repo` and `by_component` gained `through`
and rank by red plus `through`, and the HTML's summary reads "● 1 broken
link, ● 556 broken through it (558 issues total)". The manual, the golden
file and the tests follow.

## What the review and the next slices inherit

- **On 8.14's database** (`runs/p8.6-review/en-us-p8.14.duckdb`, a copy):
  `en-US/project/` is the first row, 2 broken links and 556 broken through
  them, the spec's 579 not reproduced (see below). 51 counted links
  on 24 pages are `doubled_slash` (64 in all, the rest on the doubled pages),
  10 naming the URL themselves, the rest redirecting to one
  (`arm.fedoraproject.org/`, `admin.fedoraproject.org/nagios/`). Of the
  13 doubled-slash pages, 11 have their twin, and none of the 13 is a row:
  the three `code-of-conduct//` pages hold only relative and same-page
  links, and the two calendar pages with no twin have no broken absolute one.
- **`counted` is the gate's too, by way of `report_broken`**; `introduced`
  reads `link_verdicts` directly for the baseline's side and so does not
  filter it, which no built page needs.
- **A digest is four times slower on that database** (8.5 s against 2.2 s,
  release build, a copy of the file, one run): `doubled_slash` runs a regex over every target
  of a run (0.17 s a read of the view), and `pages`, `by_repo` and
  `by_component` each evaluate `through`'s walk over `link_verdicts` again.
  Computing `through` once and joining it to the three is the remedy if every
  locale's digest (P9) makes it matter.
- **8.17 asks the toggled URL** and is untouched; a doubled slash is not a
  toggled one (`a//` against `a` differ by two).

## Negative results

- **The spec's 579 is not reproducible.** On 8.14's database the three pages
  hold 543 relative links that answer broken and 19 that now name a doubled
  URL (562), 6 of which link to the page itself and are judged as any link
  (D71), leaving 556. The 579 was read before the view existed, from another
  classification of relative links; no row of the old view gives 579.
- **A same-page `#fragment` link on a doubled page is not a doubled-slash
  link.** The first draft of the rule made all 25 per page broken, and counted
  them on the page's own row where it has no twin; the page's link to itself
  is judged as any other now (`store::views::tests::a_doubled_slash_page_linking_to_itself_is_judged_as_any_link`).
- **`relative` is the `raw_href` that is not a scheme, a leading `/`, a
  `#fragment` or a `?query`**, so a root-relative link (`/en-US/…`), which a
  doubled path does not break, is counted on its own page.
- **Rejected: dropping the twin page's links from `link_verdicts`**, which
  would take them out of the gate's baseline side and of every report that
  reads the view; a column keeps the verdict and lets each report choose.
