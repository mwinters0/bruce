# P8.4.6 — The redirected links, one `<details>` per landing: notes

What landed, for 8.6's Finding 9
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 9") and P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"What the document holds", 9): the housekeeping's "Redirected links" fold holds
one `<details>` per landing (`redirect` in `report/digest/html.rs`), no table.
The summary is the severity dot, the landing without its scheme and
"N links, M redirecting URLs"; inside are the landing as a link (with its
`pedantic_trailing_slash` reason where it has one), the first hops and the
linking pages, one to a line. The golden file, the manual's "The HTML" and
three tests follow. No `D<k>` entry: the layout is the spec's, and the JSON
(`housekeeping.redirects`) is unchanged.

## What the review inherits

- **The dot is the summary's first child**, not a table cell: a `<details>`
  summary now carries a dot where 8.19.1 allowed none on "a section's
  summary", this being a row naming a landing, not a heading. The sections'
  own summaries still have none (`a_heading_carries_no_dot_and_each_dot_is_named`).
- **The widest content is now a list, not a cell**: on 8.14's database the
  landing with 2,365 linking pages is one `<details>` closed by default and
  one `<li>` per page. It was not rendered on that database in this round,
  only on the e2e fixture (`mise run shot`, whose summaries are collapsed in
  the screen PNGs), so 8.6 resumes on 8.14's digest to see the phone.
- **The landing in a summary is plain text, its link inside**, so that
  opening a summary never follows a link.

## Negative results

- **Rejected: keeping a table with the linking pages dropped**: the first
  hops are as long as the pages, and the maintainer's finding was the empty
  width, not one column.
- **Rejected: a `<dl>` for the hops and counts**, a `dl.meta` grid being the
  header's; paragraphs read the same at a phone's width.
