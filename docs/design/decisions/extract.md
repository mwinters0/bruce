# Extraction

Part of the decision register, [`../decisions.md`](../decisions.md), whose rules are `docs/process.md`, "The decision
register"; an entry is cited by its `D<k>` against that index. What one streaming pass over a page records: every
reference kind, the anchors a browser would scroll to, the placement of each, and the content hash. `extract` holds the
pass; a profile hands it its region and handlers.

### D13 Every reference kind is extracted, each target fetched once per run
Every element that names a URL a browser loads or follows yields a link with its `kind`, so a broken image or
canonical is found, not only a broken `<a>`; an asset is asked HEAD with a GET fallback. Reports default to `kind =
anchor`, the gate (D9) to every kind. A scheme other than `http(s)` is never requested (D66); the first `<base href>`
resolves every href after it. Rejected: `<a href>` only, missing broken images and canonicals; ignoring `<base>`, which
resolves a site wrongly. Code: `extract::links::Kind`.

### D12 A fragment is valid where a browser would scroll to it
Valid: any non-empty `id`, `name` on `<a>` only, and always `#` and `#top` in any case; matched exactly, as written or
percent-decoded. Each anchor stores its tag and its placement (D34), so links to template chrome are reportable (I9).
Text fragments are `unchecked`. Rejected: every `name` attribute, which passes `#viewport` and `#generator` on every
page. Code: `extract::anchors`, `store::views` (`link_verdicts`).

### D34 An anchor's placement is `content`, `chrome` or `unknown`; the profile draws the line
Antora's region is `<article class="doc">` (I8); `generic` takes `<main>` or `[role=main]` and records `unknown` on a
page with neither, so a heuristic guess never poses as a fact and the chrome report says only what it knows. Rejected:
everything `content` when no landmark exists; `hash_root` setting the region too, a root of `html` putting every chrome
id in `content` (D15); a per-root region, a declared profile's `region` (D33) being the remedy. Code: `extract::anchors`,
`Profile::extract`'s region.

### D15 Pages store metadata, a content hash and their profile's facts; bodies only on request
The hash detects change, validators being unreliable (I3): xxh3-128, no adversary calling for BLAKE3, over what
`hash_root`/`hash_exclude` scope, by default the profile's region. `bodies` is opt-in and prunable, and nothing writes
it yet (KD27). A page records the scope its hash covered and a run its roots' keys, so `changed` tells a scope edit from
changed content. Rejected: bodies by default, about 330 MB an en-US run (2026-10-20's crawl); raw bytes,
docs.fp.o's `Last build:` moving every page every build (I19); a hash of the extracted record, blind to a prose edit; a
set root matching nothing falling back to empty or the region, hiding the misconfiguration (`root_missed`);
`runs.config_hash` differing, which cannot tell a scope edit from a rate edit. Code: `extract::hash`, `report::changed`.
Evidence: `tests/extract.rs`.
