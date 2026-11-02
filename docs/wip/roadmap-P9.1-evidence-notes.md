# P9.1 — Evidence: notes

What 9.2 to 9.9 inherit, and the negative results. No code landed. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The slice order".

## What 9.1 measured

- **A fresh en-US crawl under schema 7** (`runs/en-us-20261102-2011/`, the
  guide's config at one a second, 20:11 to 21:27 UTC, exit 0): **1:16:15** wall,
  **295 MiB** peak RSS (`time.log`), 24 s of CPU; 174,599 targets, 9,055
  pages, 68 sitemap files; the database 32,780,288 bytes (31.3 MiB). The
  database is kept for 9.2's refresh, the review (9.8) and a `max_redirects`
  trial (`report --config`, P14's inheritance).
- **The guide's figures are stale.** The 1:48:22 and 161 MiB the spec carries
  ("What the guide states") are neither this crawl's nor the previous one's
  (`runs/en-us-20261101-0005/time.log`, the same config: 1:15:49, 362 MiB,
  173,502 targets). 9.4 re-measures them when its section is written, as the
  spec says; the 33 MB a run holds.
- **Pruning alone holds a steady size** (`runs/p9.1-evidence/prune/sizes.tsv`):
  one finished run cloned as 70 successive nights under `prune --keep 30`, the
  file grew 28 MB a night until 30 runs were held and then **did not change
  by a byte for 40 nights**, at 845,426,688 bytes. A second pass whose clones
  each kept a random 70 to 100% of every table's rows, so nights differ in
  size and layout, held 732,966,912 bytes from night 30 on
  (`prune-varied/sizes.tsv`, `VARY=1`). `prune` gives almost nothing back (the
  file after is at most 0.5 MB under the file before) and the freed blocks are
  reused. **9.4's script (spec S5) therefore does not copy the DB fresh.**
  Thirty runs are 0.7 to 0.85 GB, under the spec's "about 1 GB".
- **Anubis does not challenge a GitHub runner under the guide's User-Agent**
  (`runs/p9.1-evidence/anubis-probe.log`, workflow `anubis-probe.yml`, run from
  `mwinters0/bruce-fedora-docs-ci` and since removed): from 20.55.213.112
  (Azure eastus2, ubuntu-24.04) 40 requests to docs.fp.o (3 a second) and the
  forge answered with no challenge, under the guide's and under curl's
  User-Agent; the same two hosts answered 200 with a 4 to 5 KB challenge page
  under a Chrome `Mozilla/5.0` one. The challenge is keyed on the User-Agent,
  not the address. Recorded as I27. pagure.io, fedoramagazine.org,
  getfedora.org, src.fedoraproject.org, redhat.com and bugzilla.redhat.com
  answered clean as well.

## What the next slices inherit

- **9.4**: S5 prunes and copies nothing; the weekly `--keep 8` DB is not
  measured here (an every-locale run is unmeasured until 9.9). The guide's
  `user_agent` stays the guide's (I27): a deployer who sets a browser-like one
  is challenged on Anubis-fronted hosts.
- **9.6**: the crawl's time on Actions is not yet known; here it is 1:16 on
  this machine, far inside the six-hour job.
- **The identical-clone test is the best a single crawl offers**: real nights
  differ in more ways than row counts. 9.6's nightly keeps no DB (the spec),
  so only 9.9's weekly DB is a real pruned history, and its size is the check
  on this finding.

## Negative results

- `prune-steady.sh` ran as written on the first try; its `VARY` mode is new.
  The duckdb CLI's SQL needed no change.
- The crawl log holds one line, the run line: a crawl prints nothing else
  at its default verbosity.
