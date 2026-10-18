# P3.5 The crawl: notes

What P3.6 onward inherits, and what was refused.

- **`Crawl::new(fetcher, config, registry, run, rows).run()`** seeds each
  root's URL and runs until nothing is left, at most `WORKERS` (64) targets in
  flight; a host's pace and cap stay the fetcher's. It returns a `Summary`
  (targets and pages written). P3.6 seeds a sitemap's entries beside the
  roots in `run`, claiming each through `Frontier::claim`; P3.7 stops it on
  Ctrl-C at the `while tasks.len() < WORKERS` loop, which is the only place a
  new target starts. The caller (P3.7) fills `Settings::user_agent` from
  `Config::user_agent` and checks `Config::check_robots_hosts`.
- **`Fetcher::access` is on the trait**, defaulting to `Allowed`, so a local
  fetcher (P5) needs none; `HttpFetcher`'s was an inherent method.
- **One task per target** follows its hops itself: robots.txt then fetch at
  every hop, the extraction on the blocking pool, and the rows sent from a
  second blocking task, so a full writer channel blocks a pool thread and
  never the runtime. The frontier is a mutex over a `HashSet<u64>` held for
  one insert.
- **Planning is by class, per hop.** `Frontier::plan` gives a URL a `Plan`
  and its canonical form: `Page{root}` (GET, extracted, links followed),
  `Anchors` (allowlisted and named by an `anchor` link: GET, anchors only),
  `Status` (HEAD with the GET fallback) or `Unchecked(reason)`. `hop_plan`
  turns `Unchecked` into `Status`, so a redirect is followed wherever it goes
  and nothing past a root's scope or the allowlist is read. A URL in a root's
  scope is GET for every kind of link, an image included (the spec's class 1).
  An allowlisted host that is also a root's host is allowlisted, the spec's
  order. A URL no root claims has the plain form, `canonical(url, false)`
  (P3.1's open question).
- **A chain is recorded in canonical forms** (`Hop::url`, `final_url`), the
  string a `url_id` hashes, so a chain joins `targets` without reparsing. A
  target not redirected has `final_url` NULL and an empty chain; a chain that
  does not land (a loop, past 10 hops) has `final_url` NULL, `status` NULL and
  `broken_url`; one that ends at a failure or at robots.txt has the landing
  URL as `final_url` and a row of its own with the same outcome. A hop's
  fragment is dropped.
- **`links.redirected` is written false** (KD3). It was P3.2's open question;
  holding a run's links back until their targets settle costs the whole link
  graph in memory, and D2 forbids updating them.
- **`links.fragment` is the URL's fragment as written** (percent-encoded);
  decoding for the match (D12) is the join's.
- **An href that resolves to no URL** (`http://[bad`) is a `broken_url`
  target, reason `invalid_url`, its `url` the raw text and its `url_id` the
  `xxh3_64` of it, so the link keeps a target.
- **A page of a root records `via_links` true when a link found it, false for
  a root's seed**, and `in_sitemap` false; P3.6 sets the latter where a sitemap
  claims the page first. A page both linked and listed has only the flag of
  the claimant, so orphans and unlisted pages are the join's (P4 inbox).
- **An allowlisted page's `pages` row has no `source_*`**, the edit link
  being a root page's; its profile is `auto` and its hash covers the whole
  page.
- **An extractor failure** (the HTML is refused, or a root names a profile the
  registry lacks, which `cli` refuses first, D39) is `error`, reason
  `unreadable`: it answered, but its anchors cannot be known.
- **Calls beyond the spec**, none touching a contract: the order of
  `unchecked` classes above; the `invalid_url` target; `unreadable`; a
  `bad_redirect` (a 3xx with no usable `Location`) being `broken_url`; the
  worker count; the landing URL's own row (D46).
- **Repeat requests**: a redirect to a URL already claimed is requested
  again, its answer settling only the redirecting target (D46). So is a page
  extracted before its landing URL is known to be unclaimed.
- **Negative results.** `mem::replace` on `Response::body` to take the text
  dropped `TooLarge` with it; the over-cap test caught it. `localhost` and
  `127.0.0.1` are two hosts to a crawl and one wiremock, which is how the
  tests tell an allowlisted host from a root's. A TCP-refused allowlisted
  host cannot be tested at the first hop: its robots.txt fails first and the
  host closes (`robots_unavailable`, D8); a redirect from a reachable port
  of the same host is how a connect failure is reached.
