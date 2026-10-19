# P4.3 The deployment guide's config section and `mise run sim`: notes

`docs/manual/deploy.md` holds docs.fedoraproject.org's worked config as
` ```toml file=bruce.toml ` blocks, one per decision, joined in order into
`sim/bruce.toml`. `scripts/sim.py` extracts every block tagged `file=<path>`
(D42) and runs `cargo run --example sim_config -- sim/bruce.toml`, which loads
the file through `cli::crawl::configure`, the function `bruce crawl --config`
uses, and sends no request. `cli::crawl` is `pub` for it.

## What the next slice inherits

- **`sim/bruce.toml`** is one root, `https://docs.fedoraproject.org/en-US/`
  with its sitemap set, `forge.fedoraproject.org` allowlisted, the forge's
  `issues/new*` and `/user/login*` skipped, the forge at `per_second = 1`, and
  a `user_agent`. docs.fp.o keeps its `Crawl-delay: 1` (D18). P4.4 launches it
  from `sim/` with `setsid`, a release build (`cargo build --release`, before
  the launch: nothing builds while a crawl runs), the DB and log under `runs/`:
  `setsid ../target/release/bruce crawl --config bruce.toml --db ../runs/<dir>/en-us.duckdb > ../runs/<dir>/crawl.log 2>&1 &`.
- **`sim.py` writes only the files the guide names.** A file the guide stops
  naming stays in `sim/` until it is deleted by hand; the crawl's own files
  never need to live there.
- **The skip globs were matched by hand** against P3.8's shapes: `issues/new`
  with and without a query, and `/user/login?redirect_to=…` match; a file
  under `src/branch/main/` and `issues/12` do not. No test holds this; P4.4's
  `skipped` count is the evidence on the real site.

## Negative results

- None measured. A choice the spec left open: the tag is the fence's info
  string (`file=`), not an HTML comment, so it survives in any renderer and
  sits on the block it names.
