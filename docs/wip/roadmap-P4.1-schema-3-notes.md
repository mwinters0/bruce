# P4.1 Schema 3: notes

`SCHEMA_VERSION` is 3, with no migration (D43). `links` holds
`run_id, src_url_id, target_url_id, fragment, raw_href, kind, xref_refspec`
and nothing derived; `pages.root` and `runs.roots` are new, closing KD3.

## What the next slice inherits

- **`pages.root`** is the root's `Root::url` as a string, from
  `Plan::Page { root }` in `Crawl::write_page`; NULL for a page written under
  any other plan, which is the allowlisted ones. It is the *landing* page's
  plan, so a redirect from one root's scope into another's names the second.
- **`runs.roots`** is `VARCHAR[] NOT NULL`, from `Config::root_urls()` through
  `Store::begin_run`'s new last argument; a run with no roots (a test, or a
  future `check-dir`) stores an empty list. It is bound as a list literal of
  placeholders, UF1's workaround again.
- **`link_verdicts` (P4.5)** has no stored columns to disagree with: a join
  reading `links.outcome` fails to bind rather than reading NULL. The crawl
  tests that asserted `outcome IS NULL` assert nothing about verdicts now;
  P4.5's tests carry that.
- Anything outside the repo reading a version 2 DB is refused by name (D43);
  the live-crawl DBs under `runs/` are version 2 and are not readable by this
  build.

## Negative results

- None: no measurement was taken, and the slice needed no `D<k>` entry. The
  choices (`pages.root`, `runs.roots`, the view) are the spec's.
