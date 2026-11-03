# P9.4 — The guide's scheduling, publishing and delivery: notes

What 9.5 to 9.10 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The scheduled job is one
script", "The published directory is the contract", "Delivery is pushed or pulled"
and "The workflows, step by step". The decisions are D103, D104 and I28.

## What landed

- **The guide** (`docs/manual/deploy.md`) gained the scheduled job: the script
  `bruce-nightly.sh` (S1 to S6 of the spec), the systemd template service and the
  two timers with their environment files, the one-line crontab, the every-locale
  weekly crawl and its `bruce-locales.toml`, the published directory's contract,
  what grows and what bounds it, the schema change, and delivery by `DELIVER` (an
  rsync example) or by `bruce-pull.sh`. Its config part says the hash scope, asks
  docs.fp.o three a second (`per_second = 3`) and the forge one, and quotes the
  crawl's time as the forge's share (about 1:15).
- **A block may name two paths** (`toml file=bruce.toml file=bruce-locales.toml`),
  so the weekly config shares the nightly one's sections and differs in its root.
  `sim.py`, `src/config/tests.rs` and `tests/extract.rs` read the first path as
  before; `sim.py` requires both configs.
- **`sim.py`** also runs `bash -n` over each script, writes a `#!` file
  executable, and wires the units (a timer starts an instance of a template whose
  `EnvironmentFile` and `ExecStart` script the guide tags).
- **`cli::accepts`** parses a command line as `run_from` would and runs nothing;
  `cli::tests::every_bruce_line_of_the_guides_scripts_is_a_command_line_bruce_accepts`
  reads each `bruce` line of the guide's `bash` blocks through it. It is in the
  checks `mise run check` runs, where `sim` is not. A line the test reads starts with `bruce`,
  its values `$`-words; a script that changes that style fails the test's count of lines.
- **`scripts/deployment.py`** runs the two scripts and holds a published
  directory to the contract. `test_deployment.py` runs them against a stub `bruce`
  (every branch: a schema change, a failing digest, retention, a failed stage, the
  pull's refusals); `e2e.py` runs the job twice with the real binary over the
  fixture site, then the pull over the second night's directory.

## What the next slices inherit

- **9.5**: the guide's last part is the CI workflow, and the guide's opening says
  it is not yet written (strike that sentence). C3 reads names from `SHA256SUMS`
  by pattern (`bruce-<16 hex>.tar.gz`, `config-…toml`, `baseline-…duckdb.gz`), and
  `deployment.BY_CONTENT` holds those patterns. `e2e.exercise` leaves a published
  directory at `runs/e2e/deployment/run/pub` for 9.5's e2e step to fetch from.
- **9.6**: the dev deployment's workflow sets `PUB` to the `gh-pages` branch's
  files **without `.git`**: the job renames a new directory over `PUB`, which a
  checkout's `.git` would not survive. It runs the job with `KEEP=` (no history),
  `DELIVER` unset, and commits `PUB` itself. The release is pinned by the
  workflow's own `ARCHIVE`; the first `-rc` tag is the maintainer's.
- **The crawl's time is unmeasured at three a second.** The guide's "about an
  hour and a quarter" is 9.1's measurement at one a second (1:16, 295 MiB), the
  forge's share bounding it. A detached review crawl under the final config was
  launched (`docs/status/history/2026-11-03.md`); its `time.log` is the check, and
  9.6 measures the time on Actions.
- **The weekly crawl's `bruce-locales.toml` is only loaded**: nothing has crawled
  with it. 9.9 does, and corrects the guide's 220,000 requests and 20 hours.

## Negative results

- A bash `${VAR:?message}` with an apostrophe in the message does not parse
  (`bash -n`), and the guide's scripts are read by people who copy them: no
  apostrophes in an expansion's word.
- The guide's script cannot test a `bruce` line with an array or `"$@"` in it: the
  line check substitutes a number for each `$`-word, so a conditional argument is
  written as two lines.
- `set -e` is ignored inside a function called from `||` or `if`: `stage` runs in
  a subshell as a statement of its own so a failed copy leaves `PUB` as it was.
- Naming the latest digest by content was not done, and the review kept it so
  (D103).
