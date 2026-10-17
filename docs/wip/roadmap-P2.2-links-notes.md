# P2.2 Links: notes

What the next slices inherit, and what was tried and refused. The intent is
[`roadmap-P2-extractor.md`](roadmap-P2-extractor.md), "Links and their kinds";
what exists is the code (`src/extract/links.rs`) and
[`../status/STATUS.md`](../status/STATUS.md).

## What the next slices inherit

- **The tag reaches the link by a second core handler.** The core's reader
  runs before the profile's handlers on one element, so a tag the profile set
  would not yet have a link to land on. The core therefore registers a tagger
  on the same elements last: it takes the sink's tag and puts it on every link
  the element yielded (`<img src srcset>` is several). P2.4's Antora handler
  is `a.xref.unresolved`; it calls `Sink::tag_link` and needs nothing else.
- **A tag set on an element that yields no link is dropped**, and the reader
  clears any stale one first, so a profile tagging a `<span>` cannot tag the
  next link.
- **`Link::url` is `None` where the raw value does not parse**
  (`http://[bad`). `mailto:`, `tel:`, `javascript:` and `data:` parse and
  resolve to themselves; classifying them `unchecked` is P3's (D13).
- **An empty `src`, `srcset` or `href` on anything but an anchor is no link**,
  as a browser requests nothing for it. An empty `<a href>` is the page.
- **`<base href>` is resolved at the moment it is read**, as the spec says
  ("every later href"): a link before it resolves against the page. Only the
  first `<base href>` counts, even if it fails to parse. A relative one
  resolves against the page's URL.
- **`link` kinds are decided by `rel` tokens**, case-insensitively, in the
  order `canonical`, `stylesheet`, `icon`, then any other token as `link`; a
  `<link>` with no `rel` is no link. `shortcut icon` is `icon`;
  `apple-touch-icon` is `link`.
- **`srcset` follows HTML's parse**: a URL is a run of non-whitespace and may
  hold commas, so `a.png,b.png` is one URL while `a.png, b.png` is two.
- **Fixture facts.** Over the 30 docs.fp.o pages: 7063 anchors (the survey's
  1445 and 5618), 97 images, 90 stylesheets, 29 canonicals, none
  unresolvable. A frozen version's canonical names the latest
  (`fedora/f39` to `fedora/latest`), so a page's canonical is not always its own URL.
  No fixture has a `<base>`, `srcset` or refresh; the cases are hand-written
  in `src/extract/tests.rs` and `src/extract/links/tests.rs`.

## Negative results

- **A tag taker on `*` was not needed**: the same selector list as the reader
  covers every element that can yield a link, and `*` would run the handler
  on every element of every page.
- **Test asserted a canonical is the page's own URL**, which `fedora/f39`
  refutes: a frozen version's canonical is the latest's.
