# P8.4.1 — The digest's HTML at a phone's width and in print: notes

What landed, in `report/digest/html/style.css` and `html.rs`, and the golden
file `fixtures/antora/digest.html` rewritten (`cd scripts && uv run e2e.py
--save-digest`); no `D<k>` entry, the layout being CSS the tests and the golden
file hold.

- **Tables scroll rather than crush.** `td a` and `td code` wrap with
  `overflow-wrap: break-word`, which leaves their min-content width alone, where
  `anywhere` lowered it to a few characters; the table then outgrows its
  `.scroll` and scrolls. Edge shadows on `.scroll` (`background-attachment:
  local` over `scroll`, `--cover` the container's colour) show that it does,
  and are dropped in print, which lays the tables out in the page's width with
  `anywhere` restored.
- **Every `<summary>` shows a marker**: `summary::before`, a triangle that
  turns when the `<details>` is open, there being none left once `display: flex`
  removed the native one. It is hidden in print, where all are open.
- **Print breaks**: a heading and the paragraph after it keep with what
  follows, `.sect` (a section's `<details>`) may break, which
  `avoid-page` on all `details` had made one unbreakable block, a table's header
  repeats and its rows stay whole.
- **The minor defects**: a list with no row is `None.`, not a header over
  nothing (`table`); a housekeeping fold counts in its title's unit, so
  "Unchecked links" says `81 links` where it said `2`, the reasons (`fold`);
  the summaries' counts say `fragments`, as the tables' column does, where
  `anchor` read as the Kind column's `anchor`, the `<a>` element (`counted`);
  the last row of a table has no bottom border and an open `<details>` no band
  beneath it.

## What 8.9 inherits

- **Checked in Chrome's headless shell 156** at 390 px, opened and closed, and
  printed to PDF, against the fixture's digest. Servo at 1280, 390 and print
  was rendered by `mise run shot` and its print render read; its markers and
  edge shadows at 390 were not. The recipe in
  `docs/status/history/2026-10-23.md` needs `FONTCONFIG_FILE` **absolute**:
  a relative path is "not found" and the page renders in a fallback font.
- **A PDF is read here as text per page**, no rasteriser being installed
  (`uv run --with pypdf`, `PdfReader(f).pages[i].extract_text()`): page 1 no
  longer ends on a heading and its paragraph, and "Housekeeping" no longer
  stands alone. Nothing asserts on page breaks; they are read by eye.
- **`runs/headless-browser-trial/`** stays for 8.9, which deletes it.

## Negative results

- **The golden file has no marker or shadow in it**: both are CSS, so the file
  changes by the `None.`, the fold counts and the summaries' words alone, and
  a regression of the layout is caught only by a person reading a render.
- **The URL columns still wrap at a `/`** in Chrome at 390 px, a break
  opportunity `break-word` keeps; the columns hold a readable width and the
  table scrolls for the rest.
