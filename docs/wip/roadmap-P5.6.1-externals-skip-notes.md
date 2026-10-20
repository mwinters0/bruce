# P5.6.1 notes: `--externals=skip` means never ask

Outcome: under `--externals=skip` an allowlisted URL gets the baseline's rows
where its index holds the target (`copy_or` in `Frontier::external`), else is
`unchecked`, reason `externals_skipped`; and a redirect hop onto an allowlisted
host ends the chain `unchecked`, reason `externals_skipped`, without a request
or a robots.txt read (`Frontier::hop_plan`, `Shared::walk`). D54 carries the
decision; the review that settled it is `../status/history/2026-10-20.md`.

## What the next slices inherit

- **P5.8's `check-dir.md` says `skip` governs allowlisted hosts only**, and
  that it copies the baseline's verdict where there is one: a deployer reading
  it as an offline mode is wrong, an in-scope URL the build lacks and the
  baseline lacks still going live (D22's `Rejected` line).
- **A copied external's verdict reaches the gate with its linking page**, so a
  new link to an external broken in the baseline is introduced under every
  mode; only a URL the baseline never asked about is `unchecked` under `skip`,
  and an `unchecked` is no breakage to the gate.

## Limits met, not removed

- **A hop onto an allowlisted host is ended whether or not the baseline holds
  the target**: the chain's own row says `externals_skipped`, and the hop's
  target is neither copied nor asked. A copy would need the chain row to
  borrow the baseline's verdict for a different URL.
- **The allowlist test in `Frontier::hop_plan` repeats `plan`'s order**
  (`is_skipped_external`: allowlisted host, no root's scope), because `plan`
  returns a class, not whether the URL was external; `Plan::Baseline` alone
  does not say, a root's URL outside the build's prefix being copied too.
