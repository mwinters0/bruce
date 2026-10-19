# P4.7.1 `unlisted`'s roots: notes

`Root::canonical_url` (`src/config.rs`) is the one form of a root's URL:
`Config::root_urls` writes it to `runs.roots` and `Shared::write` to
`pages.root`, so `report_roots` matches the root against its own target's
`url` by construction. `report_roots` (`src/report/views.rs`) is a recursive
CTE: a root has a sitemap when a file reached from its page by `sitemap_file`
links, the children of an index included, has `outcome = 'ok'` and no `reason`.

## What the next slice inherits

- **Older DBs keep the configured form** in both columns, so a root with an
  escape or a stripped query reads as having no sitemap there. No schema bump,
  as the spec says; P4.4's DB is not in the tree.
- **An index that answered `ok` counts as a read sitemap on its own**, even
  when every child failed: the rule is "any file reached", and the failed
  children show in `broken` or `errors`. The tests pin both a good child
  behind an over-cap index and a failed one.
- `diff` and `prune` read nothing of roots.

## Negative results

- No crawl-level test of a root with an escape: wiremock matches the request
  path, which the fetcher does not canonicalise for the mock, so the unit test
  of `root_urls` and the `report_roots` tests carry it.
