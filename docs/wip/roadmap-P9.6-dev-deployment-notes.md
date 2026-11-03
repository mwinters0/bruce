# P9.6 — The dev deployment: notes

What 9.7 to 9.10 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "The dev deployment is a
GitHub repo of its own" and "The workflows, step by step". The evidence is kept
in `runs/p9.6/` (`sizes.txt`, `SHA256SUMS`).

## What landed

- **`v0.3.0-rc.1`**, a prerelease the maintainer tagged on `2f3a729`, its notes the
  maintainer's; the release workflow published it.
- **The deployment repo** (`mwinters0/bruce-fedora-docs-ci`, `main`) holds `bruce.toml`
  and `bruce-nightly.sh` as the guide has them, and `.github/workflows/nightly.yml`,
  D1 to D4 of the spec on `fedora-minimal:44`, on a `0 1 * * *` schedule and on
  dispatch, pinning `v0.3.0-rc.1` and keeping no history (`KEEP=`).
- **The first nightly succeeded on Actions** (run `41642136109`, dispatched): 4,313
  seconds in the digest's run, 1:12:41 the job, against 1:15:19 on the maintainer's
  machine. The guide's "about an hour and a quarter" holds, and it now says the
  runner's figure. The `GITHUB_TOKEN` push made `gh-pages` (one commit); Pages was
  set to it at `/` with the API (`POST repos/.../pages`), built, and serves
  `https://mwinters0.github.io/bruce-fedora-docs-ci/`. Fetched from there,
  `SHA256SUMS` lists seven files and `sha256sum -c` passes over all of them.
- **Anubis did not challenge the runner**: the crawl's 3,200 docs.fp.o and 4,100
  forge requests went through (I27 held at crawl scale, not 40 requests).

## What the next slices inherit

- **The published directory's size is not the spec's.** One night is 21.2 MB of
  binary, 6.7 MB of baseline, 10.2 MB of HTML and 5.4 MB of JSON; the guide had 4 MB
  and 0.7 MB, measured on 2026-10-24's digest, before the redirect facts and the
  digest's view. At 30 nights of HTML and 400 of JSON the directory approaches 2.5
  GB, past Pages' 1 GB. The guide states the new figures, and the dev deployment sets
  `KEEP_JSON=30`, as long as its views (about 525 MB). The JSON's bulk is `errors` (2.3 MB: one target,
  `https://docs.fedoraproject.org/en-US/fedora/`, answering 403, repeated for each of
  its 5,657 linking pages), `housekeeping` (1.2 MB, of which the redirects 1.0 MB)
  and `pages` (1.2 MB). That 403 is the site's, the same 5,683 rows in both local
  runs. D64's `Reopens` is the digest's size; 9.9 measures the every-locale one.
- **The first night is uncompared and carried nothing.** The second night (the
  schedule's first, 2026-11-04 01:00 UTC) is the first to run `digest --since` the
  published baseline, to carry `digests/` and last night's content-named set into the
  next orphan commit, and to publish under `KEEP_JSON=30`. Its result is the first
  evidence of those paths on Actions; the stage and carry were exercised by
  `scripts/deployment.py` only. Read it with `gh run list --repo
  mwinters0/bruce-fedora-docs-ci`.
- **9.7's mirror** fetches from the Pages URL above; its `PUB_URL` is that.
- **GitHub disables a scheduled workflow after 60 days without a repository
  commit**; a quiet `main` stops the nightly. The deployment repo's README is the
  place to say so if it happens.

## Negative results

- The guide needed no change to its script, workflow or config for Actions: the
  steps ran as `workflows.py script` ran them, and the push needed no token beyond
  `permissions: contents: write`.
- Pages' `legacy` build (branch source) is used, not a workflow: the spec's
  "Pages deployed from an Actions artifact" stays rejected.
