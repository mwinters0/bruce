# P14.2 — The chain limit: notes

What P9 inherits, and the negative results. The decisions are D100 and the
amendments to D7, D70, D74 and D79; the spec is
[`roadmap-P14-redirects.md`](roadmap-P14-redirects.md).

## What landed

- **`max_redirects`**: `Policy::max_redirects` and `HostJudging::max_redirects`,
  written as `max_redirects` rows (run-wide and per host), 0 to
  `config::MAX_REDIRECTS` (10), which `crawl::hops::MAX_HOPS` is now derived
  from. No schema change: D78's rows take a new key.
- **The rule** is rule 5 of `link_verdicts` (after a target's own fault, before
  the fragment): `tj` carries `too_long`, the stored chain's length against the
  linked host's row, else `[default]`'s, else 10. The host is the linked URL's
  (`$LINKED_HOST`), not the answering host `status` is judged by.
- **`target_moved`** also holds where the chain differs while the landing does
  not; the digest's broken lists name the landing of a `too_many_redirects` row
  (`sections::landing_join`, the default view's `target_cell`).
- **Housekeeping, the policy changes and the trial `--config`** needed no code:
  they read the rows against `Policy::default().entries()`, and `is_judged` is
  true of the new key.

## What P9 inherits

- docs.fp.o's limit is the guide's to set. At a limit of 1 the 2026-11-01
  database breaks 139 targets (31 slash-only), at 0 all 716 `ok` ones (the
  spec's "What docs.fp.o's chains are"); `report --config` shows any limit
  against a kept run with no crawl, but the kept databases are of schema 6 and
  D43 refuses them, so a fresh en-US crawl comes first.
- A forge's login walls are the reason for the per-host key (four hops from
  `src.fedoraproject.org` to `id.fedoraproject.org`); the guide should say a
  host's limit is the linked URL's, not the landing's (D100).

## Negative results

- `report_redirects` and the redirect listings needed no change: the view
  already keeps every counted redirected link whatever its verdict, as a
  redirect onto a 404 shows.
- No test reproduces the crawl's own ceiling through the rule: a chain past 10
  never reaches `link_verdicts` as `ok`, the target being `broken_url` with its
  own reason (rule 4), so the rule and the crawl cannot both name one target.
- The rule's cost was not measured against a large database: it adds two policy
  joins to `tj`, each on a handful of rows per run, and a list length per
  target.
