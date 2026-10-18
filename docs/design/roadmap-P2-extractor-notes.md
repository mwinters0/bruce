# P2 The extractor: notes

What P2 left for later phases, and what it tried and refused. Its intent is
[`roadmap-P2-extractor.md`](roadmap-P2-extractor.md); what it built is the
code, [`../status/STATUS.md`](../status/STATUS.md)'s "What exists" and the
concept map.

## Reproducing the evidence

- **Tests.** `tests/extract.rs` holds the hand-written assertions over the 40
  fixtures, each citing the decision it checks, and one `insta` summary
  snapshot a fixture (`tests/snapshots/`); `tests/common/mod.rs` finds the
  fixtures and is shared with the bench by `#[path]`. A fixture's URL is its
  docs.fp.o one, or a stand-in under `example.org`. Regenerate the snapshots
  with `INSTA_UPDATE=always cargo nextest run`. The pass's own cases run
  against a stub profile in `src/extract/tests.rs`, since `extract` cannot
  name `generic` or `antora` (D39).
- **The bench runs in the suite.** `[[bench]] test = true` makes nextest run
  `bruce::bench/extract` once in criterion's test mode (about 0.3 s), so
  `--affected` reaches it through the fixtures path like any test.

## What P3, the crawler, inherits

- **`extract` is a pure function of one page**: body, URL and the root's
  `HashScope` and profile choice in, `Extracted` out. Which responses reach it
  and the charset are the caller's (`../wip/roadmap-P3-crawler.md`,
  "Which responses are read").
- **`Extracted::hash` is a `u128`** and P1's `pages.content_hash` a `BLOB`, so
  the store's writer picks the byte order; nothing reads the hash yet.
- **`Link::url` is `None` where the raw value does not parse**
  (`http://[bad`). `mailto:`, `tel:`, `javascript:` and `data:` parse and
  resolve to themselves; classifying them `unchecked` is the crawler's (D13).
  An empty `src`, `srcset` or non-anchor `href` is no link; an empty
  `<a href>` is the page.
- **Anchors are one row per distinct value per element**: `<a id=x name=x>`
  gives one, a repeated id on two elements two. Nothing deduplicates across
  elements.
- **A frozen version's canonical names the latest** (`fedora/f39` to
  `fedora/latest`), so a page's canonical is not always its own URL.
- **Fixture counts** over the 30 docs.fp.o pages: 7063 anchors, 97 images, 90
  stylesheets, 29 canonicals, none unresolvable; 300 `content` and 150
  `chrome` anchors, none `unknown`. No fixture has a `<base>`, `srcset` or
  refresh, whose cases are hand-written.

## What P5, CI mode, inherits

- **An unresolved xref's tag is its href less one leading `#`**, the rest
  kept, so `program_management::changes_policy.adoc#_system_wide_changes`
  reaches Resolve whole. Ten tagged links across the fixtures.
- **`antora`'s facts**: `rel_path` keeps its `.adoc` and directories, as a
  resource ID spells it; module, path and a URL-derived component are
  percent-decoded. 28 docs.fp.o fixtures record a version (`""` 15, `master`
  11, `f39`, `f44`); `arm-sbc` none, its component from the URL. On every page
  with a nav container its component equals the URL's first segment under
  `/en-US/`, the premise of the URL fallback.
- **The URL fallback needs lunr's `data-site-root-path`** (I13); a site
  without it gets no component on a page with no nav container.

## Negative results

- **`Element::on_end_tag` cannot carry a borrow**, its handler being
  `'static`: the title is taken by counting `<title>` elements.
- **Strict mode refuses `<select><xmp>…`**; non-strict reads the `<xmp>` body
  as text, so an id in it is no anchor, as in a browser.
- **The generator pre-pass stops at `<body>`**, so a generator meta after it
  is ignored; no fixture has one.
- **A link-tag taker on `*`** was not built: the reader's own selector list
  covers every element that can yield a link.
- **A class substring match for the edit link** (`class~=edit`) was not
  built: `generic/edit-candidates`' `a.edit` "Improve this page" would get a
  tier the spec does not give it.
- **Scanning the forge marker from the right** was not taken: a file path may
  hold an `edit` or `blob` directory more often than a repo is named for one.
- **A finish step borrowing the sink** fails to compile: it keeps every
  handler's borrow alive past the pass. It is a boxed `'static` closure that
  shares state with the profile's handlers through an `Rc<RefCell<…>>`.
- **The site root on `Settled`** was refused: it is Antora's attribute, and
  the core reading it would make the core a library of one profile (D32).
- **Scoping the hash by counting start and end tags** was not built:
  `lol_html` runs an end-tag handler only for an explicit end tag, so an
  implicitly closed root would skew the count for the rest of the page.
