# P14 — Redirects as facts: notes

What P9 inherits, and the negative results. The decisions are D100 and the
amendments to D7, D14, D47, D70, D74 and D79; the spec is
[`roadmap-P14-redirects.md`](roadmap-P14-redirects.md).

## What P9 inherits

- **Schema 7.** `pages.in_sitemap` and `via_links` are gone, so D43 refuses
  every kept database, `runs/en-us-20261101-0005/` included. `review.py crawl`
  starts a new database there. A fresh en-US crawl therefore comes before any
  limit is tried against docs.fp.o with `report --config`.
- **docs.fp.o's limit is the guide's to set.** At 1 the 2026-11-01 database
  breaks 139 targets (31 slash-only), at 0 all 716 `ok` ones (the spec's
  "What docs.fp.o's chains are"). A delay-0 refresh is a hop, so a guide
  limit of 0 breaks a CI build's home link unless the site-root refresh is
  not built (P9's spec, "P9 follows the redirect facts").
- **A host's limit is the linked URL's host's**, not the landing's (D79,
  D100): the guide says so beside a forge's key. A forge's login walls take
  four hops from `src.fedoraproject.org` to `id.fedoraproject.org`.
- **`report_pages` costs a pass over `links`** for every reader:
  0.3 s against the 1.18 million links of the 2026-11-01 run, 1.2 s for
  both runs unfiltered, timed in the duckdb CLI over
  `runs/p14.1-discovery/evidence.sql`'s copy of the views. If a report is
  slow, `report_reach` is where to look; a reader needing neither column can
  join `pages` alone.

## Negative results

- **The limit rule's cost was not measured** against a large database: two
  policy joins in `tj`, each over a handful of rows per run, and a list
  length per target.
- **No test reproduces KD38's race.** The derivation reads rows that do not
  depend on which target claimed a page, so a test of the race has nothing
  to order. The evidence is the orphan, unlisted and sitemap tests with a
  page reached only through redirects.
- **No test drives the crawl's own ceiling through the rule.** A chain past
  10 never reaches `link_verdicts` as `ok`: the target is already
  `broken_url` with its own reason (rule 4).
- **`report_redirects` needed no change.** It already kept every counted
  redirected link whatever its verdict, as a redirect onto a 404 showed.
- **Over the kept database the derivation reproduced the flags**
  (`runs/p14.1-discovery/`): `listed` direct is exactly the old
  `in_sitemap`, 2,310 pages in each run, none through a redirect. `linked`
  through a redirect only is 412 allowlisted pages and 1 crawled. The old
  `via_links` was true on 16 pages, every one also linked directly.
