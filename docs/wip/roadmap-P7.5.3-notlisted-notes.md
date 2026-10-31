# P7.5.3 — The gate lists what its `fail` list leaves out: notes

What the next slice inherits. The decision is D83, amended.

## What landed

- **`report::gate::Gate`** gains `introduced_not_failing` and
  `inverse_not_failing`, text sections after `introduced breakage` and after
  the inverse section, and JSON keys beside `introduced` and `inverse`. `fails()`
  reads the failing tables alone. `broken` takes the outcomes it judges by, so
  the not-failing section is the same query over the broken outcomes the list
  leaves out (`left_out`); `outcomes` writes `(NULL)` for none.
- **`report::inverse::split`** cuts the inverse table by `removed`: `page` and
  `file` are `broken_url`, `anchor` `broken_anchor`. `report inverse` still
  lists every row, as the check finds them; the list is the gate's.
- **An empty `fail` list loads** and is recorded as an empty `gate.fail` row;
  `Entry::written_as` writes it `gate.fail = []`. `FailOn::from_row(Some(""))`
  is empty, no row still the default.
- The manual (`check-dir`, `config`, `report`) states the sections, the
  inverse check following the list and report only.

## For P7.6

- `--config` carries `gate.fail` in its entries as before; the not-failing
  sections follow from the list with no further work.

## Negative results

- Nothing measured: one more `IN` query over the same views, and a partition of
  a table already fetched.
- "Introduced" in the not-failing section is the baseline lacking the key at a
  left-out outcome, as the failing one is at a listed outcome. A link the
  baseline had at a listed outcome and the run now has at a left-out one is
  listed there; a link broken at a left-out outcome in both runs is in no
  section, as the already-broken count holds the list's outcomes alone.
