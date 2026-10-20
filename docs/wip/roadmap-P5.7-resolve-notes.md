# P5.7 notes: Resolve

Outcome: `check-dir` resolves an xref Antora left as `#<refspec>` into a
component the build lacks to the baseline's page for it (D55). `antora`'s
Resolve is `profile/antora/resolve.rs`; `crawl/xref.rs` calls it once every
built page is read; `store::BaselineIndex` carries the baseline's page facts
and is the `Lookup`. Run on the unit fixture, `guide::install.adoc#first-boot`
becomes a link to the latest `guide` version's install page and is `ok` or
`broken_anchor` as any link; `tools::missing.adoc`, a held component, stays
`unresolved_xref`.

## What the next slices inherit

- **`link_verdicts` rule 1 changed**: an `xref_refspec` is unresolved only
  while the link's target is its own page. A resolved row keeps the refspec
  and points elsewhere; every report and the gate read the view, so P5.8's
  `introduced` needs nothing for it. A live crawl's rows are unchanged.
- **The gate key moves for a resolved xref**: its target is the resolved URL
  and its fragment the refspec's, so a page's `introduced` row for it names
  the baseline page, not the linking page. `raw_href` (`#guide::…`) still
  tells it was an xref.
- **`Lookup` gained `facts` and `built`.** `built` is `Lookup`'s way to ask
  the run's held set, false for a `RunLookup`; Resolve declines a held
  refspec itself, so `crawl` never parses one.
- **Rows for xrefs are written late.** A built page's tagged link is not sent
  with the page but in `settle_xrefs`, between the first `drive` and
  `settle_unbuilt`; a run stopped in between leaves those links out, as a stop
  leaves any target out.
- **The resolved target is planned as a link of kind `anchor`**, so one under
  the prefix is `Unbuilt`, judged with the rest (D54) and copied from the
  baseline, never asked live; one the baseline lacks is not resolvable.

## Limits met, not removed

- **KD7**: a baseline page with no edit link has no `module`/`rel_path`, so
  an xref to it stays unresolved (168 of P4.10's 2320 `antora` pages).
- **A bare version is held for every version a refspec names**: `1.0@tools::x`
  with `tools` built at `master` reads as held and stays unresolved, as
  P5.6's note has the same URL `not_built`. Both are one limit (`Held::holds`).
- **"Latest" is Antora's order by version name only.** The baseline holds no
  prerelease flag, so a `-` suffix is the only prerelease Resolve knows;
  non-semantic names compare as plain strings, not Antora's numeric collation.
- **A refspec's `family$` other than `page$` is not resolved**, and an
  `antora.version` fact that is a placeholder (`master`, `""`) matches only
  that literal text when a refspec names it.

## Negative results

- **A column marking a resolved link is out**: schema 5 would refuse the
  en-US baseline P5.2 launched (D43), and the refspec is the evidence.
- **Resolve cannot sit in the `baseline` child with the queue.** `xref` reads
  `Held` and the queue's `Xref`, so both live in `baseline.rs` and the
  resolution beside them in `xref.rs`; the reverse is a cycle
  (`module_graph.py`).
