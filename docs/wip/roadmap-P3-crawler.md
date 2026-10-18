# P3 — The crawler

What P3 does and why. Its progress is
[`../status/STATUS.md`](../status/STATUS.md)'s checklist; the decisions are
in [`../design/decisions.md`](../design/decisions.md) and are cited here,
never restated.

## Intent

The URL frontier and canonicalisation (D7, D26), sitemap seeding (D14),
per-host rate limits, backoff and robots.txt (D8, D18), redirects and retries
(D7, D16), and the single writer's remaining tables (D28). Tested against
wiremock (D24).

## What P3 delivers, and where it stops

Settled 2026-10-17. **`bruce crawl` writes a whole run**: every target with
its HTTP-level outcome (D16 but `broken_anchor`), and the `pages`, `anchors`,
`links` and `page_facts` of every page fetched. **Link and fragment verdicts
are P4's** SQL join (D28): P3 leaves `links.outcome` NULL. Tested against
wiremock, **and it ends with a small, polite live crawl of one docs.fp.o
component**, so P4's 40-minute crawl is not the first contact with the real
site. Rejected: the engine as a library with no subcommand, which leaves the
inbox's CLI questions open and P4 to meet the site first.

## What gets requested

Settled 2026-10-17, by the URL's class:

1. **In a root's scope**: GET; an HTML response is extracted and its links
   followed.
2. **On an allowlisted host**: one hop deep (D3). An `anchor`-kind link gets a
   GET, extracted for its **anchors only**; an asset kind gets HEAD, falling
   back to GET (D13).
3. **Anything else**, out-of-scope URLs on a root's own host included, and
   `mailto:`, `tel:`, `javascript:`, `data:`: a target row `unchecked`, no
   request, its reason naming which case (`out_of_scope`,
   `host_not_allowlisted`, `scheme`).
4. **Disallowed by robots.txt**: `unchecked`, reason `robots`, no request.

Rejected: checking out-of-scope URLs on a root's host one hop deep. docs.fp.o's
language switcher alone would cost about 154,000 requests a run ("Facts found
while grilling"); a deployment wanting another locale checked makes it a root
(D10).

## Redirects

Settled 2026-10-17.

- **Hops are followed by hand**, reqwest's own following off, so each hop
  passes its host's rate limit and robots.txt and the chain is recorded on
  the target; D7's 10-hop limit counts every hop.
- **A 200 HTML page whose `refresh` has delay 0 is one more hop.** So a
  docs.fp.o alias is a target whose `final_url` is the real page, not a
  `pages` row, and a fragment on a link to it is checked against the final
  page. A refresh with a delay is an ordinary link: such a page has content of
  its own.
- **The crawler records; both uses are supported, with no setting.** Some
  users care only about broken links, others track every link through a
  redirect as a task to update it to its target. Each gets its own view
  rather than a configured switch (P4's and P5's inboxes); P3 records what
  both need. A target's own canonical is already a `canonical`
  link on its page, so "links whose target names another canonical" (a frozen
  version's page naming `latest`, P2's notes) is a join, not a crawler fact.

- **Each hop records its kind**: its status beside its URL, a refresh hop
  marked as one, so a redirects view tells a permanent 301/308 (update the
  link) from a temporary 302/307 (often the site's design, I5's `f44` →
  `latest`) and from an alias. I4's slashless chain mixes 301 and 302.
  `targets.redirect_chain` becomes a list of `(url, status)` structs, the
  schema version bumped (D43).

## The command line

Settled 2026-10-17, draining the inbox's second entry.

- **`bruce crawl [URL] [--config f] [--db f]`.** A URL with a file naming
  roots is **refused**, the message saying so; a URL with a file naming none
  is the only root, the file's other settings applying; neither is refused.
  The URL path re-runs `Config::check_robots_hosts`, and `cli` refuses a
  profile name the registry lacks (D39). Rejected: the URL replacing the
  file's roots, which silently drops their `hash_exclude` and `profile`; the
  URL adding a root. Refusal can be relaxed later without breaking anyone.
- **`--db` defaults to `bruce.duckdb`** in the working directory.
- **D5's CLI overrides wait for someone to ask**: P3 ships none, and the slice
  adding `crawl` amends D5 to say so.

## Config keys

Settled 2026-10-17, draining the inbox's first entry. Each lands in
`config.rs` and `../manual/config.md` in one change.

- **`[rate] concurrency = 4`**, the in-flight cap per host, and
  `[host."<h>"] concurrency` for one host (D8). It binds only on a slow host:
  at 2 req/s and about 0.3 s a response, one request is in flight.
- **`[[root]] strip_query = false`**: drops the query from the canonical form
  of URLs in that root's scope (D7), so `url_id` there depends on the root's
  own choice.

## The stack

Settled 2026-10-18 ("Facts found while grilling", "Crates").

- **tokio and async reqwest 0.13**, extraction on the blocking pool
  (`lol_html` is synchronous), the writer keeping its own thread (D28), fed
  from those blocking tasks so its bounded channel still pushes back.
- **rustls with ring**, not reqwest's default aws-lc-rs: a C build makes D23's
  self-contained binary harder to build for the Fedora CI image.
- **`hickory-dns`**, so NXDOMAIN (`broken_url`, D16) is a typed error rather
  than a match on the system resolver's message. Cost: about 30 crates, and a
  resolver reading `resolv.conf` itself.
- **`texting_robots`** for robots.txt. **The User-Agent is a top-level
  `user_agent` key, default `bruce/<version>`**, and the robots.txt token is
  derived from it: its first product name, up to the `/`, lowercased
  (`bruce`). A deployment naming itself is matched under that name, as a
  site operator writing robots.txt expects. Rejected: a `robots_token` key,
  which can disagree with the UA.
- **A per-host next-allowed instant, not governor**, which has no per-key
  quota (boinkor-net/governor#193). The slice building it amends D11.

## Retries, timeouts and limits

Settled 2026-10-18.

- **A 5xx, a timeout or a connection reset is retried three times**, after
  1, 2 and 4 s, stretched to the host's own interval where that is slower
  (D16).
- **429 and 503 honour `Retry-After` up to 60 s**; a longer one settles the
  target `error` at once rather than stalling the run.
- **Timeouts**: 10 s to connect, 30 s for the whole response.
- **An HTML body is read up to 16 MiB** after decompression. A page over the
  cap is **not** `error`: it answered, and calling it an error reads as a
  false positive. **It is `ok`, reason `too_large`**, and is not extracted,
  half a page's anchors giving false `broken_anchor`s; P4's join makes a
  fragment into it `unchecked`, reason `too_large`. Rejected: a sixth outcome
  (D16), which every report and the CI gate (D9) would have to place, for a
  URL that works.
- **robots.txt per RFC 9309**: a 4xx allows everything; a 5xx or no answer
  disallows the host for the run, its targets `unchecked` with reason
  `robots_unavailable`.
- The 60 s and 16 MiB limits are constants until someone needs them set.

## Which responses are read

Settled 2026-10-18, draining the inbox's fourth entry.

- **Only a 2xx `text/html` or `application/xhtml+xml` body reaches the
  extractor.** A 404's body is never extracted, so docs.fp.o's 404 page never
  becomes a `generic` page.
- **Any other response is classified by its status alone**: a link to a PDF,
  an image or any non-HTML target is checked like a page (D16), its body never
  read past the headers.
- **The crawler decodes the body**, `extract` taking a `&str`: BOM, then the
  header's `charset=`, then a scan of the first 1024 bytes for `<meta
  charset>` or `http-equiv`, then UTF-8, invalid bytes becoming U+FFFD, with
  `encoding_rs`. Rejected: `extract` taking bytes and letting `lol_html`
  switch at the meta tag, which reworks P2's tested signature for no case we
  have.

## Assets and allowlisted pages

Settled 2026-10-18.

- **An asset's HEAD answered 4xx or 5xx is retried once as a GET**, its body
  unread (D13): some servers refuse HEAD with 403 or 405, and a false
  `broken_url` costs more than a request. A 2xx or 3xx HEAD is the answer.
- **A page fetched one hop deep on an allowlisted host** gets a `pages` row
  (title, profile, hash) and its `anchors`, **no `links` and no
  `page_facts`**: its links were never followed, and recording them would
  have orphan and inbound reports count a site nobody crawls.

## Sitemaps

Settled 2026-10-18 (D14, D40).

- **An index is followed into its children**, nested at most three deep; a
  `.xml.gz` file is decompressed (`flate2`, already in reqwest's tree).
  `<lastmod>` is ignored: docs.fp.o stamps every entry with the build time.
- **An entry outside every root's scope is dropped**, the run printing how
  many, a sitemap being broader than its root (D10's en-US root).
- **Each entry is a link of a new kind, `sitemap`**, its source the sitemap
  file, itself a target. A broken entry then shows in the join any broken
  link does, and orphans and unlisted pages (D14) are joins too;
  `pages.in_sitemap` stays as a convenience. Without it a broken entry has
  nowhere to show, only an HTML page earning a `pages` row. The slice amends
  D13's kinds.
- **A sitemap not found or not fetched never stops the run**: the crawl
  follows links alone, and the run records that it had no sitemap, so P4's
  orphan report can say it has nothing to go on.

## The canonical form

Settled 2026-10-18, draining the inbox's third and sixth entries (D7, D26,
D44).

- **`%XX` is decoded only where the byte is unreserved** (`A–Z a–z 0–9 - .
  _ ~`); every other escape stays, its hex uppercased, so a `%2F` never
  becomes a `/` that moves the path. The host is punycode as `url` gives it,
  an empty path is `/`, dot-segments are resolved, the fragment goes to
  `links.fragment`, and the query is dropped where the root sets
  `strip_query`.
- **`url_id` is `xxh3_64` of that string**, the crate P2 already uses.
- **`include` and `exclude` match the same string**, so a glob written with
  a decoded character matches the URL a browser shows; one written with an
  escape the form decodes never matches, and the manual says to write globs
  as the canonical URL reads. The slice amends D44 and tests a glob written
  with a decoded character.

## An interrupted run

Settled 2026-10-18. **The first Ctrl-C** stops new requests, lets those in
flight finish (the 30 s timeout bounds it), flushes the writer and leaves
`runs.finished_at` NULL; **a second** aborts. **No run resumes**: an
unfinished run's rows stay (D2) and the next run starts afresh. Rejected: a
persisted frontier, much machinery for a 40-minute job. Which run a report
reads by default is P4's (its inbox).

## Where the code lives

Settled 2026-10-18, along D39. Each file enters `../design/concepts.md`
with the slice creating it.

| Path | Holds |
|---|---|
| `url.rs` | the canonical form and `url_id` |
| `fetch.rs` | D1's fetcher trait and one hop's response |
| `fetch/http.rs` | reqwest: one hop, retries, `Retry-After`, the size cap, an error's D16 cause |
| `fetch/http/host.rs` | per-host rate, concurrency and robots.txt |
| `fetch/charset.rs` | body to `String`, which P5's local fetcher reuses |
| `crawl.rs` | the run: seeding, the workers, feeding the writer |
| `crawl/frontier.rs` | deduplication by `url_id`, and "What gets requested"'s classes |
| `crawl/hops.rs` | following hops, 3xx and refresh alike, and the 10-hop limit (D7) |
| `crawl/sitemap.rs` | `quick-xml`, indexes and gzip |
| `crawl/outcome.rs` | D16's classification |
| `store/writer.rs` | the `targets`, `anchors` and `links` rows, the schema version bumped |
| `config.rs`, `cli.rs` | the new keys; `crawl` |

**Hops are followed in `crawl`, not `fetch`**: a refresh is found by reading
the page, and P5's local build has the same static aliases with no HTTP
behind them.

## Facts found while grilling

Gathered 2026-10-17: a live probe of docs.fp.o (about 94 requests, 1 s apart)
and a crate survey built under Rust 1.99.1. Commands, bodies and the probe
project are under `runs/p3-grill/` (`probe-20261017/`,
`cratesurvey-20261017/`). What a decision comes to rely on moves to
`../design/site-invariants.md` with it.

### docs.fp.o

- **robots.txt** is unchanged from I2 and has no `Sitemap:` line; the
  Disallows are case-sensitive, so `/en-US/fedora/` is not covered.
  Re-verify: `curl -s https://docs.fedoraproject.org/robots.txt`.
- **The sitemap** is I1's index of 68 uncompressed `text/xml` children, 2307
  `<loc>`s, 2305 distinct (two repeated inside one child), every one under
  `/en-US/`, ending in `/`, none under a Disallow. Every `<lastmod>` is the
  build time. `/fr/sitemap.xml` and `/de/sitemap.xml` are byte-identical
  copies of the en-US index. Reproduce: `runs/p3-grill/probe-20261017/all-locs.txt`.
- **Every page carries a language switcher**: 68 `a.language` links, 67 to
  other locales, all relative and on docs.fp.o itself (`../../fr/a11y/`).
  Checking them one hop deep would cost about 154,000 requests a run.
  Reproduce: `grep -o 'class="language" href="[^"]*"'
  fixtures/docs-fp-o/en-US/a11y/index.html | wc -l`.
- **A missing page** is a 404 with Antora's 22,763-byte HTML 404 page, whose
  links are root-relative.
- **HEAD works** on pages and assets, `Content-Length` exact. Responses are
  gzip, never brotli, over HTTP/2. `Content-Type` carries no charset.
  Validators vary between consecutive requests (I3 stands).
- **http → https is a 302.** A page answers in about 0.34 s (median of five).
- **An alias page answers HEAD and GET alike**: 200 `text/html`, 514 bytes,
  its refresh only in the body.
- **UI assets live under `/en-US/_/`**, inside an en-US root's scope.
- **External hosts in the 30 fixtures**: 1341 absolute URLs over 59 hosts,
  spdx.org (687, 583 on `legal/all-allowed`), gitlab.com (84),
  forge.fedoraproject.org (79), fedoraproject.org (73) leading. 33 are
  `http://`, 30 of them creativecommons.org. Reproduce:
  `python3 -I runs/p3-grill/probe-20261017/hosts.py fixtures/docs-fp-o`.

### Crates

- **robots.txt: `texting_robots` 0.2.2** is the only maintained parser with
  `Crawl-delay`, `Sitemap:`, `*`/`$` and RFC 9309 longest match; it matches
  the product token exactly (`bruce`, not the UA string). `robotxt` matches an
  agent by prefix, not RFC 9309; Google's port has no `Crawl-delay`.
- **Sitemaps: `quick-xml`.** The `sitemap` crate is unmaintained since 2020.
  Neither decompresses a `.xml.gz` file (`flate2` would).
- **reqwest 0.13.5**: `Policy::none()` hands back each 3xx, so hops can be
  followed by hand; `Policy::custom` sees only the current hop's status.
  NXDOMAIN is typed only with `hickory-dns` (`NetError::is_nx_domain`); the
  system resolver tells it from a temporary failure by message string alone.
  A TLS failure is reached by descending `io::Error::get_ref`, not
  `source()`. A body can be read with a cap, after decompression. Its
  `rustls` feature means aws-lc-rs, a C build; ring needs
  `rustls-no-provider`.
- **governor 0.10.4 has no per-key quota** (boinkor-net/governor#193 open):
  a rate per host is a map of direct limiters, or a per-host next-allowed
  instant.
- **wiremock 0.6.5** serves response sequences, delays, `Retry-After`,
  redirects and IO errors, under `#[tokio::test]`.
- **Charset**: `encoding_rs` decodes by label and sniffs a BOM; no
  maintained crate does the WHATWG meta prescan; `lol_html` takes an
  ASCII-compatible encoding and can switch at the first `<meta charset>`.
- **`xxh3_64`** is in the `xxh3` feature `xxhash-rust` already enables.
- **tokio 1.53.3** suits every crate above.

## The slices

Each makes the next one's mistakes visible: the pure, I/O-free pieces land
first, wiremock carries the network until the last slice, and a live crawl
closes the phase. P3.5 is the largest, kept whole because its frontier, hops
and outcomes are reviewed only against one another.

### P3.1 Canonical URLs

`url.rs` ("The canonical form"), `strip_query` in the loader and the manual,
and `config::Root::in_scope` taking the canonical URL. Amends D44.

### P3.2 The store's rows

Schema version 2: `redirect_chain` as `(url, status)` structs with a refresh
marker ("Redirects"), the `sitemap` link kind, and the writer's rows for
`targets`, `anchors` and `links`. Amends D13.

### P3.3 One hop over HTTP

D1's fetcher trait and the HTTP fetcher ("The stack"): reqwest with ring and
`hickory-dns`, retries and `Retry-After`, timeouts, the size cap, HEAD with
GET fallback, and the charset ("Which responses are read"). Against
wiremock.

### P3.4 Per-host politeness

Rate and `concurrency` per host (D8, D18), robots.txt with
`texting_robots`, RFC 9309's unavailable cases, and `user_agent` with its
derived token, in the loader and the manual. Against wiremock. Amends D11.

### P3.5 The crawl

The frontier and "What gets requested"'s classes, hops 3xx and refresh
alike ("Redirects"), D16's outcomes, `too_large`, extraction on the blocking
pool, and feeding the writer; allowlisted pages per "Assets and allowlisted
pages". Against wiremock.

### P3.6 Sitemaps

Discovery (D40), indexes, gzip, scope filtering and `sitemap` links
("Sitemaps"). Against wiremock.

### P3.7 `bruce crawl`

The subcommand and its rules ("The command line"), Ctrl-C ("An interrupted
run"), and the manual. Amends D5.

### P3.8 The live crawl

Evidence, no code. `https://docs.fedoraproject.org/en-US/fedora-docs/`, 24
sitemap pages with the tabbed markup guide, allowlisting
`forge.fedoraproject.org` and `gitlab.com`, at `Crawl-delay: 1` under the
default UA: minutes, not a long job. Its log and DB go to `runs/`; what it
shows goes to the notes and, where a decision relies on it, the invariants.
It passes when every sitemap page is fetched, outcomes are plausible, any
alias resolves as a hop, and no `error` is unexplained.
