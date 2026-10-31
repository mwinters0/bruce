# P7.8 — Antora leaves the output: notes

What the next slice inherits. The decisions are D86 and D87.

## What landed

- **`[digest]` `group_by`** (`config::policy::digest`): a fact key, one
  `digest.group_by` row (empty by default), judged by nothing and so a trial's
  file sets it. `Digest.by_group` replaces `by_component`; the JSON key is
  `by_group`, its first column `group`, and `FORMAT_VERSION` is 2 for the
  rename. The HTML never rendered the section and still does not.
- **`Profile::sitemap_note`** (`None` by default; Antora's is the sentence the
  digest printed for every site): `report::digest` takes the `Registry` and
  prints the notes of the profiles that read the run's crawled pages. Every
  caller of `digest` passes one, the tests `report::tests::registry()`.
- **`link_hints`** (`store::views`) is made at each open from the profiles'
  `<profile>_hints` views, and `report_broken` joins it. A registry without
  Antora now opens and installs the report views.

## For P9

- The guide's config section sets `[digest]` `group_by = "antora.component"`
  for Fedora; `docs/manual/config.md`, "Digest", is the reference. Without it
  the digest's JSON has no groups, where it listed components before.
- A site that is not Antora gets no sentence above its sitemap entries.

## Negative results

- **Not measured**: `report_broken` joins a view over `antora_hints` where it
  joined `antora_hints`; with one profile shipping hints the view is that view's
  select, which DuckDB inlines.
- **A `group_by` key is checked for shape only.** The loader cannot know a
  profile's facts, so a misspelt key groups nothing and reads as none (D86).
- **The sitemap note is per profile, not per root**: a run crawling Antora and
  generic roots prints Antora's sentence once, above all of the run's entries.
