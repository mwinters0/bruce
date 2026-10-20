# P5.5 notes: the local build

Outcome: `bruce check-dir DIR --prefix URL [--config f] [--db f]` runs a
crawl over a build directory (`src/cli/check_dir.rs`). `ci::Build` reads the
directory as production URLs, `fetch::local::LocalFetcher` answers from a
map of them, `Crawl::run_seeded` begins from the seeds, and an href beginning
`/` resolves against the prefix (D53). The frontier is untouched. Run on
`runs/e2e/partial` under `http://127.0.0.1:8099/en-US/`, it writes 23
targets and 3 pages, the three `tools/` pages; `report broken` shows twelve
`broken_url`: the footer's three root-relative links on each page, now under
the prefix, and the three `unresolved_xref` on `tools/usage/`.

## What the next slices inherit

- **`check-dir` is a stub of the command.** It takes no `--baseline`,
  `--externals`, `--inverse` or `--format`, always exits 0 once the run is
  written, and prints one line. P5.6 adds `--baseline` (then required) and
  `--externals`; P5.8 the exit status, the output and the manual page. The
  command has no manual page until then.
- **What the local fetcher does with a URL is a stand-in for the routes
  P5.6 opens.** Under the prefix and not built it answers 404, so every link
  into the rest of the site, the footer's included, is `broken_url`; outside
  the prefix it returns a `Failure` (`Other`), which is `error`, so nothing
  leaves the machine. P5.6's frontier decides a URL's route before the crawl
  asks the fetcher, and then these two answers are reached only by a URL the
  frontier meant for the build. The 404 is also what `not_built` replaces for
  a held component-version.
- **The seam to the frontier is `Plan`.** `Crawl::run_seeded` plans a seed as
  a link of kind `anchor` would be, and writes an `Unchecked` plan's target
  itself, so a seed `skip` names is `unchecked`, reason `skipped`. P5.6's
  baseline class is a new `Plan`; a seed is never one, a seed being built.
- **`ci::Build::served()` is every file, `seeds()` the `.html` ones.** The
  top `index.html` is the prefix itself and `x/index.html` is `x/` and
  `x/index.html` both (two keys, one file). Antora's UI assets under `_/` are
  answered, so a stylesheet link is `ok` rather than a 404. P5.6's held set
  is read from `page_facts`, not from the build, so `Build` need not change.
- **The prefix may sit below a root's URL** (`.../docs/sub/` under
  `.../docs/`) and the root's own URL is then not asked about: a run with
  seeds seeds nothing else and reads no sitemap or robots.txt, `Crawl::run`
  being the only path that does. With no `--config` the prefix is the root.
- **Root-relative resolution is `Links::resolve`'s**, which `<base href>` and
  the edit link share, so a profile's own `resolve` hook (P5.7) sees URLs
  already under the prefix.
- **Ctrl-C is not handled**: a local run takes seconds, and a killed one
  leaves `runs.finished_at` NULL like a stopped crawl.

## Negative results

- **A fetcher cannot do the mapping**: `fetch` sits below `ci`, so it takes
  the finished map rather than the directory (the spec's placement, borne
  out by the layering and by `Build` having to know `index.html`).
- **`MAX_BODY` moved to `fetch`** from `http`'s `Settings` default, so the
  two fetchers cap an HTML body at one number.
