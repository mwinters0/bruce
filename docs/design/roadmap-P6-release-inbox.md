# P6 inbox

Facts addressed to the release and deployment-guide phase before it is
grilled. Drained when it is, and then deleted (`../process.md`, "Inboxes:
facts filed by destination").

## A full en-US crawl takes nearly two hours, hosts nearly in turn

- **Fact.** A crawl of docs.fp.o `/en-US/` with the forge allowlisted,
  from the guide's config, took 1:48:22 wall clock and 161 MiB. The hosts ran
  nearly one after the other: the sitemap's pages took the first 39 minutes,
  and the forge's 4155 requests at one a second began then, so the time is
  close to their sum. An earlier run, before the guide's skip glob was fixed,
  took 2:20:42.
- **Why P6 cares.** The guide's scheduled-crawl section sets how long a night
  the crawl may take and what a deployer is told to expect; whether the hosts
  could overlap (the frontier's order, not read) changes that figure.
- **Origin.** The first en-US crawls, 2026-10-19
  (`runs/p4.10-en-us-20261019-2125/` and `runs/p4.4-en-us-20261019-1633/`:
  `time.log` the wall clock and peak RSS, `HANDOFF.md` the launch,
  `targets.checked_at` the schedule, read with the `duckdb` CLI, the DBs being
  schema 3). Contingent on the forge staying at one request a second
  (its robots.txt's `Crawl-delay`) and about two requests a docs.fp.o page.

## A baseline of the en-US crawl is 27 MB, 5.9 MiB gzipped

- **Fact.** `bruce export-baseline` of an en-US run (172452 targets, 6435
  pages, 588552 links) wrote 27013120 bytes, which `flate2` at its default
  level takes to 6137905 (22.7%); `gzip` gives 6103638. The export takes
  about 2 s and 200 MB.
- **Why P6 cares.** The guide's publishing section says whether a deployer
  serves the file gzipped, and `check-dir --baseline` already decompresses a
  gzipped path or URL, whatever its name (`../manual/check-dir.md`);
  at 5.9 MiB a CI job downloads it in seconds, so gzip is the recommendation
  the numbers support. Trimming the copy (chrome nav links repeat on every
  page) is not needed for size.
- **Origin.** 2026-10-20 (`runs/p5.3-baseline/baseline.duckdb`, exported from
  `runs/p5.2-en-us-20261020/en-us.duckdb`; `README.md` there has the command).
  Contingent on the site's size and the schema: each new column grows it.

## A content repo's CI check of 236 pages takes nine seconds

- **Fact.** quick-docs, built by Antora 3.1.15 with the vendored UI
  (25 s with the fetches of three other repos) and checked by `check-dir`
  against the 27 MB baseline with the guide's config, took 8.9 s for 236 pages
  and 1455 targets, with no external request: only the forge is allowlisted, so
  781 external links were recorded `unchecked`, and the one live request was
  a 404 on a link the change planted.
- **Why P6 cares.** The guide's CI section says what a repo's job costs.
  The allowlist decides whether it reaches other hosts, a repo allowlisting
  more paying their pace for each link the baseline lacks; a link on the site
  the baseline lacks, D57's derived URLs included, is always asked of
  production, whatever `--externals` says (D22), so the job needs the network.
  It also has to say the build must set `CI=true`, which drops the `file://`
  edit link from the pages (`site-invariants.md`, I15), and that the job
  passes the deployment's config, published beside the baseline (D27).
- **Origin.** 2026-10-20 (`runs/p5.11-quick-docs/`; `README.md` there has the
  sources, the build and the check). Contingent on the build holding few links
  the baseline lacks.

## Every PR's gate lists each built page's home link as redirected

- **Fact.** A partial build's `index.html` refreshes to the build's start
  page, where production's is the site's, so every built page's home link is
  an introduced `redirected` row: 236 of quick-docs's 239. It never fails the
  gate. `scripts/e2e.py`'s `UNCHANGED_REDIRECTED` asserts it.
- **Why P6 cares.** The guide's CI section either tells a repo to expect it,
  or the gate drops a redirected link to the prefix when the build's
  `index.html` is a refresh, a change P6 would schedule.
- **Origin.** 2026-10-20 (`runs/p5.12-quick-docs/published.json`).
  Contingent on the playbook's start page.

## A scheduled `prune` cannot overlap the scheduled crawl

- **Fact.** While a crawl or a `duckdb` session holds the file, `prune` fails
  at open on DuckDB's lock, exit 1 (`../manual/prune.md`, "Not beside a
  crawl"). `CHECKPOINT` after pruning a DB of many runs is unmeasured.
- **Why P6 cares.** The guide's scheduling section sequences the two jobs.
- **Origin.** 2026-10-19, `prune`'s landing. Contingent on DuckDB's one
  writing process per file (D28).

## Where the release binary is published is undecided

- **Fact.** The repo's remote is GitHub (`mwinters0/bruce`); D23 says "a
  release binary on the forge", and no decision names the forge or repo that
  hosts releases.
- **Why P6 cares.** The release, and the guide's CI step fetching the binary
  in one step (D23), need the URL.
- **Origin.** The bootstrap, 2026-10-17 (`historical/initial.md`, "Open
  questions"). Contingent on nothing.

## The guide's docs.fp.o root sets no hash scope

- **Fact.** The guide's root sets neither `hash_root` nor `hash_exclude`, so
  its pages hash `article.doc` alone (D15), blind to a changed stylesheet or
  script link. The extractor's spec meant the guide to show a whole-page
  scope less `hash_exclude = ["footer"]`, the footer holding the
  `Last build:` line, and it never did.
- **Why P6 cares.** The guide decides the example's scope; P8's `changed`
  report reads whatever it decides (P8's inbox).
- **Origin.** The extractor's grilling, 2026-10-17. Contingent on the footer
  holding the build stamp.
