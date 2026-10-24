# P8.4.2 — The digest's direction: notes

What landed, for 8.6's Finding 1
([`roadmap-P8.6-digest-review-notes.md`](roadmap-P8.6-digest-review-notes.md),
"Finding 1"): the words of `report/digest/html.rs`, one rule of
`html/style.css`, the HTML's tests, the golden file `fixtures/antora/digest.html`
rewritten (`cd scripts && uv run e2e.py --save-digest`), and the manual's "The
HTML". Nothing a section computes or ranks changed, and nor did the JSON. No
`D<k>` entry: the words are the code's, and `problem`'s rustdoc says which
word `anchor` keeps.

## The scheme

- **A preposition carries the direction**: a count of links *to* something
  is by target, *on* something by page. The headings read "Broken targets,
  by the links to them", "Pages, by the broken links on them", "Repos, by the
  broken links on their pages" and "Links that errored", and the nav says
  `Targets` where it said `Fix once`. The section ids are unchanged, so an
  in-page link someone kept still lands.
- **"Fix once, fix many" leads the targets' intro** rather than heading it,
  the heading saying what a row is.
- **Every count names its unit and its side.** The figures are "Links on
  them", "Links to a broken URL", "Links to a missing fragment" and "Links
  that errored", the sentence under them giving the broken links' total, the
  sum of the two. A summary reads `113 broken links (1 to a broken URL, 112
  to a missing fragment; of 644 links on the page)`, a repo's `on 5 of its 20
  pages`. The targets' columns are `Links to it`, `Linking pages` and
  `Target's source`, where `Fix in` sat beside a section that blames the
  page's repo; the redirects' `Links to it`, `Redirecting URLs` and `Linking
  pages`; the unchecked reasons' `Targets they name`.
- **A link's two URLs are `Written as` and `Resolves to`**, in the pages'
  drill-downs and in the errors, whose `Page` and `Source` are `On page` and
  `Page's source`.
- **`anchor` means the `<a href>` alone**, the `--kind` vocabulary and the
  JSON's `kind`: the Problem column says `fragment missing` and `URL broken`
  where it said `anchor missing` and `broken`, and the pages' intro says what
  Kind `anchor` is. The repos' intro says a link counts against its page's
  repo, which may not be its target's.
- **`summary .cnt` wraps**, its `<b>` alone kept whole: the longer counts
  overflowed a phone's width on one line. Seen in Servo (`mise run shot`) at
  1280 and 390 px over the fixture's digest; not in Chrome.

## Negative results

- **"Links checked" was wrong as well as undirected**: `totals.links` counts
  every link on the pages, the unchecked among them (en-US: 586,175 holding
  187,470 unchecked). "Links on them" is what it counts.
- **8.4.1's `112 fragments` counted links as fragments**: the 266 links to one
  missing fragment read as 266 fragments. `to a missing fragment` counts the
  links.
- **Rejected: showing Kind `anchor` as `<a>` or `hyperlink`**, which would
  part the HTML's word from `--kind`'s and the JSON's for the one value, and
  needs a word for each of the other eleven kinds; with Problem no longer
  saying `anchor`, the word has one meaning on the page.
- **Rejected: a sixth figure for the broken links' total**, which adds to
  what the figures show; the sentence under them names it.
- **Left: `Status` as it was**, the intros saying it is the target's answer,
  and the manual's JSON table, whose `broken_anchor` and "anchor restored"
  are the JSON's words.
- **Open, for the maintainer** (8.6's notes): whether "Repos" should also
  count by the target's repo. The intro states the side it counts; the
  section is unchanged.
