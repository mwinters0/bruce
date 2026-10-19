# P3 The crawler: notes

What P3 left for later phases, and what it tried and refused. Its intent is
[`roadmap-P3-crawler.md`](roadmap-P3-crawler.md); what it built is the code,
[`../status/STATUS.md`](../status/STATUS.md)'s "What exists" and the concept
map.

## Reproducing the evidence

- **Tests run against wiremock.** `localhost` and `127.0.0.1` are two hosts
  to a crawl and one server, which is how a test tells an allowlisted host
  from a root's. A dropped `MockServer` keeps answering (it returns to
  wiremock's pool), so a test needing a port nobody answers on binds and drops
  a `TcpListener`. Wiremock answers an unmatched path 404, so every crawl test
  probes two `sitemap.xml`s and writes nothing for them.
- **A TCP-refused allowlisted host cannot be reached at the first hop**: its
  robots.txt fails first and the host closes (`robots_unavailable`, D8). A
  redirect from a reachable port of the same host is how a connect failure is
  tested.
- **The live crawl** of `/en-US/fedora-docs/`, 2026-10-18 at `010f25a`, is
  `runs/p3.8-live-20261018/`: `bruce.toml` (`crawl.log`, `live.duckdb`)
  discovers its sitemap; `bruce-sitemap.toml` (`crawl-sitemap.log`,
  `live-sitemap.duckdb`) sets it to I1's `/en-US/sitemap.xml`; `sm24.txt` is
  the 24 component pages of the grilling's `all-locs.txt`. Default UA,
  allowlisting `forge.fedoraproject.org` and `gitlab.com`.

  | | discovered | set |
  |---|---|---|
  | Summary | 1827 targets, 84 pages, no sitemap | 1967 targets, 87 pages, 69 sitemap files, 2283 entries dropped |
  | Wall clock | 1:44 | 2:54 |
  | Peak RSS | 54 MiB | 55 MiB |
  | `ok` / `broken_url` / `error` | 134 / 3 / 0 | 207 / 3 / 0 |
  | `unchecked` | 1650 `out_of_scope`, 39 `host_not_allowlisted`, 1 `robots` | 1717, 39, 1 |

- **Observed once, not tests** (each needs the network or a signal that would
  reach every test): `*.invalid` and an unregistered `fedoraproject.org`
  subdomain classify `NxDomain` though the resolver tried the search domain
  first, and `expired.badssl.com` classifies `Tls`. Against a local server
  holding three requests for 40 s, the first `kill -INT` ended the run 28 s
  later (the requests' own 30 s timeout, then no retry) with `1 targets, 1
  pages, no sitemap, 3 cut`; a second `kill -INT` a second later exited at
  once, the root's row flushed and `finished_at` NULL.

## What P4, verdicts and reports, inherits

- **A redirect chain** is `targets.redirect_chain`, `STRUCT(url, status,
  refresh)[]` in canonical forms, the strings a `url_id` hashes, so it joins
  `targets` without reparsing; NULL for a target not redirected, never an
  empty list. A 3xx hop has `refresh = false`; a delay-0 refresh is a hop of
  its own with the page's 200 and `refresh = true`. `final_url` is the
  landing URL, not a hop: NULL when not redirected, and NULL with `status`
  NULL and `broken_url` for a chain that never lands (a loop, past 10 hops).
  A chain ending at a failure or at robots.txt gives the landing URL a row of
  its own with the same outcome (D46). A hop's fragment is dropped.
- **`links.fragment` is as written**, percent-encoded; decoding for the match
  (D12) is the join's. `links.redirected` is false on every row (KD3).
- **An allowlisted page's `pages` row has no `source_*`**, profile `auto`, its
  hash over the whole page. `LinkRow`'s `Outcome` is left NULL for the join
  (D28). Where the join places each `reason` is P4's spec,
  `../wip/roadmap-P4-reports.md`, "Where the fragment cannot be checked".
- **`bruce crawl` exits** 0 for a finished run, broken links or not; 1 when
  the config, DB or crawl failed; 2 on a usage error; 130 when stopped. A run
  row exists only once the config, DB and fetcher are good. A stopped run's
  "cut" (claimed, never written) is a lower bound on what it lacks (D48).
- **From the live crawl** (table above): every sitemap page fetched with the
  sitemap set, 24 of 24; the alias `/en-US/fedora-docs/` a 200 whose chain is
  one refresh hop to `team-info/` (I14); no `error`. The three `broken_url`s
  are each a 404 to `curl`: `fedora-docs/content/` and
  `fedora-docs/content/style-guide/`, hand-written links missing the
  `contributing/` module, and `forge.fedoraproject.org/docs/tickets/projects/407`.
  Of 169 fragment links, 8 name an id absent from a fetched target, each
  absent to `curl` too: 7 `prerequisites` or `_prerequisites`, and one I6
  unresolved xref (`#./index.adoc#Prerequisites`). Each forge "report an
  issue" link is a 303 to `/user/login`, `ok`. Completions ran a median
  1.0 s apart on docs.fp.o and on the forge (`Crawl-delay: 1` both);
  gitlab.com's `Crawl-delay` is commented out, so it gets the default 2/s.
- **A component root does not discover docs.fp.o's sitemap**: neither
  `/en-US/fedora-docs/sitemap.xml` nor `/sitemap.xml` exists (I1), so a
  link-only run misses the one page nothing links to,
  `contributing/content/contrib-existing-documentation/`, the orphan D14's
  report exists for. A root at `/en-US/` finds the index by its probe; a
  narrower root must set `sitemap`.
- **`pages.via_links` is false for every listed page**, 23 of the 24 being
  linked too: the claimant's flag. An orphan report joins `links` (P4's spec, "What each report holds").
- **The forge costs about four requests a docs.fp.o page** when allowlisted
  (P4's spec, "The forge, and links skipped by config").
- None of KD3, KD4 or the stop's paths showed in the live crawl.

## What P5, CI mode, inherits

- **A local fetcher implements `Fetcher::fetch` alone**: `access` (allowed),
  `sitemaps` (none) and `with_stop` (ignored) are defaulted on the trait.
  `Mode::Page` reads a 2xx HTML body, `Mode::Asset` reads none, and
  `Mode::Sitemap` reads raw bytes whatever the `Content-Type`; a 3xx is
  returned as the answer, the crawler following hops.
- **An extractor failure** is `error`, reason `unreadable`.

## Calls beyond the spec

None touches a contract; each is a place a later reader may ask "why":

- A 429 with no `Retry-After` is retried on the backoff schedule; a 429 or
  503 with one up to 60 s waits that long for one retry, and over it returns
  unretried. A `Retry-After` stays one target's wait, never the host's pace.
- An asset whose HEAD fails transiently is asked again as a GET; one failing
  for good (NXDOMAIN, TLS, refused) is not. A response that is not HTTP is
  `Reset`, and retried like a dropped connection.
- Compression is gzip and deflate, not brotli: docs.fp.o answers gzip.
  reqwest's system-proxy lookup is off; its environment proxies apply. The
  cap is applied while reading after decompression, so a gzip bomb stops at
  16 MiB, and `Content-Length` is not consulted.
- Only the first 512 KiB of robots.txt is read; a file `texting_robots`
  refuses allows everything; a `Crawl-delay` over an hour is capped and a
  negative or non-finite one ignored. More than five redirects, a 3xx with no
  usable `Location`, or a status outside 2xx–5xx for robots.txt closes the
  host. `robots = false` also skips its `Sitemap:` lines.
- Hosts are keyed by host name, so two origins of one host share one pace,
  permit pool and robots.txt.
- At most 64 targets are in flight a run, and the writer channel holds 4096
  rows. An index's sitemaps are followed wherever they are, scope applying to
  pages only. Sitemap depth counts files, three deep at most (`too_deep`). A
  `<loc>` that is no absolute URL is `broken_url`, `invalid_url`, as an href
  is.
- A robots.txt fetch the stop cut caches the host as unavailable for the rest
  of the run; no row is written from it. A third Ctrl-C does nothing, read
  from the code: tokio's SIGINT handler stays registered while the runtime
  that would deliver it is gone.

## Not done

- **The duplicate-root check compares `Root.url` as written** (`Config::parse`),
  so `/~x/` and `/%7Ex/` are two roots; the crawl's seeding claims by
  `url_id`, so the second seeds nothing.

## Negative results

- **The `url` crate leaves `%7e` and `%2f` as written** and does not uppercase
  hex, so the canonical form adds its own escaping.
- **duckdb 1.10506.1 refuses a list bound into an INSERT** as well as
  appended (UF1). A chain rendered to text and cast breaks on a URL holding a
  quote, pinned by `it's,here?a=1&b=[2]`.
- **Holding a run's links back until their targets settle**, to fill
  `links.redirected`, costs the whole link graph in memory, and D2 forbids
  updating them afterwards (KD3).
- **A TLS failure is not on `source()`'s path**: it is inside the
  `io::Error` a handshake raises, so the cause walk descends `get_ref`.
- **`mem::replace` on `Response::body`** dropped `TooLarge` with the text;
  the over-cap test caught it.
- **`quick-xml` 0.42 reports `&amp;` as a `GeneralRef` event**, not inside
  the text, so a `<loc>` is assembled from text, CDATA and references.
- **Aborting a task in `drive` at the grace** would leave its rows half sent;
  the bound is per target instead (D48).
- **With `Writes::close` removed**, a 20 000-link page through a one-row
  writer channel lost all its links on three runs out of three, so the
  second Ctrl-C's hazard is not rare (D49).
