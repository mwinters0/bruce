#!/usr/bin/env python3
"""Unit tests for e2e.py, run with `uv run python -m unittest test_e2e`.

None runs Antora or touches the network; `Committed` holds the saved pages
and the content to the record the build wrote of them.
"""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import e2e


class Ui(unittest.TestCase):
    def test_a_bundle_matching_its_record_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            (d / "ui-bundle.zip").write_bytes(b"PKx")
            (d / "ui-bundle.json").write_text(json.dumps({"sha256": e2e.sha256(d / "ui-bundle.zip")}))
            self.assertEqual(e2e.check_ui(d), d / "ui-bundle.zip")

    def test_a_changed_bundle_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            (d / "ui-bundle.zip").write_bytes(b"PKx")
            (d / "ui-bundle.json").write_text(json.dumps({"sha256": "0" * 64}))
            with self.assertRaisesRegex(e2e.E2eError, "does not hash"):
                e2e.check_ui(d)

    def test_a_missing_bundle_says_how_to_vendor_it(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaisesRegex(e2e.E2eError, "fixtures.py antora-ui"):
                e2e.check_ui(Path(d))


class Playbook(unittest.TestCase):
    def test_it_names_each_repo_the_ui_and_the_indexify_style(self) -> None:
        pb = json.loads(e2e.playbook([Path("/r/a"), Path("/r/b")], "a::index.adoc", Path("/u.zip"), Path("/o"), Path("/c")))
        self.assertEqual([s["url"] for s in pb["content"]["sources"]], ["/r/a", "/r/b"])
        self.assertEqual({s["branches"] for s in pb["content"]["sources"]}, {"HEAD"})
        self.assertEqual(pb["ui"]["bundle"]["url"], "/u.zip")
        self.assertEqual(pb["urls"]["html_extension_style"], "indexify")
        self.assertEqual(pb["site"]["start_page"], "a::index.adoc")

    def test_it_sets_no_site_url_so_a_partial_build_has_none(self) -> None:
        pb = json.loads(e2e.playbook([Path("/r/a")], "a::index.adoc", Path("/u.zip"), Path("/o"), Path("/c")))
        self.assertNotIn("url", pb["site"])


class Materialise(unittest.TestCase):
    def test_each_component_becomes_a_committed_repository(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            repos = e2e.materialise(e2e.FIXTURES / "content", Path(d) / "repos")
            self.assertEqual(set(repos), set(e2e.COMPONENTS))
            for path in repos.values():
                self.assertTrue((path / ".git").is_dir())
                self.assertTrue((path / "antora.yml").is_file())
                self.assertEqual(e2e.run(["git", "status", "--porcelain"], path), "")


class Content(unittest.TestCase):
    """The hand-written site holds what the spec says it does."""

    def antora(self, name: str) -> str:
        return (e2e.FIXTURES / "content" / name / "antora.yml").read_text()

    def test_one_component_is_versioned_one_not_and_one_has_two_modules(self) -> None:
        self.assertIn("version: '1.0'", self.antora("guide"))
        self.assertIn("version: ~", self.antora("tools"))
        ops = e2e.FIXTURES / "content" / "ops" / "modules"
        self.assertEqual(sorted(p.name for p in ops.iterdir()), ["ROOT", "admin"])

    def test_a_page_alias_and_cross_component_xrefs_exist(self) -> None:
        pages = "\n".join(p.read_text() for p in (e2e.FIXTURES / "content").rglob("*.adoc"))
        self.assertIn(":page-aliases:", pages)
        for target in ("xref:tools:", "xref:guide:", "xref:ops:"):
            self.assertIn(target, pages)


class Committed(unittest.TestCase):
    def test_every_saved_page_hashes_to_the_manifest(self) -> None:
        manifest = e2e.PAGES / "manifest.json"
        if not manifest.exists():
            self.skipTest("no saved pages yet")
        doc = json.loads(manifest.read_text())
        self.assertEqual(doc["antora"], e2e.ANTORA)
        self.assertEqual(
            sorted(e["path"] for e in doc["pages"]),
            sorted(f"{b}/{p}" for b, ps in e2e.SAVED.items() for p in ps),
        )
        for e in doc["pages"]:
            f = e2e.PAGES / e["path"]
            self.assertEqual(e2e.sha256(f), e["sha256"], e["path"])
            self.assertEqual(f.stat().st_size, e["bytes"], e["path"])

    def test_the_pages_were_built_with_the_vendored_bundle(self) -> None:
        manifest = e2e.PAGES / "manifest.json"
        if not manifest.exists():
            self.skipTest("no saved pages yet")
        self.assertEqual(json.loads(manifest.read_text())["ui_sha256"], e2e.sha256(e2e.FIXTURES / "ui-bundle.zip"))


if __name__ == "__main__":
    unittest.main()
