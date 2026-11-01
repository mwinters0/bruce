# P11.3 — Views from the command line: notes

What 11.4 inherits. The spec's slice list is [`roadmap-P11-view.md`](roadmap-P11-view.md).

## What landed

- **`report digest`** takes `--view FILE` (repeatable, `default` the built-in view), `--json` / `--no-json` and
  `--write-default-view DIR`; `--format` is the other reports' alone, and `DigestFormat` is gone. `Views::load` reads and
  checks every view before the database is opened, `Views::render` renders all of them over one `model::context`, and
  `cli::report` writes nothing until all have rendered (D96).
- **`report::digest::view`** gained `loader` (`bruce:` names, the view's canonical directory, `..`, absolute names and
  links out refused; a path-join callback keeps the built-in's own includes inside it) and `declared` (the
  `{% set bruce_model = N %}` read off the text with a regex, so a view the model outgrew is refused for its version).
  The extension picks the escaping: `.html`, `.htm`, `.xml` HTML's, `.csv` one quoted field, else none.
- **`plural(one, other)`** joined `thousands` and `time`. The exported `digest.html` is the embedded one under a
  `{% set bruce_model = 2 %}` line; the embedded file carries none.
- **`Digest::html()` is test-only**; production renders through `Views`.

## What 11.4 inherits

- **The Czech date `1. 11. 2026`** takes `time('%d')|int`, no padding being in `time`; a view states it that way
  (tested). A `%-d` form would be a small addition if the fixture finds the `|int` unbearable.
- **No filter formats a decimal**: every number the model holds is an integer (counts, seconds), so the decimal comma
  is untested here; 11.4's Czech view shows whether it is needed.
- **A last-digit plural rule is a view macro** (tested in `views.rs` with Polish's); Czech's three forms are `if`s.
- **The model's reference is not written**: the manual's "Views" section says how a view is loaded, named, escaped and
  versioned; the derived part beside the JSON is 11.4's.
- **A file literally named `default`** is the built-in view for `--view`; a file is `./default`.
- **`--write-default-view` replaces no file**, so a stale export is deleted by hand before a newer one is written; the
  alternative, overwriting, loses a deployment's edits to a re-run.

## Negative results

- `Environment::set_loader` needs no feature beyond `multi_template`; no crate was added.
- The declaration read from the render's `State` was refused: a render that fails on a missing name never returns one
  (D96).
- `--json` is the default and exists for symmetry with `--no-json`; clap conflicts the two.
