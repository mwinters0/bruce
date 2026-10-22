# P6.3 — `v0.1.0` and the README's quickstart: notes

`v0.1.0` is published on GitHub, not a prerelease and the releases page's
Latest, and the README's quickstart recipe passes against its assets. What the
slice inherits is below; the wrap inherits nothing to run.

## What the release carries

- **`v0.1.0` is tagged at `aa7d0f2`**, the tree `v0.1.0-rc.3` was built from,
  after M14 replaced Publish's lookup by the draft's id. Its first attempt
  failed twice in Publish ([`../status/history/2026-10-22.md`](../status/history/2026-10-22.md),
  "`v0.1.0`'s Publish failed twice; M14 admitted"); the tag moved once, which
  is possible only while no release has published from it.
- **The release is immutable.** A fault in `v0.1.0`'s assets is fixed in
  `v0.1.1`, never in place. Only a tag with no `-` suffix is a non-prerelease,
  so only it can be "latest".
- **The recipe, as written** with `tag=v0.1.0`, downloaded, verified and
  unpacked the assets, and `bruce crawl` over `fixtures/antora/pages` exited 0,
  in `debian:trixie`, `fedora:44` and `fedora-minimal:42`. Evidence: the
  history entry's "`v0.1.0` published"; script and logs in
  `runs/release-v0.1.0/`.

## `v0.1.0-rc.2`, checked

[Run 39793421276](https://github.com/mwinters0/bruce/actions/runs/39793421276)
passed, the first static build by the runner: "Build" logged `newest glibc
symbol: GLIBC_2.38` and an `ldd` of `libm`, `libc` and the loader only. It took
14m22s against rc.1's 9m40s, during a GitHub Actions incident that also left
the run shown in progress for about 25 minutes after its last step. The
release is a prerelease with the two assets; the archive is 19,988,532 bytes,
the binary 60,829,584.

The README's recipe, with `tag=v0.1.0-rc.2`, then `bruce --version` and `bruce
crawl http://127.0.0.1:8765/whole/` over `fixtures/antora/pages`, printed
`bruce 0.1.0` and `run 1: 40 targets, 7 pages, no sitemap` with exit 0 in
`debian:trixie` (after `curl` and `ca-certificates`), `fedora:44` and
`fedora-minimal:42`. In `fedora-minimal:42` the recipe first failed at `tar
xzf`, the image having neither `tar` nor `gzip`, so the README names the
recipe's tools. Script and logs: `runs/release-rc2/`.

## Negative results

- **No quickstart claim about aarch64, Alpine's workaround or a CA bundle
  fix.** The README states what is (x86_64, glibc 2.41, no musl, the CA bundle
  needed even for `http://`); making a crawl of plain HTTP work with no bundle
  would be a code change, which no decision asks for.
