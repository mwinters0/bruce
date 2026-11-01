# P11.2 — The engine and the default view: notes

What 11.3 inherits. The spec's slice list is [`roadmap-P11-view.md`](roadmap-P11-view.md).

## What landed

- **`report::digest::view`** holds the minijinja environment and renders the default view, whose files are
  `view/default/digest.html`, `style.css` and `palette.css`, embedded and registered under those plain names. `Digest::html()` is the
  default view over the model and panics on a failure, the default being tested; 11.3 makes it a `Result`.
- **`report::digest::model::context`** presents the `Digest` and the derived part as one tree of values: maps and
  lists built eagerly, no `Object`, no serde. A table's row is a map from its column names to its cells (`is_new`
  beside them where the table marks rows new, `level` where the model judges the row); a `none` cell is a `none`, and
  an empty text is one too, as `Row::text` has it.
- **`report::digest::html` and its children are deleted**, `html::tests` (as `view::tests`) and `tally`'s tests (as
  `view::macros`) kept; both call the default view's macros through `tests::call`.
- The document is byte for byte what `html` wrote: checked by running both over every digest the tests build and
  over a matrix of every severity key at every level, both judged flags and the reach, a trial and an unfinished run
  (a throwaway test, since removed), and by the e2e golden file.
- Filters: `thousands(sep)`, `time(pattern)` (`%Y %m %d %H %M %S %%`), `short`, `slash_runs`, `final_slash_runs` (runs
  of `[text, marked]`, so the view marks them in its own format) and the test `web`.

## What 11.3 inherits

- **The environment is built per call** (`default_view`) and its three templates are added under their plain names. The `bruce:` prefix, the override's directory and the loader that resolves them are 11.3's; so is
  `--write-default-view`, which writes these three files.
- **The syntax is the environment's, so a deployment's view has it too**: `trim_blocks`, `lstrip_blocks` and
  `keep_trailing_newline`. A block tag on a line of its own prints nothing, an inline one followed by a line end eats
  it (`+%}` keeps it). The manual 11.3 writes says so.
- **The formatter prints a `none` as nothing**, as the HTML's `unwrap_or("")` did. `~` does not: `none ~ "x"` is
  `"nonex"`, so a macro concatenating a cell tests it first.
- **Macros and top-level `set`s are visible in a child's blocks and in the render context** (an experiment over
  minijinja 3.0.5, which the tests do not hold): a view extending `digest.html` may call `num`, `a`, `dot` and the rest.
  A macro must be defined before the template text calling it.
- **Blocks**: `style` (the whole stylesheet, `palette` and `style.css` included), `palette` (see
  [11.2.1](roadmap-P11.2.1-palette-notes.md)), `intro` (empty, under the header), `figures`,
  `broken_since`, `fixed_since`, `pages`, `repos`, `targets`, `errors`, `housekeeping` and `guide`, the last holding
  `guide_doubled_slash` and `guide_pedantic`.
- **Plurals are English**: the `counted` macro adds an `s` unless `n == 1`, and `is`/`are` is an `if`. A translated
  view brings its own, and the two-form plural filter the spec names is 11.3's.
- **The cause words and the unchecked reasons' meanings are in the template** (a dictionary at its top, an `if` chain),
  keyed by the cause and the reason as the tables hold them; the sitemap note reaches the template as `{key, text}`
  and the default prints the text.

## Negative results

- minijinja's HTML escape, `Environment::set_auto_escape_callback` alone, writes `'` as `&#x27;` and `/` as `&#x2f;`:
  not the bytes of D64's document, so the environment has a formatter (D95).
- The spec's feature list (`builtins`, `debug`, `multi_template`) lacks `macros`, without which a template repeats every
  link, dot and number; it is enabled. `fuel` is not (the spec).
- The model as `Object`s over an `Arc<Digest>` was not built (D95).
- Two defects of the document are kept, to hold the bytes: KD39.
