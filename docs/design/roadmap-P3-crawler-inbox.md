# P3 inbox

Facts addressed to the crawler phase before it is grilled. Drained when it
is, and then deleted (`../process.md`, "Inboxes: facts filed by destination").

## Three settings the crawler needs have no config key yet

- **Fact.** `src/config.rs` holds only the keys D5, D35 and D40 name with a
  default. Three settings other decisions give to config have none: the
  per-host concurrency cap (D8), per-host overrides of it, and the per-root
  switch that strips the query string from canonical URLs (D7). None has a
  default recorded anywhere.
- **Why P3 cares.** Its grilling picks the concurrency cap's default and the
  key names. Each key lands in `config.rs` and `../manual/config.md` in one
  change.
- **Origin.** P1.4, 2026-10-17. Contingent on D7 and D8 keeping those
  settings in config.

## How a URL on the command line meets a config file is undecided

- **Fact.** The loader offers `Config::for_url` (no file: the URL is the only
  root, D40) and `Config::load` (a file, whose roots may be empty). Nothing
  says what `bruce crawl URL --config f` does when the file has roots, nor
  which keys D5's CLI overrides cover. A profile name other than `auto` is
  kept as a string; `cli`, which builds the registry, is where an unknown
  name must be refused (D39).
- **Why P3 cares.** The `crawl` subcommand is the first caller, so whichever
  slice adds it decides both, in the manual too.
- **Origin.** P1.4, 2026-10-17.

## Scope globs match the canonical URL

- **Fact.** The maintainer settled that `include` and `exclude` globs match
  D7's canonical form of a URL, the one the store and reports show.
  `config::Root::in_scope` matches `Url::as_str()` today: percent-encoded
  where D7 decodes, the query kept where a root may strip it. D44 still says
  "the whole URL".
- **Why P3 cares.** It builds the canonicaliser. The slice that does makes
  `in_scope` take a canonical URL, amends D44 to say so, and adds a test of
  a glob written with a decoded character.
- **Origin.** The review of P1.4's config call, 2026-10-17
  (`../status/history/2026-10-17.md`).
