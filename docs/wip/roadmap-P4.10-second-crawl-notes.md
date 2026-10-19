# P4.10 The second en-US crawl: notes

The run is `runs/p4.10-en-us-20261019-2125/`: `en-us.duckdb` (one run,
`run_id` 1), `crawl.log`, `time.log` and `HANDOFF.md`, from `sim/bruce.toml`
with the finished tool (HEAD 2547df1). It passes: exit 0, `finished_at` set,
and all 2307 distinct sitemap pages fetched, `ok` with status 200. Beside it,
`combined.duckdb` holds P4.4's run as run 1 and this one as run 2, and `out/`
holds each report's text on run 2 (both gitignored with `runs/`).

## The table, as P4.4's

| | P4.10 | P4.4 |
|---|---|---|
| Summary | 172452 targets, 6433 pages, 69 sitemap files | 172239, 6425, 69 |
| Wall clock | 1:48:22 | 2:20:42 |
| Peak RSS | 161 MiB | 168 MiB |
| DB size | 27.5 MiB; 588551 `links`, 93521 `anchors`, 8938 `page_facts` | 30 MiB; 587839, 93421, 8926 |
| CPU | 16.6 s user, 2.2 s system | 20.0 s, 3.7 s |
| `ok` | 7307, all status 200; 61 reached through a redirect | 7299; 61 |
| `broken_url` | 109: 77 on docs.fp.o, 32 on the forge; all 404 | 100: 68 and 32 |
| `error` | 2, `other`, both forge `commits/` URLs | 0 |
| `unchecked` | 154581 `out_of_scope`, 8397 `host_not_allowlisted`, 1928 `skipped`, 128 `scheme`; no `robots` | 154380, 8397, 1925, 128, and 10 `robots` |
| Pages | 2320 `antora` (root `/en-US/`), 4113 `generic` (root NULL) | 2317, 4108 |
| Requests | 3263 docs.fp.o, 4155 forge | 3251, 6056 |

- **The robots changes did what P4.3.1 said.** `fesco/Fedora_Council_Engineering_Rep/`
  is fetched (`ok`, 200), and the nine `robots` 404s of P4.4 are `broken_url`,
  so no target is `robots` and `broken_url` rose by exactly those nine.
- **The glob fix did what P4.3.1 said.** The forge got 4155 requests against
  P4.4's 6056 (4148 answered then), and 1928 targets are `skipped` against 1925.
  `redirected` links fell from 2009 to 90.
- **The hosts overlapped once, not throughout.** In 20-minute buckets from
  21:20 UTC, docs.fp.o starts were 911, 1199, 481, 225, 218, 229 and the
  forge's 0, 0, 984, 1198, 1198, 775: the forge began at 22:04, 39 minutes
  in, while docs.fp.o's assets trickled on. The wall clock is about 39 + 69
  minutes, the forge's one start a second after the pages, as P4.4's
  notes expected. `duration_ms` is still queueing time (forge mean 61 s,
  max 190 s).
- **The two `error`s are transient.** Both are forge `commits/branch/main/…`
  URLs that were `ok` / 200 in P4.4, ended with no status after 45 to 48 s,
  and carry `other`; `targets` holds no message, so the cause is not known.
  They are in `errors` and `stats` and in no `diff`, which reads `broken` only.
- **Run-to-run drift is small.** 10 pages only in run 2, 2 only in run 1; no
  target of run 1 is missing from run 2; the 610 `broken_anchor`s are the same
  count; `orphans` is 74 and `unlisted` 13 in both runs.

## The reports, on run 2 of `combined.duckdb`

The sitemap, the inbound ranking and both broken stats are the deliverable
(D41: evidence only, none reported upstream from here).

- **`sitemap`**: 2320 rows, 2307 in the sitemap and `ok`, 13 `via_links` only
  (the `unlisted` ones, all `/mentored-projects/gsoc/…/index.html`).
- **`inbound`**: 2320 pages ranked; the top four have 2319 (every other page):
  `/docs/`, `/legal/`, `/legal/privacy/`, `/project/code-of-conduct/`, then 2315
  for at least seven component indexes.
- **`stats`**: 2320 pages, 552924 anchor links, 217 `broken_url`, 610
  `broken_anchor`, 2 `error`, 185117 `unchecked` (out_of_scope 157827,
  host_not_allowlisted 25052, skipped 1958, scheme 280). The worst page is
  `/legal/not-allowed-licenses/`, 112 broken anchors and 1 broken URL; the legal
  pages hold most of the 610.
- **`broken`**: 827 rows; `errors`: 2; `redirects`: 61 landing URLs, led by a
  `refresh` hop (`bugzilla-file-a-bug`, 5 links) and the forge's `/user/login`.
- **Timing**, `Store::open` installing every view, one process each on
  `combined.duckdb` (two runs): `sitemap` and `unlisted` 0.5 s and 66 MiB;
  `broken`, `inbound`, `redirects` and `orphans` 0.6 to 0.7 s and 276 to
  484 MiB. `link_verdicts` over this run alone: 0.37 s cold, grouped by outcome
  and reason, as on P4.4's (D28 stays unmaterialised).

## `diff` against P4.4

`bruce report diff --db combined.duckdb` (defaults: run 1 to run 2) prints 9
`new` and no `fixed`. The nine are the `robots` 404s above: links from
`releases/f1` to `f7`, `gsoc/2016/ideas`, `infra/join_guide/apprentice` to
`Fedora/<n>/html/…` and `Community_Services_Infrastructure/…`. They are new
only because P4.3.1 stopped skipping them: in run 1 each was `unchecked` /
`robots`, in run 2 it is `broken_url`.

- **P4.8 asked where the need for a state column shows. It shows here.** All
  nine rows read as links that broke between the runs, and the base state
  (`unchecked`, `robots`) was a choice about what to ask. Likewise the 2 forge
  `error`s, `ok` in run 1, are in no row. The nine are the only rows the
  pair has, so the column that would say "was `unchecked`" costs one join to
  `link_verdicts` and would answer the one question the first real `diff` raised.
  Whether to build it is not decided here; P5's gate, which reads a `diff` as
  introduced breakage, is its first consumer.
- Between runs 138 pages changed hash, 137 of them forge pages; filed in P8's
  inbox.

## Negative results

- No `no_anchors`, `text_fragment` or `cut` link again (KD4 and the stop's
  paths did not show).
- **P4.4's notes were wrong that nothing on docs.fp.o redirected.** Of the 61
  `ok` redirected targets, 55 are docs.fp.o URLs (a 302 or a `refresh`) and 6
  the forge's, in both runs.
- `diff` found nothing in content: of docs.fp.o's pages the hash moved on one.

## Reproducing

```sh
cd runs/p4.10-en-us-20261019-2125
cp ../p4.4-en-us-20261019-1633/en-us.duckdb combined.duckdb
duckdb combined.duckdb "attach 'en-us.duckdb' as n (read_only);
  insert into runs select 2, * exclude (run_id) from n.runs;" # likewise pages,
# page_facts, targets, anchors, links and bodies
../../target/release/bruce report diff --db combined.duckdb
```
