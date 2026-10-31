# P7.5.1 — The gate's `fail` list: notes

What the next slice inherits. The decision is D83.

## What landed

- **`[gate]` `fail`**: `config::policy::gate` holds `FailOn` (`broken_url`,
  `broken_anchor`, `error`, ordered), `DEFAULT_FAIL` and the `[gate]` table.
  `Policy::gate_fail` is recorded as one `gate.fail` row, comma-joined in
  `FailOn`'s order; `Entry::written_as` writes it as a TOML list, so
  Housekeeping states a list that has lost a default element with no other
  change.
- **`report::gate`** reads the CI run's row (`recorded_fail`, no row being the
  default) and `broken` takes the list for both `introduced` and `already`: a
  link is in them by an outcome on it, and the baseline's key counts only at an
  outcome on it. `error` joins `report_errors` to `report_broken` by name, so
  its rows carry the same columns. `report introduced` is the same `gate`.

## For P7.5.2 and P7.6

- **`--config` (P7.6) writes `gate.fail` too**: a trial policy's
  `Policy::entries()` already carries it, and the gate reads it off rows like
  `Digest.severity` is read.

## Negative results

- Nothing measured: the gate's SQL gains an `IN` list and, with `error` named,
  a union of two inlined views.
- An outcome left off the list was in no gate section, and the inverse check
  ignored the list, with an empty list refused; 7.5.3 replaced all three
  (D83).
