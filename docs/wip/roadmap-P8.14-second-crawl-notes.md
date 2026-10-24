# P8.14 — The second en-US crawl: notes

No code. `runs/p8.14-en-us-20261024-1322/en-us.duckdb`, run 1, schema 5,
finished, exit 0, from a release build of 6baecbc under the deployment guide's
config (`mise run sim`), `HANDOFF.md` beside it. **Wall clock 1:11:00**
(`time.log`; 8.1's 1:10:33, P5.2's 1:48:20), the allowlist's widening not
lengthening it. The run line: 8,990 pages, 173,288 targets, 69 sitemap files.
It is the database 8.6's review resumes on (its notes' "Where the review
stands") and the run 8.8 compares with 8.1's
(`runs/p8.1-en-us-20261023-2004/en-us.duckdb`).

## Against 8.1's run

| | 8.1 | 8.14 |
|---|---|---|
| pages | 6,436 | 8,990 |
| forge pages (`root IS NULL`) | 4,116 | 6,667 |
| en-US pages (`root` set) | 2,320 | 2,323 |
| targets | 172,453 | 173,288 |
| `report broken` | 827 on 324 pages | 1,973 on 588 pages |
| of which `broken_url` / `broken_anchor` | 217 / 610 | 1,259 / 714 |
| `report errors` | 0 | 90 on 61 pages |
| `unchecked` links | 185,116 | 168,623 |

`report stats`' unchecked reasons, links: `out_of_scope` 157,827 to 155,440,
`host_not_allowlisted` 25,051 to 10,406, `skipped` 1,958 to 1,961, `scheme`
280 to 280, and new `robots` 535 and `robots_unavailable` 1.

## What moved, by target (joined on URL)

- **2,830 `host_not_allowlisted` to `ok`**, 306 to `broken_url`, 48 to
  `error`, 26 to `broken_url/nxdomain`, 297 to `unchecked/robots`: the
  allowlist's widening (8.12), these checked and not new breakage.
- **`error` causes of those**: `timeout` 5, `tls` 4, `connect` 1, and the
  rest a host's 401, 403 or 503 (lists, fedoramagazine, os, communityblog,
  forgefiler): 90 `error` links in all, from 0.
- **`out_of_scope` to `ok` 7, to `broken_url` 5**: a URL on a root's host
  outside every scope, asked once (8.11).
- **`scheme` to `broken_url/bad_scheme` 2** (8.10); 280 stay `unchecked`.
- **Every 8.1 `broken_url` and `broken_anchor` link survives with its
  verdict**, none gone and none flipped: the 827 are unchanged, so any
  growth is a link or a target 8.1 did not ask.
- **By link (page, target, fragment), 8.1's `unchecked` to broken**:
  `broken_url` 460 from `host_not_allowlisted`, 29 `nxdomain`, 6 from
  `out_of_scope`, 2 `bad_scheme`; `broken_anchor` 74; `error` 83 (72 with a status,
  5 `timeout`, 5 `tls`, 1 `connect`). 529 `broken_url`, 1 `broken_anchor`
  and 1 `error` are on links 8.1 did not hold, the 529 below.

## The three double-slash pages

529 of the 1,259 `broken_url` links, and 3 of the 3 new en-US pages, are
`.../en-US/project/code-of-conduct//`, `.../code-of-conduct//committee/` and
`.../code-of-conduct//clarifying-statements/` (not in the sitemap, reached by
links). `https://fedoraproject.org/code-of-conduct`, newly checked under the
allowlist, redirects (302) to the first, with the doubled slash in the site's
own `Location`. The crawl follows it onto a root's host, as it follows any
redirect there, and the pages' relative links (`../../../ar/project/…`)
resolve one directory short, so about 170 per page are `broken_url`. They are
real for a reader arriving by that redirect and not a mistake of ours, but
they are not en-US content drift either: 8.8 reads the rows of those three
pages as new, and the figures without them are 730 `broken_url` and 1,444
broken links in all. Whether pages reached only by such a redirect belong among a root's
pages is not decided here.

## `robots_unavailable` is one link

`jenkins.fedorainfracloud.org` (one target, one link on
`en-US/cle/sle_services/`). It answers, but its `/robots.txt` is a 302 to
`jenkins-fedora-infra.apps.ci.centos.org//robots.txt`, another name that gives
no response (checked with curl after the run): 8.13's and 8.15's "a hop onto
another name that gives no response", so the host's links are `unchecked`, as
those slices specify. Not a misjudgement of a host that answers with a file,
which the handoff doc named the failure to look for. `nxdomain` is
36 links on 11 hosts (`www.redhat.com`, `ci.centos.org`, `design.` and
`teams.fedoraproject.org`, and others).

`unchecked/robots` is 537 links on 297 targets, the largest hosts
`bugzilla.redhat.com` 111, `fedorapeople.org` 48 and `dl.fedoraproject.org` 29.

## What 8.6 and 8.8 inherit

- **8.6 serves this database**: `target/release/bruce report digest --db
  runs/p8.14-en-us-20261024-1322/en-us.duckdb --out runs/p8.6-review/digest
  --name en-us`, then the server its notes give.
- **8.8's two runs differ in pages as well as verdicts.** The 2,551 forge
  pages and the allowlist's hosts are in run 2 only; compare `root IS NOT NULL`
  pages for content drift and read the others as newly checked.
- **The two runs are in two databases**; reading across them is 8.8's
  `--since`, which this slice does not try.
