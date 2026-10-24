# P8.18.1 — A doubled slash, asked or not: notes

What landed, for the review of 8.18's two calls
([`roadmap-P8-history.md`](roadmap-P8-history.md), "A trailing slash is
pedantic"; D71): `link_verdicts` gives a link naming a doubled-slash URL
`broken_url`, reason `doubled_slash`, whether the target was asked or not
(`skipped`, `robots`, a host off the allowlist); `relative` takes a `?query`,
which keeps the page's path, so a `?query` link on a doubled page is broken
through it as a path-relative one is; `doubled_slash` gained `twin_id`, and
the digest's `through` leaves out a link whose copy on the page's twin is
broken too. The page counts are computed once, and the digest's time is
down. The manual, D71 and the tests follow.

## What the review and the next slices inherit

- **On 8.14's database** (`runs/p8.6-review/en-us-p8.14.duckdb`, a copy):
  86 counted links on 42 pages are `doubled_slash`, against 51 on 24, the 35
  more being the links that were `unchecked` (34 off the allowlist, 1 behind
  `robots`); `en-US/project/` is still the first row, 2 broken and 554 broken
  through them, the 2 the review found broken on the twin too gone from
  556. No link on a doubled page there is a `?query` one, so that call moves
  no figure.
- **A digest takes 3.1 s on that database**, three runs of 3.09 to 3.11 s
  (release build, a copy of the file), against 8.5 s after 8.18 and the 2.2 s
  before it. `link_verdicts` costs 0.15 s a read (0.28 s) and `doubled_slash`
  0.10 s (0.18 s). The 0.9 s above the old figure is the view read again by
  each of the thirteen sections that reads it, 0.2 to 0.45 s each. Reading it
  once, into a temporary table the views read by name, is the remedy if every
  locale's digest (P9) makes it matter; the views take no table to read, so it
  is a change to how the store installs them.
- **The page counts are a temporary table**, `digest_page_counts`
  (`report::digest::sections::page_counts`), made once by `build` and joined
  by `pages`, `by_repo` and `by_component`, which take the `PageCounts` it
  returns as proof. It lives on the store's connection and is replaced by the
  next digest's.

## Negative results

- **Computing the page counts once saved 0.6 s of the 5.4 s** (8.5 s to
  7.9 s, the three evaluations of the walk being 2.3 s and the one 0.8 s). The
  rest was `doubled_slash`, read twice by every read of `link_verdicts`: its
  pattern ran over all 173,000 targets, of which 98 double a slash, and its
  twin lookup joined every page. A `contains` ahead of the pattern and a join
  of the 98 to the targets cut a section's time from 0.7 to 1.0 s to 0.2 to
  0.45 s.
- **Unchecked links turned `data:` and `mailto:` URLs red until the pattern
  read only `http(s)` URLs.** A `data:` body is base64, which holds `//`, and
  those targets are `unchecked`: the rule that skipped unchecked targets
  hid it. `store::views::tests::a_link_to_a_url_with_a_doubled_slash_is_broken_whatever_it_answered`
  holds one of each.
