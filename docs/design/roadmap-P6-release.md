# P6 — Release

What P6 does and why, settled in its grilling on 2026-10-21, which drained
its inbox, and in the split the same day that gave the deployment guide to P9
([`../wip/roadmap-P9-deployment.md`](../wip/roadmap-P9-deployment.md)). The decisions are
in [`decisions.md`](decisions.md) and are cited here,
never restated.

## Intent

Anyone can run `bruce` without building it: a release binary (D23), licensed,
carrying its dependencies' notices, and built from a tag by a workflow. What a
deployer needs beyond the binary, the deployment guide's remaining parts, is
P9's.

## P6 runs before P8

Settled 2026-10-21 (maintainer). **A release comes first, so others can
experiment with what is built**; the deployment guide, a content repo's CI
job and the dev deployment are P9's, which still follows P8. P8 is likely to
change the schema, and a 0.x release migrates nothing (below, "The first
release promises no migrations"), so someone experimenting re-crawls at the
release that carries it, a cost only a deployment's kept history would make
real. Rejected: the release waiting with the deployment for P8.

## The first release promises no migrations

Settled 2026-10-21. **A 0.x release migrates no DB**: a schema change is
named in its release notes, and the user starts a new DB, the old binary
still reading the old one. D43's "migrations wait for the first release"
becomes "for a release that promises them", amended by the release slice;
P8's run-history reports are what would make old runs worth migrating.
Rejected: migrations from the first release.

## Releases are GitHub's

Settled 2026-10-21 (maintainer). **A release is published on GitHub,
`mwinters0/bruce`**, at `releases/download/<tag>/<asset>`; D23's "on the
forge" is amended by the release slice. The forge's usage policy admits
Fedora-specific software only (`../wip/roadmap-P9-deployment.md`, "Facts the
grilling found"), and a content repo's CI fetches the binary from the
deployment anyway (`../wip/roadmap-P9-deployment.md`, "CI fetches the binary from
beside the baseline"). Rejected: a Fedora forge org, an Infra ticket and a
ruling on whether `bruce` is Fedora software.

## The release is Debian Trixie's glibc, built from the tag by a workflow

Settled 2026-10-21 (maintainer). **The release binary is
`x86_64-unknown-linux-gnu`, built in a `debian:trixie` container**, so its
glibc floor is Trixie's 2.41 (below, "Facts the grilling found"), and the
README states it: it runs on Trixie, Fedora 42 and anything newer, F44 among
them, and not on Alpine. aarch64 waits for someone to ask. **It is built by a
GitHub Actions workflow in this repo on a `v*` tag push**, `--release
--locked`, and uploaded with `gh release`, under the no-third-party-actions
rule (below), the checkout by `git`. **The repo's releases are immutable**
(maintainer, configured), so a release is published only once its assets are
attached. Rejected: F44's glibc, 2.43, which shuts out Trixie and F42; static
musl, at the risk of DuckDB's bundled C++ on musl; a `mise run release` from a
maintainer's tree, built from whatever that tree holds.

## `bruce` is MIT OR Apache-2.0, and ships its dependencies' notices

Settled 2026-10-21. **The crate is licensed `MIT OR Apache-2.0`**, with both
texts in the repo and `license` in `Cargo.toml`, and **a release bundles a
third-party notices file generated from `Cargo.lock`** (DuckDB, bundled, is
MIT; rustls and the rest Apache-2.0 or MIT), by `cargo about` or a script of
our own under the no-third-party-actions rule. Nothing licensed it before.

Settled 2026-10-22 (maintainer). **The holder is named once, in
`LICENSE-MIT`'s `Copyright (c) 2026 Michael Winters`**: the author of every
commit, a later contributor adding their own line. The year is first
publication and is never bumped. No `authors` in `Cargo.toml`, no holder line
in `LICENSE-APACHE` or a `NOTICE`, and the README names the two licenses and
points at the files. Rejected: `The bruce contributors`, naming no legal
person; a year range, upkeep with no legal effect.

Settled 2026-10-22 (maintainer). **The notices carry the license text of every
library DuckDB bundles**, since most of their terms require it beside a binary
and the crate's `duckdb.tar.gz` ships none; a pointer to DuckDB's repository
does not meet them. **The section covers every directory under the tarball's
`third_party/`**, worded as what the bundled source holds, not what is
compiled: over-crediting costs nothing, and it stays true across features and
DuckDB bumps. Rejected: exactly the libraries linked, which needs a transitive
include scan or a hand-kept list of the header-only ones. **mbedtls
(`Apache-2.0 OR GPL-2.0-or-later`) is taken under Apache-2.0**, said in one line.
**The texts are vendored in the repo** with the DuckDB source id they were
taken at, fetched from DuckDB's repository at the tag the tarball names only by
a deliberate refresh, so generating and checking the notices stay offline.
Rejected: fetching at generation, which puts the network in `mise run check`;
`libduckdb-src.zip`'s `duckdb.cpp` trailer, upstream's format and not a
contract. **A notice a library's source carries and its LICENSE omits is
vendored too**, verbatim from the tarball's source (hyperloglog's Redis BSD-3,
libpg_query's PostgreSQL and Regents notices), and is refreshed by hand.
**The one exception is libpg_query's PostgreSQL License paragraphs**
(maintainer, 2026-10-22): its source carries the two copyright lines and no
permission paragraphs, which the license asks to accompany them, so the
standard text from postgresql.org is vendored by hand beside the lines and
marked as hand-vendored. The drift check cannot hold it verbatim against the
tarball, which is accepted, the text not changing. Rejected: the copyright
lines alone, which leave the license's own condition unmet. It lands before
the release workflow, so `v0.1.0-rc.1`'s immutable assets already carry it.
**Each directory's license files are vendored**, `LICENSE*`, `COPYING*`,
`NOTICE*` and `AUTHORS*` (tdigest's `NOTICES`, re2's `AUTHORS`). **The check
fails on drift**: a vendored source id other than the tarball's, a
`third_party/` directory without texts or a vendored one the tarball lost, and
a hand-vendored notice no longer verbatim in its source file, comment markers
and whitespace aside. A bump then cannot land with stale texts.
**An MPL-2.0 crate's entry names its exact source**, the crates.io download of
that version, unmodified (MPL-2.0 §3.2(a)); its repository URL names a branch,
not what was built.
Facts: [`../status/history/2026-10-22.md`](../status/history/2026-10-22.md),
"6.1's unattended calls reviewed" and "The PostgreSQL License paragraphs".

## No third-party action, in any workflow, is a standing rule

Settled 2026-10-21 (maintainer). **No workflow uses a third-party action**:
every step is a command in the job's container, a checkout included, and what
an action would give is built from commands. A script test fails on any
`uses:` under `.github/workflows/`; P9 extends it to the workflow the guide
tags. The slice that adds the first workflow adds the rule to
`roadmap.md`'s "Standing rules" with that check, a rule landing
with its check.

## Prereleases while iterating, `v0.1.0` at the close

Settled 2026-10-21. **Releases are tagged `v0.1.0-rc.N` while the workflow
is got right, and `v0.1.0` when P6 closes**; the release workflow marks a
tag with a `-` suffix a GitHub prerelease, so `latest` never names one.
Rejected: full releases from the start, each fix while iterating an
immutable `v0.1.x`.

Settled 2026-10-22 (maintainer). **`v0.1.0-rc.1` is tagged with the binary
linking `libstdc++` and `libgcc_s` dynamically, and a static build
(`-static-libstdc++ -static-libgcc`) is tried before `v0.1.0`**; ship static or
dynamic is decided on what the attempt finds, and D23's "self-contained" with
it. The attempt is out-of-band M13 (`out-of-band.md`), ahead of 6.3.
Reasoning: [`../status/history/2026-10-22.md`](../status/history/2026-10-22.md),
"The `libstdc++` and `libgcc_s` call, answered".

## Facts the grilling found

- **This machine's glibc is 2.44**, newer than Trixie's and F44's, so a
  binary built here does not run there; no container runtime runs here today.
- **Trixie's glibc is 2.41, F44's 2.43** (`sources.debian.org/api/src/glibc/`,
  `2.41-12+deb13u4`; `mdapi.fedoraproject.org/f44/pkg/glibc`, `2.43-2.fc44`).
  docs-fp-o's image, fedora-minimal 42, is 2.41 too.

## The slice order the split proposed

Proposed 2026-10-21; `../status/STATUS.md`'s checklist is the schedule.

1. **6.1 License and notices** (above, "`bruce` is MIT OR Apache-2.0").
2. **6.2 The release workflow**: a Trixie build on a `v*` tag, the archive of
   binary, notices and licenses with its sha256, `gh release` draft, upload,
   publish; the no-third-party-actions rule and its check; D23 and D43
   amended. Closes on `v0.1.0-rc.1` built and run on Trixie and F44.
3. **6.3 `v0.1.0`**, and the README's quickstart: where the binary is, its
   glibc floor, and that a 0.x release migrates no DB.

Every tag push is the maintainer's: a round stops there.
