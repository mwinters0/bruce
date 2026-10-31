# P10.5.1 — A report reads an extension; a comparison lists policy differences: notes

Read on 2026-10-31 at `16c6d25` plus this change. With no `extends` and no comparison nothing changes: a digest
of one run is what it was, and a comparison gains a Housekeeping fold and a JSON key.

## What the next slice inherits

- **`report::digest::sections::sitemap_notes`** maps each crawled page's `pages.profile` to the run's
  `profile.<name>.extends` row where there is one, else to itself, and picks the registry's profiles by that
  name, so a report whose registry holds only the compiled profiles gives an extension's pages `antora`'s note
  once however many pages of either read it. Nothing is rebuilt from the rows.
- **`report::digest::changes`** compares the runs' *recorded* `policy` rows (never `policy_in_force`, so a trial
  changes neither side), the profile rows only for names in either run's `pages.profile`. A key holding one
  differing value on each side is one change; a key holding several (`schemes`, `region`) lists the values lost
  and gained apart. A profile's source rules are keyed by place (`source.<i>.<field>`); each profile's two lists
  are aligned by longest common subsequence of whole rules, so a rule inserted or removed is named alone, and
  between aligned rules the removed and added are paired in order, naming the fields that differ, each side
  keeping its own place in its key (`source.2.pattern` against `source.3.pattern`).
- **`Digest::policy_changes`** is `Vec<Change { from, to: Option<Entry> }>`, sorted by `Entry::listed_order`
  (M38) on the digest's run's row else the compared run's. The JSON key is `policy_changes` (`from` and `to`,
  each a `policy` row or `null`), empty without a comparison; `format_version` stays 2. The HTML fold
  "Settings that differ between the runs" is shown only in a comparison.
- **10.6** adds `locale_links` rows beside the others; they need nothing here, being plain `profile.<name>.*`.

## Behaviours, as tested

- A rule inserted, removed or edited in the middle of a list names only itself; unchanged rules after it are
  not named though their places moved. A rule gaining a template is one added field.
- A declared profile whose pages neither run used is not compared; a run that never used it still compares its
  other rows.
- A baseline file's run compares like any: its `policy` rows travel in the file (D78).

## Negative results

- A run with no row for a key is read as holding none, not as holding the default: a run records its policy
  whole (D78), so two runs of one `bruce` differ only in what a file changed, and a row one run lacks shows as
  added or removed.
- Pairing a changed value of a multi-valued key (`schemes`) was refused: "matrix to foo" says a scheme was
  edited, which no file can do; the values lost and gained are listed apart.
