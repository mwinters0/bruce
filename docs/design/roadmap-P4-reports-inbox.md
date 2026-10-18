# P4 inbox

Facts addressed to the verdicts-and-reports phase before it is grilled.
Drained when it is, and then deleted (`../process.md`, "Inboxes: facts filed
by destination").

## A changed hash scope makes every page read as changed once

- **Fact.** A page's content hash covers a per-root scope set by `hash_root`
  and `hash_exclude` (D15; `roadmap-P2-extractor.md`, "The content
  hash"). Changing either can change every page's hash in that root at the
  next run. Nothing records the scope per run: `runs.config_hash` changes on any
  config edit, so it cannot tell a scope change from a rate change.
  Nor does a hash record whether the root matched: a page a set `hash_root`
  misses is hashed whole (D15), its chrome included, and reads as changed at
  every build with nothing saying why.
- **Why P4 cares.** A report of changed pages between two runs would list
  every page after a scope change. P4 decides whether to record the scope per
  run, or how else the report tells the two apart, and whether it flags the
  pages a root missed.
- **Origin.** P2's grilling, and the review of P2.5's fallback, 2026-10-17.
  Contingent on the hash staying scoped by config.

## Links through a redirect: both uses, no setting

- **Fact.** The maintainer settled that some users care only about broken
  links, while others track every link through a redirect as a task to point
  it at its target, and that both are supported **without configuration**.
  P3 records each target's redirect chain and its final URL (D7, D46), a meta
  refresh of delay 0 counting as a hop; a target page's own `rel=canonical` is
  a `canonical` link. **`links.redirected` is false on every row** (KD3): a
  link is redirected where its target's `redirect_chain` is not NULL. A
  fragment is checked against `targets.final_url` of the link's target, which
  names a `targets` row of its own, and from there the page's `anchors`.
- **Why P4 cares.** `bruce report` serves both as separate views: the broken
  report never lists a working redirected link, and a redirects report (D27
  drafted one) lists them as tasks. P4 decides whether a link to a page whose
  canonical names another URL belongs in that second view, and whether to drop
  the `redirected` column (KD3).
- **Origin.** P3's grilling, 2026-10-17 (`../wip/roadmap-P3-crawler.md`,
  "Redirects"), and P3.5.

## What a crawl leaves for the join to place

- **Fact.** A crawl writes `targets.reason` values the join must place:
  `unchecked` for `out_of_scope`, `host_not_allowlisted`, `scheme`, `robots`,
  `robots_unavailable` and `too_deep` (a sitemap past three files deep);
  `broken_url` for `invalid_url` (an href or sitemap `<loc>` resolving
  to no URL, the target's `url` the raw text and its `url_id` the hash of
  it), `redirect_loop`, `too_many_redirects`, `bad_redirect` and `nxdomain`;
  `error` for `unreadable` (a page whose HTML the extractor refused, or a
  sitemap file that is not one), the
  transport causes and, with its status kept, a 401, 403, 429 or 5xx; `ok`
  with `too_large` for a page over the cap, which has no `anchors`. An
  allowlisted page is a `pages` row and `anchors` with no `links` and no
  `page_facts`. `pages.via_links` and `pages.in_sitemap` record the
  discovery that claimed the page, so the sitemap and link joins are the
  authority for orphans and unlisted pages. A sitemap file is a target and
  each entry naming a page a `links` row of kind `sitemap` from it; entries
  outside every root's scope have no row. Each root links to each sitemap
  file it read, and an index to each child, by kind `sitemap_file` (D47), so
  a root with no such link had no sitemap to find orphans in, and a broken
  or `unreadable` file is a broken link from it.
- **Why P4 cares.** Its verdict for a fragment link depends on which of these
  its target is: `too_large`, a target that errored and an allowlisted URL
  first named by a non-anchor link (KD4) all have no anchors to check against.
- **Origin.** P3.5, P3.6 and P3.6.1, 2026-10-18.

## An interrupted crawl leaves an unfinished run

- **Fact.** A Ctrl-C'd `bruce crawl` flushes what it fetched and leaves
  `runs.finished_at` NULL; no run resumes, and its rows stay (D2). Its
  links can name targets it never wrote, the stop having cut them.
- **Why P4 cares.** A report's default run, and a diff's pair, should be the
  latest *finished* ones, and `prune` decides what becomes of unfinished
  runs.
- **Origin.** P3's grilling, 2026-10-18 (`../wip/roadmap-P3-crawler.md`,
  "An interrupted run"); the cut targets, P3.7.1.
