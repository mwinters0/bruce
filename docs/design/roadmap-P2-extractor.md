# P2 — The extractor

What P2 does and why. It is complete; what it left for later phases is
[`roadmap-P2-extractor-notes.md`](roadmap-P2-extractor-notes.md). The decisions
are in [`decisions.md`](decisions.md) and are cited here, never restated.

## Intent

The streaming pass over one page, and everything a page yields: anchors with
tag and placement (D12, D34), links with their kinds (D13), the title, the
content hash (D15), the edit link and its forge parse (D17, D36), and the
profile's facts (D37). Antora's Extract body gives the coordinates D19 resolves
through and tags the xrefs D20 reports. Tested on P1's fixtures (D24).

## What P2 delivers, and where it stops

- **A pure function of one page**: a whole HTML body, the page's URL and the
  root's profile choice in; an in-memory `Extracted` out. It holds the profile
  chosen, the title, the content hash, each anchor with tag and placement, each
  link with raw href, absolute URL, kind and any profile tag, the chosen edit
  link with its forge parse, and the profile's facts.
- **`forge`'s URL-shape parse** (D36): repo, ref and path from an edit URL.
- **The parser bench moves into `bruce`**, `spikes/tokenizer/` is deleted
  (P1's spec, "P1.2 Tokenizer spike"), and `mise run check` runs the bench once
  in criterion's test mode, D24 worded to match. Reasoning:
  `../status/history/2026-10-17.md`, "The bench joins the check in test mode".
- **Not P2's:** the store's rows for anchors, links and facts, the canonical
  URL and its id (D7, D26, P3's `url`), and any subcommand. Whether a body is
  extracted at all (status, content type) and its charset are the caller's
  (`../design/roadmap-P3-crawler-inbox.md`).

## The Extract hook

D38's Extract does three things today's signature has no channel for. Settled
in the grilling, 2026-10-17:

- **A profile declares its content region as selectors** (`antora`
  `article.doc`; `generic` `main`, `[role=main]`). The core turns them into its
  own descendant handlers and decides each anchor's placement when the page
  ends, since `unknown` (D34) means no landmark anywhere on the page, which an
  anchor read before `<main>` cannot know. Rejected: region handlers in each
  profile, each re-deciding `unknown`, and P7's rules a third time.
- **Handlers write to a sink**, not to `Facts` alone: a fact, a tag on the
  link being read (I6), and a preferred edit link (D36).
- **A finish step** runs after the pass over the core's settled result (the
  page URL, the chosen edit link, the site-root path) and may record facts.
  Antora's module and `rel_path` come from the edit URL's
  `modules/<m>/pages/<p>.adoc`, which is the core's ranked choice, so they
  cannot be read during the pass.

All three stay inside D38's Extract, so the hooks stay four. The slice that
changes the trait amends D38 to name the declared region and the finish step.

## The edit link

The core ranks every candidate, highest first (D36), settled 2026-10-17:

1. `rel=edit`;
2. a `title` or link text reading as an edit phrase ("edit this page", "edit
   page", "edit on …");
3. a known class: `edit-this-page`, `td-page-meta--edit`,
   `theme-edit-this-page` (I12);
4. inside a tier, a URL that parses as a forge shape, then document order.

Tier 2 picks docs.fp.o's "Edit this Page" over "Page history" and "Report an
issue" (I8) unaided, so **`antora` overrides nothing**: a `title` match there
would be a docs.fp.o quirk in a generator profile, which D32 refuses, and
Antora's default UI has no title. The sink keeps `prefer_edit_link` for a theme
that needs it. The slice building the ranking drops I8 from D36's "a profile
may override the choice".

## Antora's coordinates

Settled 2026-10-17, draining the P2 inbox's first entry:

- **A page with no nav container** (I8's `without_menu` pages) takes
  `antora.component` from the URL's first segment under the site root, which
  `data-site-root-path` gives; module and `rel_path` come from the edit link as
  on every page. Recording no component was rejected, since D19 could then
  never resolve an xref into these components and a partial CI build (I11)
  leaves them all unresolved; deriving a version from the URL, since no
  versioned component uses the layout. Reasoning:
  `../status/history/2026-10-17.md`, "I8 reviewed: the nav container follows
  the layout".
- **`antora.version` is recorded verbatim** wherever a nav container exists,
  `""` and `master` included, and is absent only where none does. So a
  component with no version fact *is* a URL-derived one, and **no marker key**
  says so a second time; D19 reads a placeholder and an absent version alike.
  The slice building it says so in D19 and in the `antora_pages` view's
  comment, and drops I8's nav-container-only wording from D19.

## The forge parse

Settled 2026-10-17. A shape is recognised by its path on **any host**,
forge.fedoraproject.org being Forgejo on a host nothing else names:
`/src/branch|tag|commit/` Forgejo or Gitea, `/-/blob|edit/` GitLab,
`/blob|edit/` GitHub, `/blob/<ref>/f/` Pagure. `source_repo` is the URL up to
the shape's marker. **The ref is one segment**: a ref with a `/` mis-splits
into ref and path, which only a forge API call per repo could tell apart. That
is a `KD` of stance (a), marked in `forge`, allocated by the slice building
it; `source_url` keeps the whole URL either way.

## How the fixtures are asserted

Settled 2026-10-17. **Hand-written assertions** for every property a decision
names, each test citing it: anchor validity (D12), placement and the chrome ids
(D34, I9), the edit-link ranking (D36), Antora's facts (D19), the unresolved
xref's tag (I6, D20). Beside them **one summary snapshot per fixture** with
`insta`, a dev-dependency: counts by kind and placement, the title, the edit
link and its parse, the facts. Never every link, which on `legal/all-allowed`
is noise nobody reviews.

## Links and their kinds

Settled 2026-10-17 (D13's kinds, amended by the slice building them):

| Kind | From |
|---|---|
| `anchor` | `a[href]`, `area[href]` |
| `image` | `img[src]`, every `srcset` URL, `source[srcset]` |
| `canonical` | `link[rel~=canonical]` |
| `stylesheet` | `link[rel~=stylesheet]` |
| `icon` | `link[rel~=icon]` |
| `link` | any other `link[rel]`, except `preconnect` and `dns-prefetch`, which name an origin rather than a resource and are skipped |
| `script` | `script[src]` |
| `frame` | `iframe[src]` |
| `media` | `src` on `video`, `audio`, `source`, `track`; `poster` |
| `refresh` | `meta[http-equiv=refresh]`'s `url=`, which D13 does not yet list |

**`<base href>` is honoured** in resolving every later href, though no
fixture has one: ignoring it resolves every link on such a site wrongly and
silently. Whether a `refresh` link is followed as a redirect hop (D7) is P3's
(`../design/roadmap-P3-crawler-inbox.md`): docs.fp.o's aliases are 200 pages
whose only content is one ("Facts found while grilling").

## The content hash

Settled in part, 2026-10-17. Raw bytes were rejected: docs.fp.o's `Last
build:` footer changes every page's bytes at every build ("Facts found while
grilling"), so every page would read as changed every run. A hash over the
extracted record (title, anchors, links) was rejected for missing prose edits.

- **The hash covers a scope of the page's stream** as lol_html hands it over:
  tags, attributes and decoded text. **A user may set the scope two ways, by
  selector** (D33's vocabulary):
  1. **a content root**: by default the profile's region (D34), the whole body
     where a page has none. Most sites want this.
  2. **an exclusion list**, subtracted from the scope, so a site can hash the
     whole page but its footer and still see a changed script or stylesheet
     link as a page change. docs.fp.o wants this: its `Last build:` line is a
     `<p>` inside `<footer>`, outside `main.article` and `article.doc`
     (`fixtures/docs-fp-o/en-US/a11y/index.html`, lines 517–586).
- **The keys are per root, both optional**: `hash_root` and `hash_exclude`
  in `[[root]]`, each a list of selectors, since the scope differs by site.
  A selector lol_html cannot parse, or one outside D33's vocabulary (`+`,
  `:has`…), is refused at load with the reason. The loader, the manual's
  config reference and its tested examples change in one slice; the
  deployment guide's docs.fp.o example (P4, D42) shows docs.fp.o's setting.
- **What each combination hashes**, settled 2026-10-17, the manual giving it
  as a table:

  | `hash_root` | `hash_exclude` | Scope |
  |---|---|---|
  | unset | unset | the profile's region, or the whole page where it has none |
  | set | unset | the root |
  | unset | set | the whole page minus the exclusions |
  | set | set | the root minus the exclusions |

  So docs.fp.o writes only `hash_exclude = ["footer"]`. Rejected: one fixed
  default root, the profile's region, under which an exclusion alone barely
  acts and the common case takes two keys. The cost accepted is a default
  that depends on the other key.
- **The hash scope is not the placement region.** A root of `html` would put
  every chrome id in `content` and blind D34's chrome report; the region stays
  the profile's, its per-root override being P7's content-region selector
  (D33).
- **A scope change is P4's to read**: it changes every page's hash once, and
  `runs.config_hash` cannot tell it from any other config edit. P2 records no
  fingerprint of the scope (`../design/roadmap-P4-reports-inbox.md`).
- **The algorithm is xxh3-128** (`xxhash-rust`): change detection is not
  adversarial, and it is faster than BLAKE3 and wide enough. Recorded in D15
  by the slice building it.

## Where the code lives

Settled 2026-10-17, along D39 (`extract` above `profile` and `forge`). Each
file enters `../design/concepts.md` with the slice creating it.

| Path | Holds |
|---|---|
| `extract.rs` | the pass, `Extracted`, the sink, the title |
| `extract/anchors.rs` | validity (D12) and placement (D34) |
| `extract/links.rs` | kinds, `srcset`, `<base>`, refresh (D13) |
| `extract/edit_link.rs` | the ranking (D36) |
| `extract/hash.rs` | the scoped content hash (D15) |
| `forge.rs` | the shape parse (D36) |
| `profile/antora.rs` | Antora's Extract body; `profile/antora/` past the soft limit |
| `benches/extract.rs` | the parser bench (D24) |

## Facts found while grilling

Gathered 2026-10-17 over the 40 fixtures (paths under `fixtures/`), plus five
live docs.fp.o requests. What a decision comes to rely on moves to
`../design/site-invariants.md` with it.

- **URL-bearing markup.** docs.fp.o pages carry `<link rel>` `stylesheet` (3
  a page) and `canonical` (29 of 30, not the 404); the generator fixtures
  only `canonical`, unquoted in Docsy and Material. No icon, alternate,
  preload or `hreflang` link anywhere. None of `<base>`, `srcset`,
  `<source>`, `<iframe>`, media, `<object>`, `<embed>`, `<area>`, `<form>`,
  SVG `<use>`/`xlink:href`. docs.fp.o has 97 `<img>` (30 the chrome logo) and
  240 `<script>` (8 a page). Reproduce: `grep -ohiE '<link [^>]*>' -r .
  --include='*.html' | sed -E 's/href="[^"]*"/href=…/' | sort | uniq -c`.
- **docs.fp.o serves page aliases as 200 meta-refresh pages.** Its playbook
  (now `forge.fedoraproject.org/docs/docs-fp-o`, `prod` branch, `site.yml`)
  sets no `redirect_facility`, so Antora's `static` applies.
  `/en-US/quick-docs/howto-file-a-bug/` (an alias in `bugzilla-file-a-bug.adoc`)
  returns 200: a "Redirect Notice" with `rel=canonical` to the target, `<meta
  http-equiv="refresh" content="0; url=../bugzilla-file-a-bug/">`, a
  `location=` script and `robots noindex`. `/en-US/` itself is one, to
  `docs/`. No fixture is one. Re-verify: `curl -s
  https://docs.fedoraproject.org/en-US/quick-docs/howto-file-a-bug/ | grep -o
  '<meta http-equiv[^>]*>'`.
- **Every docs.fp.o page's bytes change on every build.** A footer carries
  `Last build: <UTC timestamp>`, the same on every page of a build, beside a
  per-page `Last content update: <date>` (not on the 404). It is the only line
  differing between `fixtures/docs-fp-o/en-US/a11y/index.html` and the live
  page ten hours later. Asset URLs carry no query, and generated ids are
  deterministic. Reproduce: `grep -l 'Last build:' -r fixtures/docs-fp-o
  --include='*.html' | wc -l` (30).
- **Charset.** All 30 docs.fp.o pages declare `<meta charset="utf-8">` first
  in `<head>`; all 40 decode as strict UTF-8, none with a BOM.
- **Edit links.** The 29 non-404 docs.fp.o pages carry exactly three
  `div.edit-this-page > a[title]`, no class, rel or text: "Page history",
  "Edit this Page", "Report an issue". "Edit this Page" points at Forgejo
  `/src/branch/` on most, GitHub `/edit/` on four (containers, iot-wg,
  java-packaging-howto, minimal) and GitLab `/edit/` on legal. Of the
  generators, Docusaurus and Docsy are told apart by class alone, Sphinx RTD by
  text alone ("Edit on GitHub", a `/blob/` href); `generic/edit-candidates`
  puts `a.edit` "Improve this page" before the `rel=edit` winner.
- **Region.** Every docs.fp.o page has one `article.doc`. Over the 30: `<a
  href>` 1445 inside, 5618 outside (81 on `arm-sbc` to 298 a page); ids 329
  inside, 150 outside (five chrome ids a page). An empty `id=""` sits inside
  the article on every non-404 page.
- **Unresolved xrefs.** Nine `a.xref.unresolved` across five fixtures, one
  href with two `#`: `#program_management::changes_policy.adoc#_system_wide_changes`
  (`fesco/Spins`). Reproduce: `grep -ohE '<a [^>]*class="[^"]*unresolved[^"]*"[^>]*>'
  -r fixtures/docs-fp-o --include='*.html' | wc -l` (9).
- **`data-site-root-path`** is on every docs.fp.o page, on
  `script#search-ui-script`: relative (`..` to `../../../..`) from the page's
  directory to `/en-US/`, except the 404's absolute `/en-US`.

## The slices

Each makes the next one's mistakes visible: P2.1's per-fixture snapshots are
what every later slice adds lines to, and the hash lands last so its default
scope is tested on Antora's real region.

### P2.1 The pass and its anchors

The Extract hook's new shape ("The Extract hook"), amending D38; the generator
pre-pass and profile choice (D35); the title; anchors with tag and placement
(D12, D34), and `generic`'s declared region; the `insta` summary harness over
all 40 fixtures ("How the fixtures are asserted"). The parser bench moves into
`bruce` and `mise run check` runs it in criterion's test mode, D24 worded to
match; `spikes/tokenizer/` is deleted.

### P2.2 Links

Every kind in "Links and their kinds", `srcset`, `<base href>`, `refresh`,
and each href resolved against the page's URL, raw href kept. Amends D13.

### P2.3 The edit link and `forge`

The core ranking ("The edit link") and the shape parse ("The forge parse"),
with the one-segment ref's `KD` of stance (a) and its marker. Amends D36.

### P2.4 Antora's Extract

`article.doc` as Antora's region, the nav container's facts, and a finish
step giving module and `rel_path` from P2.3's edit link and the URL-derived
component ("Antora's coordinates"); unresolved xrefs tagged on P2.2's links
(I6, D20). Amends D19 and the `antora_pages` view's comment.

### P2.5 The content hash

xxh3-128 over the scoped stream, and `hash_root` and `hash_exclude` in the
loader, the manual's config reference and its tested examples ("The content
hash"). Amends D15.
