# P11.2.1 — The palette block: notes

What 11.3 inherits. The spec's slice list is [`roadmap-P11-view.md`](roadmap-P11-view.md).

## What landed

- **`view/default/palette.css`** holds the screen's `:root` and `@media print`'s `:root` and nothing else; `style.css`
  keeps the rest, its `@media print` block included. `digest.html` nests `{% block palette %}{% include "palette.css"
  %}{% endblock %}` inside `{% block style %}`, ahead of `style.css`, and `default_view` registers the third file.
- **The golden file moved once**, `fixtures/antora/digest.html` (`scripts/e2e.py --save-digest`): the print `:root` is
  up with the screen one, the rest of `@media print` where it was. Custom properties resolve at computed-value time,
  so the printed document is the same; no other byte changed.
- **Tests** (`view::tests`): the default holds both `:root` rules ahead of the sheet, and a view extending
  `digest.html` that replaces `palette` keeps the sheet and loses both default rules.

## What 11.3 inherits

- **`--write-default-view` writes `digest.html`, `style.css` and `palette.css`**, and a view's `palette` block replaces
  the two `:root` rules whole: one restating only the screen's leaves the print rule gone, so the printed digest takes
  the screen's palette. A view wanting to keep the print rule writes `{{ super() }}` first.
