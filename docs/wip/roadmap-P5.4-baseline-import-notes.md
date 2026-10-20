# P5.4 notes: the baseline's import and the copy of a target

Outcome: `Store::import_baseline(file)` makes the file's run a run of the
database or finds it there, and `Store::copy_target(from, to, url_id)` copies
a target's rows between two runs (D52; `src/store/baseline/import.rs` and
`target.rs`). Nothing calls either yet. `Store::export_baseline` and the
import share `copy_run`, the six-table copy, renumbering a run as it goes.

## What the next slices inherit

- **The import returns `Imported { run, reused }`** and refuses, before
  copying, a file with no `meta` (`NotBruce`), another schema version
  (`SchemaVersion`), and one holding other than one finished run
  (`NotBaseline`, which `check-dir` maps to exit 2). A path that is not a file
  is a `Db` error, the attach being read-only and so never creating it. The
  file is left as it was.
- **`reused` also covers the run the file was exported from**: importing into
  the history database returns the original run, copying nothing.
- **`copy_target` returns false when `from` holds no target for `url_id`**,
  which is the frontier's cue to go live (spec, "What goes live"). True means
  the run now holds it, copied or already there.
- **It runs on the `Store`'s connection, which the writer thread owns during
  a crawl.** P5.6 either sends the copy to the writer as a message or does it
  before the crawl starts; the method itself takes no lock and a second
  connection would not see an uncommitted run.
- **It copies the target, the landing target one level (the way
  `link_verdicts` joins it), and each one's `pages` row and `anchors`**.
  `page_facts` are not copied, so a query for the build's held
  component-versions (P5.6) over the CI run's facts never counts a baseline
  page. A held-set query reading `pages` instead would; it reads facts.
- **A copied target is `baseline`, the landing too** (D51); a target the run
  already holds, such as a built landing page, keeps its row and its page.
- **The imported run is a finished run in the CI database**, so a `report`
  with no `--run` picks the latest finished one: the CI run once it finishes,
  but the baseline if the CI run stopped. P5.5 or P5.8 decides whether
  `check-dir` must name its run in what it prints.

## Negative results

- **`main.<table>` is not a catalog qualifier.** The opened database's
  catalog is named for its file, so `main.main.targets` fails with "Catalog
  \"main\" does not exist"; the local side is the schema `main`, an attached
  side `<alias>.main`, which `copy_run` takes whole.
