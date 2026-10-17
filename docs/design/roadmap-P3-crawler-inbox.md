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
  name must be refused (D39). `Config::parse` refuses `host."h".robots =
  false` for a host no root starts on only when the file names a root, so a
  URL added from the command line must run `Config::check_robots_hosts`
  again.
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

## The caller decides whether a body is extracted

- **Fact.** P2's extractor is a function of one HTML body, the page's URL and
  the root's profile choice; it never sees a status or a content type. P2's
  grilling recommended extracting only 2xx `text/html` bodies. docs.fp.o's
  404 page carries no generator meta, so under `auto` (D35) it would be
  profiled `generic` (`fixtures/docs-fp-o/en-US/no-such-page-bruce-fixture/404.html`).
  The charset is the caller's to hand over too.
- **Why P3 cares.** The crawler is the first caller. Its grilling settles
  which responses reach the extractor and where the charset comes from. A
  local build's `404.html` (D1) reaches P5's fetcher as an ordinary file.
- **Origin.** P1.2, refiled by P2's grilling, 2026-10-17
  (`roadmap-P2-extractor.md`, "What P2 delivers, and where it stops").

## docs.fp.o's page aliases are 200 meta-refresh pages

- **Fact.** docs.fp.o leaves Antora's `redirect_facility` at `static`, so an
  alias (`/en-US/quick-docs/howto-file-a-bug/`) and the site start page
  (`/en-US/`) answer 200 with a "Redirect Notice": `rel=canonical` to the
  target and `<meta http-equiv="refresh" content="0; url=…">`. P2's
  extractor records the refresh as a link of kind `refresh`.
- **Why P3 cares.** Whether the crawler follows a `refresh` as a redirect hop
  (D7). If not, a fragment on a link to an alias is checked against the stub
  and is a false `broken_anchor`. P2's grilling recommended following it, with
  `redirected` set.
- **Origin.** P2's grilling, 2026-10-17
  (`roadmap-P2-extractor.md`, "Facts found while grilling").
  Contingent on docs.fp.o's playbook keeping the static facility.

## `url_id` could share the content hash's crate

- **Fact.** P2 hashes page content with xxh3-128 (`xxhash-rust`). D26 names
  a u64 hash of the canonical URL and no algorithm.
- **Why P3 cares.** It builds `url` and picks the algorithm. P2's grilling
  suggested the same crate's xxh3-64, one dependency for both. A
  recommendation, not a decision.
- **Origin.** P2's grilling, 2026-10-17.
