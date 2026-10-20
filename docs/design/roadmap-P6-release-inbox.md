# P6 inbox

Facts addressed to the release and deployment-guide phase before it is
grilled. Drained when it is, and then deleted (`../process.md`, "Inboxes:
facts filed by destination").

## A full en-US crawl takes nearly two hours, hosts nearly in turn

- **Fact.** P4.10's crawl of docs.fp.o `/en-US/` with the forge allowlisted,
  from the guide's config, took 1:48:22 wall clock and 161 MiB. The hosts ran
  nearly one after the other: the sitemap's pages took the first 39 minutes,
  and the forge's 4155 requests at one a second began then, so the time is
  close to their sum. P4.4's run, before the guide's skip glob was fixed, took
  2:20:42.
- **Why P6 cares.** The guide's scheduled-crawl section sets how long a night
  the crawl may take and what a deployer is told to expect; whether the hosts
  could overlap (the frontier's order, not read) changes that figure.
- **Origin.** P4.4 and P4.10, 2026-10-19 (`roadmap-P4-reports-notes.md`,
  "Reproducing the evidence"). Contingent on the forge staying at one request
  a second and about two requests a docs.fp.o page.

## A baseline of the en-US crawl is 27 MB, 5.9 MiB gzipped

- **Fact.** `bruce export-baseline` of P5.2's en-US run (172452 targets, 6435
  pages, 588552 links) wrote 27013120 bytes, which `flate2` at its default
  level takes to 6137905 (22.7%); `gzip` gives 6103638. The export takes
  about 2 s and 200 MB.
- **Why P6 cares.** The guide's publishing section says whether a deployer
  serves the file gzipped, and `check-dir --baseline` already decompresses a
  `.gz` path or URL (`roadmap-P5-ci.md`, "`export-baseline` and `--baseline`");
  at 5.9 MiB a CI job downloads it in seconds, so gzip is the recommendation
  the numbers support. Trimming the copy (chrome nav links repeat on every
  page) is not needed for size.
- **Origin.** P5.3, 2026-10-20 (`runs/p5.3-baseline/baseline.duckdb`).
  Contingent on the site's size and the schema: each new column grows it.

## A content repo's CI check of 236 pages takes nine seconds

- **Fact.** quick-docs, built by Antora 3.1.15 with the vendored UI
  (25 s with the fetches of three other repos) and checked by `check-dir`
  against the 27 MB baseline with the guide's config, took 8.9 s for 236 pages
  and 1455 targets, with no external request: only the forge is allowlisted, so
  781 external links were recorded `unchecked`, and the one live request was
  a 404 on a link the change planted.
- **Why P6 cares.** The guide's CI section says what a repo's job costs and
  that the allowlist, not the baseline, decides whether it reaches out; a
  repo allowlisting more hosts pays their pace for each link the baseline
  lacks. It also has to say the build must set `CI=true`, which drops the
  `file://` edit link from the pages (`roadmap-P5-ci.md`, "Facts found while
  grilling").
- **Origin.** P5.11, 2026-10-20 (`runs/p5.11-quick-docs/`). Contingent on the
  build holding few links the baseline lacks.
