# P4.2 `skip`: notes

`Config::skip` is a top-level `Patterns`, compiled at load, and
`Frontier::plan` asks it first after the scheme: a match is
`Plan::Unchecked("skipped")`, so the target row is `unchecked` / `skipped`
with no request and the link row is kept (D50).

## What the next slice inherits

- **P4.3's guide** can put `skip = ["https://forge.fedoraproject.org/**/issues/new*"]`
  in docs.fp.o's worked example; `*` stops at `/` and `**` crosses it, and the
  glob meets the query (`issues/new?title=…` is the shape P3.8 saw), so
  `issues/new*` matches it. The manual's `toml` blocks are parsed by a test
  (`config::tests::every_toml_example_in_the_manual_parses`), and `docs/manual/config.md`
  now has a Skip section.
- **`link_verdicts` (P4.5)** reads `targets.reason = 'skipped'` as `unchecked`
  like any target that is not `ok`: nothing more for the join to do.
- **A skipped sitemap entry** is dropped and counted with the entries outside
  every root, since `write_sitemap` keeps only `Plan::Page` entries of a page
  sitemap.
- **A root whose `url` a glob matches** is refused at load by
  `Config::check_skipped_roots`, called by `Config::parse` and, for a root
  added from the command line, by `cli::crawl` beside `check_robots_hosts`.

## Negative results

- None measured. Two choices the spec left open are in D50: `skip` matches the
  plain canonical form (the query kept, not a root's `strip_query` form, since
  it runs ahead of scope), and a redirect hop landing on a skipped URL is still
  followed (`hop_plan` turns every `Unchecked` into a status request).
