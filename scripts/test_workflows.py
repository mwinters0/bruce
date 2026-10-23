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


class SchemaStep(unittest.TestCase):
    """The release's "Name the schema change" step, run as `script` prints it
    in a workspace whose `src` is a clone of a repo with a tag per release, and
    a `gh` that answers the latest-release call."""

    STEP = "Name the schema change"

    def schema(self, n: int) -> str:
        return f"pub const SCHEMA_VERSION: u32 = {n};\n"

    def git(self, cwd: Path, *args: str) -> None:
        env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t",
               "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}
        subprocess.run(["git", *args], cwd=cwd, env=env, check=True, capture_output=True)

    def body(self, released: tuple[str, int] | None, now: int, gh_fails: str | None = None):
        """The step's exit code, its output and the body file, for a repo
        whose latest release is `(tag, schema)` and whose tree is at `now`."""
        release = workflows.REPO / workflows.WORKFLOWS / "release.yml"
        chosen = [s for s in workflows.steps(release.read_text(), "release.yml") if s.name == self.STEP]
        self.assertEqual(len(chosen), 1)
        with tempfile.TemporaryDirectory() as tmp:
            work = Path(tmp)
            origin = work / "server" / "owner" / "repo"
            src = work / "src"
            (origin / "src" / "store").mkdir(parents=True)
            self.git(origin, "init", "-q")
            if released:
                (origin / "src" / "store" / "schema.rs").write_text(self.schema(released[1]))
                self.git(origin, "add", ".")
                self.git(origin, "commit", "-q", "-m", "release")
                self.git(origin, "tag", released[0])
            (origin / "src" / "store" / "schema.rs").write_text(self.schema(now))
            self.git(origin, "add", ".")
            self.git(origin, "commit", "-q", "--allow-empty", "-m", "tag")
            self.git(work, "clone", "-q", f"file://{origin}", "src")
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
                   "GITHUB_REPOSITORY": "owner/repo", "GH_TOKEN": "x"}
            done = subprocess.run(["bash"], input=workflows.script(chosen), text=True,
                                  capture_output=True, env=env, cwd=work)
            path = src / "dist" / "release-body.md"
            return done.returncode, done.stdout + done.stderr, path.read_text() if path.exists() else None

    def test_a_changed_schema_is_named_with_both_numbers(self):
        code, out, body = self.body(("v0.1.0", 4), 5)
        self.assertEqual(code, 0, out)
        self.assertIn("**Database schema 5, changed from 4 in v0.1.0.**", body)
        self.assertIn("refuses a database made by v0.1.0", body)

    def test_an_unchanged_schema_says_so(self):
        code, out, body = self.body(("v0.1.0", 5), 5)
        self.assertEqual(code, 0, out)
        self.assertIn("Database schema 5, as in v0.1.0", body)
        self.assertNotIn("changed from", body)

    def test_the_first_release_has_no_earlier_one(self):
        code, out, body = self.body(None, 5)
        self.assertEqual(code, 0, out)
        self.assertIn("Database schema 5. There is no earlier release.", body)

    def test_a_failed_lookup_that_is_no_404_fails_the_step(self):
        code, out, body = self.body(("v0.1.0", 4), 5, gh_fails="gh: Bad gateway (HTTP 502)")
        self.assertNotEqual(code, 0)
        self.assertIn("HTTP 502", out)
        self.assertIsNone(body)

    def test_the_publish_step_sends_the_body_file(self):
        release = workflows.REPO / workflows.WORKFLOWS / "release.yml"
        publish = workflows.steps(release.read_text(), "release.yml")[-1]
        self.assertIn("-F body=@release-body.md", publish.run)


if __name__ == "__main__":
    unittest.main()
