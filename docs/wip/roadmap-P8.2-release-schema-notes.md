# P8.2 — A release names its schema change: notes

What landed, per the P8 spec's "A release names its schema change itself":
`release.yml`'s step "Name the schema change", between Archive and Publish,
writes `src/dist/release-body.md`, and Publish sends it with `-F body=@…` in
place of its fixed sentence. D43 says why.

## What it does

- The latest published release is `gh api repos/…/releases/latest` (drafts and
  prereleases are never it), so an `-rc.N` tag compares with the last full
  release. The tag's `SCHEMA_VERSION` and that release's, from `git show` of
  its fetched tag, are read by the same `sed` over `src/store/schema.rs`.
- The body names three cases: changed (both numbers, and that each binary
  refuses the other's database), unchanged, and no earlier release (a `404`).
  Any other `gh` failure, an unfetchable tag or an unparsable constant fails the
  step, before anything is published, and the tag may still move (D62).
- `SchemaStep` in `scripts/test_workflows.py` runs the step as `workflows.py
  script` prints it, in a temporary workspace with a git repo as the server,
  a clone as `src` and a stand-in `gh`. It was not run against GitHub: the
  first real exercise is the next `-rc` tag, and `-F body=@file` reading a
  file is `gh api`'s documented behaviour, asserted only as text.

## What the next slice inherits

Nothing. `store::schema` carries a comment that `release.yml` reads the
`SCHEMA_VERSION` line by its text, so reformatting it fails the step.

## Negative results

- No `D` entry of its own: D43 is amended in place, which already said the
  release notes name the change.
- `generate_release_notes=true` stays, GitHub appending its list of changes
  below the body.
