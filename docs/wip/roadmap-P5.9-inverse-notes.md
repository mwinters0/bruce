# P5.9 notes: the inverse check

Outcome: `bruce check-dir --inverse fail|warn` and `bruce report inverse --run
N` list the baseline's links from pages outside the build to what the change
removed, and `Gate::fails` fails on them under `fail` (the default). The code
is `report/inverse.rs` (D56); the `Gate` carries `inverse` and `mode`, the
section sits between the introduced breakage and the redirected links, and the
JSON object has an `inverse` array. The manual is `check-dir.md`'s "The
inverse check" and `report.md`'s `inverse`.

## What the next slices inherit

- **Held is read back, not carried.** `held()` takes the CI run's built pages
  (the pages with an `antora.component` fact; a copied page has none, D52),
  and `prefix()` cuts each URL after the component's segment and a named
  version's. It is `crawl::baseline::Held::holds` again (D56): a change to
  I16 moves both.
- **P5.10's planted changes** each map to one `removed` value: a deleted page
  (`page`), a removed anchor others link to (`anchor`), a deleted image
  another component embeds (`file`). The fixture's
  versioned component is the test of a named version's prefix.
- **`--top` truncates the count line**, `report inverse`'s notes counting the
  rows kept, as `stats`' totals do not (they sum the page rows before the cut).
- **The cost** on `runs/p5.2-en-us-20261020/en-us.duckdb` as a baseline (588k
  links) with 284 `quick-docs` pages as the build is 2.5 s in a debug build.

## Limits met, not removed

- **The component's segment is the first one equal to its name**, below
  any site root. A prefix whose own path holds a segment named as a
  component (`/quick-docs/` serving a component `quick-docs`) cuts one segment
  early. No deployment has one.
- **A linking page's `component` is its baseline fact**; a page with none
  (a non-Antora page) is grouped by its repo, else by nothing, so the count of
  components counts that group as one.
- **An alias page the build emits is built** and a baseline page behind a
  redirect is read at its landing: `landing_gone` handles the one a baseline
  redirect leads into a held prefix, and a built alias that now leads to a
  renamed page is judged by the anchors of where it lands in the run.

## Negative results

- **"A page not in the build" taken literally counts a deleted page's own
  links.** A page of a held component-version deleted by the change is not
  built, and its links to other removed pages read as breakage; the linking
  pages are those outside every held prefix.
- **The runs' ids are written into the statement's text**, as numbers, so the
  many positional placeholders `Sql` binds stay for the user's values (the
  held prefixes, `--repo`, `--fact`, `--kind`).
