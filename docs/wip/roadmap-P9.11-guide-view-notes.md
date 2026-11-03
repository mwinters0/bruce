# P9.11 — The guide's view: notes

What 9.12 and 9.8 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The slice order", item 7, and
"What the digest's view left".

## What landed

- **`fedora.html`**, a tagged block of `docs/manual/deploy.md` ("The digest's view"): it extends
  `bruce:digest.html` and replaces the `palette` block with docs.fp.o's palette (`git show
  e30483c:src/report/digest/html/style.css`, both `:root` rules, the screen's and `@media print`'s).
  It declares `bruce_model = 2`.
- **The script names it with `VIEW`** (default `default`, the built-in view), passed to both of S3's
  digest calls as `--view "$VIEW"`. The systemd environment files and the cron line set
  `VIEW=/etc/bruce/fedora.html`.
- **A view in a file is published under the contract's names**, `digest.html` and the dated
  `bruce-YYYY-MM-DD.html` ([9.11.1](roadmap-P9.11.1-view-names-notes.md)).
- **`sim`** requires the view (`sim.check_view`: tagged, extends the built-in view, declares its model
  version). **`deployment.exercise`** runs both nights with `VIEW` set and holds the published
  directory to the contract's names (`check_published`'s `ext`). **`e2e`**
  renders the fixture crawl's digest through the guide's view and holds it to the built-in page with
  palette lines changed and nothing else (`check_guide_view`); the golden files are untouched.
- **The deployment repo** (`mwinters0/bruce-fedora-docs-ci`) carries `fedora.html`, the guide's script
  and `VIEW=fedora.html` in `nightly.yml`; its pinned `v0.3.0-rc.1` has every view feature the
  view uses (no change to `src/report/digest/view/` or `cli::report` since the tag).

## What the next slices inherit

- **The nightly after the push is the view's first run on Actions.**
- **9.8 reviews the digest as the deployment publishes it**, `digest.html`.

## Negative results

- The print rule must be in the view: a `palette` restating only the screen's `:root` leaves a
  printed digest dark (`digest.md`, "Views"), so e2e counts both `:root` rules in the page.
- Piping the view's name through the `--view` flag as an array (`"${view[@]}"`) was not used:
  `cli::tests` reads a `$`-word as one positional argument, and `--view "$VIEW"` with `default`
  as the unset value is one command line for both cases.
