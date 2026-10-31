---
name: release
description: Prepare a `bruce` release on the maintainer's machine — check the tree, draft the release notes from the template, and stop with the tag commands the maintainer runs. Use when the user invokes /release, or asks to cut, prepare or draft the notes of a release.
---

A release is the maintainer's push of an annotated `v*` tag:
`.github/workflows/release.yml` builds it, and its message, with the schema
line the workflow places in it, is the Release body (`docs/design/decisions.md`,
"D43"). **This skill drafts that message and nothing leaves the machine**: no
tag, no `git push`, since a published release is immutable (D62). It makes no
commit either: a version bump is the maintainer's, before this runs.

## 1. Refuse

Stop, saying which, on any of:

- `git status --porcelain` printing anything.
- `mise run check` not ending green (the whole check, not `--affected`).
- `V`, `[package] version` in `Cargo.toml`, already released:
  `git ls-remote --tags origin refs/tags/v<V>` names a tag. Read it from the
  remote, a local tag proving nothing; a `git ls-remote` that fails is a stop,
  not an empty answer. Say the version needs a bump.

## 2. Draft the notes

Draft them into `runs/release-v<V>/notes.md` from
[`notes-template.md`](notes-template.md), off `git log <previous>..HEAD`,
`<previous>` the latest tag without a `-` suffix (`git tag --list 'v*'
--sort=-v:refname`), so an `-rc.N` carries the whole release's notes.

- **The opening sentence is the five-second summary**: what a user gets, and
  that something breaks if it does — "Site policy in config, and some breaking
  renames for uniformity." It may grow further paragraphs, as a commit's
  subject grows a body, only when one sentence truly cannot carry what a user
  must know before the lists.
- **Sections in the template's order, each left out when empty.** ⚠️ CAUTION
  never holds the database schema: the workflow adds that item itself, under
  the notes' section of that name or as one ahead of their first section
  (`release.yml`'s "Write the release body").
- **💥 Breaking changes is every change that needs a user to act**: a renamed
  subcommand, flag or config key, a key the config now refuses, a report's text
  or JSON (or the digest's `format_version`) changing shape, an exit status
  meaning something else — whatever makes a deployment's config, its CI job or
  a script reading `bruce`'s output stop working. Each item says what to do
  where the rename alone does not.
- **🚀 Features takes `feat`; 🔧 Fixes takes `fix` and `perf`.** A commit no
  user can observe — docs, tests, harness, refactors, the record — is left out.
- **An item is one or two short sentences of user-visible effect**, prefixed
  `<kind>(<scope>):`, never the mechanism or a docs link unless a user needs it
  to act, and ends with `( <URL> )`, one
  `https://github.com/mwinters0/bruce/commit/<full SHA>` per commit it covers,
  space-separated.

## 3. Hand over

Print the path of the notes and say **the text is the maintainer's, the draft
an aid to it**: they edit it before tagging. Then print, for them to run:

```sh
# a release is tried as prereleases first (D62); the bare tag follows
git tag -a v<V>-rc.<N> HEAD -F runs/release-v<V>/notes.md --cleanup=whitespace
git push origin v<V>-rc.<N>
```

`--cleanup=whitespace` keeps the `##` headings, which the default cleanup strips
as comments. **The tag must be annotated with text**: the workflow refuses a
lightweight tag and an empty annotation before it builds. Say that **publishing
makes the release immutable**, so a mistake in its notes or assets is a new
version.
