# P6.1.2 — libpg_query's PostgreSQL License paragraphs: notes

What landed, per the P6 spec's "`bruce` is MIT OR Apache-2.0, and ships its
dependencies' notices": `licenses/duckdb/libpg_query/SOURCE-NOTICE-TERMS`
holds the standard PostgreSQL License permission and disclaimer paragraphs,
copied from `postgresql.org/about/licence/` on 2026-10-22, under a header that
says so and says why. `THIRD-PARTY-NOTICES.txt` prints it right after the
`SOURCE-NOTICE` lines it accompanies. `HAND_TEXTS` in `scripts/notices.py`
names it; its docstring says how it is checked.

## What 6.2 inherits

- **Nothing new for the workflow.** The file is in the committed notices, so
  `v0.1.0-rc.1`'s archive carries it.
- **`--refresh-duckdb` keeps it**, as it keeps every `SOURCE-NOTICE`: the
  refresh deletes `licenses/duckdb/` and re-copies from DuckDB's repository,
  and restores the hand-kept files from `HAND_VENDORED` and `HAND_TEXTS`.

## Decisions made on the way

- **The paragraphs only, not the website's copyright lines.** The site prints
  `1996-2026, The PostgreSQL Global Development Group`; libpg_query's source,
  and so `SOURCE-NOTICE`, says `1996-2017, ... PGGroup`. Printing the site's
  lines would assert a year the source does not carry. The paragraphs say "the
  above copyright notice", which is the source's lines printed above them.
- **A separate file named to sort after `SOURCE-NOTICE`**, so the lines print
  first and the paragraphs follow them. Not appended to `SOURCE-NOTICE`, which
  the check holds verbatim in the tarball's source file.

## Negative results

- **The check cannot hold the text.** It verifies the file is present and
  opens with `Hand-vendored from https://www.postgresql.org/`; a changed
  paragraph is not detected, which the spec accepts (the text does not change).
