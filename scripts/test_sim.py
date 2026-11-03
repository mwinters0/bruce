#!/usr/bin/env python3
"""Unit tests for sim.py, run with `uv run python -m unittest test_sim`.

Each test breaks exactly one thing and asserts the failure names it;
`ThisRepo` holds the real guide to the extraction.
"""

from __future__ import annotations

import contextlib
import io
import os
import shutil
import tempfile
import unittest
from pathlib import Path

import sim

GUIDE = """# Guide

Prose.

```toml file=bruce.toml file=bruce-locales.toml
allowlist = ["a.example"]
```

An untagged block is the reader's:

```sh
bruce crawl --config bruce.toml
```

More config, below the first, in the same file:

```toml file=bruce.toml
[[root]]
url = "https://a.example/"
```

```toml file=bruce-locales.toml
[[root]]
url = "https://a.example/all/"
```

````md file=notes/readme.md
```toml file=nested.toml
```
````
"""


class Extraction(unittest.TestCase):
    def test_blocks_of_one_file_join_in_order_and_untagged_ones_are_left(self):
        blocks = sim.tagged_blocks(GUIDE)
        self.assertEqual(list(blocks), ["bruce.toml", "bruce-locales.toml", "notes/readme.md"])
        self.assertEqual(len(blocks["bruce.toml"]), 2)
        self.assertNotIn("bruce crawl", "".join(blocks["bruce.toml"]))

    def test_a_block_naming_two_paths_belongs_to_both(self):
        blocks = sim.tagged_blocks(GUIDE)
        self.assertEqual(blocks["bruce-locales.toml"][0], 'allowlist = ["a.example"]\n')
        self.assertEqual(blocks["bruce.toml"][0], blocks["bruce-locales.toml"][0])
        self.assertEqual(blocks["bruce-locales.toml"][1], '[[root]]\nurl = "https://a.example/all/"\n')

    def test_a_longer_fence_holds_a_shorter_one_as_text(self):
        blocks = sim.tagged_blocks(GUIDE)
        self.assertEqual(
            blocks["notes/readme.md"], ["```toml file=nested.toml\n```\n"]
        )

    def test_an_unclosed_block_names_its_line(self):
        with self.assertRaisesRegex(sim.GuideError, r"guide:3: .*never closed"):
            sim.tagged_blocks("a\nb\n```toml file=x\nbody\n")

    def test_files_are_written_joined_with_a_blank_line(self):
        with tempfile.TemporaryDirectory() as d:
            guide, out = Path(d) / "g.md", Path(d) / "sim"
            guide.write_text(GUIDE, encoding="utf-8")
            out.mkdir()
            (out / "crawl.log").write_text("kept", encoding="utf-8")
            sim.extract(guide, out)
            self.assertEqual(
                (out / "bruce.toml").read_text(),
                'allowlist = ["a.example"]\n\n[[root]]\nurl = "https://a.example/"\n',
            )
            self.assertTrue((out / "notes" / "readme.md").exists())
            self.assertEqual((out / "crawl.log").read_text(), "kept")

    def test_a_path_leaving_sim_is_refused(self):
        for bad in ("../x", "/etc/x", "a/../../x"):
            with self.subTest(bad), tempfile.TemporaryDirectory() as d:
                guide = Path(d) / "g.md"
                guide.write_text(
                    f"```toml file=bruce.toml\n```\n```toml file=bruce-locales.toml\n```\n```sh file={bad}\nx\n```\n",
                    encoding="utf-8",
                )
                with self.assertRaisesRegex(sim.GuideError, "stays inside sim/"):
                    sim.extract(guide, Path(d) / "sim")
                self.assertFalse((Path(d) / "x").exists())

    def test_a_guide_without_tags_or_without_the_config_fails(self):
        for text, want in (
            ("```toml\nx = 1\n```\n", "no code block is tagged"),
            ("```toml file=other.toml\nx = 1\n```\n", "file=bruce.toml"),
            ("```toml file=bruce.toml\nx = 1\n```\n", "file=bruce-locales.toml"),
        ):
            with self.subTest(want), tempfile.TemporaryDirectory() as d:
                guide = Path(d) / "g.md"
                guide.write_text(text, encoding="utf-8")
                with self.assertRaisesRegex(sim.GuideError, want):
                    sim.extract(guide, Path(d) / "sim")

    def test_a_missing_guide_says_where(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaisesRegex(sim.GuideError, "absent.md"):
                sim.extract(Path(d) / "absent.md", Path(d) / "sim")

    def test_run_reports_a_bad_guide_without_loading(self):
        with tempfile.TemporaryDirectory() as d:
            err = io.StringIO()
            with contextlib.redirect_stderr(err):
                code = sim.run(Path(d) / "absent.md", Path(d) / "sim", load=True)
            self.assertEqual(code, 1)
            self.assertIn("sim:", err.getvalue())


SERVICE = """[Service]
ExecStart=/usr/local/libexec/job.sh
EnvironmentFile=/etc/job/%i.env
"""
TIMER = """[Timer]
OnCalendar=daily
Unit=job@nightly.service
"""


class Wiring(unittest.TestCase):
    def out(self, files: dict[str, str]) -> Path:
        d = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, d, True)
        for name, text in files.items():
            (d / name).parent.mkdir(parents=True, exist_ok=True)
            (d / name).write_text(text)
        return d

    def test_a_timer_starting_a_template_with_its_environment_and_script_is_wired(self):
        out = self.out({
            "systemd/job@.service": SERVICE, "systemd/job.timer": TIMER,
            "job.sh": "#!/bin/sh\n", "etc/job/nightly.env": "A=1\n",
        })
        self.assertEqual(sim.check_units(out), [])

    def test_each_loose_end_is_named(self):
        for files, want in (
            ({"systemd/job@.service": SERVICE, "systemd/job.timer": TIMER, "etc/job/nightly.env": ""}, "no script"),
            ({"systemd/job@.service": SERVICE, "systemd/job.timer": TIMER, "job.sh": ""}, "EnvironmentFile"),
            ({"systemd/job.timer": TIMER}, "tags no job@.service"),
            ({"systemd/job.timer": TIMER.replace("job@nightly", "job")}, "no instance"),
            ({"systemd/job.timer": TIMER.replace("OnCalendar=daily\n", "")}, "no OnCalendar"),
        ):
            with self.subTest(want):
                problems = sim.check_units(self.out(files))
                self.assertTrue(any(want in p for p in problems), problems)

    def test_a_script_that_does_not_parse_is_named(self):
        out = self.out({"good.sh": "echo ok\n", "bad.sh": "if then\n", "x.toml": "if then\n"})
        problems = sim.check_scripts([out / "good.sh", out / "bad.sh", out / "x.toml"])
        self.assertEqual(len(problems), 1)
        self.assertIn("bad.sh", problems[0])

    def test_a_script_is_written_executable_and_a_config_is_not(self):
        with tempfile.TemporaryDirectory() as d:
            guide = Path(d) / "g.md"
            guide.write_text(
                "```toml file=bruce.toml\n```\n```toml file=bruce-locales.toml\n```\n"
                "```bash file=job.sh\n#!/usr/bin/env bash\necho hi\n```\n",
                encoding="utf-8",
            )
            sim.extract(guide, Path(d) / "sim")
            self.assertTrue(os.access(Path(d) / "sim" / "job.sh", os.X_OK))
            self.assertFalse(os.access(Path(d) / "sim" / "bruce.toml", os.X_OK))


WORKFLOW = """name: x
on:
  pull_request:
jobs:
  j:
    runs-on: ubuntu-latest
    steps:
      - name: One
        run: |
          echo one
"""


class Workflow(unittest.TestCase):
    def out(self, text: str | None) -> Path:
        d = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, d, True)
        if text is not None:
            (d / sim.WORKFLOW).parent.mkdir(parents=True)
            (d / sim.WORKFLOW).write_text(text)
        return d

    def test_a_workflow_of_run_steps_is_accepted(self):
        self.assertEqual(sim.check_workflow(self.out(WORKFLOW)), [])

    def test_each_way_it_is_not_one_this_repository_can_run_is_named(self):
        for text, want in (
            (None, "tags no"),
            (WORKFLOW.replace("      - name: One\n        run: |\n          echo one\n", "      - uses: x/y@v1\n"), "`uses:`"),
            (WORKFLOW.replace("run: |\n          echo one\n", "shell: bash\n"), "no `run:`"),
            (WORKFLOW.split("steps:")[0], "no steps"),
        ):
            with self.subTest(want):
                problems = sim.check_workflow(self.out(text))
                self.assertTrue(any(want in p for p in problems), problems)


VIEW = """{% set bruce_model = 2 %}
{% extends "bruce:digest.html" %}
{% block palette %}:root { --bg: red; }
{% endblock %}
"""


class View(unittest.TestCase):
    def out(self, text: str | None) -> Path:
        d = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, d, True)
        if text is not None:
            (d / sim.VIEW).write_text(text)
        return d

    def test_a_view_extending_the_built_in_one_with_its_version_is_accepted(self):
        self.assertEqual(sim.check_view(self.out(VIEW)), [])

    def test_each_way_it_is_not_a_view_bruce_takes_is_named(self):
        for text, want in (
            (None, "tags no"),
            (VIEW.replace("{% extends \"bruce:digest.html\" %}\n", ""), "does not extend"),
            (VIEW.replace("{% set bruce_model = 2 %}\n", ""), "no bruce_model"),
        ):
            with self.subTest(want):
                problems = sim.check_view(self.out(text))
                self.assertTrue(any(want in p for p in problems), problems)


class ThisRepo(unittest.TestCase):
    def test_the_guide_extracts_both_configs_its_scripts_and_units(self):
        with tempfile.TemporaryDirectory() as d:
            written = sim.extract(sim.REPO / sim.GUIDE, Path(d))
            for config in sim.CONFIGS:
                self.assertIn(Path(d) / config, written)
            self.assertEqual(sim.check_scripts(written), [])
            self.assertEqual(sim.check_units(Path(d)), [])
            self.assertEqual(sim.check_workflow(Path(d)), [])
            self.assertEqual(sim.check_view(Path(d)), [])
            self.assertTrue((Path(d) / "bruce-nightly.sh").exists())


if __name__ == "__main__":
    unittest.main()
