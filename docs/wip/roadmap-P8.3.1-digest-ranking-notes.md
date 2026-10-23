# P8.3.1 — Targets ranked by links, and the sitemap's unfetched entries: notes

What landed, per `../status/history/2026-10-23.md`, "The digest's sections
reviewed": `broken_targets` is ordered by `links`, then `pages`, and the
housekeeping gains `sitemap_entries`, JSON `housekeeping.sitemap_entries`
(`format_version` stays 1, a field being added). D64 and
`docs/manual/digest.md` say what is built.

## What 8.4 inherits

- **Section 4's "named by more than one link" is `links > 1` on
  `broken_targets`**, whose rows are already in order; the rest are
  `count(links = 1)` and sit in section 5's drill-downs.
- **`sitemap_entries` has `page`, `outcome`, `reason` and `status`**, the
  entries that did not answer `ok` first. It is empty when every entry was
  fetched as a page, as on en-US.

## Measured

`runs/p8.1-en-us-20261023-2004/en-us.duckdb`, run 1: `broken_targets` leads
with `legal/license-review-process/#_what_is_licenseref` (266 links, 3
pages), then Updates_Policy's `updates-testing-activation` (7, 6), a quick-docs
`README.md` (5, 5) and a grub2 anchor (5, 1); `sitemap_entries` is empty,
the 494 targets unchanged.

## Negative results

- **`sitemap_entries` is a query on `report_sitemap`, not a call to
  `reach::sitemap`**: that report returns the 2,320 pages as well, with columns
  an entry leaves null, and the digest keeps the rows of the entries alone. The
  view is the shared query, `via_links IS NULL` being what marks an entry
  (`src/report/views.rs`).
- **The synthetic test store holds the ranking apart**: a target with more
  links on fewer pages, and a tie on links broken by pages, so a regression to
  ordering by pages alone fails.
