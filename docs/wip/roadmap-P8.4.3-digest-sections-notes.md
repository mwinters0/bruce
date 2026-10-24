# P8.4.3 — The digest's sections folded and reordered: notes

What landed, for 8.6's Finding 2
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 2"): `report/digest/html.rs` writes the sections in the spec's order
1, 2, 5, 6, 4, 8, 9 (3 and 7 not built), every one after the figures a
`<details class="sect">` that starts closed, its `<h2>` inside the
`<summary>`; `html/style.css` styles that summary as the heading it was;
the nav follows the order; the HTML's tests, the golden file
`fixtures/antora/digest.html` (`cd scripts && uv run e2e.py --save-digest`)
and the manual's "The HTML" follow. No `D<k>` entry: the order and the folding
are the spec's, and the id's placement is `open_section`'s rustdoc.

## A link into a closed `<details>`

- **A fragment naming an element inside a closed `<details>` opens it, and
  every closed one around it, then scrolls** (HTML's "ancestor revealing
  algorithm", run by "scroll to the fragment"), in **Chrome 97**, **Firefox
  139** and **Safari 26.2** on. Firefox: bug 1724299, `PresShell::GoToAnchor`
  calling `AncestorRevealingAlgorithm` on the target. WebKit: commit
  `23cdbeb3cd` ("Auto-expand details element for find-in-page and scrolling
  to fragments", 2025-06-26), on by default from `5bc51f2367` (2025-08-02),
  `LocalFrameView::scrollToAnchorFragment` calling
  `revealClosedDetailsAndHiddenUntilFoundAncestors`; WebKit's "WebKit
  Features for Safari 26.2" names the feature. Chrome's version is its
  release notes', not read in source.
- **Only the ancestors open; the target does not.** Chrome's headless shell
  156 over the fixture's digest, `--dump-dom 'file://…/digest.html#p1'`:
  the pages' `<details class="sect">` gains `open`, `p1`'s own `<details>`
  does not, and `#pages` or `#housekeeping` opens nothing.
- **An older browser does nothing visible**: the element has no box, so there
  is nowhere to scroll. Servo 0.7.0 is one: `servoshell … digest.html#p3`
  renders the page at its top, the pages closed.
- **The choice**: the ids stay on the `<section>`, outside its `<details>`,
  so a nav link lands on the closed section's heading in every browser, one
  click opening it. A repo's link to a page's row (`#p<n>`) opens the pages
  and lands on the row, marked by `details:target`, in the three browsers
  above, and does nothing in an older one until the pages are opened; the
  manual says so.

## What 8.7 and 8.8 inherit

- **A section opens with `open_section(out, id, heading, note)` and closes
  with `close_section`**; it starts closed and is styled by `.sect`.
  `the_sections_after_the_figures_are_pages_repos_targets_errors_housekeeping`
  lists the order, nav included: section 3 goes after the figures, section 7
  after the targets (`fix-once`), each with a nav entry in that place.
- **Prose naming another section's place** says "above" or "below" by this
  order: the targets' one-link count says its links are "among the pages
  above". A section added between them leaves that true.
- **Rendered** by `mise run shot` (Servo) at 1280 and 390 px and in print, and
  by Chrome's headless shell at 390 px, closed and opened: at 390 px a long
  heading wrapped below its marker until the `<h2>` was given `flex: 1 1
  10rem`; it now wraps beside it. Chrome's `--screenshot` of a URL with a
  fragment paints blank above the target, so the reveal is read off
  `--dump-dom`.

## Negative results

- **Rejected: the id inside the section's `<details>`** (on its first
  paragraph), which would open the section from the nav in the three browsers
  and land nowhere in an older one, and scroll the heading off the top.
- **Rejected: `details:has(:target)::details-content { content-visibility:
  visible }`** as an older browser's fallback: `::details-content` arrived
  in Chromium 131, Firefox 143 and Safari 18.4, so it helps Safari 18.4 to
  26.1 alone, and it shows a `<details>` whose `open` is unset, which a click
  on its summary then cannot close while the fragment stands.
- **Kept: the housekeeping's list in its summary**, beside the heading (on
  its own line at a phone's width), its heading alone saying nothing of what
  it holds; the other headings say what a row is.
- **No browser behaviour went to `site-invariants.md`**, whose scope is the
  sites checked and the tools that build them; the versions are here and in
  the manual, a property of where the reader opens the page.
