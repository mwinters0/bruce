#!/usr/bin/env python3
"""Unit tests for workflows.py, run with `uv run python -m unittest test_workflows`.

A test of a failure breaks exactly one thing and asserts the failure names
it; `ThisRepo` holds the real workflows to the rule and to the parser.
"""

from __future__ import annotations

import io
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Sequence

import workflows

WORKFLOW = """name: demo

on:
  push:
    tags: ["v*"]

jobs:
  demo:
    runs-on: ubuntu-latest
    container: debian:trixie
    env:
      GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    steps:
      - name: Install
        run: apt-get install -y git

      # A comment between steps.
      - name: Build
        working-directory: src
        run: |
          # A comment in the script is the script's.
          cargo build \\
            --release

          echo done

      - name: Publish
        run: gh release create "$GITHUB_REF_NAME"
"""


class Uses(unittest.TestCase):
    def test_a_step_s_uses_is_found_with_its_line(self):
        text = "jobs:\n  j:\n    steps:\n      - uses: actions/checkout@v4\n"
        self.assertEqual(workflows.uses(text), [4])

    def test_a_uses_after_other_keys_and_a_reusable_workflow_are_found(self):
        text = "    steps:\n      - name: x\n        uses: a/b@c\n  j:\n    uses: ./.github/w.yml\n"
        self.assertEqual(workflows.uses(text), [3, 5])

    def test_a_comment_or_a_command_naming_it_is_not_one(self):
        text = "# no step uses: an action\n      - run: echo uses: nothing\n"
        self.assertEqual(workflows.uses(text), [])


class Steps(unittest.TestCase):
    def test_the_steps_are_read_in_order_with_their_directory(self):
        got = workflows.steps(WORKFLOW)
        self.assertEqual([s.name for s in got], ["Install", "Build", "Publish"])
        self.assertEqual([s.directory for s in got], [".", "src", "."])

    def test_a_block_keeps_its_lines_dedented_and_drops_the_trailing_blank(self):
        build = workflows.steps(WORKFLOW)[1]
        self.assertEqual(
            build.run,
            "# A comment in the script is the script's.\ncargo build \\\n  --release\n\necho done",
        )

    def test_a_scalar_run_is_the_line(self):
        self.assertEqual(workflows.steps(WORKFLOW)[0].run, "apt-get install -y git")

    def test_a_step_with_no_run_is_an_error_naming_its_line(self):
        text = "    steps:\n      - name: Check out\n        uses: a/b@c\n"
        with self.assertRaisesRegex(workflows.WorkflowError, r"w:2: a step with no `run:`"):
            workflows.steps(text, "w")

    def test_a_folded_run_is_refused_rather_than_misread(self):
        text = "    steps:\n      - run: >\n          a\n"
        with self.assertRaisesRegex(workflows.WorkflowError, r"folded"):
            workflows.steps(text, "w")


class Script(unittest.TestCase):
    def test_each_step_is_a_subshell_in_its_directory(self):
        out = workflows.script(workflows.steps(WORKFLOW))
        self.assertIn('cd "$GITHUB_WORKSPACE"/src\n# A comment', out)
        self.assertEqual(out.count("\n(\n"), 3)

    def test_the_script_is_valid_bash(self):
        out = workflows.script(workflows.steps(WORKFLOW))
        done = subprocess.run(["bash", "-n"], input=out, text=True, capture_output=True)
        self.assertEqual(done.returncode, 0, done.stderr)

    def test_skip_and_before_pick_steps_by_name(self):
        all_steps = workflows.steps(WORKFLOW)
        self.assertEqual(
            [s.name for s in workflows.select(all_steps, ["Install"], "Publish")], ["Build"]
        )

    def test_an_unknown_name_lists_the_steps(self):
        with self.assertRaisesRegex(workflows.WorkflowError, r"no step named 'Nope'.*Install"):
            workflows.select(workflows.steps(WORKFLOW), ["Nope"], None)


class Check(unittest.TestCase):
    def run_in(self, files: dict[str, str]) -> tuple[int, str]:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            (repo / workflows.WORKFLOWS).mkdir(parents=True)
            for name, text in files.items():
                (repo / workflows.WORKFLOWS / name).write_text(text)
            out = io.StringIO()
            return workflows.check(repo, out=out), out.getvalue()

    def test_a_workflow_of_commands_passes(self):
        code, out = self.run_in({"a.yml": WORKFLOW})
        self.assertEqual(code, 0, out)
        self.assertIn("1 checked", out)

    def test_an_action_fails_naming_file_and_line(self):
        bad = WORKFLOW.replace("      - name: Install\n", "      - uses: actions/checkout@v4\n      - name: Install\n")
        code, out = self.run_in({"a.yml": WORKFLOW, "b.yaml": bad})
        self.assertEqual(code, 1)
        self.assertIn("b.yaml:14: `uses:`", out)

    def test_no_workflows_directory_is_none_checked(self):
        with tempfile.TemporaryDirectory() as tmp:
            out = io.StringIO()
            self.assertEqual(workflows.check(Path(tmp), out=out), 0)
            self.assertIn("0 checked", out.getvalue())


class ThisRepo(unittest.TestCase):
    def test_no_workflow_uses_an_action(self):
        out = io.StringIO()
        self.assertEqual(workflows.check(out=out), 0, out.getvalue())

    def test_the_release_workflow_is_read_whole(self):
        release = workflows.REPO / workflows.WORKFLOWS / "release.yml"
        got = [s.name for s in workflows.steps(release.read_text(), "release.yml")]
        self.assertEqual(got[0], "Install the build tools")
        self.assertEqual(got[-1], "Publish")

    def test_the_release_script_is_valid_bash(self):
        release = workflows.REPO / workflows.WORKFLOWS / "release.yml"
        out = workflows.script(workflows.steps(release.read_text(), "release.yml"))
        done = subprocess.run(["bash", "-n"], input=out, text=True, capture_output=True)
        self.assertEqual(done.returncode, 0, done.stderr)


NOTES = (
    "Every link checked twice, and a renamed flag.\n\n"
    "## 💥 Breaking changes\n- BREAKING(cli): `--a` is now `--b`.  ( url )\n\n"
    "## 🚀 Features\n- feat(crawl): x.  ( url )\n"
)


def git(cwd: Path, *args: str) -> None:
    env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t",
           "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}
    subprocess.run(["git", *args], cwd=cwd, env=env, check=True, capture_output=True)


def release_step(name: str) -> workflows.Step:
    release = workflows.REPO / workflows.WORKFLOWS / "release.yml"
    chosen = [s for s in workflows.steps(release.read_text(), "release.yml") if s.name == name]
    assert len(chosen) == 1, name
    return chosen[0]


class TagStep(unittest.TestCase):
    """The release's "Check the tag against the crate" step: the notes are the
    tag's message, so only an annotated tag with text is a release."""

    def run_step(self, tag_args: Sequence[str]) -> tuple[int, str]:
        with tempfile.TemporaryDirectory() as tmp:
            work = Path(tmp)
            src = work / "src"
            src.mkdir()
            git(src, "init", "-q")
            (src / "Cargo.toml").write_text('[package]\nname = "bruce"\nversion = "0.3.0"\n')
            git(src, "add", ".")
            git(src, "commit", "-q", "-m", "release")
            git(src, "tag", *tag_args, "v0.3.0")
            env = {**os.environ, "GITHUB_WORKSPACE": str(work), "GITHUB_REF_NAME": "v0.3.0"}
            done = subprocess.run(["bash"], input=workflows.script([release_step("Check the tag against the crate")]),
                                  text=True, capture_output=True, env=env, cwd=work)
            return done.returncode, done.stdout + done.stderr

    def test_an_annotated_tag_with_notes_passes(self):
        code, out = self.run_step(["-a", "-m", "Notes."])
        self.assertEqual(code, 0, out)

    def test_a_lightweight_tag_is_refused(self):
        code, out = self.run_step([])
        self.assertNotEqual(code, 0)
        self.assertIn("lightweight tag", out)

    def test_an_empty_annotation_is_refused(self):
        code, out = self.run_step(["-a", "-m", "", "--cleanup=verbatim"])
        self.assertNotEqual(code, 0)
        self.assertIn("annotation is empty", out)


class BodyStep(unittest.TestCase):
    """The release's "Write the release body" step, run as `script` prints it
    in a workspace whose `src` is a clone of a repo with a tag per release, the
    release's own annotated with its notes, and a `gh` that answers the
    latest-release call."""

    STEP = "Write the release body"

    def schema(self, n: int) -> str:
        return f"pub const SCHEMA_VERSION: u32 = {n};\n"

    def body(self, released: tuple[str, int] | None, now: int, gh_fails: str | None = None,
             notes: str = NOTES):
        """The step's exit code, its output and the body file, for a repo
        whose latest release is `(tag, schema)` and whose tree is at `now`."""
        with tempfile.TemporaryDirectory() as tmp:
            work = Path(tmp)
            origin = work / "server" / "owner" / "repo"
            src = work / "src"
            (origin / "src" / "store").mkdir(parents=True)
            git(origin, "init", "-q")
            if released:
                (origin / "src" / "store" / "schema.rs").write_text(self.schema(released[1]))
                git(origin, "add", ".")
                git(origin, "commit", "-q", "-m", "release")
                git(origin, "tag", released[0])
            (origin / "src" / "store" / "schema.rs").write_text(self.schema(now))
            git(origin, "add", ".")
            git(origin, "commit", "-q", "--allow-empty", "-m", "tag")
            (work / "notes.md").write_text(notes)
            git(origin, "tag", "-a", "v0.3.0", "-F", str(work / "notes.md"), "--cleanup=whitespace")
            git(work, "clone", "-q", "--branch", "v0.3.0", f"file://{origin}", "src")
            bin_ = work / "bin"
            bin_.mkdir()
            gh = bin_ / "gh"
            if gh_fails:
                answer = f"echo '{gh_fails}' >&2; exit 1"
            elif released:
                answer = f"echo {released[0]}"
            else:
                answer = "echo 'gh: Not Found (HTTP 404)' >&2; exit 1"
            gh.write_text(f"#!/bin/sh\n{answer}\n")
            gh.chmod(0o755)
            env = {**os.environ, "PATH": f"{bin_}:{os.environ['PATH']}", "GITHUB_WORKSPACE": str(work),
                   "GITHUB_REPOSITORY": "owner/repo", "GH_TOKEN": "x", "GITHUB_REF_NAME": "v0.3.0"}
            done = subprocess.run(["bash"], input=workflows.script([release_step(self.STEP)]), text=True,
                                  capture_output=True, env=env, cwd=work)
            path = src / "dist" / "release-body.md"
            return done.returncode, done.stdout + done.stderr, path.read_text() if path.exists() else None

    def test_a_changed_schema_is_a_caution_ahead_of_the_notes_first_section(self):
        code, out, body = self.body(("v0.1.0", 4), 5)
        self.assertEqual(code, 0, out)
        self.assertTrue(body.startswith("Every link checked twice, and a renamed flag.\n\n## ⚠️ CAUTION\n"), body)
        self.assertIn("- **Database schema 5, changed from 4 in v0.1.0.**", body)
        self.assertIn("refuses a database made by v0.1.0", body)
        self.assertLess(body.index("## ⚠️ CAUTION"), body.index("## 💥 Breaking changes"))
        self.assertIn("## 🚀 Features\n- feat(crawl): x.  ( url )", body)

    def test_a_changed_schema_joins_the_notes_own_caution(self):
        notes = NOTES.replace("## 💥", "## ⚠️ CAUTION\n- No stability is promised until v1.0.\n\n## 💥")
        code, out, body = self.body(("v0.1.0", 4), 5, notes=notes)
        self.assertEqual(code, 0, out)
        self.assertEqual(body.count("## ⚠️ CAUTION"), 1)
        self.assertIn("## ⚠️ CAUTION\n- **Database schema 5, changed from 4 in v0.1.0.**", body)
        self.assertIn("- No stability is promised until v1.0.", body)

    def test_notes_with_no_section_take_the_caution_at_their_end(self):
        code, out, body = self.body(("v0.1.0", 4), 5, notes="Only a summary.\n")
        self.assertEqual(code, 0, out)
        self.assertTrue(body.startswith("Only a summary.\n"), body)
        self.assertTrue(body.rstrip().endswith("so start a new database."), body)

    def test_an_unchanged_schema_is_a_closing_line(self):
        code, out, body = self.body(("v0.1.0", 5), 5)
        self.assertEqual(code, 0, out)
        self.assertTrue(body.rstrip().endswith("Database schema 5, as in v0.1.0: a database made by either opens in the other."))
        self.assertNotIn("CAUTION", body)
        self.assertNotIn("changed from", body)

    def test_the_first_release_has_no_earlier_one(self):
        code, out, body = self.body(None, 5)
        self.assertEqual(code, 0, out)
        self.assertTrue(body.rstrip().endswith("Database schema 5. There is no earlier release."))

    def test_a_failed_lookup_that_is_no_404_fails_the_step(self):
        code, out, body = self.body(("v0.1.0", 4), 5, gh_fails="gh: Bad gateway (HTTP 502)")
        self.assertNotEqual(code, 0)
        self.assertIn("HTTP 502", out)
        self.assertIsNone(body)

    def test_the_publish_step_sends_the_body_file_and_nothing_generated(self):
        release = workflows.REPO / workflows.WORKFLOWS / "release.yml"
        publish = workflows.steps(release.read_text(), "release.yml")[-1]
        self.assertIn("-F body=@release-body.md", publish.run)
        self.assertNotIn("generate_release_notes", publish.run)


if __name__ == "__main__":
    unittest.main()
