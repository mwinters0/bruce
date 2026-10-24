# P8.18.2 — The doubled slash shown where it is: notes

The digest's `broken` and `broken_targets` rows carry `landing`, the URL a
`doubled_slash` target redirected to where that URL doubles a slash and the
target's own URL does not, else `null` (`sections::landing_join`). The
drill-down's *Resolves to* shows it on a second line, "→ lands on …", and the
fix-once table's *Broken target* shows it the same way. Every doubled run of `/` in a
`doubled_slash` row's *Written as*, *Resolves to* and landing is a `<mark>`
(`html::slashes`), red and bold, read by the rule `doubled_slash` reads
(D71). The JSON gains the field without a `format_version` change, an added
field being none.

On 8.14's database 41 of the 86 `doubled_slash` links, on 16 pages, show a
landing, against 35 targets. `en-US/project/`'s two are
`arm.fedoraproject.org/` (lands on `fedoraproject.org//`) and
`fedoraproject.org/code-of-conduct` (lands on `…/code-of-conduct//`).

**Negative results on the digest's time**, release builds on a copy of
8.14's database, baseline 3.76 s on this machine:

- Joining the `doubled_slash` view to tell a target's own doubled slash from
  its landing's took the digest to 4.2 s, the view scanning every target
  again for each of the two queries.
- Testing the target's URL inline, through `store::TARGET_URL_DOUBLES` (the
  view's own expression, exported so it is not copied), but joining every
  target by URL, still took 4.1 s. That join added 0.15 s a query.
- Joining only the redirected targets (799 of 173,288) that double no slash
  themselves gives 3.78 s, indistinguishable from the baseline.

`fixtures/antora/digest.html` changes only in the `mark` rule and the pages
section's note, the fixture site holding no doubled slash. The review's
digest is rewritten under `runs/p8.6-review/digest/`.
