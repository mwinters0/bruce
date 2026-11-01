# P10.6 — Locale learning in the core, `locale_links`: notes

Read on 2026-10-31 at `9a0f676` plus this change. With no `locale_links` a profile learns no locale, `antora` now
among them; docs.fp.o's locales come from the guide's `fedora` profile alone.

## What the next slice inherits

- **`profile::locale_prefix(page, link)`** is D67's departing-prefix computation, moved out of `profile::antora`
  to the parent; `Declarative`'s finish step resolves each `locale_links` href through `Settled::resolve` and
  records its prefix by `Sink::other_locale`, after the base's finish. `Definition::locale_links` is recorded as
  the row `profile.<name>.locale_links`, after `edit_link` and only where set, so every definition recorded
  before 10.6 is unchanged.
- **The guide's profile** (`docs/manual/deploy.md`, "How the pages are read") is the one `[[profile]]` block
  tagged `file=bruce.toml`: `fedora`, `extends = "antora"`, `locale_links = "a.language[href]"`, no `generator`
  (it claims what `antora` claims) and no root naming it. `tests/extract.rs` and `scripts/e2e.py` read the
  block from the guide rather than copying it; `examples/sim_config.rs` builds the registry, so `mise run sim`
  refuses a profile extending what is not compiled.
- **10.8 compares two runs that differ by design in two places**: `pages.profile` of the 2,329 Antora pages is
  `fedora` in the after run, `antora` in the before, and the digest's comparison lists the three
  `profile.fedora.*` rows as differing settings. Neither is a fact, locale or verdict. The before run has no
  `profile` rows. `fixtures/antora/digest.html` was regenerated for the same reason: its Housekeeping now lists
  the three rows.
- The crawl and baseline test modules' `registry()` is the extension (`cli::registry_for`), so their pages'
  profile is `fedora` where a test asserts it.

## Acceptance, as tested

- **docs.fp.o:** over the 29 non-404 pages `fedora` names the same 67 locales, the first `ar`, and compiled
  `antora` none; every fixture extracts as `antora` does through the profile, its name and locales aside
  (`the_guides_profile_extracts_what_antora_does`). The 404 has no generator meta and is read by `generic`, as
  before. `mise run e2e` crawls and gates `fixtures/antora` through the guide's profile; only the digest's
  Housekeeping moved.
- **docs.antora.org** (`docs_antora_org_is_read_right_by_compiled_antora_alone`): every page is `antora` by its
  generator meta; component, version (the unversioned component's `""`), the forge shape of the edit link, its
  ref and a content region are right with no config, and the 404 gives no fact. No page has a language menu, so
  locale learning is checked on docs.fp.o alone.

## Negative results

- Compiled `antora` is right on docs.antora.org except one page shape: `assembler/latest/` has its ROOT module's
  `pages/` directly under the start path (`docs/assembler/pages/index.adoc`), which `module_and_path` does not
  read, so the page records no `module` or `rel_path`. That is Antora's own layout and not Fedora's, so it is
  `antora`'s to learn: KD34, unowned.
- `locale_links` takes no `attr` or pattern: a menu link's `href` is the value, and `hreflang` alternates, which
  carry the locale in an attribute and the page in another, are P12's (its inbox).
