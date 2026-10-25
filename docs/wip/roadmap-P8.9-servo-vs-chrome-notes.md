# P8.9 — Servo's renders beside Chrome's headless shell's: notes

No code. The renders are under `runs/p8.9-compare/`: `chrome-*.png` and
`servo/digest-open-*.png`, of `fixtures/antora/digest.html` as it stands (8.8.1's
golden file) closed, and of `digest-open.html`, a copy with every `<details>`
open, at 390, 1280 and 794 px (print rules applied, as `mise run shot` does), and
`chrome-print.pdf`, Chrome's own print. Servo is `mise run shot`'s 0.7.0 and
Chrome the headless shell 156.0.8078.47 of `runs/headless-browser-trial/`.

## The verdict: not alone

**Servo is enough for a first look at a desktop's and a phone's layout, and not
enough to check what the digest is for.** Closed, at 1280 and 390 px, the two
renders agree to the line: the header, the figures, the legend, every summary and
its marker. Opened, they differ where 8.4.1 and 8.4.4 went wrong:

- **A phone's tables.** At 390 px Chrome breaks `/en-US/legal/privacy/` at its
  hyphen and slashes into a few lines and scrolls the table with a shadow on the
  edge it scrolls toward; Servo breaks at every `/`, makes the cells taller and
  narrower, scrolls at another column (Kind, where Chrome's cut falls at
  Fragment) and paints no edge shadow. A fix to the table's widths, or to the
  shadows, cannot be read off Servo.
- **Print.** Servo has no paged media and no PDF (`scripts/shot.py`), so a break
  rule (`break-inside`, a heading kept with what follows, a repeated header) is
  invisible to it. Chrome's PDF is nine pages, none ending on a heading or on a
  heading's paragraph, read per page as text (below). Its colours and opened disclosures Servo shows as Chrome does.
- **Text.** Servo 0.7.0 still drops the underscores of `host_not_allowlisted`
  and `out_of_scope` in the housekeeping's `<h3>` `<code>` (8.4.4's notes), the
  words reading "host not allowlisted"; Chrome shows them.
- **A fragment.** `digest.html#p3` renders at the top in Servo, which does not
  open the ancestors (8.4.3's notes); Chrome's `--dump-dom` does.

**Chrome stays the reference for a phone's tables and for print**, as
`scripts/shot.py` already says of a phone's layout. Servo stays what
`mise run shot` is: a render for a person to read, nothing asserting on it.

## The Chrome recipe, as it works

`runs/headless-browser-trial/` holds the headless shell, the three libraries it
needs (`archpkgs/`, which `LD_LIBRARY_PATH` finds) and a `fonts.conf`; the font
config of `mise run shot` (`runs/shot/fonts.conf`, **absolute**, a relative
`FONTCONFIG_FILE` being "not found") serves as well.

```sh
T=$PWD/runs/headless-browser-trial
LD_LIBRARY_PATH=$T/archpkgs/root/usr/lib FONTCONFIG_FILE=$PWD/runs/shot/fonts.conf \
  $T/mise-data/installs/http-chrome-headless-shell/156/chrome-headless-shell \
  --no-sandbox --hide-scrollbars --user-data-dir=$T/chrome-profile \
  --window-size=390,5000 --screenshot=$OUT.png file://$PWD/$PAGE
```

`--print-to-pdf=$OUT.pdf --no-pdf-header-footer` in place of the last two flags
prints; `uv run --with pypdf` reads a PDF's pages as text,
`PdfReader(f).pages[i].extract_text()`, as no rasteriser is installed. Opened
copies are made by adding `open` to every `<details>`, as
`scripts/shot.py`'s `print_copy` does.

## Negative results

- **`mise run shot PAGE` takes an absolute path**: the task runs in `scripts/`,
  so `runs/…` is "No such file". Not fixed: no slice asked for it.
- **Chrome's `--screenshot` of a URL with a fragment** paints blank above the
  target (8.4.3's notes); a reveal is read off `--dump-dom`.
- **The closed summaries differ by 4 px a row** between the two (Chrome's taller),
  a line-height metric, not a layout.

## What 8.20 inherits

The digest the maintainer reviews is read in a real browser, not through Servo:
the checks above are what a render here can and cannot say. The Chrome trial
stays in `runs/` (a call under STATUS's "Decisions worth another look").
