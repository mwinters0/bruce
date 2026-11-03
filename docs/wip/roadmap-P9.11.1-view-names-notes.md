# P9.11.1 — The view under the contract's names: notes

What 9.12 and 9.8 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The published directory is the contract" and
"The slice order", item 7.

## What landed

- **The script renames the view's file once**, after S3's digest call: `bruce` writes
  `STEM-DATE-FILE` for a view in a file, and the job moves it to `STEM-DATE.EXT` (`deploy.md`'s
  `bruce-nightly.sh`), so S4's loop, unchanged but for the carried-over globs, publishes `digest.<ext>` and
  `digests/<stem>-<date>.<ext>`. The `digest-*` globs are gone from S4: nothing makes a `digest-` file now.
- **`VIEW` is refused before the crawl** (exit 1, a message on stderr) when it is not `default` and its file
  name has no extension or a `.json` one in any case: the first names no format, the second would overwrite
  `digest.json` and `STEM-DATE.json`.
- **`deployment.check_published(pub, ext)`** holds the directory to the contract: `digest.json` and
  `digest.<ext>`, no `digest-*` at the root, every entry of `digests/` named `<stem>-<date>.<ext>`.
  `exercise` (so `e2e`) runs the guide's own `fedora.html` and checks `ext = html`. The unit tests' stub `bruce` names
  the view's file as the real one does (`--view`), so the trim test publishes a `.csv` view.
- **The deployment repo** (`../bruce-fedora-docs-ci`, uncommitted and unpushed): the guide's script and the
  README's line about the digest's name.

## What the next slices inherit

- **9.12's index lists the fixed names**: `digest.json`, `digest.<ext>` and `digests/`, whatever the view's file.
- **The dev deployment's first night on the new script** drops the root `digest-fedora.html` (nothing
  carries it) and keeps `digests/bruce-YYYY-MM-DD-fedora.html` of the nights between 9.11's push and
  this one, listed in `SHA256SUMS` and trimmed by `KEEP_VIEWS` with the rest; they do not match
  `digests/<stem>-<date>.<ext>` and a check of the live directory would name them.
- **9.8 reviews `digest.html`**.

## Negative results

- None beyond the 2026-11-03 history entry, "Review of the guide's view's published name": a view file
  named `html` publishes `digest-html`.
