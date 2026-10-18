# P3.7.1 A stop bounded in time: notes

What P3.8 onward inherits, and what was refused.

- **`fetch::Stopper`** is a `watch` of the instant the first stop came, so a
  waiter wakes at once and the grace is measured from the stop. It lives in
  `fetch`, the lower layer, and `crawl` re-exports it. `Crawl::new` makes it
  and hands it to its fetcher by `Fetcher::with_stop`, a defaulted trait
  method a fetcher that never waits ignores, so the crawl and its fetcher
  cannot hold two stops.
- **`HttpFetcher`** races its host's `admit()` and each backoff against the
  stop, `biased` toward the stop, and returns `FailureKind::Stopped`, which
  the crawler never writes. A sent request is not raced. `host.rs` is
  untouched.
- **The 30 s bound is per target, not in `drive`**: `Shared::follow` runs a
  target's hops under `in_grace`, which drops them `STOP_GRACE` after the
  stop, and the write that follows is left to finish. The settled shape
  (history, "A stop is bounded in time") had `drive` abort what is left;
  aborting a task in its write would leave its rows half sent from the
  blocking pool and make the cut count wrong (D48). The bound is the same.
  The sitemap discovery's robots.txt and probes are under the same grace.
- **"Cut" is every target claimed and not written**, `Frontier::claimed`
  less the target rows sent, since each claim writes exactly one row in a
  whole run. So the URLs queued and never started count as well as those
  dropped mid-hop: a lower bound on what the run lacks (D48).
- **A robots.txt fetch the stop cut caches the host as unavailable** for
  the rest of the run. Harmless: `walk` checks the stop after every
  `access`, so no row is written from it.
- **The second Ctrl-C** drops the crawl's future, shuts the runtime down in
  the background without waiting for its blocking tasks, and finishes the
  writer; the wait for writes begun is P3.7.2's. It prints `run N:
  abandoned at the second Ctrl-C, …` and exits 130. `process::exit` is gone.
- **Observed once, not a test**: against a local server holding three
  requests for 40 s, the first `kill -INT` ended the run 28 s later (the
  requests' own 30 s timeout, then no retry) with `1 targets, 1 pages, no
  sitemap, 3 cut`; a second `kill -INT` a second after the first exited at
  once, the root's row flushed and `finished_at` NULL.
