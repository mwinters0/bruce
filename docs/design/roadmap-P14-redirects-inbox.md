# P14 inbox

Facts addressed to the phase reading redirects and discovery from the store's
facts before it is grilled. Drained when it is, and then deleted
(`../process.md`, "Inboxes: facts filed by destination").

## The page flags are a claim, not a fact

- **Fact.** `pages.in_sitemap` and `via_links` are the flags of the target
  that claimed the page first (`crawl::Shared::write_page`), so no page has
  both: in `runs/en-us-20261101-0005/en-us.duckdb`, 2,310 pages are listed,
  16 linked and none both, in each run. A page a redirect lands on takes the
  redirecting target's (KD38). None of docs.fp.o's 2,310 sitemap entries
  redirects in either run, so KD38 costs it nothing today.
- **Why P14 cares.** Deriving listed and linked from `links` and
  `targets.final_url` drops two columns whose only reader is `report_pages`
  and the reach reports (`report::reach`, `report::views`'s `SITEMAP`); the
  docs.fp.o figures are what a derived view must reproduce.
- **Origin.** P10's wrap repoint, 2026-11-01, read off the 10.8 crawl.
  Contingent on the flags still being written at claim time.

## The chain is already stored, once per target

- **Fact.** `targets.redirect_chain` holds every hop (URL, status, refresh)
  and `final_url` the landing, written once per target per run (D46), and
  `report_inbound` already counts a link for the page it lands on. The 10.8
  crawl's run 2 has 798 redirected targets: 646 of one hop, 131 of two, 14
  of three, 6 of four and 1 of five.
- **Why P14 cares.** A chain's length is `len(redirect_chain)`, so the
  configurable limit needs no new storage, and the distribution is what a
  default would flag on docs.fp.o: a limit of one flags 152 targets, of none
  all 798.
- **Origin.** P10's wrap, 2026-11-01, read off
  `runs/en-us-20261101-0005/en-us.duckdb`. Contingent on the chain's shape
  (`store::schema`).
