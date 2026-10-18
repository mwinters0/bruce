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

- **KD3** — `links.redirected` is `false` on every row, though the schema has
  the column. **(c) unowned**: P4's grilling would decide to drop the column
  (a schema change, D43) or fill it by a view. Detail: `src/crawl.rs`
- **KD4** — The first link to an allowlisted URL decides how it is asked, so a
  URL named by an image before an anchor link has no anchors to check that
  link's fragment against. **(c) unowned**: a design holding every link's kind
  until the crawl ends would promote it. Detail: `src/crawl/frontier.rs`

<!-- deficiency-watermark: KD4 -->
