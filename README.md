# bruce

A link checker that checks anchors as well as pages, and remembers what it found.

Most link checkers stop at the page. If `guide/#install` loads `guide/`, they
call the link fine, even when the page has no `#install` anchor any more.
`bruce` checks the fragment too, and keeps every page, anchor and link of each
crawl in a [DuckDB](https://duckdb.org/) database. From that one database you
get broken-link reports, inbound-link rankings, orphans, the sitemap compared
with what is actually linked, and the difference between two runs.

It works on any site with no configuration at all. Its main target is
[docs.fedoraproject.org](https://docs.fedoraproject.org/), and it understands
Antora sites. It traces each page back to the repository and file it was built
from, and it can act as a CI gate for one content repository's partial build:
the check fails only on breakage that the change introduced.

## Quickstart

There is no release yet, so build it from source. The toolchain is pinned in
`mise.toml`; with [mise](https://mise.jdx.dev/) installed:

```sh
mise install
cargo build --release        # the binary is target/release/bruce
```

Plain `cargo` (Rust 1.99 or later) works as well. To put `bruce` on your
`PATH`, run `cargo install --path .`.

Crawl a site. Given only a URL, `bruce` crawls everything under that URL and
obeys the site's `robots.txt`, including any `Crawl-delay`:

```sh
bruce crawl https://docs.example.org/en-US/
```

This writes one **run** to `bruce.duckdb` in the current directory. To read it:

```sh
bruce report broken          # broken pages and missing anchors
bruce report errors          # links that could not be checked (timeouts, 5xx, 403)
bruce report stats --top 20  # the pages with the most breakage
bruce report diff            # what changed since the previous run
```

Each report can also print JSON (`--format json`). The database is ordinary
DuckDB, so `duckdb bruce.duckdb` lets you query it directly.

A large site takes hours at a polite rate. Start a long crawl detached from
your terminal:

```sh
setsid bruce crawl --config bruce.toml > crawl.log 2>&1 &
```

## Gating a content repo's CI

A scheduled full crawl is exported as a **baseline**, and each change's build
is checked against it:

```sh
# on a schedule, against the live site
bruce crawl --config bruce.toml
bruce export-baseline --out baseline.duckdb

# in the content repo's CI, after its Antora build
bruce check-dir build/site --prefix https://docs.example.org/en-US \
  --baseline https://example.org/baseline.duckdb --config bruce.toml
```

`check-dir` exits 1 only on breakage the change introduced. That includes
links elsewhere on the site to pages or anchors the change removed. Breakage
already present in the baseline is counted but never fails the check.

## Getting started

The manual is in [`docs/manual/`](docs/manual/):

- **[Deploying `bruce`](docs/manual/deploy.md)**: start here. It walks through
  writing a configuration for a real site, using docs.fedoraproject.org as the
  example, and explains each choice.
- [Configuration reference](docs/manual/config.md): every key in `bruce.toml`,
  including roots, scope, the external-host allowlist, `skip` globs, request
  rates and `robots.txt`.
- [`bruce crawl`](docs/manual/crawl.md): what a run records, its output, and
  how to stop one.
- [`bruce report`](docs/manual/report.md): every report, its columns, and the
  views for querying the database yourself.
- [`bruce check-dir`](docs/manual/check-dir.md): the CI gate, how each link is
  answered, and what fails.
- [`bruce export-baseline`](docs/manual/export-baseline.md): writing the
  baseline file that CI checks against.
- [`bruce prune`](docs/manual/prune.md): deleting old runs so the database
  does not grow without limit.

`bruce help <command>` prints each command's arguments.

## Development

```sh
mise run test     # the test suite, under cargo-nextest
mise run check    # everything a change is held to: fmt, clippy, tests, docs checks
mise run e2e      # build a fixture Antora site, crawl it and gate it (slow, needs node)
```

[`docs/status/STATUS.md`](docs/status/STATUS.md) describes what is built and
[`docs/design/roadmap.md`](docs/design/roadmap.md) what comes next. The design
decisions, each with its reasoning, are in
[`docs/design/decisions.md`](docs/design/decisions.md). The development process
is in [`docs/process.md`](docs/process.md).
