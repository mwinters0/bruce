# P6.3 — `v0.1.0` and the README's quickstart: notes

What landed: the README's quickstart, which replaces "There is no release yet"
with the release page, a download-verify-unpack recipe keyed on the tag, and
the three facts a downloader needs (below). What did not: the `v0.1.0` tag,
which is the maintainer's push (P6 spec, "The slice order the split proposed"),
so the box stays empty.

## What the tag push inherits

- **`v0.1.0-rc.2` first.** The workflow that builds the static `libstdc++` and
  `libgcc_s` (M13, D23) has been rehearsed in a container but never run by the
  GitHub runner, and a published release is immutable. A prerelease is the
  cheap check: "Build" must print `newest glibc symbol:` at or under
  `GLIBC_2.41` and pass its `ldd` check, and the release must be a prerelease
  ([`roadmap-P6.2-release-workflow-notes.md`](roadmap-P6.2-release-workflow-notes.md),
  "What the rest of 6.2 and 6.3 inherit").
- **`v0.1.0` needs `Cargo.toml`'s `version` to be `0.1.0`** (it is) and the
  tree at the tag to be the one wanted: the workflow refuses a `HEAD` moved
  since the push. Without a `-` suffix the release is not a prerelease, and
  only then does the releases page's "latest" name one.
- **The README's recipe is untested against a real release asset** until a tag
  with no `-` exists: its `tag=v0.1.0` names none yet. It follows the asset
  layout 6.2 checked on `v0.1.0-rc.1` (archive of a directory of the same name,
  `<asset>.sha256` in `sha256sum`'s format).
- **Ticking 6.3** is the change after the tag publishes: check the published
  release's assets against the recipe, then tick the box.

## Negative results

- **No quickstart claim about aarch64, Alpine's workaround or a CA bundle
  fix.** The README states what is (x86_64, glibc 2.41, no musl, the CA bundle
  needed even for `http://`); making a crawl of plain HTTP work with no bundle
  would be a code change, which no decision asks for.
