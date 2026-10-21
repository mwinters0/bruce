# Known deficiencies

The deficiency register, beside [`STATUS.md`](STATUS.md) and read with it.
Every known deficiency carries a stable `KD<k>`, allocated on discovery and
never reused, and **one line here**: what it costs, its stance, and the `.rs`
file carrying its `deficiency: KD<k>` marker. **The comment at that marker is
the detail.** It sits on the mechanism, so the session editing the mechanism
reads it without being sent anywhere. This is an index, not the document.

There are three stances, because these are not one kind of thing, and the
difference decides whether anyone should act:
- **(a)** a consequence of a deliberate tradeoff, never to be worked;
- **(b)** a defect with a known fix and a named destination;
- **(c)** a defect with a known fix and no owner. That is a legitimate resting
  state, said in those words, naming whatever would promote it.

Two things are not here at all:
- A limitation whose remedy the user already has today is a property of how
  the system works. It lives beside its mechanism with no identifier.
- A defect in the code's layout that costs only the next person reading it is
  an `S<k>` in [`seam-findings.md`](seam-findings.md).

The full rules, striking and slice pairing included, are
[`../process.md`](../process.md), "Known deficiencies".
`cd scripts && uv run deficiencies.py` reconciles this index against the code
markers, against [`STATUS.md`](STATUS.md)'s slice checklist, and against the
roadmap's phase index.

- **KD1** — A forge edit URL whose ref holds a `/` (a branch `feature/x`) parses
  into the wrong ref and path, though `source_repo` and `source_url` stay right.
  **(a) deliberate tradeoff**: telling them apart takes one forge API call per repo. Detail:
  `src/forge.rs`
- **KD2** — A forge edit URL whose repo or group is named for a shape's marker
  (`edit`, `blob`) parses at that name, giving the wrong repo, ref and path.
  **(a) deliberate tradeoff**: the right-to-left scan that avoids it misreads a
  file path holding such a directory, which is likelier. Detail: `src/forge.rs`

- **KD4** — The first link to an allowlisted URL decides how it is asked, so a
  URL named by an image before an anchor link has no anchors to check that
  link's fragment against. **(c) unowned**: a design holding every link's kind
  until the crawl ends would promote it. Detail: `src/crawl/frontier.rs`

- **KD5** — A component split across repos' start paths and only partly built reads
  its unbuilt pages as `not_built`, and the inverse check skips their links, a
  component-version being held whole.
  **(a) deliberate tradeoff**: telling the pages a change owns from the rest takes a
  flag naming their globs, which every repo's CI keeps current by hand. Detail:
  `src/crawl/baseline.rs`

- **KD6** — A build of a site whose pages carry no `antora` facts holds nothing, so a
  link to a page the change deletes passes on the baseline's `ok`. **(c) unowned**: a
  non-Antora site checked in CI would promote it. Detail: `src/crawl.rs`

- **KD10** — Resolve's parse departs from Antora's resource-ID grammar (I21): it
  takes the family before the component, not after the module, refuses a version
  with no component, `_` and any family but `page`, and leaves `./` and `..`
  unnormalised, so such an xref out of the build stays `unresolved_xref` and
  fails the gate.
  **(c) unowned**: a content repo writing any of these forms would promote it.
  Detail: `src/profile/antora/resolve.rs`

- **KD14** — Where no version sits off its own segment, Resolve takes a
  component's prerelease for its latest, so a versionless xref into a site
  serving its latest at its own name beside a prerelease (`2.0`, `3.0-beta`)
  resolves to the prerelease. **(c) unowned**: such a site checked in CI would
  promote it. Detail: `src/profile/antora/resolve.rs`

- **KD16** — `Held` knows a version only by its own name, so in a build whose
  latest sits under a renamed segment (`latest_version_segment`), a URL there
  that the change deleted is the baseline's answer, not `not_built`. **(c)
  unowned**: a CI build of a component at its renamed segment would promote
  it. Detail: `src/crawl/baseline.rs`

<!-- deficiency-watermark: KD16 -->
