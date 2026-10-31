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

- **KD4** — The first link to a URL asked live (an allowlisted one, one on a
  root's host outside every scope, or one a build lacks) decides how it is asked, so a URL named by an image before an
  anchor link has no anchors to check that link's fragment against.
  **(c) unowned**: a design holding every link's kind until the crawl ends
  would promote it. Detail: `src/crawl/frontier.rs`

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

- **KD17** — A build serving its latest at its own name holds no `latest`
  segment, so a link under production's `latest` to a page the change deleted
  passes on the baseline's `ok`, and one to a page it adds fails as introduced.
  **(c) unowned**: a content repo's CI linking under `latest`, or a decision on
  how a run learns the served latest, would promote it.
  Detail: `src/crawl/baseline.rs`

- **KD18** — A versionless xref out of the build resolves to an older
  version's page where the component's latest lacks it, so the gate passes a
  link production's Antora leaves unresolved. **(c) unowned**: a content repo
  linking without a version to a page a later version removed would promote
  it. Detail: `src/profile/antora/resolve.rs`

- **KD20** — `report introduced` and `report inverse` with `--top N` count
  the breakage already in the baseline, the inverse links, and their inbound
  pages and components over the N rows they print. **(c) unowned**: anyone
  reading those counts under `--top` would promote it.
  Detail: `src/report/gate.rs`

- **KD22** — A two-run report refused because its default `--to` is a later
  unfinished `check-dir` run's baseline tells the user to name a run with
  `--run`, which that report refuses; `--to` is the answer. **(c) unowned**:
  anyone meeting the refusal on a two-run report would promote it.
  Detail: `src/report/error.rs`

- **KD23** — A target's answer is judged by the policy only inside
  `link_verdicts`, so the queries reading a target's stored outcome show the
  crawl's: a link a remapped status breaks is a new break's `unexplained`, not
  `target_removed`, and `report sitemap` and the digest's sitemap entries show
  the status as the crawl judged it. **(c) unowned**: a judged view of
  targets for those queries to read would promote it. Detail:
  `src/store/views.rs`

- **KD25** — Under `--externals=skip`, a redirect's hop onto a URL on a root's
  host outside every scope and locale is asked for its status, where a link
  to it asks nothing and an allowlisted hop stays `externals_skipped`.
  **(b) owned by M27**. Detail: `src/crawl/frontier.rs`

- **KD26** — `[severity]` cannot name `invalid_url`, a reason the crawl writes
  on a `broken_url` link, so a site cannot move such links off red.
  **(b) owned by M28**. Detail:
  `src/config/policy/severity.rs`

- **KD27** — No option asks a crawl to keep a page's HTML, so the opt-in,
  prunable `bodies` table D15 provides is never written and no run's bodies can
  be read back. **(c) unowned**: anyone wanting a page's HTML kept beside its
  run would promote it. Detail: `src/store/writer.rs`

- **KD28** — A digest whose `--from` names a later run judges every section by
  that run's policy, while its severity map, guide, reach and grouping read its
  own run's, and `report diff` judges the same pair by the run it reports on.
  **(b) owned by M30**. Detail: `src/report/digest.rs`

<!-- deficiency-watermark: KD28 -->
