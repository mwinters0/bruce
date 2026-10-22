# P6 — Release: notes

P6 delivered `v0.1.0` on GitHub, built by `.github/workflows/release.yml`
from the tag, licensed `MIT OR Apache-2.0` with its third-party notices. The
decisions are D23, D43 and D60, and `roadmap.md`'s "Standing rules"; how
the notices are made and checked is `scripts/notices.py`'s docstring, and
the workflow's is its header comment. This keeps what the next release, P8
and P9 inherit, and what was tried and refused.

## What the next release inherits

- **The tag is `v<x>.<y>.<z>[-<pre>]`, its base `Cargo.toml`'s `version`.**
  The workflow refuses any other, and a tag moved since the push. A `-`
  suffix makes a GitHub prerelease, so only a bare tag is Latest. Every tag
  push is the maintainer's.
- **A published release is immutable.** A fault in its assets is fixed in
  the next patch, never in place. A tag can move only while nothing has
  published from it, as `v0.1.0` did once (below).
- **What a run must show**: every step green; Build printing `newest glibc
  symbol:` at or under `GLIBC_2.41` and an `ldd` of `libm`, `libc` and the
  loader only; Publish logging `draft <id>`, both uploads, `draft <id>
  holds:` the two assets, and `published … prerelease=<bool>`.
- **The rehearsal before a tag**: `cd scripts && uv run workflows.py script
  release.yml --skip "Check out the tag" --before Publish`, run with `bash`
  in `debian:trixie` (rootful `nerdctl`) over a copy of the tree committed in
  a scratch repo; about 3 minutes on 24 cores. On GitHub's runner a build
  took 9m40s to 14m22s.
- **The recipe check after one**: `runs/release-v0.1.0/recipe.sh <image>
  <tag>` runs the README's quickstart and a `bruce crawl` over
  `fixtures/antora/pages` in `debian:trixie`, `fedora:44` and
  `fedora-minimal:42`.
- **A DuckDB bump fails `mise run check`** (`ThisRepo` in
  `scripts/test_notices.py`) until `uv run notices.py --refresh-duckdb` and
  `uv run notices.py` are run and each `SOURCE-NOTICE` is re-read by hand.
  The refresh needs the network and `git`, and stops when the tag's commit is
  not the tarball's source id.
- **The sizes**: the archive about 20 MB, the binary about 61 MB; the
  notices file about 680 KB.

## The releases

`v0.1.0-rc.1` links `libstdc++` and `libgcc_s` dynamically and stays so,
being immutable; M13 linked them in from `rc.2`. `rc.3` was the first run of
M14's Publish, which makes, fills and publishes the draft by the id its
create returns. `v0.1.0` is tagged at `aa7d0f2`, `rc.3`'s tree, after its
first tag's Publish failed twice during GitHub incidents and its two drafts
were deleted ([`../status/history/2026-10-22.md`](../status/history/2026-10-22.md),
"`v0.1.0` published"). Each was downloaded, verified and run on the three
images; logs in `runs/release-rc1/`, `runs/release-rc2/`,
`runs/release-static/` and `runs/release-v0.1.0/`.

## Negative results

- **A draft is found neither by tag nor reliably by listing.** `GET
  /releases/tags/<tag>` omits drafts, and a listing seconds after a create
  missed the draft, so the workflow keeps the id the create returns. `gh
  release create` prints a URL, not an id, and `gh release upload` looks a
  draft up by tag, so the create is `gh api -X POST` and the upload `curl`.
- **A shell substitution inside a pipeline hides its failure**: an archive
  whose `--mtime` came from `$(git log …)` inside `tar`'s options was written
  with no mtime when `git` failed. Each such value is its own statement under
  `set -e`.
- **A bare `debian:trixie` cannot crawl**, even over `http://`: the TLS
  client is built whether or not a request uses it and needs the system CA
  bundle (the README states it). Making a plain-HTTP crawl work with no
  bundle would be a code change no decision asks for.
- **The crates**: `duckdb` and `selectors` ship no license file and are
  listed with their expression and repository; `duckdb`'s MIT text is
  `libduckdb-sys`'s, `selectors`' MPL-2.0 is `cssparser`'s.
- **DuckDB's tarball is `v1.5.6` at `069cc9f9b5` exactly**, the tag's commit
  being the source id in `pragma_version.cpp`. Four upstream `third_party/`
  directories (`catch`, `imdb`, `jemalloc`, `snowball`) are not in the
  tarball and are not vendored.
- **libpg_query's source carries two copyright lines and no PostgreSQL
  License text.** Its `LICENSE` is Lukas Fittl's BSD-3. The hand-copied
  paragraphs follow the source's `1996-2017` lines, not postgresql.org's
  `1996-2026`, which the source does not carry, and sit in a file sorting
  after `SOURCE-NOTICE`, which the check holds verbatim. The check holds the
  paragraphs only by their header, so a changed paragraph goes undetected.
- **The Bison skeleton's FSF notice** in libpg_query's generated grammar is
  not vendored: its output carries the Bison exception, which asks for none.
- **Identical library texts are not shared**, each printed under its
  library's name so a reader finds it; sharing would save under 20 KB.
