# P11.4 — The litmus tests: notes

What the wrap inherits. The spec's slice list is [`roadmap-P11-view.md`](roadmap-P11-view.md).

## What landed

- **`fixtures/antora/views/cs.html`**, a whole-document view of the digest in Czech, and **`links.csv`**, one record to
  each finding. Both declare `bruce_model = 2`. `cs.html` includes `bruce:palette.css` and `bruce:style.css` rather than
  copies of them.
- **`scripts/e2e.py`** writes the digest through `default`, both views and the JSON, parses the CSV and counts its
  records against the JSON's `broken` and `errors`, checks the Czech document for English and for the share of broken
  links in the JSON's own totals, and holds both outputs to `fixtures/antora/views/golden/` (`--save-digest` writes them
  anew with the HTML's). `check.py`'s `E2E_READS` gained `fixtures/antora/views/`.
- **`report::digest::view::litmus`** (tests) renders both views over every digest the tests build, and holds Czech's
  `counted` (22 and 24 take the genitive, only 2 to 4 the few form; 1 234 has a no-break space), `cz_day` and `percent`
  (the decimal comma), the sitemap note keyed by `antora.sitemap_from_build` with the profile's text as the fallback,
  the CSV with a title holding a comma, quotes and a line end, and that `docs/manual/digest.md`'s new "The model a view
  reads" names exactly `model::context`'s top-level names.
- **The manual** gained that section, the derived part beside the JSON, and two lines under "Views": booleans print
  `True` or `False`, and a whole-document view can include the built-in sheets by `bruce:` name.

## Negative results

- **The model lacked nothing.** Neither view asked for a name the model does not hold, so `context` is unchanged and the
  version stays 2. Two things a CSV author meets: a finding of `error` is a row of the JSON's `errors`, which has no
  `outcome` and no `hint`, so a view tests the finding's `error` before reading them (`links.csv` does); a boolean
  prints as Jinja's `True`, so a view wanting the JSON's `true` writes the conditional. An `outcome` on each finding
  would make the first one line shorter and was left out: the default view's own `f.error` branches are the same shape.
- **No decimal filter.** Every number the model holds is an integer, so Czech's decimal comma is the view's tenths by
  integer division (`percent`); a filter taking the separator would have no model value to format. Polish's last-digit
  plural rule is already `views.rs`'s test; Czech's needs no more than comparisons.
- **The Czech view is not the default's sentences one for one**: it adds the share of broken links to the figures'
  paragraph, to show the decimal comma on a real value, and writes the "By: Source Page" paragraph with its `<p>` that
  the default lacks (KD39).

## What the wrap inherits

- The Czech wording was written without a Czech speaker; the spec has the maintainer's contacts verify it. A correction
  is an edit to `fixtures/antora/views/cs.html` and `--save-digest`, no code.
- P11's intent is delivered: a deployment's prose, guide and palette are its view (the spec's "Intent"), and the two
  litmus tests are green in `mise run check`. Remaining is the wrap: a repoint, the consolidation of the five notes
  docs, the spec and the notes to `docs/design/`, the phase index row to `Complete`, and the README's status list.
