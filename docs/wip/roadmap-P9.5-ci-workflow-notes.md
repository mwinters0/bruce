# P9.5 — The guide's CI section: notes

What 9.6 to 9.10 inherit, and the negative results. The spec is
[`roadmap-P9-deployment.md`](roadmap-P9-deployment.md), "One workflow for the
forge and GitHub, with no third-party actions", "The CI job runs on Fedora 44 and
installs Antora" and "The workflows, step by step". The pin's invariant is I29.

## What landed

- **The guide** (`docs/manual/deploy.md`, "Checking a content repo's build") gained
  the workflow `.github/workflows/links.yml` (C1 to C5 of the spec), its
  flowchart and a paragraph a step. The workflow's `env:` holds the five
  settings that are a deployment's: `PUB_URL`, `PREFIX`, `PLAYBOOK`,
  `SITE_DIR`, `ANTORA_VERSION` (and `ANTORA_DIR`). The opening sentence saying
  the section was unwritten is struck.
- **`sim.check_workflow`** requires the tagged workflow, fails a `uses:` and reads
  its steps as `workflows.py` does. `cli::tests` reads the `run:` blocks of a
  tagged `yaml` block as it reads a `bash` one, and asserts the `check-dir` line
  is among the lines read.
- **`deployment.ci_job`** runs the workflow's steps as `workflows.py script`
  joins them, the workflow's own `env:` supplying what the caller does not set;
  `yaml_steps` has Antora's `js-yaml` parse the same file. `e2e.py` runs the
  workflow over the `tools` repo (its build step builds the playbook, its fetch
  takes what the second night published, served locally): the unchanged repo
  passes, the planted one exits 1 with the report in stdout, a set whose baseline
  fails its sum and a directory with no `SHA256SUMS` exit 2. The install and
  checkout steps are skipped by name (`deployment.INSTALL_STEPS`); `test_e2e` holds
  the guide's `ANTORA_VERSION` equal to `ANTORA`.
- **The change is fetched by its SHA** after a `git init`, so the checkout has
  no `origin` and the build writes no edit link (I15), as the partial build does.

## What the next slices inherit

- **9.6**: the dev deployment's workflow can copy the guide's C3 fetch for the
  mirrors (9.7); `PUB_URL` there is the Pages URL. A job's exit 2 on a stale or
  missing set is the could-not-run the spec asks for, though a forge shows only
  red: the message is the difference.
- **The Forgejo runner is not exercised.** The workflow's `runs-on: ubuntu-latest`
  is GitHub's; the Fedora forge's label is `podman` (the spec, "Facts the
  grilling found"), and the guide says so beside the example. Whether the forge's
  runner supplies `GITHUB_SERVER_URL`, `GITHUB_REPOSITORY` and `GITHUB_SHA` as the
  C1 fetch needs is unmeasured here; 9.7 on GitHub measures the GitHub side.
- **The workflow's install step ran once by hand in `fedora-minimal:44`**
  (`sudo nerdctl run`): `microdnf` installed `nodejs-npm`, `git`, `tar` and `gzip`
  (Node 22.23.1, git 2.56.0) and npm installed Antora 3.1.15, 24 s in all. The
  build, the fetch and the check were not run in that container; `e2e` runs them
  on the host. 9.6 measures the whole job's time on Actions.
- **The Antora pin matches production's.** `docs/site-builder` `prod` installs
  `@antora/cli@3.1`, the newest 3.1.x, 3.1.15 today (I29).

## Negative results

- `--to-dir` was not used: a playbook with `output.destinations` (quick-docs' has
  an `archive` provider) may not honour it, so `SITE_DIR` is the playbook's own
  `output.dir`, as `docsbuilder.sh` builds.
- `sim` does not parse the workflow as YAML: the scripts are standard library
  only and Python has no YAML parser. `sim` reads the steps as `workflows.py` does
  and `e2e` has `js-yaml` parse the file and agree with it.
