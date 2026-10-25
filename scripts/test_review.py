#!/usr/bin/env python3
"""Unit tests for review.py, run with `uv run python -m unittest test_review`.

Nothing is built, crawled or served here: the directories a crawl leaves are
written by hand, and `mise run crawl` is the check that one launches.
"""

from __future__ import annotations

import os
import shlex
import subprocess
import tempfile
import unittest
from pathlib import Path

import review


def finished(runs: Path, name: str, ended: str, code: int = 0, db: bool = True) -> Path:
    run = runs / name
    run.mkdir(parents=True)
    (run / "started").write_text("x\n", encoding="utf-8")
    (run / "exit").write_text(f"{code}\n", encoding="utf-8")
    (run / "ended").write_text(f"{ended}\n", encoding="utf-8")
    if db:
        (run / review.DB).write_bytes(b"")
    return run


class State(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.runs = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_an_exit_of_0_is_finished_and_another_failed(self):
        self.assertEqual(review.state(finished(self.runs, "a", "t")), review.State("finished", 0))
        self.assertEqual(review.state(finished(self.runs, "b", "t", 143)), review.State("failed", 143))

    def test_no_started_is_unlaunched(self):
        run = self.runs / "a"
        run.mkdir()
        self.assertEqual(review.state(run).name, "unlaunched")

    def test_a_live_group_is_running_and_a_gone_one_vanished(self):
        run = self.runs / "a"
        run.mkdir()
        (run / "started").write_text("x\n", encoding="utf-8")
        (run / "pgid").write_text(f"{os.getpgrp()}\n", encoding="utf-8")
        self.assertEqual(review.state(run).name, "running")
        proc = subprocess.Popen(["true"], start_new_session=True)
        proc.wait()
        (run / "pgid").write_text(f"{proc.pid}\n", encoding="utf-8")
        self.assertEqual(review.state(run).name, "vanished")

    def test_no_pgid_is_vanished(self):
        run = self.runs / "a"
        run.mkdir()
        (run / "started").write_text("x\n", encoding="utf-8")
        self.assertEqual(review.state(run).name, "vanished")


class Baseline(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.runs = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_the_newest_finished_crawl_by_its_end_is_copied(self):
        finished(self.runs, "p9-en-us", "2026-10-23T21:14:50+00:00")
        newest = finished(self.runs, "a-en-us", "2026-10-24T14:33:49+00:00")
        self.assertEqual(review.baseline(self.runs), newest / review.DB)

    def test_a_failed_crawl_or_one_with_no_database_is_passed_over(self):
        kept = finished(self.runs, "a", "2026-10-23T00:00:00+00:00")
        finished(self.runs, "b", "2026-10-24T00:00:00+00:00", code=1)
        finished(self.runs, "c", "2026-10-25T00:00:00+00:00", db=False)
        self.assertEqual(review.baseline(self.runs), kept / review.DB)

    def test_no_finished_crawl_is_none(self):
        self.assertIsNone(review.baseline(self.runs))


class Newest(unittest.TestCase):
    def test_the_latest_review_directory_by_its_name(self):
        with tempfile.TemporaryDirectory() as d:
            runs = Path(d)
            for name in ("en-us-20261025-0900", "en-us-20261026-0800", "p8.14-en-us-20261027-1322"):
                (runs / name).mkdir()
            self.assertEqual(review.newest(runs), runs / "en-us-20261026-0800")

    def test_none_says_to_crawl(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaisesRegex(review.ReviewError, "mise run crawl"):
                review.newest(Path(d))


class CrawlScript(unittest.TestCase):
    def test_the_crawl_is_timed_and_its_exit_written_before_its_end(self):
        line = review.crawl_script(Path("/r/en us"), Path("/b/bruce"))
        words = shlex.split(line)
        self.assertIn("/r/en us/en-us.duckdb", words)
        self.assertIn("--config", words)
        self.assertEqual(words[words.index("--config") + 1], "bruce.toml")
        self.assertLess(line.index("/usr/bin/time"), line.index("echo $?"))
        self.assertLess(line.index("echo $?"), line.rindex("ended"))

    def test_the_line_runs_and_writes_each_file(self):
        with tempfile.TemporaryDirectory() as d:
            run = Path(d)
            fake = run / "bruce"
            fake.write_text("#!/bin/sh\necho run 2\nexit 3\n", encoding="utf-8")
            fake.chmod(0o755)
            subprocess.run(["sh", "-c", review.crawl_script(run, fake)], cwd=run, check=True)
            self.assertEqual(review.state(run), review.State("failed", 3))
            self.assertIn("run 2", (run / "crawl.log").read_text(encoding="utf-8"))
            self.assertIn("Exit status: 3", (run / "time.log").read_text(encoding="utf-8"))
            self.assertTrue((run / "ended").exists())


class Digest(unittest.TestCase):
    def test_an_unfinished_crawl_is_refused_with_its_state(self):
        with tempfile.TemporaryDirectory() as d:
            run = finished(Path(d), "en-us-20261025-0900", "t", code=143)
            with self.assertRaisesRegex(review.ReviewError, "failed, exit 143"):
                review.digest(run)


if __name__ == "__main__":
    unittest.main()
