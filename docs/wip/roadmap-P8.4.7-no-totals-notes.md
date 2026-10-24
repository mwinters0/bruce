# P8.4.7 — The rows' summaries without their total: notes

`tally` (`src/report/digest/html.rs`) ends at its last count: a page's and a
repo's summary reads "● 1 broken link, ● 554 broken through it", the
`<small>` total gone (8.6's notes, "Finding 12"). The JSON never carried a
total, so it does not change. The golden file `fixtures/antora/digest.html`
changes only in its eleven summaries. The `small, .muted` rule in
`html/style.css` stays, other markup using it.
