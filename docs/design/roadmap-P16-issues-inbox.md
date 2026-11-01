# P16 inbox

Facts addressed to the phase filing issues for broken links before it is
grilled. Drained when it is, and then deleted (`../process.md`, "Inboxes:
facts filed by destination").

## Who answers for a break is already decided, by its cause

- **Fact.** The digest's `new_breakage` names the target page's source for
  a break caused by `target_removed`, `target_moved` or `anchor_removed`,
  and the linking page's for the rest (`report::digest::cause`, `CAUSES`).
  `broken_targets` counts the links per target, so one anchor restored in
  the target's repo fixes them all (`../manual/digest.md`).
- **Why P16 cares.** "The repo that owns it" is that rule, and one issue
  per link files a removed anchor's N links as N issues in the target's
  repo, where one per target is one fix; which unit an issue is, and whose
  repo it goes to, is the call.
- **Origin.** P16's sketching, 2026-11-01, read off the source at `9fb39b1`.
  Contingent on the causes and their sources staying as they are.

## Newly broken and fixed are a diff of two runs, keyed by page, target and fragment

- **Fact.** The digest's sections 3 and 4 key a link as `diff` does,
  `(page, target, fragment)`, and its JSON carries each row with
  `source_repo` and `source_url` (`report::digest::cause`,
  `../manual/digest.md`). D9 accepts that a renamed page's links all look
  new.
- **Why P16 cares.** Those rows are the hooks' "newly broken" and "fixed"
  events ready made, but an issue's lifetime spans more than two runs: a
  rename closes every issue on the page and opens them again, and a run
  skipped, or a link fixed and broken again, is invisible to a diff. Whether
  issues reconcile against the tracker's open set or against the last run
  is the call.
- **Origin.** P16's sketching, 2026-11-01, read off the source at `9fb39b1`.
  Contingent on the digest's key.

## A page's repo is a URL; which forge serves it is not kept

- **Fact.** `forge::parse` tells Forgejo and Gitea, GitLab, Pagure and
  GitHub apart by an edit URL's path on any host, but the store keeps only
  `source_url`, `source_repo`, `source_ref` and `source_path`
  (`store::schema`), not which shape matched. A repo's issues need not live
  in it: Forgejo, GitLab and GitHub each let a repo disable its tracker or
  point at an external one.
- **Why P16 cares.** Choosing a tracker's API per repo needs the forge's
  kind, re-parsed, stored or configured, and an owner whose tracker is
  another repo or disabled needs a mapping; where each lives is the call.
- **Origin.** P16's sketching, 2026-11-01, read off the source at `9fb39b1`.
  Contingent on `pages`' source columns.
