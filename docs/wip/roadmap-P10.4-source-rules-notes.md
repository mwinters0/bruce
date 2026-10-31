# P10.4 — Source rules: notes

Read at `85c9306` on 2026-10-31. With no `[[profile.source]]` every extraction is what it was: the
snapshots and the 30-page corpus pass untouched, and `KD1` and `KD2` are properties at `forge.rs`.

## What the next slice inherits

- **`forge::Rule`** (`forge/rule.rs`) is the compiled rule and `forge::source(rules, page, edit_link)` the
  one place a page's source is decided: the first rule giving a repo and a path, else `forge::parse` of the
  edit link. `config::SourceRule::build` makes a `Rule` from the file's strings, the loader calling it to
  refuse what would not build and `Declarative::new` to keep it, so the checks live once, in `Rule::new`.
- **`Extract::sources`** carries a profile's rules to the core (`Arc<[Rule]>`, empty by default). The core,
  not the profile, applies them, after the edit link is chosen and before the finish step, because a rule
  over the URL gives a source where there is no edit link. 10.5's `extends` has the extension's `Extract`
  carry its rules ahead of its base's (a base states none today).
- **`EditLink` is gone.** `Extracted::edit_link` is the `Url` and `Extracted::source` the source, from a rule
  or from the parse; `Settled` gains `source`, and `antora` reads `module` and `rel_path` from it instead of
  from the edit link's parse, so a rule that states `modules/<m>/pages/…` in its path gives them too.
  `Source::git_ref` is an `Option<String>`, `pages.source_ref` having been nullable all along.
- **`profile.<name>.source.<i>.{from,pattern,repo,ref,path}`** are the `policy` rows (`i` from 0, the
  file's order); the digest's Housekeeping lists them as it lists the rest.

## Behaviours a rule has, as tested

- A template left out takes the capture of its own name; `ref` takes none where there is no such capture, so
  a rule with no ref gives `source_ref` null.
- A rule whose expansion is no source (an empty path, a path ending `/`, a repo that is no `http(s)` URL
  with a host and a path, or one with a query or fragment) is passed over and the next rule is tried. A
  repo is normalised to `origin/path` without a final `/` or userinfo, the form `parse` gives.
- A `repo` template is checked at load as far as it can be: with no `$` it must be the URL form; with one,
  what precedes the first `$` must be empty or begin `http://` or `https://`. Every `${name}` and `$n` must
  name a capture the regex has.
- A rule over `edit_link` is passed over for a page with none. The edit-link ranking still prefers a
  candidate `parse` reads (D36), the rules playing no part in which link is chosen.

## Negative results

- A site mixing source formats under one profile gets the first matching rule for every page it matches
  (the manual says so): a rule asserts one format and the rest fall to the edit link's parse. Nothing here
  tells a rule edited from a site changed; 10.5.1's comparison of recorded rules is the place.
- No crawl-level test of a rule: `write_page` reads `Extracted::source` alone, and the extraction tests
  cover the Sphinx fixture end to end (`profile::declarative::tests`).
