# P10.3.1 — Declared profiles as a `[[profile]]` array: notes

Read at `be4341f` on 2026-10-31. The 692 tests pass with no extraction or crawl changed: only the
config's syntax moved.

## What the next slice inherits

- **`config::profile::validate`** takes the `[[profile]]` array (`RawProfile`, now holding `name`) and returns
  the `Definition`s in file order. It refuses a name given twice (`profile.x: the name of an earlier profile`)
  as it refuses the rest, `profile.<name>` leading the message; a `[[profile]]` without a `name` is serde's
  `missing field`. The `Profiles` visitor and `toml`'s `preserve_order` are gone, `toml` back to its default
  features, as before 10.3.
- **The manual's key rows are `profile.<key>` and `profile.fact.<key>`**, what `every_key_the_config_reads_is_in_the_manual_with_its_default`
  derives from `RawProfile` and `RawFact`; `profile.<name>.*` remains the name of the rows a run records (D78),
  which are unchanged.

## Negative results

- A TOML array of tables nests its sub-arrays under the last entry (`[[profile.fact]]` after `[[profile]]`), so
  a fact rule written before any `[[profile]]` is an error, not a rule with no profile; the tests that wrote
  `[[profile.x.fact]]` alone now open with a `[[profile]]`.
- The old table form (`[profile.sphinx]`) is refused by serde's type error, not by a message naming the
  array; no deployment had written one (nothing was released with it).
