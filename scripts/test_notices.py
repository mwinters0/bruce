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

    def test_an_mpl_crate_names_its_exact_source(self):
        with tempfile.TemporaryDirectory() as t:
            mpl = package(Path(t), "m", license="MPL-2.0")
            dual = package(Path(t), "d", license="MIT OR (MPL-2.0 AND Apache-2.0)")
            mit = package(Path(t), "x", license="MIT OR Apache-2.0")
            out = notices.render([mpl, dual, mit])
            self.assertIn("m 1.0.0: MPL-2.0 (https://example.invalid/m); the crate ships no license text\n"
                          "    source (MPL-2.0): https://crates.io/api/v1/crates/m/1.0.0/download, unmodified", out)
            self.assertIn("crates/d/1.0.0/download", out)
            self.assertNotIn("crates/x/", out)
            self.assertIn("CRATES (3)", out)


def duckdb(root: Path, libs: list[str], version: str = "v9.9.9", source_id: str = "abc123",
           extra: dict[str, str] | None = None) -> dict:
    """A `libduckdb-sys` package whose tarball holds `libs` under
    `third_party/` and the version header, plus `extra` members."""
    p = package(root, notices.BUNDLED_CRATE, {"LICENSE": "duck"})
    header = f'#define DUCKDB_VERSION "{version}"\n#define DUCKDB_SOURCE_ID "{source_id}"\n'
    members = {notices.BUNDLED_VERSION_SOURCE: header, **(extra or {})}
    for lib in libs:
        members[f"duckdb/third_party/{lib}/code.cpp"] = ""
    archive = root / notices.BUNDLED_CRATE / notices.BUNDLED_ARCHIVE
    with tarfile.open(archive, "w:gz") as tar:
        for name, body in {"duckdb/third_party/": "", "duckdb/src/main.cpp": "", **members}.items():
            info = tarfile.TarInfo(name)
            data = body.encode()
            info.size = len(data)
            tar.addfile(info, io.BytesIO(data))
    return p


def vendor(libs_dir: Path, files: dict[str, str], source: str = "v9.9.9 abc123\n") -> None:
    for name, body in files.items():
        f = libs_dir / name
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text(body)
    (libs_dir / notices.SOURCE_FILE).write_text(source)


class Bundled(unittest.TestCase):
    def test_the_bundled_libraries_are_read_from_the_archive(self):
        with tempfile.TemporaryDirectory() as t:
            p = duckdb(Path(t), ["fmt", "re2"])
            b = notices.bundled(p)
            self.assertEqual((b.version, b.source_id, b.libraries), ("v9.9.9", "abc123", ["fmt", "re2"]))
            self.assertEqual(notices.bundled_libraries(p), ["fmt", "re2"])
            self.assertIsNone(notices.bundled(package(Path(t), "other")))

    def test_every_library_s_texts_are_printed_at_the_source_id(self):
        with tempfile.TemporaryDirectory() as t:
            root = Path(t)
            libs = root / "licenses"
            vendor(libs, {"fmt/LICENSE": "fmt text\n", "re2/LICENSE": "re2 text", "re2/AUTHORS": "re2 authors"})
            out = notices.render([duckdb(root, ["fmt", "re2"])], libs)
            self.assertIn("LIBRARIES BUNDLED IN DUCKDB", out)
            self.assertIn("v9.9.9 (abc123)", out)
            for want in ("fmt: LICENSE", "fmt text", "re2: AUTHORS", "re2 authors", "re2: LICENSE", "re2 text"):
                self.assertIn(want, out)
            self.assertNotIn("LIBRARIES BUNDLED", notices.render([package(root, "other")], libs))


class Problems(unittest.TestCase):
    def setUp(self):
        self._t = tempfile.TemporaryDirectory()
        self.root = Path(self._t.name)
        self.libs = self.root / "licenses"
        self.addCleanup(self._t.cleanup)

    def test_vendored_texts_that_match_the_tarball_have_no_problem(self):
        vendor(self.libs, {"fmt/LICENSE": "x"})
        p = duckdb(self.root, ["fmt"])
        self.assertEqual(notices.bundled_problems(p, self.libs), [])

    def test_a_source_id_other_than_the_tarball_s_is_drift(self):
        vendor(self.libs, {"fmt/LICENSE": "x"}, source="v9.9.9 old\n")
        problems = notices.bundled_problems(duckdb(self.root, ["fmt"]), self.libs)
        self.assertEqual(len(problems), 1)
        self.assertIn("--refresh-duckdb", problems[0])

    def test_a_library_without_texts_and_a_vendored_one_the_tarball_lost_are_drift(self):
        vendor(self.libs, {"gone/LICENSE": "x"})
        problems = notices.bundled_problems(duckdb(self.root, ["fmt"]), self.libs)
        self.assertEqual(len(problems), 2)
        self.assertIn("third_party/fmt has no vendored text", problems[0])
        self.assertIn("gone is vendored but the tarball has no", problems[1])

    def test_nothing_vendored_is_drift_not_a_crash(self):
        problems = notices.bundled_problems(duckdb(self.root, ["fmt"]), self.libs)
        self.assertTrue(any("no SOURCE" in m for m in problems))

    def test_a_hand_vendored_notice_must_stay_verbatim_in_its_source(self):
        lib, src = notices.HAND_VENDORED[0]
        comment = "/* head\n * Copyright (c) 2014, A\n *   * Redistributions\n */\n"
        p = duckdb(self.root, [lib], extra={notices.BUNDLED_THIRD_PARTY + src: comment})
        vendor(self.libs, {f"{lib}/LICENSE": "x",
                           f"{lib}/{notices.HAND_NAME}": "Copyright (c) 2014, A\n  * Redistributions\n"})
        self.assertEqual(notices.bundled_problems(p, self.libs), [])
        (self.libs / lib / notices.HAND_NAME).write_text("Copyright (c) 2015, A\n")
        problems = notices.bundled_problems(p, self.libs)
        self.assertEqual(len(problems), 1)
        self.assertIn("no longer verbatim", problems[0])
        (self.libs / lib / notices.HAND_NAME).unlink()
        self.assertIn("missing", notices.bundled_problems(p, self.libs)[0])

    def test_comment_markers_and_whitespace_do_not_matter(self):
        self.assertEqual(notices.normalise(" * a  b\n *   * c\n */"), "a b c")


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


class CheckDrift(unittest.TestCase):
    def test_a_drifted_vendored_text_fails_the_check_and_says_why(self):
        out = io.StringIO()
        real = notices.generate
        def drifted(repo):
            raise notices.Drift("fmt: no vendored text")
        notices.generate = drifted
        try:
            self.assertEqual(notices.check(Path("."), out), 1)
        finally:
            notices.generate = real
        self.assertIn("drifted", out.getvalue())
        self.assertIn("fmt: no vendored text", out.getvalue())


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

    def test_every_bundled_library_has_vendored_texts_at_the_tarball_s_source_id(self):
        for p in notices.shipped(notices.cargo_metadata()):
            self.assertEqual(notices.bundled_problems(p, notices.REPO / notices.LICENSES), [], p["name"])

    def test_no_shipped_crate_lacks_an_expression(self):
        for p in notices.shipped(notices.cargo_metadata()):
            self.assertTrue(p.get("license"), p["name"])


if __name__ == "__main__":
    unittest.main()
