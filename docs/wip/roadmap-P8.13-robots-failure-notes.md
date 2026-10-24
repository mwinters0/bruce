# P8.13 — A host that gives no response is its links' failure: notes

What landed, for 8.6's Finding 4
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md))
and the maintainer's ruling of 2026-10-24 that a host which does not resolve
or never answers must not hide its links as `unchecked`: `Access::Failed`
beside the `Fetcher` trait in `fetch`, `Listing::Failed` and `Robots::Failed`
in `fetch::http::host`, `HttpFetcher::robots_txt` returning the failure of the
request on the host's own name, `Shared::walk` settling it as `End::Failed` (D16),
D8 amended, KD21 struck, and the manual (`config.md`, `deploy.md`,
`digest.md`) and the digest's `robots_unavailable` legend reworded. `Access`
is no longer `Copy`.

## The line between failed and unavailable

- **Failed**: a robots.txt request on the host's own name, the first or one
  a redirect kept on the name (`http` to `https`, another port), produced no
  response after its retries, a `Failure` in `fetch`'s sense: `nxdomain` is
  `broken_url`, and `connect`, `timeout`, `tls`, `reset` and `other` are
  `error`, the verdict a request for the link itself would have had. A link
  redirected onto the host fails at that hop the same way. `checked_at` and
  `duration_ms` are kept, the host's request standing for the target's own.
- **Unavailable, as before**: the host answered, but with no file to read
  (a 5xx, a 429 that outlasts the retries), or its robots.txt redirected
  nowhere usable (more than five hops, a non-web `Location`, or **a hop onto
  another host that gives no response**): another host's silence says
  nothing of this one. The name is the line because a host's state is keyed
  by it: with the first hop alone, a host silent on `https` was `error` or
  `robots_unavailable` by which scheme a link asked it on first.
- **A stop** stays `Unreachable`; the crawl writes nothing for it.

## What the next slices inherit

- **8.14's crawl reclassifies, not newly breaks.** KD21 counted 29 en-US
  targets on 16 hosts under the deployment guide's allowlist as
  `robots_unavailable` (8.12's notes); 8.8 counts those as moving to
  `broken_url` or `error`.
- **The failure is settled once a run**, as robots.txt is: a host down or
  slow at its first request has every link `error` (`timeout`, `connect`)
  for the run, even if it recovers, where it had them `unchecked`.
- **`robots_unavailable` now means the host answered with no file**, so a
  run's count of it should be near zero; a large one is a host failing with
  5xx or 429.
- **`src/crawl.rs` stands at the soft limits exactly**: 1000 code lines and
  40 functions (`file_limits.py --all`). It is not past them, so no seams
  round was due, but the next line of production code the file gains makes
  one due.

## Negative results

- **`fetch::http::host` could not name `fetch::Failure` while `Access` lived
  in it**: the trait's default `access` named `Access` there, a
  `fetch` ⇄ `fetch::http::host` cycle `module_graph.py` refuses. `Access` is
  the trait's vocabulary, as `Response` and `Failure` are, so it moved up
  beside them, and every fetcher's module reads it from `fetch`. Keeping
  host free of `fetch` by giving it a failure type of its own was refused: a
  second copy of D16's causes.
- **No test resolves a name to NXDOMAIN through the crawl**: it needs a
  resolver answering NXDOMAIN offline. `FailureKind::NxDomain` to
  `broken_url` is asserted in `crawl::outcome::tests`, its classification in
  `fetch::http::tests::the_cause_is_found_down_the_chain`, and a host's
  NXDOMAIN settled once for all its URLs in `fetch::http::host::tests`. The
  crawl's test uses a refused connection (`connect`, `error`).
