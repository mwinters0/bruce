# P11 — The digest's view: notes

P11 made the digest's HTML a view. `report::digest::model` holds the derived
part (each row's level, the findings' counts and ranking under the severity
map, the policy's rows) and no words. `report::digest::view` renders minijinja
templates over it: the default view, embedded under `view/default/`, and any a
deployment writes, extending the default as `bruce:digest.html` or replacing it
whole. `report digest` takes `--view`, `--json` / `--no-json` and
`--write-default-view`. A Czech view and a CSV view over the e2e fixture site
are the two litmus tests, held to golden files by `mise run e2e`. The
decisions are D95 and D96, with D64, D76 and D87 amended. The register held
every one at the wrap, and the audit found nothing to move in. A view's author
reads `../manual/digest.md` ("Views", "The model a view reads"). This file
keeps the evidence, what later phases inherit and what was tried and refused.

## The evidence

| Where | What it holds |
|---|---|
| `runs/p11-view-cost-20261101/` | `report digest` over the 2026-11-01 en-US crawl before P11 and after 11.2, three runs each: 8.7 s and 2.4 GB both, the HTML byte-identical |
| `fixtures/antora/views/` | `cs.html` and `links.csv`, the litmus views, and `golden/`, what they write over the fixture site (`scripts/e2e.py --save-digest` rewrites them with `fixtures/antora/digest.html`) |
| `report::digest::view::litmus` | both views over every digest the tests build; Czech's plurals, date and decimal comma; the CSV's quoting; the manual naming exactly `model::context`'s top-level names |

- **The move lost nothing.** 11.2's default rendered `html`'s document byte for
  byte over every digest the tests build, over a matrix of every severity key
  at every level, and over the golden file (a throwaway test, removed). The
  golden file moved once since, in 11.2.1, for the print `:root` moving up.
- **The model lacked nothing.** Neither litmus view asked for a name the model
  does not hold, so the model's version stayed 2.

## What later phases inherit

- **P9: the guide may document a deployment's view.** P11 added no config
  key, a run recording nothing of its views. The palette docs.fp.o's
  deployment may want back is `git show
  e30483c:src/report/digest/html/style.css`, its `:root` and `@media print`
  rules, which become a view's `palette` block.
- **The Czech wording was written without a Czech speaker.** The spec has the
  maintainer's contacts verify it; a correction is an edit to
  `fixtures/antora/views/cs.html` and `--save-digest`, no code.
- **The environment's syntax is a view's**: `trim_blocks`, `lstrip_blocks` and
  `keep_trailing_newline`, a block tag on its own line printing nothing.
  Macros and top-level `set`s are visible in a child's blocks, which minijinja
  3.0.5 does and no test holds.
- **`none ~ "x"` is `"nonex"`**: the formatter prints a `none` as nothing, but
  concatenation does not, so a macro joining a cell tests it first.
- **A CSV author meets two things**: a finding of `error` is a row of the JSON's
  `errors`, with no `outcome` and no `hint`, so a view tests `f.error` first;
  and a boolean prints Jinja's `True`, so a view wanting `true` writes the
  conditional.

## Tried and refused

- **`Judged` in `model`**, where the spec placed it: `Digest` holds a `Judged`
  and the model reads the `Digest`, a cycle (`module_graph.py`). It stays in
  `document`.
- **`tally` whole in `model`**: it is a count's words and dots, so only the
  counts moved.
- **minijinja without `macros`**, the spec's feature list, which left a
  template repeating every link, dot and number. `fuel` stays off.
- **A decimal filter**: every number the model holds is an integer, so a ratio
  is the view's arithmetic (M42).
- **An `outcome` on every finding**, sparing the CSV view its `f.error` test:
  the default view's own branches are the same shape.
- **`--write-default-view` overwriting**, which loses a deployment's edits to a
  re-run: a stale export is deleted by hand.

## Limits left in place

- KD39: the default view keeps two malformed paragraphs, to hold the bytes.
- A file literally named `default` is the built-in view for `--view`; the file
  is `./default`.
- `time` has no unpadded day: a Czech date is `time('%d')|int`.
