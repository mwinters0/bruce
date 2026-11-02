# P14.1 — Discovery as facts: notes

What 14.2 and P9 inherit, and the negative results. The decisions are D14 and
D47 as amended; the spec is [`roadmap-P14-redirects.md`](roadmap-P14-redirects.md).

## What landed

- **Schema 7**: `pages.in_sitemap` and `via_links` are gone, `Item` carries
  neither, and `dispatch` takes no `via_links`. D43 refuses every kept
  database; `review.py crawl` starts a new one where the newest kept is of
  another schema (`opens` returns false for the one refusal it answers so).
- **`report_reach`** (a view of its own, beneath `report_pages`) holds
  `listed` and `linked` per `(run_id, url_id)`; `report_pages` joins it, and
  `report_sitemap` joins it for its entries. `report_inbound` gained `via`,
  which is how `linked` tells a redirect from a direct link. The spec put the
  columns in `report_pages`; they are still its columns, the derivation
  being one view the entries half of `report_sitemap` shares.
- **`report_sitemap`** gained `entry`, true for a sitemap entry no page was
  fetched for. The digest's sitemap entries and the report's ordering read it,
  there being no NULL flag left to mean it. An entry's `linked` is that of the
  URL it lands on.
- **Reproduction** over the kept 2026-11-01 database: `runs/p14.1-discovery/`
  (`evidence.sql`, its output `evidence.out`). Listed `direct` is exactly the
  old `in_sitemap`, 2,310 pages in each run, none through a redirect; linked
  through a redirect only is 412 allowlisted pages and 1 crawled, in both. The
  old `via_links` was true on 16 pages, every one also `linked` directly.

## What 14.2 inherits

- `report_pages` now costs a pass over `links` for every reader of it: 0.3 s
  against the 1.18 million links of the kept database, 1.2 s for both runs
  unfiltered (timed in the duckdb CLI, not through a report). If a report is
  slow, `report_reach` is where to look; a reader needing neither column could
  join `pages` alone.
- The `report_redirects` view is untouched. 14.2's `link_verdicts` rule and
  `target_moved` do not meet this slice's views.

## Negative results

- No test reproduces KD38's race (two orders of one redirect's hops); the
  derivation reads rows that do not depend on who claimed a page, so a test
  of the race has nothing to order. The orphan/unlisted/sitemap tests with a
  page reached only through redirects are the evidence.
- A stopped run that wrote no `sitemap` links lists nothing, so its pages read
  `listed` NULL, where the flags would have said false: the same.
