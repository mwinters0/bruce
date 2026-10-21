#!/usr/bin/env python3
"""Unit tests for notices.py, run with `uv run python -m unittest test_notices`.

Each test builds a small crate graph on disk and asserts what the file says of
it; `ThisRepo` holds the committed file and the dual license to the tree.
"""

from __future__ import annotations

import io
import tarfile
import tempfile
import tomllib
import unittest
from pathlib import Path

import notices


def package(root: Path, name: str, files: dict[str, str] | None = None, **fields) -> dict:
    """A `cargo metadata` package whose source is `root/<name>`."""
    src = root / name
    src.mkdir(parents=True, exist_ok=True)
    for fname, body in (files or {}).items():
        (src / fname).write_text(body)
    return {
        "id": name,
        "name": name,
        "version": "1.0.0",
        "license": "MIT",
        "repository": f"https://example.invalid/{name}",
        "manifest_path": str(src / "Cargo.toml"),
        **fields,
    }


def graph(packages: list[dict], edges: dict[str, list[tuple[str, str | None]]], root: str) -> dict:
    """The metadata for `packages` with `edges`: id to (dependency id, kind),
    kind `None` for a normal dependency."""
    nodes = [
        {"id": p["id"], "deps": [{"pkg": d, "dep_kinds": [{"kind": k}]} for d, k in edges.get(p["id"], [])]}
        for p in packages
    ]
    return {"packages": packages, "resolve": {"root": root, "nodes": nodes}}


class Shipped(unittest.TestCase):
    def test_only_the_normal_closure_without_the_root_is_listed(self):
        with tempfile.TemporaryDirectory() as t:
            ps = [package(Path(t), n) for n in ("root", "a", "b", "dev", "build", "far")]
            meta = graph(
                ps,
                {
                    "root": [("a", None), ("dev", "dev"), ("build", "build")],
                    "a": [("b", None)],
                    "dev": [("far", None)],
                },
                "root",
            )
            self.assertEqual([p["name"] for p in notices.shipped(meta)], ["a", "b"])

    def test_a_dependency_shared_by_a_normal_and_a_dev_edge_is_shipped(self):
        with tempfile.TemporaryDirectory() as t:
            ps = [package(Path(t), n) for n in ("root", "a")]
            meta = graph(ps, {"root": [("a", "dev")]}, "root")
            meta["resolve"]["nodes"][0]["deps"][0]["dep_kinds"].append({"kind": None})
            self.assertEqual([p["name"] for p in notices.shipped(meta)], ["a"])


class Render(unittest.TestCase):
    def test_crates_sharing_a_text_share_one_entry(self):
        with tempfile.TemporaryDirectory() as t:
            same = {"LICENSE": "The text\r\nof it  \n\n"}
            ps = [package(Path(t), "a", same), package(Path(t), "b", same)]
            out = notices.render(ps)
            self.assertEqual(out.count("The text\nof it\n"), 1)
            self.assertIn("Used by: a 1.0.0, b 1.0.0", out)
            self.assertIn("LICENSE TEXTS (1)", out)

    def test_only_license_files_are_read(self):
        with tempfile.TemporaryDirectory() as t:
            p = package(Path(t), "a", {"LICENSE-MIT": "mit text", "README.md": "readme", "license-apache": "apache text"})
            texts = notices.license_texts(p)
            self.assertEqual(sorted(texts), ["apache text", "mit text"])

    def test_a_crate_with_no_text_says_so(self):
        with tempfile.TemporaryDirectory() as t:
            out = notices.render([package(Path(t), "a", {"README.md": "x"})])
            self.assertIn("a 1.0.0: MIT (https://example.invalid/a); the crate ships no license text", out)

    def test_the_output_is_a_function_of_the_packages(self):
        with tempfile.TemporaryDirectory() as t:
            ps = [package(Path(t), "a", {"LICENSE": "x"}), package(Path(t), "b", {"LICENSE": "y"})]
            self.assertEqual(notices.render(ps), notices.render(list(ps)))
            self.assertTrue(notices.render(ps).endswith("\n"))

    def test_the_bundled_libraries_are_listed_from_the_archive(self):
        with tempfile.TemporaryDirectory() as t:
            p = package(Path(t), notices.BUNDLED_CRATE, {"LICENSE": "duck"})
            archive = Path(t) / notices.BUNDLED_CRATE / notices.BUNDLED_ARCHIVE
            with tarfile.open(archive, "w:gz") as tar:
                for member in ("duckdb/third_party/", "duckdb/third_party/fmt/format.cc",
                               "duckdb/third_party/re2/re2.h", "duckdb/src/main.cpp"):
                    info = tarfile.TarInfo(member)
                    tar.addfile(info, io.BytesIO(b""))
            self.assertEqual(notices.bundled_libraries(p), ["fmt", "re2"])
            self.assertIn("LIBRARIES BUNDLED IN DUCKDB", notices.render([p]))
            self.assertNotIn("LIBRARIES BUNDLED", notices.render([package(Path(t), "other")]))


class Check(unittest.TestCase):
    def test_a_missing_file_is_out_of_date(self):
        with tempfile.TemporaryDirectory() as t:
            out = io.StringIO()
            real = notices.generate
            notices.generate = lambda repo: "want\n"
            try:
                self.assertEqual(notices.check(Path(t), out), 1)
                (Path(t) / notices.NOTICES.name).write_text("want\n")
                self.assertEqual(notices.check(Path(t), out), 0)
                (Path(t) / notices.NOTICES.name).write_text("stale\n")
                self.assertEqual(notices.check(Path(t), out), 1)
            finally:
                notices.generate = real
            self.assertIn("out of date", out.getvalue())


class ThisRepo(unittest.TestCase):
    """The committed file and the license, held to the tree."""

    def test_the_committed_notices_are_what_the_lockfile_yields(self):
        out = io.StringIO()
        self.assertEqual(notices.check(out=out), 0, out.getvalue())

    def test_the_crate_is_licensed_mit_or_apache_and_carries_both_texts(self):
        manifest = tomllib.loads((notices.REPO / "Cargo.toml").read_text())
        self.assertEqual(manifest["package"]["license"], "MIT OR Apache-2.0")
        mit = (notices.REPO / "LICENSE-MIT").read_text()
        self.assertTrue(mit.startswith("MIT License\n"))
        self.assertIn("Permission is hereby granted", mit)
        apache = (notices.REPO / "LICENSE-APACHE").read_text()
        self.assertIn("Apache License", apache)
        self.assertIn("Version 2.0, January 2004", apache)

    def test_no_shipped_crate_lacks_an_expression(self):
        for p in notices.shipped(notices.cargo_metadata()):
            self.assertTrue(p.get("license"), p["name"])


if __name__ == "__main__":
    unittest.main()
