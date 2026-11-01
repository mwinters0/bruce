# P10 — Declarative extraction rules: notes

P10 moved everything Fedora-specific out of the compiled profiles and into the
deployment's config. A `[[profile]]` entry is a profile in the registry beside
`generic` and `antora`. It holds a region, a generator pattern, fact rules,
source rules tried before the edit link's parse, an edit link and the links of
a language menu, and it may extend one compiled profile. A run records each
definition as `policy` rows, and a comparison digest lists every row that
differs between its runs. Locale learning moved into the core, so compiled
`antora` reads no menu and the deployment guide's `fedora` extension names it.
Compiled `antora` reads Antora 3.2's implicit `ROOT` module. The help examples
and the digest's palette are neutral. The decisions are D93 and D94, with D19,
D33, D35, D67, D78 and D91 amended. The register held every one at the wrap,
and the audit found nothing to move in. The keys are `../manual/config.md`'s
("Profiles"), and the guide's extension is `../manual/deploy.md`'s ("How the
pages are read"). This file keeps the evidence, what later phases inherit and
what was tried and refused. Facts for P11 and P12 are in their inboxes,
`roadmap-P11-prose-inbox.md` and `roadmap-P12-generators-inbox.md`.

## The evidence

| Where | What it holds |
|---|---|
| `fixtures/docs-antora-org/`, `fixtures/sphinx-doc-org/` | 13 and 9 saved pages, each `manifest.json` saying why a page is there (`SITES` in `scripts/fixtures.py`); `tests/common`'s `fixtures()` skips both, so the snapshots and the bench corpus did not move |
| `runs/p10-before-20261031-1647/` | the "before": the binary at `d88ee2b`, its `bruce.toml`, and `en-us.duckdb` holding run 1 |
| `runs/en-us-20261101-0005/` | the "after": a copy of that database with run 2 (`b0fc6ac`, the guide's `fedora` extension) beside run 1; `compare.sql`, run with `duckdb -readonly`, and the digest under `digest/` |

- **The crawls agree on everything P10 could change.** `pages.profile` reads
  `fedora` on exactly the 2,329 pages that read `antora`. The only new
  `policy` rows are the three `profile.fedora.*` rows, and the `antora.*`
  facts are the same rows in both runs. No page's title, hash scope, source,
  placement or sitemap membership differs, and the out-of-root locale
  targets are the same 154,770. Of 590,047 link verdict rows, three differ,
  each caused outside `bruce`: a mirror redirect drawn per request
  (`releases/branched/`, P9's spec, "What the declarative extraction rules
  left"), a host timing out on an unchanged page, and one page edited
  between the runs.
- **Counts alone mislead.** `report stats` moved by one broken URL and one
  error, both explained by the first two verdicts above. A verdict can move
  on a page whose content hash is unchanged, so a test that assumes a target
  answers alike twice flags the mirror and the timeout.
- **A SAML sign-in URL on `sso.redhat.com` never matches across runs.** Its
  signature is fresh on each fetch, so `compare.sql` leaves those targets
  out.
- The 10.8 digest was read as JSON, and its HTML was not opened.

## What later phases inherit

- **P9: the guide's config is an extension.** `fedora` is `extends =
  "antora"` with `locale_links = "a.language[href]"`. It has no `generator`,
  so it claims what `antora` claims, and no root names it. `tests/extract.rs`,
  `scripts/e2e.py` and `examples/sim_config.rs` read the block from the guide
  rather than copying it, so a re-grilled guide keeps them true by keeping
  the `file=bruce.toml` tag on one `[[profile]]` block.
- **P9: a run before P10 is comparable with one after.** It has no `profile`
  rows, so a comparison lists them as added. A run that lacks a row is read
  as holding nothing, not the default (D78).
- **P11: the palette replaced** is in its inbox verbatim, and the digest's
  sentence on a forge's new-issue form or a login page answering 200 stays
  (I17).
- **P12: the line between compiled and config was drawn by best effort**
  against docs.antora.org alone. A compiled `sphinx` cannot claim a page by
  its generator, and `hreflang` locales are its inbox's. Compiled `antora` is
  right on every saved docs.antora.org page with no config, the implicit
  `ROOT` included (I26).
- **P13: every selector is CSS under a key named for it** (`selector`,
  `region`, `locale_links`, `edit_link`), so a richer language is a key
  beside it.

## Tried and refused

- **Compiling selectors once saved little.** The 30 docs.fp.o pages'
  `extract` bench went from 49.5 ms to 48.5 ms, inside criterion's noise.
  The cost that remains is lol_html building its matching program in
  `HtmlRewriter::new` on every page, which its public API cannot reuse
  across rewriters. More would take fewer handlers per page (the hash's `*`
  and the `{r} *` descendants are the widest), which changes the pass.
- **An extension's facts in `antora.*`.** `antora.component` is read by name
  in `crawl`, `ci` and `report` (D54, D56), and a declared rule there could
  collide with a compiled fact.
- **A changed value of a multi-valued key paired as an edit.** For
  `schemes` or `region` the values lost and gained are listed apart, since
  "matrix to foo" describes an edit no file can make.
- **The `[profile.<name>]` table form.** A TOML table's keys have no order,
  and `auto` tries profiles in file order. The array form needs toml's
  `preserve_order` no longer. Nothing was released with the table form, and
  serde's type error refuses it, not a message naming the array.

## Limits left in place

- **A site mixing source formats under one profile** gets the first matching
  rule for every page that rule matches. A URL rule asserts one extension,
  and the rest fall to the edit link's parse (`../manual/config.md`).
- **A declared profile's `edit_link` yields to a preference its base set**,
  the base's handler coming first. `antora` sets none, so this binds nothing
  today.
- **The implicit `ROOT` read cannot see where a start path ends**, so a
  directory named `pages` above the real one is read as the implicit
  module's (`module_and_path`'s rustdoc). On Antora 3.1, docs.fp.o's version,
  a `pages/` under a start path is not a page, so the read cannot misfire on
  its built pages.
- **A fact rule reads one element**, and a multi-valued fact waits for a site
  that needs one.
