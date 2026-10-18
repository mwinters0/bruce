# P3.7 `bruce crawl`: notes

What P3.8 onward inherits, and what was refused.

- **`cli::run_from(args)`** parses with clap's builder (no derive, colour or
  suggestions: clap was already in criterion's tree), one child module a
  subcommand, handed the registry by the parent and reading nothing of it,
  so the edge runs one way (`module_graph.py` refused `super::registry`). `cli::crawl::configure` is the pure half: the roots from the
  file or the URL, the URL's `check_robots_hosts`, the profile check, and
  the `runs.config_hash`, the `xxh3_64` of the file's text, a NUL and the URL
  as given.
- **The order of a run**: the config, the DB opened (a bad or wrong-version
  file refused before any request), the fetcher and the runtime built, and
  only then `begin_run`, so a run row means a crawl started. The writer is
  finished before the crawl's own result is read, its error naming the cause
  of a `WriterGone`. `finish_run` is called only for a run not stopped.
- **`crawl::Stopper`**, from `Crawl::stopper`, is checked where a target
  would start (`drive`) and before each root's discovery. `Summary::stopped`
  is true only when the stop found work left, so a stop after the last
  target still finishes the run. A stopped run wrote whole targets: a task
  started is followed to its end and written.
- **Exit status**: 0 a run finished, broken links or not (verdicts are P4's);
  1 the config, DB or crawl failed; 2 clap's usage error; 130 stopped, at the
  first Ctrl-C after the flush or at once at the second (`process::exit`,
  the writer unflushed). The output is one line, `run N: … targets, … pages,
  … sitemap files`, `no sitemap` when none was read.
- **Calls beyond the spec**, none touching a contract: the exit statuses;
  the writer channel's 4096 rows; a file naming no root and no URL refused
  as "neither"; the profile error listing `auto` and the registry's names;
  the wait at the first Ctrl-C ("Decisions worth another look").
- **Observed once, not a test** (a signal to the test process would reach
  every test): `kill -INT` on `bruce crawl` against a local `http.server`
  printed the stop line, wrote four pages and exited 130.
