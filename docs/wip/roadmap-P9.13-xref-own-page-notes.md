# P9.13 — An xref whose target is `broken_url` stays on its own page: notes

What 9.13.1 and later slices inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The slice order", item 7; the decision is D57
(amended), the deficiency it closes KD40, found by [9.7](roadmap-P9.7-closing-evidence-notes.md).

## What landed

- **The link of a resolved xref is written last.** `crawl::xref::Kept` holds both forms of it, pointing at the
  resolved target and on its own page with the refspec as fragment; `Shared::send_kept` sends them after
  `copy_from_baseline`, through `WriteHandle::xref_link`. The writer flushes what its appenders hold, reads the
  target's `outcome` in the run, and appends the own-page form where it is `broken_url`, the resolved form otherwise
  (`store::writer`, `Msg::Xref`). A target the run lacks (a stopped run) takes the resolved form, as before.
- **The request for a derived URL is still made**, since its outcome is what decides the link; a broken one stays in
  the run as a target nothing links to. The writer cannot tell a derived URL from a page the baseline's facts found, and
  does not need to: a page with facts is never `broken_url`.
- **Tests**: `crawl::baseline::tests` asserts where each xref's link points (`aims`), in the two tests D57 cites.
  The `store::views` rule 1 clause on a `broken_url` target stays, for a target the policy judges `broken_url`
  where the crawl did not (KD23).

## Not delivered

- **The mirror's clean pass** is 9.13.1. The mirror's workflow fetches the binary from beside the baseline, which is
  `v0.3.0-rc.1`'s, so the pass waits on a release carrying this change (a tag the maintainer pushes), the deployment
  repo's pin bumped and a nightly publishing the new set. Nothing here was run against production's baseline: the
  fixture tests are the evidence so far.

## Negative results

- A link row cannot be chosen where `settle_xrefs` makes it, the derived URL's outcome being known only after the
  request, which the drive after it makes; hence the writer's late choice.
