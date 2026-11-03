# P9.12 — The published directory's index: notes

What 9.8 and 9.10 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The slice order", item 7.

## What landed

- **`INDEX`**, a variable of `docs/manual/deploy.md`'s `bruce-nightly.sh` (off when empty, as `DELIVER`
  and `KEEP` are): S4's `stage` calls `index` after the trim and lists `index.html` in `SHA256SUMS`.
  The page links `digest.*`, the three fixed-name files and `SHA256SUMS`, then one line a night, newest
  first, linking that night's files in `digests/`; names are escaped with `sed`.
- **`deployment.check_index`**, called by `check_published` whenever `index.html` exists: it links
  exactly the fixed names, the latest digests, `SHA256SUMS` and the dated digests, none twice, the nights
  newest first, and `SHA256SUMS` lists it. `check_published(index=True)` requires one; `exercise` (so
  `e2e`) sets `INDEX` and requires it. `test_deployment` holds the off-by-default, the order, a name's
  escape and each way `check_index` fails.
- **The deployment repo** (`../bruce-fedora-docs-ci`, uncommitted and unpushed): the guide's script,
  `INDEX=1` in `nightly.yml`, and the README's line.

## What the next slices inherit

- **The nightly after the push is the index's first run on Actions**; the page is at the Pages root,
  `https://mwinters0.github.io/bruce-fedora-docs-ci/`. 9.8 reviews the digest from it.
- **The index is a listing, not a report**: it states no run, version or size, which are `digest.json`'s.

## Negative results

- `${1//&/&amp;}` in bash 5.2 expands `&` in the replacement to the matched text
  (`patsub_replacement`, on by default), so `<` would have become `<lt;`; the escape is `sed`.
