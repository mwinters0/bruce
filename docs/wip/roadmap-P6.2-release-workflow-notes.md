# P6.2 — The release workflow: notes

What landed, per the P6 spec's "The release is Debian Trixie's glibc, built
from the tag by a workflow": `.github/workflows/release.yml`, whose header
comment says what it does; `scripts/workflows.py`, which fails on a `uses:` in
any workflow and prints a workflow's steps as one shell script; the rule in
`roadmap.md`'s "Standing rules"; and D23 and D43 amended. **The box is
unticked**: the slice closes on a tag that has been pushed, built by the
runner, published, downloaded, and run on Trixie and F44.

## What the rest of 6.2 and 6.3 inherit

- **The tag to push is `v0.1.0-rc.1`**, on a commit holding this slice. The
  workflow refuses a tag that is not `v<x>.<y>.<z>[-<pre>]` or whose base is not
  `Cargo.toml`'s `version` (0.1.0), and one moved since the push (`HEAD` is not
  `GITHUB_SHA`). A `-` suffix makes a prerelease.
- **What to check in the run**: every step green; "Build" printing `newest
  glibc symbol:` at or under `GLIBC_2.41` (the rehearsal's was `GLIBC_2.38`);
  the draft carrying both assets before it is published; the release a
  prerelease. A rerun after a failure deletes the tag's draft and creates it
  again; once published, a release is immutable and the next try is
  `v0.1.0-rc.2`.
- **Not exercised without a tag**: `gh release create`, the API delete and the
  publishing `PATCH`, the runner's `container:` job, the clone with
  `GITHUB_TOKEN`, and `permissions: contents: write` sufficing. The `drafts`
  query was run read-only against the repo (no draft, exit 0). Debian's `gh` is
  2.46.0, which has `--verify-tag` and `--generate-notes`.
- **The asset** is `bruce-<tag>-x86_64-unknown-linux-gnu.tar.gz` holding a
  directory of the same name with `bruce`, `THIRD-PARTY-NOTICES.txt`,
  `LICENSE-MIT` and `LICENSE-APACHE`, beside `<asset>.sha256` in `sha256sum`'s
  format. Archiving the same tree twice gave the same sha256 (a sorted tar with
  the commit's time, `gzip -n`), so a rebuild's archive differs only by the
  binary.
- **Closing 6.2**, after the release is published: download both assets
  (`gh release download v0.1.0-rc.1 -R mwinters0/bruce`), `sha256sum -c` the
  file, extract it, and run `bruce --version` and a crawl of a local server in
  `debian:trixie` (with `ca-certificates`, below) and `fedora:44`.
- **6.3's README states** that `bruce` needs the system CA bundle even for
  an `http://` crawl, and links `libstdc++` and `libgcc_s` dynamically.

## The rehearsal

`uv run workflows.py script release.yml --skip "Check out the tag" --before
Publish`, run with `bash` in `debian:trixie` (rootful `nerdctl`) over a copy of
the tree, at the tree of this slice: 2m41s to build on 24 cores, a 60 MB
binary, log at `runs/release-rehearsal/trixie-build.log`. The binary then
printed its version and crawled `fixtures/antora/pages/whole` served on
localhost in `fedora-minimal:42` (docs-fp-o's image), `fedora:44` and
`debian:trixie`; the archive's `sha256sum -c` passed in `fedora:44`.

## Negative results

- **The binary is not fully self-contained.** `ldd` lists `libstdc++.so.6`,
  `libgcc_s.so.1`, `libm` and `libc`. Trixie, `fedora:44` and
  `fedora-minimal:42` all carry the first two, so nothing was changed;
  `-static-libstdc++` would remove a floor on the libstdc++ version and was not
  tried, no image checked lacking it.
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
