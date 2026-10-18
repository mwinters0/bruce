# P3.4 Per-host politeness: notes

What P3.5 onward inherits, and what was refused.

- **`Hosts::new(&Config)` is the politeness a crawl builds once** and hands to
  `HttpFetcher::new(settings, hosts)`; `Settings::user_agent` is still the
  caller's to fill from `Config::user_agent` (P3.7 does; `Hosts` reads
  `Config::robots_token` itself). `HttpFetcher` and
  `Hosts` clone cheaply and share state.
- **`fetch` paces and `access` gates.** Every request, retries and robots.txt
  included, passes `Host::admit` (the in-flight permit, then the host's next
  slot), so a retry waits for the later of its backoff and the host's own
  interval. `fetch` never consults robots.txt: the crawler calls
  `Fetcher::access(&Url) -> Access` before each hop, and maps
  `Disallowed` to `unchecked`/`robots` and `Unavailable` to
  `unchecked`/`robots_unavailable`. `access` fetches the host's robots.txt
  on its first call and every later call is a lookup.
- **Hosts are keyed by the name `url::Host` prints**, ports and schemes
  sharing one pace, one permit pool and one robots.txt, fetched from the
  origin of the first URL asked about (an `http://` first URL reaches the
  `https://` file through the host's redirect). The spec's `[host]` keys are
  by host already; two origins of one host are not distinguished.
- **The interval is the configured rate until robots.txt is in**, then D18's
  combination (`host::interval`, tested as a pure function). The robots.txt
  request itself is therefore paced at the configured rate, never at a
  `Crawl-delay` it has not yet told us.
- **Calls beyond the spec**, none touching a contract:
  - A **429 for robots.txt that outlasts the retries closes the host**
    (`Unavailable`) rather than counting as a 4xx that allows all: it says
    the host is limiting us, not that the file is absent. D8 records it.
  - **More than five redirects, a 3xx with no usable `Location`, or any
    status outside 2xx to 5xx** closes the host.
  - **A file `texting_robots` refuses to parse allows everything.** Its
    parser is forgiving (an HTML page served at `/robots.txt` parses to no
    rules), so a refusal is rare, and no rule can be obeyed from it.
  - **Only the first 512 KiB of robots.txt is read**; the rest is ignored.
  - **A `Crawl-delay` is capped at an hour** (`MAX_INTERVAL`), and a
    negative or non-finite one is ignored, so a hostile file cannot stall a
    run or panic `Duration`.
  - A `Retry-After` stays one target's wait: it does not move the host's
    pace for the other requests in flight.
  - `robots = false` also means the host's robots.txt `Sitemap:` lines are
    not read; P3.6 discovers a sitemap on such a host from `sitemap.xml`
    alone. `Robots` keeps `Robot::sitemaps` for the hosts that are read, and
    P3.6 adds the accessor it needs.
- **`user_agent`'s validation** is stricter than a header's: visible ASCII
  starting with a product name of letters, digits, `-`, `_` or `.`, since
  that name is what robots.txt matches (`texting_robots` compares the token
  exactly, lowercased, and falls back to `*`).
- **Negative result**: a dropped `MockServer` kept answering (it appears to
  return to wiremock's pool), so a test needing a port nobody answers on
  binds and drops a `TcpListener` itself.
