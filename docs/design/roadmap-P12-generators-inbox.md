# P12 inbox

Facts addressed to the compiled profiles for other generators before the
phase is grilled. Drained when it is, and then deleted (`../process.md`,
"Inboxes: facts filed by destination").

## Locales by `hreflang`

- **Fact.** Locale learning reads the links a profile names
  (`locale_links`, D67); `<link rel="alternate" hreflang>` is the web's
  standard way to name a page's translations, and docs.fp.o emits none.
- **Why P12 cares.** A generator that emits `hreflang` (Docusaurus is said
  to; unverified) could learn locales with no config, which D67's
  Reopens names; whether the core reads it for every profile is the call.
- **Origin.** The declarative extraction rules' grilling, 2026-10-31.
  Contingent on locale learning staying in the core.

## A compiled `sphinx` cannot claim a page by its generator

- **Fact.** Sphinx's pages carry no `<meta name="generator">`
  (`fixtures/sphinx-doc-org`), so `Profile::detect`, which is handed the
  generator string alone (D35), cannot claim one under `auto`. What marks a
  page as Sphinx's is `html[data-content_root]`, and its version only the
  Read the Docs addon's `meta[name=readthedocs-version-slug]`. The
  declarative profile `profile::declarative::tests` reads it with is chosen
  by a root naming it.
- **Why P12 cares.** A compiled profile for a generator that emits no
  generator meta needs Detect to see more of the page, or `auto` to be
  settled by a second pre-pass; whichever it is changes D35's hook.
- **Origin.** The standalone declarative profile's slice, 2026-10-31.
  Contingent on Sphinx still emitting no generator meta.
