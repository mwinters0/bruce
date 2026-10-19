# P4.8 `diff`: notes

`bruce report diff` is `report/diff.rs`: a `UNION ALL` of two halves over
`report_broken`, `new` (broken in `to`, no matching broken row in `from`) and
`fixed` (the reverse). A link's key is its page's `url_id`, its target's URL
and its fragment (`''` for none), all stable across runs (D26), and `QUALIFY
row_number()` keeps one row per key. `report/run.rs` chooses the pair and
returns a `Chosen`; `Output.from` carries the base run and whether it
finished, and `cli::report::check` refuses `--run` with `diff` and
`--from`/`--to` with the rest as a usage error (exit 2).

## What the next slice inherits

- **`prune` needs nothing from here.** It is `cli/prune` calling `store`
  directly, as the spec places it.
- **Not broken includes absent.** A broken link whose page or text is gone
  is `fixed`, and one on a page the base run did not reach is `new`; the
  manual says so. A column naming the link's state in the other run (absent,
  `ok`, `unchecked`) would split them, and costs a join to `link_verdicts`
  that no consumer has asked for. P4.10's `diff` against P4.4 is where to
  look for the need.
- **`--from` alone reads against the latest finished run, `--to` alone
  against the finished run before it.** The spec names only the default and
  both options; this is the reading that keeps either option useful alone.
- **A filter is read in the run a row comes from**, so a page that moved
  repo between runs appears under both in `fixed` and `new`.
- **`--kind` applies to both sides**: a link broken as an image in `from`
  and as an anchor in `to` is `new` under `--kind anchor` and `fixed` under
  `--kind image`.

## Negative results

- Not run on a real DB: P4.4's is not in the tree (see P4.7's notes), and
  P4.10 is its first real input. The tests build both runs by hand.
