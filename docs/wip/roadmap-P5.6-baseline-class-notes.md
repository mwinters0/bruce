# P5.6 notes: the baseline class

Outcome: `bruce check-dir DIR --prefix URL --baseline FILE|URL [--externals
skip|baseline|live] [--config f] [--db f]` imports the baseline as a run
(`file.rs`, D52), records it in the CI run's `baseline_run`, and answers a URL
the build lacks as D54 has it. A built page's target is `local` (D51), a
`not_built` verdict too, a copy `baseline`, a request `live`. Run on
`runs/e2e/partial` against a baseline crawled from `runs/e2e/whole`, it writes
the 23 targets and 3 pages P5.5 did, with nothing new broken: the three
`unresolved_xref` on `tools/usage/` wait for P5.7, and the footer's
`/legal/…` links are broken in the baseline too, so they are copies.

## What the next slices inherit

- **P5.7 cannot resolve a link while the page is written.** Whether an xref's
  component-version is held is known only once every built page is read, the
  same wait an unbuilt URL has (`Shared::settle_unbuilt`). A tagged link's row
  is written with the page, its target the page itself; Resolve has to hold
  such rows back until the set is final, then write them with the resolved
  target, or the row is a rewrite of an appended one. `Pending` is where the
  held set lives (`crawl/baseline.rs`), `Held::holds` takes a URL, not a
  refspec: Resolve answers the refspec's component against `Held`'s keys.
- **P5.9's touched pages come from the CI run's `page_facts`**, not from
  `Held`, which is gone with the process. A copied page carries no facts
  (D52), so a query for the run's `antora.component` and `antora.version`
  never counts a baseline page.
- **P5.8's exit status needs typed errors.** `check_dir::check` returns
  strings, so every failure is exit 1; a bad config, an unreadable baseline,
  a schema mismatch and a prefix out of scope are exit 2 in the spec. The
  baseline's own errors are `store::Error` (`NotBruce`, `SchemaVersion`,
  `NotBaseline`) and `file.rs`'s strings.
- **The output names the baseline.** The line says `against baseline run N`;
  the spec's date of the baseline's run is a query of `runs.started_at`, not
  printed yet. `report` with no `--run` picks the latest finished run, the CI
  run once it finishes (P5.4's note), which `check-dir` leaves unfinished if
  the process is killed.
- **Live requests now leave `check-dir`.** A URL under the prefix the build
  lacks, outside every held component-version and unknown to the baseline,
  is asked of production (`Routed` sends anything the build does not hold to
  HTTP), at the config's pace and robots.txt. Ctrl-C is still not handled, so
  a run waiting on a slow host is killed, not stopped.
- **`Summary::targets` counts a copy once**, though `copy_target` may write
  its landing target as well; `cut` is then still claimed less written.
- **The baseline file's temporary copies are in `std::env::temp_dir()`**
  (`bruce-baseline-<pid>-<n>.duckdb`) and go with the `BaselineFile`.

## Limits met, not removed

- **KD5, KD6**: the two the spec named, filed with their markers.
- **A hop onto a URL the build lacks is asked live.** An alias in a built
  page whose target is another component's unbuilt page is a request to
  production, `Frontier::hop_plan` mapping `Baseline` and `Unbuilt` to
  `Status`. Antora aliases stay in their component, so none is known.
- **A bare version holds every version-less URL of its component**, so a
  component held at `master` whose baseline also has `1.0` reads a link to
  `component/1.0/…` as `not_built`. The tie-break from the baseline's versions
  would have put `antora.*` keys in `store` (D39); not built.
- **`latest_version_segment`** (I16): a held version served under another
  name is not recognized, and a page deleted from it is the baseline's `ok`.

## Negative results

- **Routing by the prefix is wrong.** `Routed` first sent every URL under the
  prefix to the build, which made a URL the build lacks and the baseline does
  not know a local 404 where D22 says live; it routes by what the build holds.
- **The crawl's methods cannot sit in the `baseline` child.** An `impl
  Shared` there and `Pending` used by the parent are a cycle
  (`module_graph.py`), so `baseline.rs` holds what a run waits on and
  `crawl.rs` the run.
- **`Frontier::against` cannot take `Baseline` from the child either**: the
  frontier plans with it and `baseline` names `Plan`, so the two types are the
  frontier's.
- **Copying at claim time doubles a landing page.** `targets` has no key, and
  a copy written before the build's own row for a page both reach leaves two;
  hence the copies go last, and `Msg::Copy` flushes the appenders first so the
  copy sees rows still buffered.
