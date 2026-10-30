# P7.2 — A run records its policy: notes

What the next slice inherits. The decision is D78.

## What landed

- `config/policy`: `Policy`, the judging settings with their defaults, and
  `Policy::entries()`, the run-wide `(key, value)` rows. It holds one setting,
  `pedantic_trailing_slash`, the only judging setting that exists (D77's macro
  parameter). `Config.policy` is always the default until a key can set it.
- Schema 6: the `policy(run_id, host, key, value)` table. `begin_run` takes the
  policy and writes its rows with the run in one transaction; the baseline
  file's `TABLES` and `prune`'s carry it. `host` is NULL for every row today.
- `[rate]` is `[default]`; the loader refuses `[rate]` naming `[default]`. The
  Rust type stays `Rate` and the field `Config.rate`.

## For P7.3

- **A setting is a `Policy` field and a row in `entries()`**, plus its parse in
  `config`; no schema change. The test
  `store::views::tests::a_redirect_that_only_adds_or_drops_a_final_slash_is_ok_and_pedantic`
  pins the macro's default against `Policy::default()`: each setting the macro
  takes or reads needs the same pin, there being two copies of a default until
  the macro reads the table.
- **`entries()` is run-wide only.** A per-host setting (`status`,
  `doubled_slash`) needs a host-keyed form, `host` being already a column; the
  macro joins a target's host row to the NULL one.
- **`[default]` holds `per_second` and `concurrency`, as `[rate]` did.**
  `robots` is not a default: an external host's robots.txt is always obeyed
  (D8). The asking settings are not in `policy` yet; P7.7 records them.
- **Constants that become settings are still constants**: the 401/403/429
  `error` in `crawl/outcome.rs` and the unchecked schemes in
  `crawl/frontier.rs`. Moving a status to read time is P7.3's, and its default
  map must equal the code it replaces.
- `runs.config_hash` hashes the file's text, so a policy change changes it and
  `import_baseline`'s reuse test (`started_at`, `config_hash`) is unaffected.

## Negative results

Nothing was measured; judging is unchanged and no view reads the table yet.
