# The concept map

Each concept the code holds, the crates and modules holding it, and why each
crate holds its part. **Read it before deciding where new code goes.** The rule
it serves is [`../process.md`](../process.md), "Seams". `scripts/concepts.py`
holds every production file in the workspace to exactly one concept here.

A line `- **<crate>** — …` claims the backticked paths in it, relative to that
crate's `src/`. A path is a file, or a directory written with a trailing `/`,
which claims everything under it and the `<dir>.rs` beside it. A crate root
(`lib.rs`, `main.rs`) holding only its crate's doc, `mod` lines and re-exports
is claimed by none. `bruce` is one crate (D31), so a concept lives in one top-level module and
spans others only along the layering [`decisions.md`](decisions.md), "D39"
records, each part saying why its module holds it. **Not in one place** names what today's tree scatters and the layering
does not explain.

No code exists yet, so no concept is listed. P1's spec places the first
concepts (`../wip/roadmap-P1-scaffold.md`, "Where the concepts live"), and each
lands here with the slice that creates its files.
