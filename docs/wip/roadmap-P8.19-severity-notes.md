# P8.19 — Severity marked in the digest: notes

What landed, for P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"Severity is marked"): `report/digest/html/severity.rs` (the `Level`, read
off a link's outcome and reason, the dot, the legend), a dot column with no
heading opening the table of the broken targets, each page's links, the errors
and the redirected links, one legend after the figures, the stylesheet's `.sev`,
the golden file, the manual's "The HTML". No `D<k>` entry: the levels and the
rows that carry them are the spec's.

## What the next slices inherit

- **A row's level is `Level::of_link(outcome, reason)`**, a redirected link's
  row (outcome-less) `Level::of_redirect(reason)`. 8.7's `changed` and
  `anchors-removed` sections and 8.8's two-run sections take a dot from the
  same two functions (8.19.1 moved the levels).
- **A table's header named `""` is the dot's column** (`column_class`), so a
  new table with a dot passes `("", false)` first and `mark(level)` first.

## Negative results

- **Broken and attention differ by colour alone in print's greys**, both
  filled, as the spec has it; `--bad` and `--warn` print as dark red and dark
  amber, close in grey. The row's own text (`URL broken`, `fragment missing`,
  the errors section) says which, and the legend names them; the ring is what
  stays distinct.
- **Rendered** by `mise run shot` (Servo) at 1280 px and in print: the dots
  sit before each section heading, beside the closed marker, and before each
  row in the printed, opened tables; the legend wraps at a phone's width as
  the figures' paragraph does. The pedantic ring is not on the e2e site's
  digest (no slash-only redirect there), so only the unit test shows it.
