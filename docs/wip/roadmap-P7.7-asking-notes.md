# P7.7 — The asking settings: notes

What the next slice inherits. The decision is D85.

## What landed

- **`config::policy::asking`**: `Asking` (the run-wide values, the hosts' own
  in `hosts`) and `HostAsking` (a host's, each key an `Option`). `[default]` and
  `[host]` take `connect_timeout`, `timeout`, `retries`, `max_retry_after` and
  `slash_probe`; `max_body_mib` is top-level. They are rows of `Policy::entries`
  like any setting (D78), the defaults included, so no schema change and a
  baseline file carries them.
- **`fetch::http`**: `Settings::of(&Config)` reads the run-wide values and
  `Settings::default()` reads `Asking::default()`, so the defaults are written
  once. A host the file names has its own over them (`Host::asking`); the total
  timeout is set on each request, and a host with a connect timeout of its own
  gets a client of that timeout, kept in `HttpFetcher::others`.
- **`crawl::slash::toggled`** asks `Asking::probes` of the toggled URL's host.
  **`LocalFetcher::with_max_body`** holds `check-dir` to the same cap.
- **`Store::try_policy` writes the judging rows alone** and
  `report::digest::sections::policy` reads the asking rows from the stored
  `policy`, so a trial's digest states the file's judging settings and the run's
  asking ones (D85). The Housekeeping paragraph says so.
- `fetch::MAX_BODY` is gone: the cap is `Asking::max_body`.

## For P7.8 and P9

- **A reader of the policy that wants asking rows reads `policy`**, a reader
  that wants a trial's judging rows reads `policy_in_force`. `ASKING_KEYS`
  (`config`) lists the keys; `Entry::is_asking` tells a row.
- The deployment guide's config section (P9) can set a site's timeouts and
  `slash_probe` per host; `docs/manual/config.md`, "Asking", is the reference.

## Negative results

- **A run recorded before this slice has no asking rows**, and the digest then
  states none: a missing row reads as the default, which is what those runs
  asked with. Nothing is inferred from a run's data.
- **`retries` is a count, not a list of waits**, capped at 10 at load (a wait of
  512 s before the last); the cap is this slice's choice and nothing measured
  it.
- **Each client built for a host's own connect timeout has its own hickory
  resolver and so its own DNS cache**; a host is one name, so nothing is
  looked up twice for it. Not measured.
- Not tested end to end: `cli::check_dir`'s passing the cap to `LocalFetcher`
  is one line, and `LocalFetcher::with_max_body` has its own tests.
