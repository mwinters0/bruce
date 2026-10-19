#!/usr/bin/env python3
"""Unit tests for sim.py, run with `uv run python -m unittest test_sim`.

Each test breaks exactly one thing and asserts the failure names it;
`ThisRepo` holds the real guide to the extraction.
"""

from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

import sim

GUIDE = """# Guide

Prose.

```toml file=bruce.toml
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

````md file=notes/readme.md
```toml file=nested.toml
```
````
"""


class Extraction(unittest.TestCase):
    def test_blocks_of_one_file_join_in_order_and_untagged_ones_are_left(self):
        blocks = sim.tagged_blocks(GUIDE)
        self.assertEqual(list(blocks), ["bruce.toml", "notes/readme.md"])
        self.assertEqual(len(blocks["bruce.toml"]), 2)
        self.assertNotIn("bruce crawl", "".join(blocks["bruce.toml"]))

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
                    f"```toml file=bruce.toml\n```\n```sh file={bad}\nx\n```\n",
                    encoding="utf-8",
                )
                with self.assertRaisesRegex(sim.GuideError, "stays inside sim/"):
                    sim.extract(guide, Path(d) / "sim")
                self.assertFalse((Path(d) / "x").exists())

    def test_a_guide_without_tags_or_without_the_config_fails(self):
        for text, want in (
            ("```toml\nx = 1\n```\n", "no code block is tagged"),
            ("```toml file=other.toml\nx = 1\n```\n", "file=bruce.toml"),
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


class ThisRepo(unittest.TestCase):
    def test_the_guide_extracts_a_config(self):
        with tempfile.TemporaryDirectory() as d:
            written = sim.extract(sim.REPO / sim.GUIDE, Path(d))
            self.assertIn(Path(d) / sim.CONFIG, written)


if __name__ == "__main__":
    unittest.main()
