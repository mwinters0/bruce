# P7.4 — Comparisons judged by the newer run's policy: notes

What the next slice inherits. The decision is D81.

## What landed

- **`store::JudgedBy`**, a guard setting the connection's `judged_by` variable,
  which `link_verdicts_with` reads where `policy_run` is NULL. It is held by
  `report::gate::gate` and the `inverse` report (the CI run), `diff::diff` (`to`)
  and `digest::build` when a comparison exists (the digest's run), so the
  baseline's links, `diff`'s other run, and the compared run's totals, causes
  and `new` marks (D74, D75) are judged by the newer run's policy.
- The guide's ⓘ for a reason the digest's policy does not judge can no longer
  appear on a compared run's rows, which P7.3's notes expected P7.4 to remove.

## For P7.5 and P7.6

- **`--config` (P7.6) names a trial policy, not a run**: `JudgedBy` takes a run
  whose recorded rows are the policy. A trial file has none, so P7.6 needs the
  guard (and the macro) to read a policy from something other than a run's rows,
  or to write the trial's rows under a scratch run. The variable is the one
  place judging is chosen for the views.
- **Queries reading a target's stored outcome are judged by no policy**, a
  comparison's included: KD23. The severity map and `fail` list of P7.5 read
  verdicts, not these.
- The gate's `fail` list (P7.5) is the CI run's, which is the policy the gate now
  judges the baseline by.

## Negative results

- A parameterised copy of every report view was not built: D81's Rejected.
- `changed` and `anchors-removed` read no verdict of the older run (a content
  hash; its anchors), so they hold no guard.
- Not measured: the guard adds one `SET VARIABLE` per comparison, and the macro's
  `getvariable` is a constant per query.
