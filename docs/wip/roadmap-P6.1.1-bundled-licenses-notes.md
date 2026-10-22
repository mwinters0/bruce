# P6.1.1 — DuckDB's bundled licenses: notes

What landed, per the P6 spec's "`bruce` is MIT OR Apache-2.0, and ships its
dependencies' notices": `THIRD-PARTY-NOTICES.txt` prints the license files of
all 26 `third_party/` directories in `libduckdb-sys`'s tarball, and gives each
MPL-2.0 crate (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`) its
exact crates.io source. The texts are vendored under `licenses/duckdb/`;
`scripts/notices.py`'s docstring says how they are kept and checked.

## What 6.2 inherits

- **Nothing new for the workflow.** The notices file stays committed and is
  copied into the archive; `licenses/duckdb/` is not shipped, its texts being in
  the file.
- **A DuckDB bump fails `mise run check`** (`ThisRepo` in `test_notices.py`)
  until `uv run notices.py --refresh-duckdb` and `uv run notices.py` are run,
  and any `SOURCE-NOTICE` re-read by hand. The refresh needs the network and
  `git`; it stops when the tag's commit is not the tarball's source id.

## Negative results

- **The tarball is `v1.5.6` at `069cc9f9b5` exactly**: the tag's commit
  matches the source id in `pragma_version.cpp`, so the repository's texts are
  the ones for this source. Four upstream directories (`catch`, `imdb`,
  `jemalloc`, `snowball`) are not in the tarball and are not vendored.
- **libpg_query's source carries copyright lines and no PostgreSQL License
  text.** Its `LICENSE` is Lukas Fittl's BSD-3; the Postgres-derived files say
  only `Portions Copyright (c) 1996-2017, PostgreSQL Global Development PGGroup`
  and `... 1994, Regents of the University of California`, so `SOURCE-NOTICE`
  holds those two lines and the license's permission paragraphs are in no file
  the tarball ships (vendored from postgresql.org by 6.1.2).
- **The Bison skeleton's FSF notice** in libpg_query's generated grammar is not
  vendored: its output carries the Bison exception, which asks for no notice.
- **Identical texts are not shared.** Unlike the crate section, the library
  texts are printed per library, each under its name, so a reader finds a
  library by name; sharing would save under 20 KB.
