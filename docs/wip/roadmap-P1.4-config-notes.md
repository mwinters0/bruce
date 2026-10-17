# P1.4 Config: notes

What landed, and what the next slices inherit. The row is `P1.4` in
[`../status/STATUS.md`](../status/STATUS.md), its intent in
[`roadmap-P1-scaffold.md`](roadmap-P1-scaffold.md).

## What landed

- **`config.rs` is the loader**: `Config::for_url` for a run with no file,
  `Config::load` and `Config::parse` for one with. A file is read into
  private serde shapes that refuse unknown keys, then validated into
  `Config`, so later modules get parsed URLs, compiled globs and normalised
  host names. Errors name the key (`root[1]: include: …`).
- **The keys** are those D5, D35 and D40 name: `[[root]]` with `url`,
  `include`, `exclude`, `sitemap` and `profile`; `allowlist`; `rate`; and
  per-host `[host]` settings (D8; `roadmap-P1.4.1-config-hosts-notes.md`). A
  file may name no root, leaving the URL on the command line to.
- **A root's scope** is D44. `Root::in_scope` is in `config` because it is
  what `include` and `exclude` mean, and the crawler will call it.
- **The manual starts** as `../manual/config.md`, one table a section. A test
  parses each of its `toml` examples, and `--affected` runs it when the
  manual changes.
- **New dependencies:** `serde`, `toml`, `globset` and `url`; `tempfile` for
  tests.

## What the next slices inherit

- **P1.5:** `config` names no profile beyond `auto`; a root's other profile
  is `ProfileChoice::Named(String)`, for `cli` to check against the registry
  (D39).
- **The keys still missing, and how a URL on the command line meets a
  file,** are in the P3 inbox (`../design/roadmap-P3-crawler-inbox.md`).
- **The `[host]` settings** are loaded but enforced nowhere yet; their
  precedence is stated in the manual and in rustdoc, for the crawler to
  implement.

## Negative results

- **`toml` 1.x reads an integer into an `f64` field**, so `per_second = 2`
  needs no decimal point; the manual's examples hold this.
- **`url::Host::parse` refuses a URL or `host:port` only as `IdnaError`**,
  so `allowlist` and `host` refuse a `/` or `:` first, to say why.
