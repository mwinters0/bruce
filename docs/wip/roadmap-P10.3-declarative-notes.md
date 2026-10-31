# P10.3 — A standalone declarative profile: notes

Read at `0c33470` on 2026-10-31. With no `[[profile]]` every crawl, report and
extraction is what it was: the 30-page snapshots and the bench corpus pass untouched.

## What the next slice inherits

- **`config::profile`** parses `[[profile]]` into `Definition` (name, `generator`,
  `region`, `edit_link`, `Fact` rules), refusing at load what the built profile could not
  run, and writes the rows a run records (`Definition::entries`, `profile.<name>.…`).
  They live on `Policy::profiles`, so `Config::policy` is the one place a run's
  definitions are read from; `Entry::is_profile` / `is_judged` keep them out of a trial
  (`Store::try_policy`) and `digest::sections::policy` reads them from `policy` as
  recorded, Housekeeping listing them as it lists the rest (`profile.x.region += "…"`).
- **`profile::declarative::Declarative`** is the trait built from a `Definition`. Its
  fact values are gathered in handlers and written in the `finish` step, because an
  end-tag handler is `'static` and cannot hold the sink. 10.4's source rules and 10.5's
  `extends` add to this type: a base profile to wrap, and a rule list beside `facts`.
- **`cli::registry::registry_for(&[Definition])`** (re-exported as `cli::registry_for`) builds generic, then the file's profiles in
  file order, then `antora`; `crawl::configure` returns it with the config, and
  `crawl::run` / `check_dir::run` take no registry. **Reports still build the compiled
  registry alone**: a page extracted by a declarative profile reaches no profile at
  report time until 10.5 builds one from the recorded rows. `registry_for` refuses the
  names `generic` and `antora`; 10.5's `extends = "antora"` is a field, not a name.
- **`config::selector`** holds `selectors` (a list in one string, split and parsed), moved out of
  `config.rs`, and `AUTO_PROFILE` sits in `config::profile`: `module_graph.py` counts a child
  naming its parent as an edge, so `policy` → `profile` → `config` was a cycle until neither
  reached up. `Definition::rows` returns pairs for the same reason (`Policy` makes the `Entry`s).
- **File order is the array's**: `[[profile]]` entries named by `name`, so `toml` runs without `preserve_order`.
- **`Profile::name` and the region stay `'static`**: `Declarative::new` takes the name
  and the region slice from `profile::interned`, which leaks each distinct text once, as
  `selectors::compiled` does its selectors, so the trait and every consumer of
  `Extract::region` are unchanged (10.2's notes expected a widening).

## Behaviours a rule has, as tested

- An attribute fact is the attribute as written, an empty one a value; an element
  without it is passed over, and a `pattern` that does not match gives nothing, so a
  later element is read. A text fact is the element's text through its descendants,
  decoded and trimmed; one with no text is passed over; a nested match does not start a
  second reading; a void element has none.
- An element the parser closes implicitly never fires its end-tag handler (lol_html's
  documented limit), so the `finish` step closes any reading still open with the text it
  held, which is the element's own, the text handler having stopped at its end.
- A `region` entry is one compound selector (a space or `>`, `+`, `~` outside brackets
  and quotes is refused at load); the core extends it with `[id]`, ` [id]` and ` a[name]`.
  None given takes `generic`'s (`generic::REGION`).
- `edit_link` takes the first match with an `href` as the sink's preference, which the
  core still resolves against `<base>` and parses, so a rule naming a link that is no
  forge shape yields an `edit_link` with no `source`.

## What the Sphinx fixture showed

- With `region = ["div.document"]` the page has more `content` anchors than under
  `generic` (`div.body[role=main]` only): the sidebar's ids join them. The fixture checks
  the region took effect by that difference, as 10.1's notes said it must.
- `h1`'s text is `Getting started¶` (the permalink's text is inside the heading), so a
  heading fact needs a `pattern` to strip it. `html[data-content_root]` and the Read the
  Docs `meta` both read with `attr` and no pattern.
- No generator meta: the profile is picked by a root naming it. What a compiled `sphinx`
  would need is filed in `../design/roadmap-P12-generators-inbox.md`.

## Not done here, by the spec's order

`extends`, source rules (and the 10.1 notes' gap in them), `locale_links` and a report's
registry from the run are 10.4 to 10.6. The digest names a profile rule that differs
between two runs it compares only as far as Housekeeping lists the recorded rows of the
run it reports on; a comparison naming the rule that changed is not built, and needs a
decision on what `compare` shows of a policy (10.5, where the registry is rebuilt from
the rows, is the place to decide).
