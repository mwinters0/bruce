# P4.5 `link_verdicts`: notes

`link_verdicts` is `store`'s (`src/store/views.rs`), installed by
`Store::init` at every open ahead of the profiles' views, so an ad-hoc
`duckdb` user has it after a crawl alone. D28 is amended to say the verdict
is this view. Its rules are the module doc's, in the order they apply.

## What the next slice inherits

- **Columns.** `run_id, src_url_id, target_url_id, fragment, raw_href, kind,
  xref_refspec, outcome, reason, redirected`: a link's own fields and its
  verdict, one row per `links` row, so `report` joins only to `targets`
  (for the source's and target's URLs) and `pages` (for `source_repo`,
  `source_path`). `kind` is not filtered: `broken` takes `kind = 'anchor'`
  by default (D13) and that is 4.6's.
- **Two reasons the spec did not name.** `text_fragment` (a fragment holding
  `:~:`, whole, even one with an id before the directive) and `cut` (a link
  whose target has no row, which only a stopped run leaves: nothing is known,
  so it is `unchecked`). `stats` counts `unchecked` by reason, so both show.
- **`reason` is NULL for most breakage.** A 404 target is `broken_url` with a
  NULL reason (`outcome::answered`), as is every `broken_anchor`; only
  `unresolved_xref`, a loop, `invalid_url` and the like carry one. 4.6's text
  table prints the status for a `broken_url` from `targets` if it wants more.
- **`#top` is matched in any case**, HTML's rule, where D12's "always `#top`"
  is silent; an id is matched exactly, as the raw fragment or percent-decoded
  (a fragment not valid UTF-8 decodes to nothing and is matched raw only).
- **A link's `redirected`** is `targets.final_url IS NOT NULL`, so it is true
  for a redirect that ended `unchecked` / `skipped` or `robots` too.

## On P4.4's DB (`runs/p4.4-en-us-20261019-1633/en-us.duckdb`)

Timing: the whole view, grouped by outcome and reason, over its 587839 links
and 172239 targets takes 0.1 s (0.37 s on the first, cold run); the count of
one run's `broken_anchor` 0.09 s, `duckdb` 1.5.5 CLI, peak RSS 650 MB of the
CLI over the file. **Materialising is not needed**; D28's Reopens names the
size at which it would be.

| `outcome` | `reason` | links |
|---|---|---|
| `ok` | | 399737 |
| `unchecked` | `out_of_scope` | 157623 |
| `unchecked` | `host_not_allowlisted` | 27390 |
| `unchecked` | `skipped` | 1955 |
| `broken_anchor` | | 610 |
| `unchecked` | `scheme` | 280 |
| `broken_url` | | 117 |
| `broken_url` | `unresolved_xref` | 92 |
| `unchecked` | `robots` | 35 |

2009 links are `redirected`, 4 of them with a fragment. No link is
`no_anchors`, `text_fragment` or `cut`: none of the run's 874 `ok` targets
without a page is named with a fragment. Of the 610 broken anchors, 266 name
`_what_is_licenseref`, 50 `index.adoc` and 48 `ambassador-mentor.adoc`
(evidence only, D41; not looked into).

## Negative results

- KD4 did not show: no `no_anchors` link in P4.4's DB.
- A join of `anchors` on `a.anchor IN (fragment, decoded)` was not built; two
  `EXISTS` keep each an equality, which DuckDB plans as hash joins.

## Reproducing

```sh
duckdb -readonly runs/p4.4-en-us-20261019-1633/en-us.duckdb
```

then `.timer on`, `CREATE TEMP VIEW link_verdicts AS` with `LINK_VERDICTS`'s
select from `src/store/views.rs`, and `select outcome, reason, count(*) from
link_verdicts group by all`.
