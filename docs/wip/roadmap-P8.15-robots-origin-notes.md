# P8.15 — robots.txt read per origin: notes

What landed, for P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"robots.txt is read per origin"): `Hosts::read` in `fetch::http::host`, a
table of each origin's raw read beside the host table; `Host::access` and
`Host::sitemaps` keyed by origin; `HttpFetcher::listing`, which shares a
redirect's end with the origin it ends at; `Listing::Failed` carrying the
name that was silent; D69, D8 amended, `config.md` and `digest.md`.

## What the next slices inherit

- **8.14's crawl changes no verdict by order any more.** A name answering on
  `https` and silent on `http` or an old port had every link `error` when an
  `http` link asked first (8.13's notes); now only the silent origin's do.
  8.8 counts any `error` that goes the other way as reclassified, not new.
- **The digest's `robots_unavailable` legend still says "the whole site"**
  (`report::digest::html`, and the golden file would move with it). The reason
  is now an origin's. Left, since the usual host has one origin and the golden
  file is the maintainer's to see change.
- **An origin's read is held twice**: `Hosts::read` keeps the raw listing,
  shared with any origin whose redirect ends at it, and `Host::access` keeps
  what a host makes of it (its switch, its name, the parsed rules). One cell
  could not do both: a Failed listing means `Failed` on the name that was
  silent and `Unreachable` on another's.

## Negative results

- **A loop between two origins would deadlock on the cell.** `A` redirecting
  to `B`'s `/robots.txt` while `B`'s redirects to `A`'s, each read waiting on
  the other's `OnceCell`. `Reads::waiting` holds who waits on whom, and the read
  that would close the cycle is `Unreachable`; a chain-local visited set alone
  misses two tasks starting at once. Asserted by
  `origins_redirecting_to_each_other_do_not_wait_for_ever`.
- **No test serves two schemes.** The mock servers speak `http` alone, so the
  origins differ by port; a scheme is part of the key (`Url::origin`) but is
  not exercised against a TLS server.
