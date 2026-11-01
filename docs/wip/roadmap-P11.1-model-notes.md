# P11.1 — The model: notes

What 11.2 inherits. The spec's slice list is [`roadmap-P11-view.md`](roadmap-P11-view.md).

## What landed

- **`report::digest::model`** holds the derived part as plain Rust values and no words:
  `standing` (each link's finding under the severity map, a page's and a repo's counts, their ranking, and a page's
  findings, red first), `targets` (the "By: Link Target" split), `policy` (the
  settings off their defaults and the changes between runs split by whether they judge), `row` (`Row`, `rows`, moved
  from `html::markup`) and `figure_changes` (the five figures' deltas). `html` reads it and holds the words: its `tally`
  child words a count (`standing`'s `tally`, split from its counts), `compared::changes` signs a delta.
- **`Profile::sitemap_note` returns `SitemapNote { key, text }`**; Antora's key is `antora.sitemap_from_build`.
  `Digest::sitemap_notes` holds them; the HTML prints `text` (D87).
- Output is unchanged: `report::digest::html::tests` and the digest tests pass as they were, bar the two lines that
  read a note's text.

## What 11.2 inherits

- **The model still borrows**: `Row<'a>` points into `Digest`'s tables, `Page`, `Repo`, `Finding` and `Targets` hold
  `Row`s. minijinja's `Object`s are `'static`-ish (`Arc`), so 11.2 either wraps the `Digest` in an `Arc` and has the
  objects index it, or owns the rows; the shapes (fields, ordering) are what this slice settled.
- **Words still in the model's neighbours**: `severity::class`/`name` (a level's dot class and its aria-label),
  `guide::pedantic_marking` (a level's words), `kinds_at` in the legend and `Entry::question` are the view's and
  move into the default template in 11.2. The model gives them the `Severity` enum.
- **Levels not yet precomputed per row**: `html` still asks `SeverityMap::of_link` / `of_redirect` for the dots of
  `broken_targets`, `errors`, `redirects` and the compared sections' rows, and filters them with `listed`. A template
  cannot call the map, so 11.2 gives those rows a level (as `Finding` has one) in the model.
- **`Judged` stays in `document`**, the spec placing it in `model`: the model reads the `Digest`, and `Digest` holds a
  `Judged`, so a `Judged` in `model` is a cycle (`module_graph.py`). It is already a Rust value the document carries; 11.2's
  objects expose it from there.

## Negative results

- Moving `tally` whole into `model` was refused: it is the words of a count ("broken link", "unverifiable") with the
  dot markup, so only the counts it reads moved, with their `reasons` as a list.
