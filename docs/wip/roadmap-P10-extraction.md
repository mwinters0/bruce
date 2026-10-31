# P10 — Declarative extraction rules

What P10 does and why, settled in the grilling of 2026-10-31. The decisions
are in [`../design/decisions.md`](../design/decisions.md) and are cited here,
never restated. Its inbox was drained into it ("What the inbox asked").

## Intent

TOML rules that let a site extend extraction without writing Rust: a selector
and attribute giving a fact, a content-region selector, and a URL regex giving
a source (D33). They write the same key/value facts a compiled profile does
(D37). Resolution logic stays code. **P10 runs before P9** (P9's spec, "P9
follows P10 and P11"), so the deployment guide documents the rules once.

**What P10 is for** (maintainer, 2026-10-31): **everything Fedora-specific
moves out of the compiled profiles before `bruce` is deployed to CI.** The tool
is generic and the deployment holds its site's specifics, D32's two layers
made true of the code, not only of the design. The rules are the means: what
leaves the code is expressed in the deployment's config. Other sites may be
fixtures, as a check on where the line between config and compiled falls.

## The rules are a declarative profile

Settled 2026-10-31 (maintainer). **A `[[profile]]` entry in the config,
named by its `name`, is a profile in the registry beside `generic` and
`antora`**, an array of tables as `[[root]]` is, because the order `auto`
tries them in is the file's and a TOML table's keys have none (amended
2026-10-31, the history entry "P10.3's calls reviewed"). It is chosen by a
root's `profile = "<name>"` or, under `auto`, by a pattern over the page's
generator meta (D35). It holds a region (D34), fact rules (a selector and an
attribute, the key namespaced `<name>.*` as the Sink does) and source rules;
its Resolve and Views are empty, as `generic`'s are (D38). A root only names
it, so nothing is per root (D90).

## The line: compiled is Antora and its default UI

Settled 2026-10-31 (maintainer). **Compiled `antora` knows what Antora, its
default UI and its official extensions emit; what only Fedora's UI bundle or
infrastructure emits is the deployment's config.** The test of an item is
whether docs.antora.org, on the default UI, needs it. A stock Antora site
then needs no config (D40), and what the deployment adds is exactly
Fedora's.

## A declarative profile may extend a compiled one

Settled 2026-10-31 (maintainer). **One level of `extends = "<compiled>"`**:
the extension adds its rules to everything its base does (region, handlers,
Resolve, Views), and may replace the base's region. The base's facts keep the
base's namespace, so `group_by = "antora.component"` and the base's views are
unchanged; the extension's own facts are `<name>.*`. A root names the
extension; under `auto` the config's profiles are tried before the compiled
ones, in file order. `pages.profile` records the extension's name.
docs.fp.o's deployment is then an extension of `antora`.

## Other sites check the line

Settled 2026-10-31 (maintainer). **docs.antora.org** checks that compiled
`antora` alone, with no config, gets the component, version, region and edit
link right; **one Sphinx site** checks a declarative profile end to end.
Their pages are saved as fixtures in an evidence-first slice that lands no
code, beside the Fedora fixtures.

## P10 draws the line by best effort, and ships no new generator

Settled 2026-10-31 (maintainer). **The line is drawn by best effort against
docs.antora.org**; it is settled only once several kinds of site are
supported, which is P12's (`../design/roadmap.md`). So **the Sphinx site's
config is a test fixture**, checking the mechanism, not the line: what it
shows a compiled `sphinx` profile would need goes to P12's inbox, and P10
compiles no profile for another generator.

## A fact rule reads one element

Settled 2026-10-31 (maintainer). **`[[profile.fact]]`, under its profile, is `key`,
`selector`, an optional `attr` (without one, the element's trimmed text) and
an optional `pattern`, a regex whose one capture is the value; the first
matching element on a page that yields a value gives the fact.** An element
without the attribute, with no text, or whose value the `pattern` does not
match is passed over; an empty attribute is a value, as `antora`'s
`data-version=""` is (amended 2026-10-31, the history entry "P10.3's calls
reviewed"). A multi-valued fact waits for a site needing one, `group_by` and
`--fact` counting a page once per value.

## Source rules come before the edit link's parse

Settled 2026-10-31 (maintainer). **A profile's source rules are tried in file
order, the first matching giving the page's source, and `forge::parse` reads
the edit link only where none matches**: the deployment's statement beats the
heuristic. A rule captures `repo` and `path`, `ref` optionally. KD1 and KD2,
the heuristic's tradeoffs, then have a remedy in config, so the slice landing
source rules makes them a property at `forge.rs` and strikes them.

## Locale learning is the core's, its links a profile's selector

Settled 2026-10-31 (maintainer). **D67's mechanism moves out of `antora`
into the core**, beside the Sink's `other_locale`, and **a declarative
profile names the links it reads, `locale_links = "<selector>"`**;
docs.fp.o's extension sets `a.language[href]`, and compiled `antora` reads
no menu. D67 is amended in the slice landing it: a locale is a prefix the
profile's locale links name. `<link rel="alternate" hreflang>`, the standard
and D67's Reopens, is P12's (its inbox), docs.fp.o emitting none to test it
on.

## The edit link's `title` stays core, and a profile may name its edit link

Settled 2026-10-31 (maintainer). **D36's phrase tier keeps matching a link's
`title`**, the accessible name of an icon-only link being the web's, not
Fedora's. **A declarative profile may set `edit_link = "<selector>"`**, the
Sink's `prefer_edit_link`, for a site whose link earns no tier (another
language or theme). Matching on strings at all is P13's to replace
(`../design/roadmap.md`).

## Selectors are CSS, under a key that leaves room

Settled 2026-10-31. **Every selector a rule takes is lol_html's streaming CSS
(D11), under a key named for it (`selector`, `region`, `locale_links`,
`edit_link`)**, parsed and refused at load as `hash_root` is, so a richer
language (P13) is a key beside it and P10's config stays valid.

## The nav-less component fallback stays compiled

Settled 2026-10-31 (maintainer). **`antora` keeps taking a page's component
from its URL under `data-site-root-path` (I13) where no nav container is
rendered**: the computation is Antora's URL layout (I16) and holds for any
bundle omitting the nav, and an extension's facts being `<name>.*`, no rule
could write `antora.component`. D19 says so in the slice that touches it.

## Fedora-flavoured text goes

Settled 2026-10-31 (maintainer). **`report`'s help examples take neutral
names** (`antora.component=docs`, `fr`), and **the digest's palette a
neutral default**, theming it from config being P11's (its inbox holds the
palette replaced, verbatim). The digest's sentence on a new-issue form or a
login page answering 200 stays, a login wall being the web's (I17).

## Auto picks a declarative profile by its generator pattern

Settled 2026-10-31 (maintainer). **A declarative profile's optional
`generator = "<regex>"` is matched against the generator meta under `auto`**
(D35); an extension without one inherits its base's detection, so
docs.fp.o's claims every Antora page, and a standalone profile without one
is chosen only by a root naming it.

## Acceptance: Fedora's extraction unchanged, from fixtures and a crawl

Settled 2026-10-31 (maintainer). **Compiled `antora` with the deployment
guide's extension extracts what today's code does**: the same facts, edit
links, sources, locales and placements on `fixtures/docs-fp-o` and
`fixtures/antora` through the guide's config, and docs.antora.org's saved
pages right under compiled `antora` with none. **Then one en-US crawl under
the guide's config, compared with one before P10** by `report changed` and
the digest's comparison: no fact, locale or verdict differs. The crawl is
about two hours, run detached.

## A run records its declarative profiles, as policy rows

Settled 2026-10-31 (maintainer). **A run records the definition of each
declarative profile its config defines**, used or not, as it records each
root's hash scope (D15), so a comparison tells a rule edited from a site
changed, and the digest names a rule that differs between the runs it
compares, **among the profiles `pages.profile` shows either run used**
(amended 2026-10-31, the history entry "P10.3's calls reviewed"). **A source rule is named by its place in each run's
list**, its order being its meaning (D93), **and the naming aligns the two
runs' lists**, so a rule inserted or removed is named alone, never every rule
after it, and rules removed and added in one gap are paired in order, the
fields that changed named (amended 2026-10-31, the history entry "10.4's call reviewed").
**A comparison digest lists every `policy` row that differs between its
runs under Housekeeping**, the profile rows among those profiles, apart from
the rows that differ from the defaults, and its JSON under a key of its own:
that list says what one run configured and cannot show a row removed, and
rules alone would hide a changed root or scope, which changes what a run
fetched (amended 2026-10-31, the history entry "10.5's call reviewed"; D91
amended as 10.5.1 lands). **They are
`policy` rows (D78)**, keys under `profile.<name>.`, so the schema does not
change (D43), the digest's Housekeeping states them as it states the rest
(D91), and a crawl before P10 stays comparable with one after.

## A report reads a declarative profile from its run

Settled 2026-10-31 (maintainer; amended 2026-10-31, the history entry "10.5's
call reviewed"). **A report reads a declared profile's base from the run's
`profile.<name>.extends` row**, not from the config it is given, and gives the
pages `pages.profile` names it that compiled profile's sitemap note
(`report::digest::sections`), read as they were extracted, as a digest is
judged by its own run's policy (D81). A declarative profile has no report-time
behaviour but its base's, so nothing is rebuilt from the rows, and a report of
an old run never refuses over a definition a later `bruce` would not build.
**Xref resolution reads no recorded profile**: it runs inside the crawl,
`check-dir`'s included, through the registry of the run whose extraction named
the profile (`crawl::xref`).

## Where the code lives

Settled 2026-10-31 (maintainer). **`config::profile`**, a new child, parses
and validates `[[profile]]`, its selectors and regexes refused at load.
**`profile::declarative`**, a new child, builds a profile from it: the
`extends` wrapper and the fact, source, edit-link and locale rules (`profile`
sits above `config`, D39). **The departing-prefix computation moves from
`profile::antora` to the `profile` parent**, beside `Sink::other_locale`,
for every profile. **`cli` adds the config's profiles to the registry ahead
of the compiled ones** (D39). **Every selector, the compiled profiles'
included, is compiled once per run**, not on every page. The concept map's
`config` and `profile` lines gain the two children in the slices adding them.

## The first slice gathers evidence, and the before crawl runs in it

Settled 2026-10-31 (maintainer). **10.1 lands no code**: docs.antora.org's
and a Sphinx site's pages saved as fixtures, and an en-US crawl under today's
code and the guide's config launched detached into `runs/`, the "before" the
closing slice's crawl is compared with in the same database.

## A source rule reads the page's URL or its edit link

Settled 2026-10-31 (maintainer). **One rule shape, `from = "url" |
"edit_link"`, a regex with named captures `repo`, `ref` and `path`**: over
the page's URL for a site with no edit link, over the edit link's for a forge
shape `forge::parse` does not know, a ref holding a `/` among them (KD1).
**A rule may set `repo`, `ref` and `path` as templates over its captures**
(`${page}`, the `regex` crate's expansion), each defaulting to the capture of
its own name, so a rule states what the URL does not: Sphinx's
`/en/<ref>/<page>.html` is `repo = "https://github.com/sphinx-doc/sphinx"`,
`path = "doc/${page}.rst"` (amended 2026-10-31, the history entry "P10.3's
calls reviewed"; `fixtures/sphinx-doc-org`). A rule's `repo` is refused at
load unless it has the URL form `forge::Source::repo` holds, and a URL rule
asserting one extension leaves a site mixing source formats to its edit link,
a property 10.4 states.

## What the inbox asked

Drained 2026-10-31. The site policy's grilling filed that the edit link's
phrases are English and its classes three themes', `generic`'s region `main`
or `[role=main]`, and locales learned only from Antora's language menu
(`extract/edit_link.rs`, `profile/generic.rs`, D67), asking whether a rule
can add to each. A declarative profile's `edit_link` names a link no tier
earns, its `region` replaces the content region, and `locale_links` names the
menu (above); D36's tiers and the generic region are unchanged.

## The slices

Settled 2026-10-31 (maintainer). Evidence first, and a rework of a tested
core path a slice of its own (`../process.md`, "The loop", step 3).

1. **10.1 Evidence**, no code: docs.antora.org's and a Sphinx site's pages
   saved as fixtures (`scripts/fixtures.py`), and the "before" en-US crawl
   launched detached under the guide's config from a copy of today's binary,
   into a database 10.8 reuses.
2. **10.2 Selectors compiled once per run**, the compiled profiles' and the
   core's included, with no change of behaviour.
3. **10.3 A standalone declarative profile**: `config::profile` and
   `profile::declarative`, `region`, `generator`, fact rules and `edit_link`,
   the registry, the `policy` rows and the manual's rows; D33, D35 and D78
   amended; checked on the Sphinx fixture, what a compiled `sphinx` would
   need filed to P12's inbox.
4. **10.4 Source rules**, before `forge::parse`; KD1 and KD2 a property at
   `forge.rs`, struck.
5. **10.5 `extends`**: an extension over a compiled profile, `pages.profile`,
   and a report reading an extension's base from the run's recorded `extends`.
6. **10.6 Locale learning in the core**: `locale_links`, `antora` reading no
   menu, D67 amended and I25 repointed; the guide's `fedora` extension in
   `../manual/deploy.md`, loaded by `sim`; `fixtures/docs-fp-o` and
   `fixtures/antora` through it extracting what today's code does, and
   docs.antora.org's fixtures right with no config.
7. **10.7 The Fedora-flavoured text**: neutral help examples and palette.
8. **10.8 The closing evidence**: the "after" crawl into 10.1's database,
   compared by `report changed` and the digest, no fact, locale or verdict
   differing.

## Facts the grilling found

Found 2026-10-31, at `e30483c`.

- **The Sink already offers every write a rule needs.** `profile::Sink` has
  `fact` (namespaced by the profile's name, `<ns>.<key>`),
  `prefer_edit_link` (an href the edit-link choice takes first; no profile
  calls it, `extract/edit_link.rs:11`), `other_locale` (a locale prefix,
  D67) and `tag_link` (Resolve's input). A profile's `Extract` is a region
  (`&'static [&'static str]`, each a single compound selector), handlers and
  a finish step that sees the page URL, the resolver and the chosen edit link.
- **`generic` is a region and nothing else** (`main`, `[role=main]`):
  no handlers, no facts, no views, `detect` always false, and it is
  `Registry::choose`'s fallback. `antora`'s facts are `component` and
  `version` from `.nav-container`'s data attributes, `module` and `rel_path`
  from the edit URL's `modules/<m>/pages/<rest>`; its locales come from
  `a.language[href]`.
- **The edit-link vocabulary and the forge shapes are constants**:
  three tiers (`rel~=edit`; the phrases `edit this page`, `edit page`,
  `edit on …`; the classes `edit-this-page`, `td-page-meta--edit`,
  `theme-edit-this-page`), and four forge markers (Forgejo, GitLab, Pagure,
  GitHub), the ref one segment (KD1). No config reaches either.
- **Config already parses user selectors**: `hash_root` and `hash_exclude`
  are parsed as `lol_html::Selector` at load and refused when lol_html cannot
  stream them, then discarded; every selector, the profiles' included, is
  parsed again on every page (`extract::hash`, `extract::anchors`,
  `Profile::extract`).
- **`profile` sits above `config`** (D39) and already imports
  `config::ProfileChoice`; no profile reads a config value today, and `cli`
  alone builds the registry.
- **Fact keys are free text downstream**: `report --fact KEY=VALUE` and
  `[digest] group_by` take any key, `group_by` checking only its shape.
- **A run does not record its extraction**: `runs.config_hash` hashes the
  config's text, `policy` (D78) holds judging, asking, gate and digest keys,
  and `pages.profile` names the profile each page was extracted by.
- **Hand-written generator pages exist** under `fixtures/generators/`
  (Docsy, Docusaurus, Furo, Material, Sphinx RTD, generic), read by
  `tests/extract.rs` (I12); no real non-Antora site is saved.

## What the compiled code holds that is Fedora's

Found 2026-10-31, at `e30483c`, from upstream source (antora-ui-default
`0e38223a`, Antora 3.1's `compute-origin.js`, `content-catalog.js` and
`html5.js`, `@antora/lunr-extension`'s `search-scripts.hbs`, Fedora's UI
bundle in `fixtures/antora/ui-bundle.zip`, docs-fp-o's `rebuild-site.py` and
`site.yml` on `prod`) and docs.antora.org's live pages.

- **No Fedora host, rate, locale or version is compiled in.** Every default
  is generic (D10's scope is the root's prefix, D18's rate is 2 a second,
  `Crawl-delay` is read at run time); `en-US`, `per_second`, `robots` and
  Anubis are only the guide's (`docs/manual/deploy.md`).
- **Antora's own, compiled by the line:** `a.xref.unresolved` (I6), the
  `modules/<m>/pages/` edit-path tail, the refspec grammar (I21), the URL
  layout and its `latest_version_segment` inference (I16, I5, a playbook
  option), version order (I20), and the `antora.component`/`antora.version`
  keys `crawl`, `ci` and `report` read by name (D54, D56).
- **The default UI's, compiled by the line:** `article.doc`, `.nav-container`
  with `data-component`/`data-version` (rendered on every page), the
  generator meta, and `div.edit-this-page > a` with the text "Edit this Page"
  (no `title`, no `rel`).
- **Fedora's UI bundle's:** the edit link's `title="Edit this Page"` on an
  icon-only link, which only the `title` half of D36's phrase tier catches;
  the nav container rendered on three layouts alone, which is why `antora`
  falls back to the component under `data-site-root-path` (I13); and a
  language menu `a.language` (`page-languages.hbs`), which the default UI
  lacks.
- **docs.fp.o's build script's:** the menu's locale list
  (`generate_lang_switch_ui`, `{{siteRootPath}}/../<lang>{{page.url}}`),
  which D67's `profile::antora::other_locale` and `crawl::frontier`'s
  locale learning read (I25), the one build artefact compiled in; the lunr
  extension, enabled by `antora --extension=lunr` though `site.yml` disables
  it, so `search-ui-script[data-site-root-path]` (an official extension's)
  is on docs.fp.o and not on docs.antora.org (DocSearch); the forge's
  `edit_url`, whose shape `forge::parse` matches on any host (I23); and
  `CI=true` (I15), which no code reads.
- **Tuned on docs.fp.o, not reading it:** D65's 0.3 hint bound and D66's
  scheme list, from the 2026-10-23 crawl.
- **Fedora-flavoured text:** the digest's two blues (`style.css`, no stated
  origin), `report`'s help examples (`en-us`, `antora.component=fedora-docs`),
  and the digest's sentence on a forge's new-issue form or login page (I17,
  observed on Fedora's forge).
- **Invariants:** I8, I13, I25 and I5's naming are the bundle's or the
  build's; I1–I4, I11, I17–I19 and I22 are docs.fp.o's and no production
  code reads them.
