# P8.8.2 — Broken and fixed apart, no anchors-removed section, the compared run a sub-title: notes

Earned by 8.20's findings 1 to 3.

- **The headings say "the previous run" whatever the comparison is**, as the
  maintainer worded them, though `--from` may name an older run and `--since` a
  baseline file. The sub-title says which run it was, and the sections'
  paragraphs name it ("since run 4", "the baseline file's run").
- **The fixed links are one table with no drill-down**: their section holds
  nothing else, so 8.8.1's single fold "Broken then, not now" would be a second
  click for nothing.
- **`Compared::shares_day`** is read once, in `compare::compared`, against the
  database's other crawl runs by UTC day, the day a digest is named for. A
  baseline file's run never shares, its database being another.
- **`anchors_removed` is still computed** for the JSON; the HTML reads no field
  of it.
