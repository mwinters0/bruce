#!/usr/bin/env python3
"""Unit tests for e2e.py, run with `uv run python -m unittest test_e2e`.

None runs Antora or touches the network; `Committed` holds the saved pages
and the content to the record the build wrote of them.
"""

from __future__ import annotations

import json
import tempfile
import unittest
import urllib.error
import urllib.request
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

    def test_each_has_the_hosted_origin_antora_makes_an_edit_link_from(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            repos = e2e.materialise(e2e.FIXTURES / "content", Path(d) / "repos")
            for name, path in repos.items():
                self.assertEqual(e2e.run(["git", "remote", "get-url", "origin"], path).strip(), e2e.ORIGIN.format(name))


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


class Plant(unittest.TestCase):
    def planted(self, d: str) -> Path:
        tools = e2e.materialise(e2e.FIXTURES / "content", Path(d) / "repos")["tools"]
        e2e.plant(tools)
        return tools

    def test_it_deletes_the_page_and_the_image_and_renames_the_anchor(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            tools = self.planted(d)
            self.assertFalse((tools / "modules/ROOT/pages/reference.adoc").exists())
            self.assertFalse((tools / "modules/ROOT/images/diagram.svg").exists())
            usage = (tools / "modules/ROOT/pages/usage.adoc").read_text()
            self.assertIn("[#run]", usage)
            self.assertNotIn("[#running]", usage)

    def test_it_edits_the_working_tree_without_committing(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            self.assertNotEqual(e2e.run(["git", "status", "--porcelain"], self.planted(d)), "")

    def test_a_plant_over_text_the_fixture_lacks_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / "a.adoc"
            f.write_text("x")
            with self.assertRaisesRegex(e2e.E2eError, "no 'y' to plant over"):
                e2e.replace(f, "y", "z")

    def test_the_fixture_embeds_the_image_the_plant_deletes_from_another_component(self) -> None:
        guide = (e2e.FIXTURES / "content/guide/modules/ROOT/pages/upgrade.adoc").read_text()
        self.assertIn("image::tools:ROOT:diagram.svg", guide)
        ops = (e2e.FIXTURES / "content/ops/modules/ROOT/pages/index.adoc").read_text()
        self.assertIn("xref:tools:ROOT:reference.adoc", ops)


class Server(unittest.TestCase):
    def test_it_serves_the_directory_under_the_site_path_only(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            site = Path(d)
            (site / "a").mkdir()
            (site / "a" / "index.html").write_text("<p>a</p>")
            (site / "top.html").write_text("top")
            server = e2e.serve(site, 0)
            base = f"http://127.0.0.1:{server.server_address[1]}"
            try:
                get = lambda path: urllib.request.urlopen(base + path).read()
                self.assertEqual(get(e2e.SITE_PATH + "/a/"), b"<p>a</p>")
                self.assertEqual(get(e2e.SITE_PATH + "/top.html"), b"top")
                for path in ("/top.html", "/robots.txt", e2e.SITE_PATH + "/missing/"):
                    with self.assertRaises(urllib.error.HTTPError) as err:
                        get(path)
                    self.assertEqual(err.exception.code, 404, path)
            finally:
                server.shutdown()
                server.server_close()

    def test_the_configuration_names_the_site_as_its_one_root_and_allowlists_nothing(self) -> None:
        cfg = e2e.bruce_config(8099)
        self.assertIn('url = "http://127.0.0.1:8099/en-US/"', cfg)
        self.assertNotIn("allowlist", cfg)


class Verify(unittest.TestCase):
    """`verify` accepts the documents the planted changes should give and
    names each way a section differs."""

    P = "http://127.0.0.1:8099/en-US/"

    def doc(self, **sections: object) -> dict:
        doc = {
            "run": 2, "baseline_run": 1, "introduced": [], "inverse": [], "redirected": [], "errors": [],
            "already_broken": {"count": 0, "links": None},
        }
        doc.update(sections)
        return doc

    def introduced(self, row: tuple) -> dict:
        page, href, kind, outcome, reason, baseline = row
        return {"page": self.P + page, "raw_href": href, "kind": kind, "outcome": outcome, "reason": reason, "baseline": baseline}

    def inverse(self, row: tuple) -> dict:
        page, href, kind, removed, component = row
        return {"page": self.P + page, "raw_href": href, "kind": kind, "removed": removed, "component": component}

    def redirect(self, row: tuple) -> dict:
        page, href, hop = row
        return {"page": self.P + page, "raw_href": href, "chain": f"{hop} {self.P}"}

    def good(self) -> tuple:
        unchanged = (
            0,
            self.doc(
                redirected=[self.redirect(r) for r in e2e.UNCHANGED_REDIRECTED],
                already_broken={"count": 9, "links": None},
            ),
            "",
        )
        changed = (
            1,
            self.doc(
                run=3,
                introduced=[self.introduced(r) for r in e2e.PLANTED_INTRODUCED],
                inverse=[self.inverse(r) for r in e2e.PLANTED_INVERSE],
                redirected=[self.redirect(r) for r in e2e.PLANTED_REDIRECTED],
                already_broken={"count": 6, "links": None},
            ),
            "",
        )
        image = self.doc(inverse=[self.inverse(r) for r in e2e.IMAGE_ONLY_INVERSE])
        return unchanged, changed, ((1, image, ""), (0, image, ""))

    def test_the_documents_the_plant_should_give_are_accepted(self) -> None:
        unchanged, changed, image = self.good()
        reports = (changed[1]["introduced"], changed[1]["inverse"])
        self.assertEqual(e2e.verify(unchanged, changed, 8099, image, reports), [])

    def test_an_unchanged_build_that_introduces_something_is_named(self) -> None:
        unchanged, changed, image = self.good()
        unchanged[1]["introduced"] = [self.introduced(next(iter(e2e.PLANTED_INTRODUCED)))]
        problems = e2e.verify(unchanged, changed, 8099, image)
        self.assertEqual(len(problems), 1)
        self.assertIn("unchanged introduced", problems[0])

    def test_a_missing_inverse_row_and_a_wrong_exit_status_are_named(self) -> None:
        unchanged, changed, image = self.good()
        changed[1]["inverse"].pop()
        problems = e2e.verify(unchanged, (0, changed[1], ""), 8099, image)
        self.assertEqual(sorted(p.split(":")[0] for p in problems), ["planted exit status", "planted inverse", "planted inverse count"])

    def test_a_warned_inverse_that_still_fails_is_named(self) -> None:
        unchanged, changed, image = self.good()
        problems = e2e.verify(unchanged, changed, 8099, (image[0], (1, image[1][1], "")))
        self.assertEqual([p.split(":")[0] for p in problems], ["image only, --inverse=warn, exit status"])

    def test_a_run_that_printed_no_json_is_named_with_its_stderr(self) -> None:
        unchanged, changed, _ = self.good()
        problems = e2e.verify((2, None, "bad config\n"), changed, 8099)
        self.assertTrue(any("bad config" in p for p in problems))


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
