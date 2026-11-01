# P11 — The digest's view

What P11 does and why, settled in the grilling of 2026-11-01. The decisions
are in [`../design/decisions.md`](../design/decisions.md) and are cited here,
never restated.

## Intent

A deployment presents the digest as its own: its prose, its guide ("How to
read this report"), its palette, without a compiled profile (D32) and without
a fork of `bruce`.

## The digest's HTML is a view, and the default ships inside the binary

Settled 2026-11-01 (maintainer). **The HTML is a view over a model `bruce`
builds**, after MVC: `bruce` is the model and the controller, and **a default
view is packaged inside the binary**, so a release is still one file (D23).
**The default view can be written out to a file**, which a deployment edits
and passes back as an override. This replaces the sketch's per-text overrides:
which texts a deployment may change, and whether it may theme the digest, are
answered by the view being the deployment's whole, the stylesheet included.
The technology is open (below).

## Two litmus tests

Settled 2026-11-01 (maintainer). The phase is not done until both hold, and
each is its acceptance evidence:

1. **A view entirely in a language other than English** can be written.
2. **A custom CSV** can be written with the view layer.

What they settle:

- **The model holds no words.** Every sentence the HTML prints today,
  the guide's words stating a setting and the legend's included, is the
  view's, and the model carries the values they state (enums, numbers,
  booleans, timestamps). What a site wrote itself (a declared profile's
  rule, a URL, a page ID hint, D65) is data, in whatever language it was
  written.
- **A profile's note is a key and its text**: a compiled profile's sitemap
  note (D87, Antora's sentence in `profile::antora`) reaches the model as a
  stable key (as `antora.sitemap_from_build`) and the profile's English, the
  default view printing the text and a translation keying its own sentence
  by the key, the text its fallback. The profile stays the one place
  knowing the generator's fact (D87). Rejected: the default view holding
  each compiled profile's sentence, generator knowledge in the core's view.
- **The model is the JSON document and a derived part**: each row's level,
  the findings' counts and ranking under the severity map, and which of the
  guide's reasons the policy judges, computed in Rust (D76, D82, D92), so a
  view lays out and never re-implements a judgement.
- **A view is any text format, not HTML.** Escaping follows the view's output
  format, HTML's for HTML and none for CSV, which needs its own quoting.

## Numbers and dates are the view's, through filters taking their conventions

Settled 2026-11-01 (maintainer). **The model gives raw numbers and UTC
timestamps; the view formats them through filters taking explicit
arguments**, such as a thousands separator or a date pattern, set once at a
translated view's top. Month and weekday names, and plural forms, are the
view's words and conditionals, with a filter for a two-form plural. Rejected:
a locale name over CLDR data (icu4x), megabytes in the one binary for what a
view states in a line; raw values alone, arithmetic in every view.

## Views are named per digest, and no run records them

Settled 2026-11-01 (maintainer).
- **`report digest --view FILE` is repeatable**, each writing a file; with
  none the built-in HTML view is written. **Naming any drops the built-in**,
  and `--view default` keeps it beside them.
- **The built-in view writes `DIR/STEM-DATE.html`; a user's is named for its
  file**, `--view links.csv` writing `DIR/STEM-DATE-links.csv`, so two CSV
  views cannot collide. The view's extension picks its escaping.
- **The JSON is no view**: it is the model's serialization, stays in code
  under its `format_version` (D64), and is switched by `--json` /
  `--no-json`, replacing `--format json|html|both`.
- **A run records nothing of its views.** The words stating the policy are
  read from the run's recorded policy (D78), so today's view renders an old
  run truly, and a reworded view needs no crawl. A view overrides no key (D5),
  `report digest` reading no config outside a trial (D84).

## An override extends the default or replaces it

Settled 2026-11-01 (maintainer). **The default view is named blocks**: the
palette, each section, each guide entry, and an empty intro under the
header. An override either extends the built-in default, replacing the
blocks it names and inheriting every block a later release adds, or is a
whole document, as a translation is. Rejected: replacement alone, a
deployment that set only its palette missing every section a release adds.

Settled 2026-11-01 (maintainer, reviewing 11.2). **The palette is a
`palette` block inside a `style` block** holding the whole sheet:
`palette` includes `palette.css`, both `:root` rules (the screen's and
`@media print`'s) and nothing else, and `style` the rest from `style.css`.
A view replaces `palette`, or writes `{{ super() }}` before its own `:root`
to inherit a property a later release adds, or replaces `style` whole.
Rejected: one `style` block, where a view's palette is appended after the
sheet and a `:root` without its print twin turns the printed digest dark.

## The engine is minijinja, and a CSV view quotes every value

Settled 2026-11-01 (maintainer). **Views are minijinja 3 templates**, with
`default-features = false`, and undefined names strict, so a view naming a
field the model lacks fails with its file, line and expression. Its
Apache-2.0 notice is the release's (D60). **A `.csv` view's escaping quotes
each value it prints as one RFC 4180 field**, the view writing the commas and
line ends. Rejected: tera, whose template semantics changed in minor
releases; handlebars, twenty crates and no inheritance; upon, neither; a
`csv` filter the author must remember on every value, a title's comma
shifting a column where it is forgotten. (The survey is under "Facts the
grilling found".)

## The model has one version, and a view declares the one it was written for

Settled 2026-11-01 (maintainer). **The JSON's `format_version` (D64) versions
the derived part too**, bumped only for a change a view or a loader would
misread, the derived part documented beside the JSON in
`docs/manual/digest.md`. **A view declares the version it was written for**
(as `{% set bruce_model = 3 %}`), and `bruce` refuses one declaring another,
naming the manual's section; one declaring none relies on strict undefined
names, and an exported view always declares. Rejected: a `view_version` of
its own, a view reading the JSON's fields as well.

## The default is written out, and an override extends it as `bruce:`

Settled 2026-11-01 (maintainer). **`report digest --write-default-view DIR`
writes the default's files and exits** without reading a database:
`digest.html`, `style.css`, and `palette.css` apart so a palette edit is
small. **An
override extends the default as `{% extends "bruce:digest.html" %}`**,
needing no exported copy; what else it includes or extends resolves inside
its own directory, and nothing outside it. A whole-document view is the
exported `digest.html`, edited.

## A failing view writes nothing

Settled 2026-11-01 (maintainer). **The JSON and every view are rendered in
memory first, and no file is written unless all succeed**; a failure exits
non-zero naming the view, its line and the expression, so a published
directory is never yesterday's HTML beside today's JSON. No fuel bounds a
view, which is the deployer's own file, trusted as its config is, and no
gate renders one (`check-dir` writes no digest); minijinja's recursion limit
stands. Rejected: writing what rendered and failing after.

## The view layer is the digest's alone

Settled 2026-11-01 (maintainer). **Only the digest has views**; `report
<kind>` keeps `--format text|json`. The digest's model holds every broken
and error row and every crawled page's counts, so the CSV litmus test is
met there. A view over another report needs a model and a version of its
own, a later phase's to propose.

## Where the parts live

Settled 2026-11-01 (maintainer). All of it is `report::digest`'s, above
`profile` as D39 has it, so no edge is new:
- **`report::digest::model`**: the derived part, each link's level and the
  findings' ranking (today `html::standing` and `html::severity`), the
  reasons the policy judges (the guide's `Judged`) and the policy changes'
  rows, and the `Digest` presented to minijinja as maps and lists, no serde
  (D95).
- **`report::digest::view`**: the environment, its loader (`bruce:` names, and
  the override's own directory), the filters, the CSV escaping, the version
  check and rendering before writing; **the default's templates in
  `view/default/`** (`digest.html`, `style.css`, `palette.css`), embedded.
- **`report::digest::html` and its children are deleted** once the default
  reproduces them, `markup`'s helpers with them.
- **`profile`**: `sitemap_note` gives a key and a text.
- **`cli::report`**: `--view`, `--json` / `--no-json`, `--write-default-view`.

## The default view renders today's HTML byte for byte

Settled 2026-11-01 (maintainer). **The e2e golden file and
`report::digest::html::tests` pass unchanged** over the default view: the one
evidence that the move lost no empty state, policy-stated sentence or ⓘ.
Rewording the default is a change after it. Rejected: an equivalent document,
the golden file regenerated on review.

## The translation is Czech

Settled 2026-11-01 (maintainer). **The litmus translation is Czech**, which
the maintainer's Czech-speaking contacts verify: three integer plural forms
(1; 2 to 4; 0 and 5 up, 22 included), a decimal comma, a space between
thousands, a date as `1. 11. 2026`, and diacritics through HTML and CSV. A
unit test of a last-digit plural rule (Polish's: 2 to 4 by the last digit,
not 12 to 14) shows a view can compute one, which Czech's range does not.
Rejected: Polish as the fixture, harder but verifiable by nobody available.

## The slices

Each makes the next one's mistakes visible:

1. **11.1 The model.** The derived part leaves `html` for `model` as plain
   Rust values, `html` reading it; the sitemap note becomes a key and a text
   in `profile` (D87 amended). Output unchanged, golden file and tests as
   they are.
2. **11.2 The engine and the default view.** minijinja; `model`'s maps and lists;
   the default's templates rendering byte for byte; `html` deleted; the
   notices regenerated; the concept map, D64 and D76 amended.
   **11.2.1 The palette block**, earned: 11.2 shipped one `style` block
   for the spec's `palette`; `palette.css` and the nested blocks, the golden
   file regenerated once for the print `:root` moving up.
3. **11.3 Views from the command line.** `--view` (repeatable, `default`, the
   files' names), `--json` / `--no-json`, `--write-default-view`, `bruce:`
   and the directory's bound, the version declaration, writing all or
   nothing, the model built once per call and every view rendered over it,
   the filters and the CSV escaping; the manual's rows.
4. **11.4 The litmus tests.** A Czech view and a CSV view, fixtures
   rendered over the e2e fixture site by `scripts/e2e.py`, a last-digit
   plural rule's unit test beside them; the derived part documented beside
   the JSON. Whatever they show the model lacks is fixed here.

## Facts the grilling found

- **The guide** (`report::digest::html::guide`) has two entries,
  `doubled_slash` and `pedantic_trailing_slash`, each shown only while the
  run's policy judges its reason, and each one's last words stating a setting
  of the run (`doubled_slash_reach`; the `[severity]` marking). The ⓘ beside a
  row's reason links to its entry. **An override that does not read the run's
  policy can be false for the run it is printed in** (from the P11 inbox,
  the site policy's wrap, 2026-10-31).
- **The rest of the HTML's prose** is spread through `report::digest::html`
  and its `compared`, `policy` and `severity` children: the title, the
  subtitle, each section's opening paragraph, the empty-state lines and the
  legend.
- **The HTML computes what the JSON does not hold**: each link's level and the
  findings' counts and ranking under the severity map (D76, D82, D92), which
  guide entries the policy judges, the words stating a setting, and the
  profiles' sitemap notes (D87). D64's "the HTML shows nothing the JSON lacks"
  is of facts, not of these.
- **The precedent for a digest setting** is `[digest]` `group_by` (D86):
  recorded with the run's policy (D78), judged by nothing, read from the
  policy in force so `report digest --config` tries another (D84).
  `report digest` reads no config otherwise.
- **The palette docs.fp.o's deployment may want back** is
  `src/report/digest/html/style.css` at `e30483c`, `:root`'s custom properties
  and its `@media print` block (`git show e30483c:src/report/digest/html/style.css`);
  the current sheet's palette is a neutral default. The sentence on a forge's
  new-issue form or a login page answering 200 is built in, a login wall
  being the web's (I17). From the P11 inbox (the declarative extraction
  rules' grilling, 2026-10-31: P11 must be able to recall the scheme).
- **Nothing in `src/` derives `Serialize`**: serde serves the config's
  `Deserialize`, serde_json is a dev-dependency, and `report::digest::json`
  writes the JSON by hand over `Digest` (`report::digest::document`), most of
  whose sections are `report::table::Table` (named columns, rows of `Cell`).
- **The runtime template engines, surveyed 2026-11-01** from crates.io, their
  changelogs and sources (`cargo tree -e normal` over scratch crates):
  - **minijinja 3.0.5** (2026-10-29), Apache-2.0 only: no crate beyond itself
    with `default-features = false` and `multi_template`, `builtins`,
    `debug`, `fuel`, serde optional since 3.0. `extends`/blocks from an
    in-memory loader; auto-escape by the template's name (`.html`, `.htm`,
    `.xml`) and none otherwise; `set_fuel` and a recursion limit; a strict
    undefined mode, an error naming the expression with line and excerpt;
    Rust filters and functions; template-behaviour changes only in a major
    release, with `UPDATING.md`.
  - **tera 2.4.0** (2026-09-11), MIT: serde required, inheritance, the same
    auto-escape suffixes, always-strict undefined, no fuel; template
    semantics changed in minor releases (2.2, 2.3).
  - **handlebars 6.4.4**, MIT: 36 crates, 20 new to the lock, serde_json into
    the release; no `extends`, logic in Rust helpers, always HTML-escaping.
  - **upon 0.11.0**, MIT OR Apache-2.0: no inheritance, no HTML escaping by
    default, pre-1.0. liquid (no release since 2025-02, 22 new crates),
    tinytemplate and ramhorns are stale; askama is compile-time only.
- **P9 follows P11** (P9's spec, "P9 follows the declarative extraction
  rules and P11"), and its guide documents P11's config.
