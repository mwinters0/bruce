# P8.1 — A hash records its scope: notes

What landed, per the P8 spec's "A hash records its scope": `extract::Covered`
(`root`, `region`, `whole`) from `extract::hash`, `Extracted.covered`,
`pages.hash_scope` (`NOT NULL`, checked), `runs.hash_scopes` (a list of
`STRUCT(root, hash_root, hash_exclude)`, one per root, beside `runs.roots`),
and `SCHEMA_VERSION` 5. D15 says why.

## The en-US crawl under schema 5

`runs/p8.1-en-us-20261023-2004/en-us.duckdb`, run 1, finished, exit 0:
6,436 pages, 172,453 targets, 1:10:33 (`time.log`; P5.2's 1:48:20 under the
same config, the cause not sought). `report broken` gives 827 broken links
on 324 pages, P5.2's figures. It is the schema-5 en-US evidence 8.3 to 8.7
read, and the run 8.8's second crawl compares with.

- **Every page is `region`**: the 2,320 a root covers (`article.doc`), so no
  en-US page lacks the region and none is `whole`, and `root` is 0, the
  config setting no `hash_root`. `runs.hash_scopes` is one struct, the en-US
  root with empty `hash_root` and `hash_exclude`.
- **The forge's 4,116 pages (`root IS NULL`) are `region` too**, not
  `whole`: they are read under `auto` and no keys, `generic`'s region is
  `main` or `[role=main]`, and Forgejo wraps its content in
  `<div role="main">`. So their hash already excludes the forge's chrome, and
  the churn the spec cites is inside that region; `changed` leaves them out
  by `pages.root`, not by scope.

## What 8.3 and 8.7 inherit

- **A hash is comparable only within one `hash_scope` and one root's keys.**
  `changed` (8.7) lists a page whose `hash_scope` is the same in both runs
  and whose `content_hash` differs; a page whose scope differs is "scope
  changed"; a page that is `whole` under a root whose `hash_scopes` entry has
  a non-empty `hash_root` is the root missed (`hash_scope = 'whole'` joined
  to `unnest(runs.hash_scopes)` on `pages.root = s.root`). The pages with
  `root IS NULL` are the allowlisted ones, hashed under no root's keys.
- **`root` means `hash_root` matched.** `hash_exclude` alone is `whole`, as the
  hash is the page less the exclusions, so `whole` is not by itself a miss.
- **`runs.roots` is unchanged**, and `report_roots` still unnests it. The two
  lists are written together by `Store::begin_run` from `RunRoot::of(config)`
  and share an order, which no constraint holds; join on `root`, never on
  position.
- **`RunRoot` replaced `Config::root_urls`** (removed), the store taking the
  root's canonical URL and keys as strings. `begin_run` takes `&[RunRoot]`;
  `RunRoot: From<&str>` gives a root with default keys, for a test.
- **`copy_run` names `runs`' columns**, so a column added to `runs` is added in
  `store/baseline.rs`' select too, or an import fails with a count mismatch.

## Negative results

- **No D entry of its own**: the scope is D15's and the schema bump D43's, and
  both are amended in place.
- **A `run_roots` table was not made**: it would join more easily than a
  struct list, but `prune`'s and `baseline`'s table lists and the six-element
  counts would all change for one small table; the list of structs holds the
  same rows and unnests in one expression.
