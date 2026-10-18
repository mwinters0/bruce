# P3.2 The store's rows: notes

What P3.3 onward inherits, and what was refused.

- **Schema 2**: `targets.redirect_chain` is `STRUCT(url, status, refresh)[]`,
  NULL for a target not redirected (never an empty list), and a 2-hop chain
  reads `redirect_chain[1].url`, `.status`, `.refresh`. A 3xx hop has
  `refresh = false`; a page's delay-0 refresh is a hop of its own with the
  page's 200 and `refresh = true`. `final_url` is the last hop's target, not
  a hop.
- **Rows are in `store/rows.rs`**, not `writer.rs` as the spec's file table
  has it: `PageRow`, `FactRow`, `TargetRow`, `Hop`, `AnchorRow`, `LinkRow`,
  `Outcome` and `Row`. The writer is the thread and the appenders. The store
  names no `extract` type (D39 puts the two on one tier), so a link's `kind`
  and an anchor's `placement` are `&'static str`, taken from `Kind::as_str`
  and `Placement::as_str`; the crawler converts.
- **`Kind::Sitemap`** is in `extract::Kind` (D13), never yielded by the pass.
  `links.kind` has no CHECK, so a wrong string is not refused by the schema.
- **`Outcome` is D16's five**, used by `TargetRow` and by `LinkRow` (an
  `Option`, which the crawler leaves `None`: P4's join fills the verdict, D28).
- **`TargetRow.checked_at` is a `SystemTime`**, converted to microseconds;
  `None` for an `unchecked` target that made no request.
- **A chain cannot go through the appender** (UF1): a target with one is
  inserted by a cached statement per chain length, binding each hop's fields
  inside a list literal. The rest use the appender. Both write the same
  table in arrival order, which nothing reads.
- **`links.redirected` is a NOT NULL column the writer cannot know**: a page's
  links are written when it is extracted, before the targets they name have
  been fetched, and D2 lets only `runs.finished_at` change. The crawl writes it
  false (KD3), and the truth is the target's `redirect_chain`.
- **Negative results**: binding a list into the INSERT is refused by duckdb
  1.10506.1 as well as appending one (UF1); a chain rendered to text and cast
  breaks on any URL holding a quote, which the test pins with
  `it's,here?a=1&b=[2]`.
