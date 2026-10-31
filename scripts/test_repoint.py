#!/usr/bin/env python3
"""Unit tests for repoint.py, run with `uv run python -m unittest test_repoint`.

Each cap and each exemption has a case, because a check that misses reports the
tree cleaner than it is and one that cries wolf stops being run. The meter's
cases build a real git repository in a temporary directory: what it counts is
defined by `git diff`, so a fake would test the fake.
"""

from __future__ import annotations

import io
import subprocess
import tempfile
import unittest
from pathlib import Path

import file_limits
import repoint

FILES = {
    "docs/design/decisions.md": "# Decisions\n\n<!-- decision-watermark: D1 -->\n\n## A (`a.rs`)\n- **D1** x\n",
    "docs/design/decisions/a.md": "# A (`a.rs`)\n\n### D1 x\nbody\n",
    "docs/status/STATUS.md": (
        "# Status\n\n<!-- repointed: STAMP -->\n\n## What exists\n\n"
        "| a | b |\n|---|---|\n| x | y |\n\n## Not started\n\n- a thing.\n"
    ),
    "docs/status/history/README.md": "# History\n",
    "docs/design/roadmap.md": "# Roadmap\n",
    "CLAUDE.md": "# C\n",
    "docs/process.md": "# P\n",
    ".claude/skills/x/SKILL.md": "---\nname: x\n---\n",
    "Cargo.toml": '[workspace]\nmembers = ["crates/*"]\n',
    "crates/core/src/lib.rs": "//! lib\nfn f() {}\n",
    "vendor/some-crate/src/lib.rs": "//! vendored\nfn v() {}\n",
}


def git(repo: Path, *args: str) -> str:
    return subprocess.run(
        ["git", "-c", "user.name=t", "-c", "user.email=t@t", *args],
        cwd=repo, check=True, capture_output=True, text=True,
    ).stdout.strip()


def write(repo: Path, rel: str, text: str) -> None:
    path = repo / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def make_repo(tmp: str) -> Path:
    repo = Path(tmp)
    for rel, text in FILES.items():
        if not (repo / rel).exists():  # a test's own manifest stands
            write(repo, rel, text)
    git(repo, "init", "-q", "-b", "main")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "base")
    sha = git(repo, "rev-parse", "--short", "HEAD")
    write(repo, "docs/status/STATUS.md", FILES["docs/status/STATUS.md"].replace("STAMP", sha))
    git(repo, "commit", "-qam", "stamp")
    return repo


class Register(unittest.TestCase):
    INDEX = "# D\n\n<!-- decision-watermark: D4 -->\n\n## S\n- **D3** t\n- **D4** t\n"

    def check(self, topic: str, index: str = INDEX) -> list[str]:
        with tempfile.TemporaryDirectory() as tmp:
            write(Path(tmp), "docs/design/decisions.md", index)
            write(Path(tmp), "docs/design/decisions/s.md", topic)
            return repoint.register_problems(Path(tmp))

    def test_an_entry_over_the_character_cap_is_named(self):
        long = "x" * (repoint.ENTRY_CHARS + 1)
        problems = self.check(f"# S\n### D3 t\n{long}\n### D4 t\nl\n")
        self.assertEqual(len([p for p in problems if "D3" in p and "characters" in p]), 1)
        self.assertFalse([p for p in problems if "D4" in p])

    def test_blank_lines_and_the_lead_do_not_count(self):
        lead = "x" * (repoint.ENTRY_CHARS + 1)
        self.assertEqual(self.check(f"# S\n\n{lead}\n\n### D3 t\nl\n\nl\n### D4 t\nl\n"), [])

    def test_a_line_that_could_wrap_is_named_and_an_unbreakable_one_is_not(self):
        wrap = "w " * repoint.LINE_COLUMNS
        token = "t" * (repoint.LINE_COLUMNS + 40)
        heading = "### D4 " + ("h " * repoint.LINE_COLUMNS).strip()
        index = self.INDEX.replace("- **D4** t", "- **D4** " + ("h " * repoint.LINE_COLUMNS).strip())
        problems = self.check(f"# S\n### D3 t\n{wrap}\n{heading}\n{token}\n", index)
        self.assertEqual(
            [p for p in problems if "could wrap" in p],
            [f"docs/design/decisions/s.md:3 runs past {repoint.LINE_COLUMNS} columns and could wrap"],
        )

    def test_the_index_and_the_topic_files_name_the_same_entries(self):
        index = "# D\n\n<!-- decision-watermark: D5 -->\n\n- **D3** t\n- **D4** t\n"
        problems = self.check("# S\n### D3 t\nl\n### D5 t\nl\n", index)
        self.assertTrue(any("D5" in p and "does not index" in p for p in problems))
        self.assertTrue(any("D4" in p and "no topic file" in p for p in problems))

    def test_a_title_differing_between_index_and_file_is_named(self):
        problems = self.check("# S\n### D3 other\nl\n### D4 t\nl\n")
        self.assertTrue(any("D3" in p and "indexed as" in p for p in problems))

    def test_an_entry_held_twice_and_a_watermark_below_an_entry_are_named(self):
        with tempfile.TemporaryDirectory() as tmp:
            index = "# D\n\n<!-- decision-watermark: D3 -->\n\n- **D3** t\n- **D4** t\n"
            write(Path(tmp), "docs/design/decisions.md", index)
            write(Path(tmp), "docs/design/decisions/s.md", "# S\n### D3 t\nl\n### D4 t\nl\n")
            write(Path(tmp), "docs/design/decisions/u.md", "# U\n### D4 t\nl\n")
            problems = repoint.register_problems(Path(tmp))
        self.assertTrue(any("D4 is held by" in p for p in problems))
        self.assertTrue(any("watermark D3 is below" in p for p in problems))

    def test_an_index_without_a_watermark_is_named(self):
        problems = self.check("# S\n### D3 t\nl\n### D4 t\nl\n", self.INDEX.replace("<!-- decision-watermark: D4 -->", ""))
        self.assertTrue(any("watermark" in p for p in problems))


class History(unittest.TestCase):
    def test_every_dated_entry_is_held_to_the_cap_and_the_readme_is_not(self):
        with tempfile.TemporaryDirectory() as tmp:
            long = "x\n" * (repoint.HISTORY_LINES + 1)
            write(Path(tmp), "docs/status/history/2026-09-14.md", long)
            write(Path(tmp), "docs/status/history/2020-01-01.md", long)
            write(Path(tmp), "docs/status/history/2026-09-15.md", "x\n" * repoint.HISTORY_LINES)
            write(Path(tmp), "docs/status/history/README.md", long)
            problems = repoint.history_problems(Path(tmp))
        self.assertEqual(len(problems), 2)
        self.assertTrue(any("2026-09-14" in p for p in problems))
        self.assertTrue(any("2020-01-01" in p for p in problems))


class Orientation(unittest.TestCase):
    def test_a_skill_and_claude_md_are_capped(self):
        with tempfile.TemporaryDirectory() as tmp:
            write(Path(tmp), ".claude/skills/big/SKILL.md", "x\n" * (repoint.SKILL_LINES + 1))
            write(Path(tmp), ".claude/skills/ok/SKILL.md", "x\n")
            write(Path(tmp), "CLAUDE.md", "x\n" * (repoint.ORIENTATION_CAPS["CLAUDE.md"] + 1))
            problems = repoint.orientation_problems(Path(tmp))
        self.assertEqual(len(problems), 2)
        self.assertTrue(any("big/SKILL.md" in p for p in problems))
        self.assertTrue(any("CLAUDE.md" in p for p in problems))


class Comments(unittest.TestCase):
    def check(self, line: str) -> list[str]:
        with tempfile.TemporaryDirectory() as tmp:
            write(Path(tmp), "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
            write(Path(tmp), "crates/core/src/a.rs", line + "\n")
            return repoint.comment_problems(Path(tmp))

    def test_provenance_in_a_comment_is_named(self):
        for bad in ("// landed in P3.2", "/// see M12", "//! since 2026-09-01"):
            self.assertEqual(len(self.check(bad)), 1, bad)

    def test_handles_dates_as_data_and_code_are_not(self):
        for ok in ("// D12, KD3, I34, S2, UF1", "/// epoch 1970-01-01", 'let s = "P3.2";'):
            self.assertEqual(self.check(ok), [], ok)

    def test_a_root_package_s_comments_are_read_and_vendor_s_are_not(self):
        with tempfile.TemporaryDirectory() as tmp:
            write(Path(tmp), "Cargo.toml", '[package]\nname = "x"\n')
            write(Path(tmp), "src/main.rs", "// landed in P1.2\n")
            write(Path(tmp), "vendor/v/src/lib.rs", "// landed in P1.2\n")
            write(Path(tmp), "target/debug/build/out.rs", "// landed in P1.2\n")
            problems = repoint.comment_problems(Path(tmp))
        self.assertEqual(len(problems), 1)
        self.assertIn("src/main.rs", problems[0])

    def test_no_manifest_is_no_source(self):
        with tempfile.TemporaryDirectory() as tmp:
            write(Path(tmp), "src/main.rs", "// landed in P1.2\n")
            self.assertEqual(repoint.comment_problems(Path(tmp)), [])


class Stamp(unittest.TestCase):
    def test_one_stamp_reads_and_zero_or_two_do_not(self):
        with tempfile.TemporaryDirectory() as tmp:
            write(Path(tmp), "docs/status/STATUS.md", "<!-- repointed: abc1234 -->\n")
            self.assertEqual(repoint.read_stamp(Path(tmp)), "abc1234")
            write(Path(tmp), "docs/status/STATUS.md", "<!-- repointed: none -->\n")
            self.assertEqual(repoint.read_stamp(Path(tmp)), repoint.NO_BASELINE)
            write(Path(tmp), "docs/status/STATUS.md", "none\n")
            self.assertIsNone(repoint.read_stamp(Path(tmp)))
            write(Path(tmp), "docs/status/STATUS.md", "<!-- repointed: abc1234 -->\n<!-- repointed: abc1235 -->\n")
            self.assertIsNone(repoint.read_stamp(Path(tmp)))


class Meter(unittest.TestCase):
    def test_the_fixture_is_green_and_at_zero(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            out = io.StringIO()
            self.assertEqual(repoint.check(repo, out=out), 0)
            self.assertIn("net +0", out.getvalue())
            self.assertIn("repoint: green", out.getvalue())

    def test_live_docs_count_and_centering_and_history_do_not(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            write(repo, "docs/design/roadmap.md", "# Roadmap\n" + "x\n" * 10)
            write(repo, "docs/design/roadmap-P9-thing.md", "x\n" * 100)
            write(repo, "docs/design/roadmap-P9.2-thing-notes.md", "x\n" * 100)
            write(repo, "docs/wip/roadmap-P10-other.md", "x\n" * 100)
            write(repo, "docs/design/historical/initial.md", "x\n" * 100)
            write(repo, "docs/status/history/2026-09-14.md", "x\n" * 50)
            write(repo, "docs/design/new.md", "x\n" * 5)  # untracked
            g = repoint.growth_since(repo, repoint.read_stamp(repo))
        self.assertEqual(g.docs, 15)

    def test_comment_lines_count_in_both_directions_and_code_does_not(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            write(repo, "crates/core/src/lib.rs", "/// a\n/// b\n// c\nfn f() {}\nfn g() {}\n")
            write(repo, "crates/core/src/new.rs", "//! new\nfn h() {}\n")  # untracked
            g = repoint.growth_since(repo, repoint.read_stamp(repo))
        self.assertEqual(g.comments, 3)  # +3 -1 committed, +1 untracked

    def test_a_member_moved_since_the_stamp_is_counted_where_it_stood(self):
        # Git pairs a moved file with its old path only while the two are
        # alike; one it does not pair is a deletion at the stamp's member root
        # and an addition under `CODE_ROOTS`, and the deletion counts too.
        with tempfile.TemporaryDirectory() as tmp:
            write(Path(tmp), "Cargo.toml", '[workspace]\nmembers = ["old", "crates/*"]\n')
            write(Path(tmp), "old/src/lib.rs", "// a\n// b\nfn o() {}\n")
            repo = make_repo(tmp)
            git(repo, "rm", "-q", "old/src/lib.rs")
            write(repo, "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
            write(repo, "crates/old/src/lib.rs", "// x\n// y\nfn n() { let z = 1; }\n")
            g = repoint.growth_since(repo, repoint.read_stamp(repo))
        self.assertEqual(g.comments, 0)

    def test_a_re_vendor_is_not_this_project_s_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            write(repo, "vendor/some-crate/src/lib.rs", "//! a\n// b\n// c\nfn v() {}\n")
            write(repo, "vendor/some-crate/src/new.rs", "//! new\n")  # untracked
            g = repoint.growth_since(repo, repoint.read_stamp(repo))
        self.assertEqual(g.comments, 0)

    def test_an_untracked_binary_is_not_read(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            (repo / "fixtures").mkdir()
            (repo / "fixtures" / "bundle.zip").write_bytes(b"PK\x03\x04\xa2\xff")
            g = repoint.growth_since(repo, repoint.read_stamp(repo))
        self.assertEqual((g.docs, g.comments), (0, 0))

    def test_past_the_budget_is_red(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            write(repo, "docs/design/roadmap.md", "x\n" * 20)
            self.assertEqual(repoint.check(repo, budget=10, out=io.StringIO()), 1)
            self.assertEqual(repoint.check(repo, budget=100, out=io.StringIO()), 0)

    def test_no_baseline_measures_from_the_empty_tree_before_any_commit(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            git(repo, "init", "-q", "-b", "main")
            write(repo, "docs/status/STATUS.md", "# S\n<!-- repointed: none -->\n")
            write(repo, "docs/design/decisions.md", "# D\n<!-- decision-watermark: D0 -->\n")
            write(repo, "docs/status/history/2026-10-17.md", "x\n" * 9)
            out = io.StringIO()
            self.assertEqual(repoint.check(repo, out=out), 0, out.getvalue())
            self.assertIn("since the empty tree: live docs +4,", out.getvalue())
            out = io.StringIO()
            self.assertEqual(repoint.check(repo, budget=3, out=out), 1)
            self.assertIn("!! the live record has grown 4 lines since the empty tree; repoint", out.getvalue())

    def test_no_baseline_counts_committed_and_staged_files_too(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            write(repo, "docs/design/new.md", "x\n" * 5)
            git(repo, "add", "docs/design/new.md")
            g = repoint.growth_since(repo, repoint.empty_tree(repo))
        live = sum(len(t.splitlines()) for rel, t in FILES.items() if repoint.is_live_doc(rel))
        self.assertEqual(g.docs, live + 5)
        self.assertEqual(g.comments, 1)

    def test_the_register_s_growth_is_metered_in_bytes(self):
        topic = FILES["docs/design/decisions/a.md"]
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            grown = topic.replace("\n\n### D1", "\n\n" + "x" * (repoint.REGISTER_GROWTH_BYTES + 1) + "\n\n### D1")
            write(repo, "docs/design/decisions/a.md", grown)
            out = io.StringIO()
            self.assertEqual(repoint.check(repo, out=out), 1)
            self.assertIn("decision register has grown", out.getvalue())
            write(repo, "docs/design/decisions/a.md", topic.replace("\n\n### D1", "\n\n" + "x" * 100 + "\n\n### D1"))
            self.assertEqual(repoint.check(repo, out=io.StringIO()), 0)

    def test_a_stamp_that_is_not_an_ancestor_is_red(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = make_repo(tmp)
            problems = repoint.meter_problems(repo, "0000000", 10, io.StringIO())
        self.assertEqual(len(problems), 1)
        self.assertIn("ancestor", problems[0])


class Source(unittest.TestCase):
    def test_a_package_below_the_root_that_is_no_member_is_no_source(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            (repo / "spikes/s/src").mkdir(parents=True)
            (repo / "spikes/s/Cargo.toml").write_text('[package]\nname = "s"\n')
            (repo / "a").mkdir()
            (repo / "a/Cargo.toml").write_text('[package]\nname = "a"\n')
            self.assertFalse(repoint.is_source("spikes/s/src/lib.rs", (".", "a"), repo))
            self.assertTrue(repoint.is_source("a/src/lib.rs", (".", "a"), repo))
            self.assertTrue(repoint.is_source("src/lib.rs", (".",), repo))


class ThisRepo(unittest.TestCase):
    def test_the_stamp_is_readable_and_the_scope_is_as_documented(self):
        stamp = repoint.read_stamp(repoint.REPO)
        self.assertIsNotNone(stamp)
        self.assertTrue(repoint.is_live_doc("docs/design/decisions.md"))
        self.assertTrue(repoint.is_live_doc(".claude/skills/go/SKILL.md"))
        self.assertFalse(repoint.is_live_doc("docs/status/history/2026-09-14.md"))
        self.assertFalse(repoint.is_live_doc("docs/design/roadmap-P10-some-slug-inbox.md"))
        self.assertFalse(repoint.is_live_doc("docs/wip/roadmap-P1-scaffold.md"))
        self.assertFalse(repoint.is_live_doc("docs/design/historical/initial.md"))
        self.assertFalse(repoint.is_live_doc("vendor/some-crate/README.md"))
        self.assertTrue(repoint.is_source("crates/core/tests/it.rs", ("crates",)))
        self.assertTrue(repoint.is_source("src/main.rs", (".",)))
        self.assertFalse(repoint.is_source("vendor/some-crate/src/lib.rs", (".",)))

    def test_every_workspace_crate_is_counted(self):
        """A crate the meter does not read grows the record unmetered."""
        roots = repoint.code_roots(repoint.REPO)
        for path in file_limits.rust_files(repoint.REPO):
            rel = path.relative_to(repoint.REPO).as_posix()
            self.assertTrue(repoint.is_source(rel, roots, repoint.REPO), rel)
        if not (repoint.REPO / "Cargo.toml").exists():
            self.assertEqual(roots, ())


if __name__ == "__main__":
    unittest.main()
