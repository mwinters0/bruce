# P6.2 — The release workflow: notes

What landed, per the P6 spec's "The release is Debian Trixie's glibc, built
from the tag by a workflow": `.github/workflows/release.yml`, whose header
comment says what it does; `scripts/workflows.py`, which fails on a `uses:` in
any workflow and prints a workflow's steps as one shell script; the rule in
`roadmap.md`'s "Standing rules"; and D23 and D43 amended. The slice closed on
`v0.1.0-rc.1`, pushed, built by the runner, published, downloaded and run on
Trixie and F44 (below).

## What the rest of 6.2 and 6.3 inherit

- **The tag is `v<x>.<y>.<z>[-<pre>]` and its base must be `Cargo.toml`'s
  `version`.** The workflow refuses any other, and one moved since the push
  (`HEAD` is not `GITHUB_SHA`). A `-` suffix makes a prerelease. A rerun after a
  failure deletes the tag's draft and creates it again; once published, a
  release is immutable and the next try is `v0.1.0-rc.2`. Debian's `gh` is
  2.46.0, which has `--verify-tag` and `--generate-notes`.
- **What to check in a run**: every step green; "Build" printing `newest glibc
  symbol:` at or under `GLIBC_2.41`; the release a prerelease.
- **The asset** is `bruce-<tag>-x86_64-unknown-linux-gnu.tar.gz` holding a
  directory of the same name with `bruce`, `THIRD-PARTY-NOTICES.txt`,
  `LICENSE-MIT` and `LICENSE-APACHE`, beside `<asset>.sha256` in `sha256sum`'s
  format. Archiving the same tree twice gave the same sha256 (a sorted tar with
  the commit's time, `gzip -n`), so a rebuild's archive differs only by the
  binary.
- **6.3's README states** that `bruce` needs the system CA bundle even for
  an `http://` crawl, and that glibc is its only library (M13, below).

## The release, checked

[Run 39703334491](https://github.com/mwinters0/bruce/actions/runs/39703334491)
passed; "Build" logged `newest glibc symbol: GLIBC_2.38`. The release is
published, a prerelease, not a draft, with the two assets. Its downloaded
assets are `runs/release-rc1/dl/` (the archive is 19.7 MB, the binary 60 MB).
`sha256sum -c` passed; the archive holds the four files; the digest GitHub
shows for the archive is the `.sha256` file's.

`bruce --version` printed `bruce 0.1.0`, and `bruce crawl
http://127.0.0.1:8765/whole/` (`fixtures/antora/pages` served by `python3 -m
http.server`, `--net host`, rootful `nerdctl`, the archive mounted read-only)
printed `run 1: 40 targets, 7 pages, no sitemap` and exited 0, in `debian:trixie`
(after `apt-get install ca-certificates`), `fedora:44` and
`registry.fedoraproject.org/fedora-minimal:42`. The three were identical.

## The rehearsal

`uv run workflows.py script release.yml --skip "Check out the tag" --before
Publish`, run with `bash` in `debian:trixie` (rootful `nerdctl`) over a copy of
the tree, at the tree of this slice: 2m41s to build on 24 cores, a 60 MB
binary, log at `runs/release-rehearsal/trixie-build.log`. The binary then
printed its version and crawled `fixtures/antora/pages/whole` served on
localhost in `fedora-minimal:42` (docs-fp-o's image), `fedora:44` and
`debian:trixie`; the archive's `sha256sum -c` passed in `fedora:44`.

## Negative results

- **rc.1's binary linked `libstdc++` and `libgcc_s` dynamically**, which M13
  ended from `v0.1.0-rc.2` (D23; `../status/history/2026-10-22.md`, "M13
  landed"). rc.1 is immutable and keeps them.
- **A bare `debian:trixie` cannot crawl.** Without `ca-certificates`,
  `bruce crawl http://…` fails with `tls: builder error: … No CA certificates
  were loaded from the system`, the TLS client being built whether or not a
  request uses it. Fedora's images carry a bundle. It is not fixed here (a
  crawl of plain HTTP is not the use), and it is 6.3's to state.
- **No third-party action, so no `actions/checkout`**: the clone is
  `git clone --depth 1 --branch <tag>` with the token in an `http.extraheader`,
  which keeps it out of `.git/config`. `rustup` comes from `sh.rustup.rs`
  over TLS, pinned to `mise.toml`'s `rust` version, which the workflow reads, so
  the version is stated once.
- **A shell scalar in a pipeline hides a failure**: the archive's first draft
  took `$(git log …)` inside `tar`'s option, where a failure (git's dubious
  ownership, in the rehearsal) printed and archived with no mtime; it is now its
  own statement under `set -e`.
- **A draft is not found by tag.** `GET /releases/tags/<tag>` omits drafts, so
  the workflow lists releases and selects by `tag_name`, deleting and
  publishing by id through `gh api`; assets go up inside `gh release create
  --draft`, so no `gh release upload` looks a draft up by tag.
