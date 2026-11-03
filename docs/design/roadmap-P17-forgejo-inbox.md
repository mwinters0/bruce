# P17 inbox

Facts addressed to the phase running the guide's CI workflow on Forgejo Actions,
before it is grilled. Drained when it is, and then deleted (`../process.md`,
"Inboxes: facts filed by destination").

## The Fedora forge gives a personal repository no runner, and Codeberg does

- **Fact.** The forge's Usage Policy (`forge/documentation`, `usage_policy.adoc`)
  allows only forks in a personal namespace and new repositories only under an
  organization, Fedora-specific ones alone. Runners are requested by an org's Owners
  and provisioned per org (`runners.adoc`, the `new_runner.yml` template). All 32
  production runner definitions in `infra/ansible`
  (`roles/openshift-apps/forgejo/runners/production/`) are org-scoped, and personal
  forks show `has_actions: false`. The policy lets a dedicated runner attach to an
  org or a repository, under the same Fedora-only rule. Codeberg's hosted runners
  (`codeberg.org/actions/meta`) serve any public, free-licensed repository with no
  approval: `codeberg-tiny`, `-small` and `-medium` (2, 5 and 10 minutes), Actions
  being off per repository until enabled. Codeberg lets a self-hosted runner register
  to an account, an org or a repository.
- **Why P17 cares.** Which Forgejo host and runner the evidence runs on is its
  first call: the Fedora forge needs the docs org or a registered runner of our own,
  and Codeberg's 10-minute `medium` is the forge's own cap.
- **Origin.** The P9 review of 9.7's mirror, 2026-11-03, read off the documents
  named and the forge's API (Forgejo `v15.0.11`). Contingent on the policy and
  Codeberg's offer staying as they are.

## Forgejo sets the guide's `GITHUB_*` variables, the server URL being the runner's

- **Fact.** Forgejo sets every `FORGEJO_*` variable as `GITHUB_*` too. For
  `pull_request`, `GITHUB_SHA` is the pull request's head, not a merge commit
  (v15.0 `notifier_helper.go`; `forge/documentation` run 10 carries PR #21's head).
  `GITHUB_SERVER_URL` is the address the runner registered with, not the public
  one (runner v12.10.2, `runner.go:65`): on the Fedora forge
  `http://forgejo-http.forgejo.svc.cluster.local:3000`, which a job container reaches,
  fetching a SHA from it working. `container:` runs on the forge's rootless Podman
  runners; `permissions:` parses and nothing in the v15 server reads it.
- **Why P17 cares.** The guide's C1 fetch should work unchanged there, which the
  phase measures rather than assumes; and any URL a workflow builds from that
  variable for a reader, a link in a report say, is the internal one.
- **Origin.** The P9 review of 9.7's mirror, 2026-11-03, from Forgejo's source and the
  forge's own runs, none of them ours. Contingent on the Forgejo and runner versions.
