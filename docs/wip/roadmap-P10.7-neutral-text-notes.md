# P10.7 — Neutral help examples and palette: notes

Read on 2026-11-01 at `0be4feb` plus this change. Nothing a run extracts, stores or judges changes; the digest's
HTML and two help messages do.

## What the next slice inherits

- **`report`'s two help examples** are `antora.component=docs` (`--fact`'s refusal and the manual's row) and `fr`
  (`--name`'s refusal). No other production text names a Fedora component or locale: the remaining `fedora`
  strings in `src/` are test fixtures and a comment on a measured count.
- **The digest's palette** (`src/report/digest/html/style.css`, `:root` and `@media print`) is neutral greys
  with a violet accent, which `--accent`, `--accent-deep`, `--new` and `--shade` (the accent at .45) share; the
  severity colours (`--bad`, `--warn`, `--good`) are unchanged. The sheet it replaces is recoverable as P11's
  inbox says (`git show e30483c:src/report/digest/html/style.css`).
- **`fixtures/antora/digest.html`**, the golden file, carries the sheet inline and moved by the same fourteen
  values; `mise run e2e` compares it, so 10.8's digest is the neutral one on both sides of its comparison.

## Negative results

- The digest's sentence on a forge's new-issue form or a login page answering 200 stays (P10's spec); no other
  Fedora-flavoured text was found in the code, the manual's worked example being the deployment guide's by design.
