# P4.4 The first en-US crawl: notes

The run is `runs/p4.4-en-us-20261019-1633/`: `en-us.duckdb` (one run,
`run_id` 1), `crawl.log`, `time.log` and `HANDOFF.md` (the launch command).
It is the release build run from `sim/bruce.toml`: one root
`https://docs.fedoraproject.org/en-US/` with its sitemap set. It passes: exit
0, `finished_at` set, and 2304 of the sitemap's 2305 distinct pages fetched,
the other being forbidden by docs.fp.o's own `robots.txt` (below). Every
number below comes from that DB (`duckdb -readonly`) or `time.log`.

## The table, as P3.8's

| | |
|---|---|
| Summary | 172239 targets, 6425 pages, 69 sitemap files (`crawl.log`) |
| Wall clock | 2:20:42 (estimate 77 min) |
| Peak RSS | 168 MiB (estimate about 110 MiB) |
| DB size | 31.5 MB (30 MiB); 587839 `links`, 93421 `anchors`, 8926 `page_facts` |
| CPU | 20.0 s user, 3.7 s system |
| `ok` | 7299, all status 200; 61 of them reached through a redirect |
| `broken_url` | 100: 68 on docs.fp.o, 32 on the forge; all 404 |
| `error` | 0 |
| `unchecked` | 154380 `out_of_scope`, 8397 `host_not_allowlisted`, 1925 `skipped`, 128 `scheme`, 10 `robots` |
| Pages | 2317 `antora` (root `/en-US/`), 4108 `generic` (allowlisted, root NULL) |
| Links by kind | 552256 `anchor`, 16219 `script`, 7718 `image`, 6951 `stylesheet`, 2317 `canonical`, 2307 `sitemap`, 69 `sitemap_file`, 2 `frame` |
| Links with a fragment | 16498; 92 with `xref_refspec` set |

`link_verdicts` does not exist yet; its timing is P4.5's. The broken anchors
are P4.5's to count.

## What the next slice inherits

- **The skip glob `issues/new*` misses most of the forge's links.** `*` stops
  at `/` (D44) and the query is matched, so a link whose query holds a `/`
  escapes: the edit-page `issues/new?title=[main]%20Doc%20issue%20in%20file%20modules/ROOT/pages/index.adoc`
  is the shape of 1908 of the 1925 `skipped` targets. They were asked, each
  answered 303 to `/user/login`, and the hop stopped at the second glob
  (`user/login*`, D50), ending `unchecked` / `skipped` with the chain
  recorded. The outcome is the one wanted; the cost is 1908 forge requests,
  32 minutes at 1/s, of 6056 the forge got. Only 16 `issues/new` targets
  matched the first glob (no query, or a query without `/`), and
  `issues/new/choose` matches neither. P4.3's "matched by hand" note had the
  shape wrong. Offline, with the crate's `GlobBuilder` as `Patterns::new`
  builds it, over the 1921 `issues/new` URLs in `targets`: the guide's glob
  matches 16; `https://forge.fedoraproject.org/**/issues/new?*/**` matches
  1904; with the guide's glob beside it, 1920. **The guide's glob is fixed
  before P4.10**, or the second crawl repeats this and its `diff` against
  this run compares like with like only by accident. The guide's "and no
  request is sent" (`docs/manual/deploy.md`) is false for those links until it
  is.
- **The other four `skipped` are what the second glob is for**:
  `/user/login` itself, and three `settings` URLs that redirect to it
  (`/org/%3Corganization_name%3E/settings`, `/user/settings/applications`,
  `.../tokens/new`), each chain one 303 hop.
- **A sitemap lists a page its own `robots.txt` forbids.**
  `/en-US/fesco/Fedora_Council_Engineering_Rep/` is a sitemap entry and
  matches `Disallow: /*/Fedora*/` (I2); it is `unchecked` / `robots`, no page.
  `curl` answers it 200. The crawl is right to leave it; "every sitemap page
  fetched" can hold only up to such a page, and the second crawl's check
  should count it as the one expected. The other nine `robots` targets are
  linked paths under `Fedora*` (release notes, an installation guide) and
  `Community_Services_Infrastructure/`.
- **The hosts ran one after the other.** `targets.checked_at` is the instant
  a target's hops began (`src/crawl.rs`), so its distribution is the
  schedule. In 20-minute buckets from 16:20 UTC, docs.fp.o starts were 481,
  1200, 855, 83, 77, 218, 217, 120 and the forge's 0, 0, 563, 1199, 1140,
  1192, 1191, 771: the sitemap's 2317 pages ran in the first 47 minutes, the
  forge from 17:11 to 18:52 at one start a second, as `per_second = 1`. The
  wall clock is nearly the sum of the two hosts' times, not the longer. The
  frontier's order, not a rate, is the likely cause; the code was not read
  for it. Of 3251 docs.fp.o requests, 2317 are pages and 69 sitemap files;
  the rest are assets (a `_images/` SVG or PNG), which kept trickling in
  after the pages.
- **The estimate's 77 minutes was the forge at two requests a page**: 4600
  at 1/s. The forge got 6056 (4148 answered, 1908 the redirects above), and
  fixing the glob gives 4148, 69 minutes. The wall clock with the glob fixed
  is about 47 + 69 minutes if the hosts still run in turn.
- **`duration_ms` includes waiting for the host's turn**: its mean is 49 s on
  docs.fp.o and 63 s on the forge, against one start a second. It is not
  a response time; a report that wants latency has no column for it.
- **Peak RSS is 168 MiB**, over the 110 estimate and three times P3.8's
  55 MiB, for 172239 targets against P3.8's 1967. It is a record, not a
  finding about any cap.

## Evidence only (D41)

The 100 `broken_url`s are all 404s; 99 are named by an `anchor` link and one
by an `image`. By component: `fedora` 28, `quick-docs` 10, `infra` 8,
`fedora-docs` 4, `commops` 4, the other 14 across a dozen more. Two forge
shapes among the 32: `.../-/issues` (GitLab's path) and `council/tickets/issue/N`
(`issue`, not `issues`). They are the deployer's to report; none is filed
from here.

## Negative results

- None of KD4's or the stop's paths showed. No target ended `error`.
- Nothing on the site redirected a docs.fp.o link, as in P3.8: every
  redirected target is a forge URL, 1908 of them skipped hops, 61 `ok`.

## Reproducing

```sh
duckdb -readonly runs/p4.4-en-us-20261019-1633/en-us.duckdb \
  "select outcome, reason, count(*) from targets group by all order by 3 desc"
```

The glob check is a throwaway program over the `globset` crate with
`literal_separator(true)`, fed `select url from targets where url like
'https://forge.%issues/new%'`; it is not kept.
