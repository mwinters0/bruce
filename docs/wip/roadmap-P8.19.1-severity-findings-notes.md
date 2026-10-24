# P8.19.1 — A dot marks a finding: notes

What landed, for P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"Severity is marked", amended by 8.6's "Finding 8"): the dot on a section's
`<summary>` is gone, with `highest` and `open_section`'s level; `broken_url`
and `error` are `Level::Broken` (red), `broken_anchor` and a redirect that is
not pedantic `Level::Attention` (amber), the legend, the manual's "The HTML"
and the golden file following. No `D<k>` entry: the levels are the spec's.

## What the next slices inherit

- **A heading never carries a dot**; a section built from rows that are links
  or targets puts `mark(Level::of_link(..))` first in each row, as 8.19's
  notes say.
- **An `error` row is red**, so the errors section's dot is `Broken` whatever
  its reason; the section's own text still says the target was not asked.

## Negative results

- **Red and amber now split the old way's pair**: print's greys still tell
  broken from attention by colour alone, the ring being what stays distinct.
  The row's text (`URL broken`, `fragment missing`, the errors section) says
  which.
