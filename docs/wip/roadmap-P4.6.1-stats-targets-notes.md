# P4.6.1 `stats` counts unchecked targets beside links: notes

The totals' reason lines read `  skipped: 1955 links, 1925 targets`. The
targets are `count(DISTINCT v.target_url_id)` over the same `link_verdicts`
join as the links (`totals` in `src/report/query.rs`), so `--repo`, `--fact`
and `--kind` narrow both and the link-only reasons (`text_fragment`,
`no_anchors`, `cut`) keep their lines. The header is `unchecked by reason:`.

## What the next slice inherits

- **A target count is a distinct count over a link view.** 4.7's `inbound`
  and `orphans` count distinct linking pages the same way; the filter still
  names the page a link is on.
- **The page rows' `unchecked` column is still links.** Only the totals
  carry targets; a per-page target count would need `count(DISTINCT …)` in
  `report_stats` and has no consumer.

## Negative results

- No NULL-target caveat: `links.target_url_id` is `NOT NULL`
  (`src/store/schema.rs`), so `count(DISTINCT)` skips no link.
