# P5.9.1 notes: every file of the build is a seed

Outcome: `Build::seeds` (`src/ci.rs`) is every file of the build directory, an
`index.html` as its directory, so a `check-dir` run holds a `local` target for
each file the build has, an image, a stylesheet or an attachment included.
`report::inverse` reads `removed = file` as a baseline non-page in a held prefix
with no `local` `ok` target in the run, and its `asked` condition is gone (D56).
The manual's `check-dir.md` and `report.md` no longer say a file no built page
links to is unread.

## What the next slices inherit

- **P5.10's deleted cross-component image is `file` whether or not a built page
  still names it**; the fixture needs no stub page linking to it.
- **A run's target count grows by the build's non-HTML files**, the Fedora UI's
  `_/` assets among them, and only `stats`' count shows it: a seed writes no
  page, link or anchor (`fetch::local` reads no bytes of a non-HTML file in
  `Page` mode), pinned by
  `crawl::baseline::tests::a_seeded_file_no_page_names_is_a_local_target_and_no_page`.

## Negative results

- **Seeding only the baseline's targets under held prefixes** was refused at the
  review: the held set is read off the built pages after the crawl.
