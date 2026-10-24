# P8.16 — A trailing slash is pedantic: notes

What landed, for P8's spec ([`roadmap-P8-history.md`](roadmap-P8-history.md),
"A trailing slash is pedantic", first bullet and last): the reason in
`store::views`' `link_verdicts` (no new rule: it sets the reason of a link
that would be `ok`), `report_redirects`' `reason` column, the `redirects`
report's and the digest's rows split and ordered by it, the gate's introduced
redirected links showing and ordering by it, the digest's *Reason* column,
D70 and D7 amended, and the manual's `report`, `check-dir` and `digest`.

## What the next slices inherit

- **8.19 reads a link's level off `outcome` and `reason`**: `ok` with
  `pedantic_trailing_slash` is the grey ring, and `store::PEDANTIC_TRAILING_SLASH`
  is the constant. In the digest's redirects the reason is a column already,
  `null` or the pedantic one, so a row's level needs no new field.
- **The redirects rows are keyed by landing and pedantic-or-not.** A landing
  that links reach both through a slash-only redirect and through another
  has two rows, `reason` the only difference. Grouping by the raw reason
  split a landing by `robots`, `no_anchors` and the like, so the report maps
  every other reason to `null` (`report::redirects`).
- **A pedantic link with a missing fragment is `broken_anchor`**, and a
  slash-only redirect onto a broken landing is the landing's verdict: only an
  `ok` link takes the reason. On 8.14's database, 8 more links go through a
  slash-only redirect and are not pedantic: 3 `broken_url`, 1 `error`, 4
  `unchecked`/`robots`.
- **8.17 asks the toggled URL**, which is the other half of the same ruling
  and touches `crawl`, not these reports. 8.18's doubled slash is a redirect
  whose final URL has `//`, which the comparison here refuses (`a//` against
  `a` differ by two slashes), so the two do not overlap.

## Shown on 8.14's database

`runs/p8.14-en-us-20261024-1322/en-us.duckdb` (a copy, opened with this
slice's build): **505 links to 267 targets are `ok`, `pedantic_trailing_slash`**
(every one an `anchor`). The digest's redirects are 709 rows, the 267 with the
reason listed after the other 442. The review's 496 links to 259 targets
(8.6's notes, "Finding 6") read the same rows by another count; the 8 to 9 the
view adds are the URLs with a query, such as
`pagure.io/…/issues?status=all` to `…/issues/?status=all`, which a comparison
of the whole URL's ends misses.

## Negative results

- **`rtrim(url, '/')` is the wrong comparison**: it strips every trailing
  slash, so `a//` against `a` would count, and it misses a slash before a
  query. The view strips one slash before the end or the query on both sides
  (`store::views`).
- **No new column in `link_verdicts`**: the reason is the existing one.
  The spec's "a sixth outcome" refusal is D70's Rejected line.
