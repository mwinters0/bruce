# P8.17 — A toggled slash that answers: notes

A broken 4xx target with no hop before it has its URL with the final `/`
toggled asked once, for its status alone (`crawl::slash`, called from
`Shared::walk`), and a 2xx sets the target's reason to `wrong_trailing_slash`
(`store::WRONG_TRAILING_SLASH`, `crawl::outcome::answered`'s new argument).
`link_verdicts` passes a non-`ok` target's reason through, so no view changed.
The digest's `broken` and `broken_targets` gain `fix`, the target's URL toggled
again (`url::toggle_slash`, the one toggle), shown as "→ try …" under the
target in the drill-down and the fix-once table. No `format_version` change,
an added field being none. D72 holds the decision; the manual's `crawl`,
`check-dir`, `report` and `digest` say what a reader sees.

## What the next slices inherit

- **The reason is a flag on the target, not a verdict.** The link keeps
  `broken_url`; 8.7 and 8.8 compare outcomes, so a target gaining the reason
  between two runs is not a change to them, but a section 3 row for a link
  newly broken will show it in its `reason`.
- **The yield is unmeasured.** No run was made: a probe is a live request per
  broken 4xx target, and 8.14's database cannot say what the toggled URLs
  answer. The next full crawl's `report broken` answers it
  (`reason = 'wrong_trailing_slash'`), and the notes of whichever slice reads
  that crawl first should give the number.
- **The cost is one HEAD and, on a 4xx, one GET** per probed target
  (`Mode::Asset`), under the host's pace; the digest's time is unchanged, the
  work being the crawl's.

## Negative results

- **No target row for the toggled URL.** Claiming it would have made it a
  page of the run where it is in scope, read and its links followed, counted
  in the pages and orphans of a run that nothing links to; and a link naming
  it later would lose the claim race to a status-only read (KD4's shape).
- **A target redirected before its 4xx is not asked about.** Its landing's
  row is written with the same reason as the redirecting target's, so a reason
  set by the toggle of one would be read as the toggle of the other
  (`slash::toggled`, `form == item.form` in `walk`).
- **A toggled URL that redirects is not an answer.** Its destination is
  unknown without following hops, and may be a 404 itself.
