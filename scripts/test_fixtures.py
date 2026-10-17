#!/usr/bin/env python3
"""Unit tests for fixtures.py, run with `uv run python -m unittest test_fixtures`.

None touches the network: `fetch` is the only function that does, and the
rest are pure functions of the strings and dicts given them.
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

import fixtures

PAGE = """<html><head>
<meta name="generator" content="Antora 3.1.15">
<link rel="canonical" href="https://docs.fedoraproject.org/en-US/fedora/latest/x/">
</head><body><div class="nav-container" data-component="fedora" data-version="f39">
<div class="tabs is-loose"></div><div class="tabs">
<a href="#a::b.adoc" class="xref unresolved">b</a>
<h2 id="one">One</h2><a id="two"></a>
<a href="https://forge.fedoraproject.org/x/src/branch/rawhide/a.adoc" title="Edit this Page">e</a>
</body></html>"""


def page(url: str, **over) -> dict:
    base = {
        "url": url, "status": 200, "bytes": 10, "generator": "Antora 3.1.15",
        "canonical": None, "component": "c", "version": None, "ids": 1,
        "tabs": 0, "unresolved": 0, "edit_titles": [],
    }
    return base | over


class Measure(unittest.TestCase):
    def test_reads_the_features_the_selection_uses(self) -> None:
        m = fixtures.measure("https://docs.fedoraproject.org/en-US/x/", 200, PAGE.encode())
        self.assertEqual(m["generator"], "Antora 3.1.15")
        self.assertEqual(m["canonical"], "https://docs.fedoraproject.org/en-US/fedora/latest/x/")
        self.assertEqual((m["component"], m["version"]), ("fedora", "f39"))
        self.assertEqual((m["tabs"], m["unresolved"], m["ids"]), (2, 1, 2))
        self.assertEqual(m["edit_titles"], ["Edit this Page"])

    def test_tabs_are_counted_by_class_token_or_set_id(self) -> None:
        # Antora's own markup puts the id first, which a `<div class="tabs`
        # prefix match never sees.
        page = (fixtures.ROOT / "fixtures/generators/antora/tabs.html").read_text()
        self.assertEqual(fixtures.tab_sets(page), 1)
        self.assertEqual(fixtures.tab_sets('<div id="_tabs_2" class="x">'), 1)
        self.assertEqual(fixtures.tab_sets('<p class="tabsy tab"><li id="_tabs_1_tab_a">'), 0)

    def test_a_page_with_no_features_measures_empty(self) -> None:
        m = fixtures.measure("u", 404, b"<html></html>")
        self.assertIsNone(m["generator"])
        self.assertEqual((m["tabs"], m["ids"], m["edit_titles"]), (0, 0, []))


class Sampling(unittest.TestCase):
    def test_short_lists_are_kept_whole(self) -> None:
        self.assertEqual(fixtures.sample(["a", "b"], 5), ["a", "b"])

    def test_long_lists_are_spaced_and_start_at_the_first(self) -> None:
        got = fixtures.sample([str(i) for i in range(100)], 4)
        self.assertEqual(got, ["0", "25", "50", "75"])

    def test_locs(self) -> None:
        self.assertEqual(fixtures.locs("<url><loc>a</loc></url><loc>b</loc>"), ["a", "b"])

    def test_robots_disallowed_paths(self) -> None:
        d = fixtures.DISALLOWED
        self.assertTrue(d.match("/en-US/Fedora_Core/x/"))
        self.assertTrue(d.match("/en-US/Community_Services_Infrastructure/x"))
        self.assertFalse(d.match("/en-US/fedora/latest/"))


class Selecting(unittest.TestCase):
    def test_each_rule_takes_its_page_and_says_why(self) -> None:
        h = "https://docs.fedoraproject.org/en-US/"
        pages = [
            page(h + "big/", bytes=999, component="big"),
            page(h + "tabbed/", tabs=4, component="tabbed"),
            page(h + "xref/", unresolved=2, component="xref"),
            page(h + "pkg/", component="pkg", version="master"),
            page(h + "fedora/f39/x/", component="fedora", version="f39",
                 canonical=h + "fedora/latest/x/"),
            page(h + "fedora/latest/", component="fedora", version="f44"),
            page(h + "bare/", component=None),
            page(h + "gone/", status=404),
            page(h + "plain/", generator=None),
        ]
        got = {e["url"]: e["why"] for e in fixtures.select(pages)}
        self.assertTrue(any("largest" in w for w in got[h + "big/"]))
        self.assertTrue(any("tabs" in w for w in got[h + "tabbed/"]))
        self.assertTrue(any("unresolved" in w for w in got[h + "xref/"]))
        self.assertTrue(any("unversioned" in w for w in got[h + "pkg/"]))
        self.assertTrue(any("frozen" in w for w in got[h + "fedora/f39/x/"]))
        self.assertTrue(any("latest" in w for w in got[h + "fedora/latest/"]))
        self.assertTrue(any("no component" in w for w in got[h + "bare/"]))
        self.assertNotIn(h + "gone/", got)
        self.assertNotIn(h + "plain/", got)

    def test_the_invariants_witnesses_are_always_taken(self) -> None:
        url = next(iter(fixtures.WITNESSES))
        got = {e["url"] for e in fixtures.select([page(url), page("https://h/x/")])}
        self.assertIn(url, got)

    def test_the_set_never_exceeds_the_target(self) -> None:
        pages = [page(f"https://h/{i}/", component=f"c{i}", bytes=i) for i in range(100)]
        self.assertLessEqual(len(fixtures.select(pages, 30)), 30)


class Paths(unittest.TestCase):
    def test_a_page_is_stored_by_its_url_path(self) -> None:
        self.assertEqual(
            fixtures.rel_path("https://docs.fedoraproject.org/en-US/quick-docs/x/"),
            "en-US/quick-docs/x/index.html",
        )

    def test_an_error_page_is_named_for_its_status(self) -> None:
        self.assertEqual(fixtures.rel_path("https://h/en-US/gone/", 404), "en-US/gone/404.html")


class Committed(unittest.TestCase):
    """The manifest in the tree and the pages it names agree."""

    manifest = fixtures.MANIFEST

    def test_every_page_is_present_and_hashes_to_its_record(self) -> None:
        if not self.manifest.exists():
            self.skipTest("no manifest yet")
        import hashlib

        for e in json.loads(self.manifest.read_text())["pages"]:
            body = (self.manifest.parent / e["path"]).read_bytes()
            self.assertEqual(hashlib.sha256(body).hexdigest(), e["sha256"], e["path"])
            self.assertEqual(len(body), e["bytes"], e["path"])


if __name__ == "__main__":
    unittest.main()
