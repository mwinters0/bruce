#!/usr/bin/env python3
"""The third-party notices a release carries, generated from `Cargo.lock`.

`THIRD-PARTY-NOTICES.txt` at the repository root names every crate the
release binary is built from with its license expression, and prints each
license text the crates ship. The set is the **normal-dependency closure** of
`bruce` for `x86_64-unknown-linux-gnu`, the release's target, read from `cargo
metadata --locked --offline`: a dev-dependency, a build-dependency and a
crate only another platform pulls in are not listed. A proc-macro crate and
what it pulls in are normal dependencies, so they are listed though compiled
for the host and not linked. A license text is each top-level `LICENSE*`,
`LICENCE*`, `COPYING*`, `NOTICE*` or `UNLICENSE*` file of the crate's source;
crates sharing one text share one entry. A crate that ships none is listed
with its expression and its repository, and says so.

DuckDB is bundled (D23): `libduckdb-sys` compiles a C++ source tree that holds
libraries of its own under `third_party/`, and its tarball ships none of their
license texts. The file prints the texts of **every** `third_party/` directory
the tarball holds, whether or not it is compiled in (D60), from
`licenses/duckdb/`: the `LICENSE*`, `LICENCE*`, `COPYING*`, `NOTICE*` and
`AUTHORS*` files DuckDB's repository holds at the tag the tarball names,
vendored by `--refresh-duckdb` and by it alone, so generating and checking
stay offline; plus `SOURCE-NOTICE`, a notice
a library's source carries and its LICENSE omits, vendored by hand from the
tarball's source file (`HAND_VENDORED`); plus a text the source does not carry
at all, copied by hand from the license's own site (`HAND_TEXTS`). The check
fails on drift: a vendored source id other than the tarball's, a directory with
no texts, a vendored one the tarball lost, a hand-vendored notice no longer in
its source, a hand-copied text missing or no longer marked as hand-vendored. A
crate whose expression names MPL-2.0 is listed with its exact source (§3.2(a)).

The file is committed, so the release workflow copies it and a reader sees it
in the tree. `--check` fails when it differs from what the lockfile now
yields, and `ThisRepo` in `test_notices.py` runs that check in the scripts'
`unittest`, so a dependency change that does not regenerate it fails
`mise run check`.

Usage:

    cd scripts
    uv run notices.py            # rewrite THIRD-PARTY-NOTICES.txt
    uv run notices.py --check    # fail if it is out of date
    uv run notices.py --refresh-duckdb   # re-vendor DuckDB's texts (network)
    uv run python -m unittest test_notices
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Sequence, TextIO

REPO = Path(__file__).resolve().parent.parent
NOTICES = REPO / "THIRD-PARTY-NOTICES.txt"
#: The release's target; `cargo metadata` drops what only other platforms need.
TARGET = "x86_64-unknown-linux-gnu"
#: A top-level file whose name starts so is a license text, any case.
LICENSE_PREFIXES = ("LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE")
#: The crate whose source archive holds DuckDB's bundled libraries.
BUNDLED_CRATE = "libduckdb-sys"
BUNDLED_ARCHIVE = "duckdb.tar.gz"
BUNDLED_ROOT = "duckdb/"
BUNDLED_THIRD_PARTY = BUNDLED_ROOT + "third_party/"
BUNDLED_VERSION_SOURCE = BUNDLED_ROOT + "src/function/table/version/pragma_version.cpp"
#: Where the bundled libraries' texts are vendored, and the file naming the
#: DuckDB tag and source id they were taken at.
LICENSES = Path("licenses") / "duckdb"
SOURCE_FILE = "SOURCE"
#: A file under a library's directory is a text when its name starts so.
BUNDLED_PREFIXES = ("LICENSE", "LICENCE", "COPYING", "NOTICE", "AUTHORS")
#: The upstream repository the refresh reads, at the tag the tarball names.
UPSTREAM = "https://github.com/duckdb/duckdb.git"
#: A notice a library's source carries and its LICENSE omits, vendored by hand
#: as `licenses/duckdb/<library>/SOURCE-NOTICE`: (library, the source file
#: under `third_party/` it is verbatim in).
HAND_VENDORED = (
    ("hyperloglog", "hyperloglog/hyperloglog.cpp"),
    ("libpg_query", "libpg_query/src_backend_parser_parser.cpp"),
)
HAND_NAME = "SOURCE-NOTICE"
#: A text the tarball holds nowhere, copied by hand from the license's site and
#: kept beside the library's other texts: (library, file, the marker its first
#: line must carry). The check cannot hold it verbatim against the tarball; it
#: holds that it is present and says where it came from.
HAND_TEXTS = (
    ("libpg_query", "SOURCE-NOTICE-TERMS", "Hand-vendored from https://www.postgresql.org/"),
)
#: Where a crate's exact source is downloaded.
CRATE_SOURCE = "https://crates.io/api/v1/crates/{name}/{version}/download"

RULE = "=" * 72


def cargo_metadata(repo: Path = REPO) -> dict[str, Any]:
    """The workspace's resolved graph for the release target, offline and
    locked: a lockfile out of date with `Cargo.toml` is an error here."""
    out = subprocess.run(
        ["cargo", "metadata", "--locked", "--offline", "--format-version", "1",
         "--filter-platform", TARGET],
        cwd=repo, capture_output=True, text=True,
    )
    if out.returncode != 0:
        raise SystemExit(f"cargo metadata failed:\n{out.stderr}")
    return json.loads(out.stdout)


def shipped(meta: dict[str, Any]) -> list[dict[str, Any]]:
    """The packages the binary is built from: the root's normal-dependency
    closure, proc-macro crates included, the root excluded, sorted by name
    and version."""
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    root = meta["resolve"]["root"]
    seen: set[str] = set()
    stack = [root]
    while stack:
        pid = stack.pop()
        if pid in seen:
            continue
        seen.add(pid)
        for dep in nodes[pid]["deps"]:
            if any(k["kind"] is None for k in dep["dep_kinds"]):
                stack.append(dep["pkg"])
    seen.discard(root)
    return sorted((packages[i] for i in seen), key=lambda p: (p["name"], p["version"]))


def license_texts(package: dict[str, Any]) -> list[str]:
    """The license files at the top of a crate's source, normalised: LF line
    ends, no trailing blank lines, sorted by file name."""
    src = Path(package["manifest_path"]).parent
    texts = []
    for f in sorted(src.iterdir(), key=lambda p: p.name.upper()):
        if f.is_file() and f.name.upper().startswith(LICENSE_PREFIXES):
            body = f.read_bytes().decode("utf-8", errors="replace")
            texts.append("\n".join(line.rstrip() for line in body.splitlines()).strip("\n"))
    return texts


@dataclass(frozen=True)
class Bundled:
    """What `libduckdb-sys`'s tarball says of the DuckDB it holds."""

    archive: Path
    version: str
    source_id: str
    libraries: list[str]


def bundled(package: dict[str, Any]) -> Bundled | None:
    """The bundled DuckDB's version, source id and `third_party/` directories,
    or none for any other crate."""
    if package["name"] != BUNDLED_CRATE:
        return None
    archive = Path(package["manifest_path"]).parent / BUNDLED_ARCHIVE
    with tarfile.open(archive) as tar:
        names = {
            m.name[len(BUNDLED_THIRD_PARTY):].split("/")[0]
            for m in tar.getmembers()
            if m.name.startswith(BUNDLED_THIRD_PARTY)
        }
        version_source = tar.extractfile(BUNDLED_VERSION_SOURCE)
        text = version_source.read().decode() if version_source else ""
    def define(key: str) -> str:
        m = re.search(rf'#define {key} "([^"]*)"', text)
        return m.group(1) if m else ""
    return Bundled(archive, define("DUCKDB_VERSION"), define("DUCKDB_SOURCE_ID"),
                   sorted(n for n in names if n))


def bundled_libraries(package: dict[str, Any]) -> list[str]:
    """The directories under the bundled DuckDB's `third_party/`, or none for
    any other crate."""
    b = bundled(package)
    return b.libraries if b else []


def normalise(text: str) -> str:
    """A text's words: comment markers (`/`, `*`) and whitespace dropped, so a
    notice vendored out of a source comment compares with the comment."""
    return " ".join(re.sub(r"[/*]", " ", text).split())


def vendored_texts(libs_dir: Path, lib: str) -> list[tuple[str, str]]:
    """A library's vendored files, as (path under the library, text)."""
    root = libs_dir / lib
    return [
        (str(f.relative_to(root)), clean(f.read_bytes().decode("utf-8", errors="replace")))
        for f in sorted(root.rglob("*"))
        if f.is_file()
    ]


def clean(body: str) -> str:
    return "\n".join(line.rstrip() for line in body.splitlines()).strip("\n")


def bundled_problems(package: dict[str, Any], libs_dir: Path) -> list[str]:
    """What is out of step between the tarball and the vendored texts."""
    b = bundled(package)
    if b is None:
        return []
    out = []
    source = libs_dir / SOURCE_FILE
    want = f"{b.version} {b.source_id}"
    have = source.read_text().strip() if source.exists() else "no " + SOURCE_FILE
    if have != want:
        out.append(f"{libs_dir}/{SOURCE_FILE} says {have!r}, the tarball is {want!r}; "
                   "run `uv run notices.py --refresh-duckdb`")
    vendored = {d.name for d in libs_dir.iterdir() if d.is_dir()} if libs_dir.is_dir() else set()
    for lib in b.libraries:
        if lib not in vendored or not vendored_texts(libs_dir, lib):
            out.append(f"third_party/{lib} has no vendored text under {libs_dir}")
    for lib in sorted(vendored - set(b.libraries)):
        out.append(f"{libs_dir}/{lib} is vendored but the tarball has no third_party/{lib}")
    with tarfile.open(b.archive) as tar:
        members = set(tar.getnames())
        for lib, src in HAND_VENDORED:
            if lib not in b.libraries:
                continue
            notice = libs_dir / lib / HAND_NAME
            member = tar.extractfile(BUNDLED_THIRD_PARTY + src) if BUNDLED_THIRD_PARTY + src in members else None
            if not notice.exists() or member is None:
                out.append(f"{lib}: {HAND_NAME} or its source {src} is missing")
            elif normalise(notice.read_text()) not in normalise(member.read().decode(errors="replace")):
                out.append(f"{lib}: {HAND_NAME} is no longer verbatim in {src}")
    for lib, name, marker in HAND_TEXTS:
        if lib not in b.libraries:
            continue
        text = libs_dir / lib / name
        if not text.exists():
            out.append(f"{lib}: {name} is missing")
        elif not text.read_text().startswith(marker):
            out.append(f"{lib}: {name} no longer opens with {marker!r}")
    return out


def mpl_source(package: dict[str, Any]) -> str:
    """The line naming an MPL-2.0 crate's exact source, unmodified (§3.2(a))."""
    if "MPL-2.0" not in re.split(r"[\s()]+", package.get("license") or ""):
        return ""
    url = CRATE_SOURCE.format(name=package["name"], version=package["version"])
    return f"    source (MPL-2.0): {url}, unmodified"


def render(packages: Sequence[dict[str, Any]], libs_dir: Path | None = None) -> str:
    """The file's text, a function of the packages, their source and the
    vendored texts alone."""
    shared: dict[str, list[str]] = {}
    crates: list[str] = []
    duckdb: Bundled | None = None
    for p in packages:
        label = f"{p['name']} {p['version']}"
        expr = p.get("license") or "no license expression"
        repo = p.get("repository") or "no repository"
        texts = license_texts(p)
        note = "" if texts else "; the crate ships no license text"
        crates.append(f"{label}: {expr} ({repo}){note}")
        if source := mpl_source(p):
            crates.append(source)
        for t in texts:
            shared.setdefault(t, []).append(label)
        duckdb = bundled(p) or duckdb
    out = [
        "THIRD-PARTY NOTICES",
        "",
        "bruce is licensed MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE). The",
        "release binary is built from the crates below, each under the license named.",
        "Generated from Cargo.lock by scripts/notices.py; do not edit by hand.",
        "",
        RULE,
        f"CRATES ({len([c for c in crates if not c.startswith('    ')])})",
        RULE,
        "",
        *crates,
        "",
    ]
    if duckdb and libs_dir is not None:
        out += [
            RULE,
            "LIBRARIES BUNDLED IN DUCKDB",
            RULE,
            "",
            f"The DuckDB source {duckdb.version} ({duckdb.source_id}) compiled into the",
            "binary (libduckdb-sys, MIT) holds the libraries below under its",
            "third_party directory. Every directory is listed, whether or not it is",
            "compiled in, with the license files DuckDB's repository holds for it at",
            "that tag; a notice in a library's source that its LICENSE omits is",
            "marked SOURCE-NOTICE, and a license text no source carries, copied by",
            "hand from the license's site, opens with Hand-vendored. mbedtls is",
            "offered under Apache-2.0 OR GPL-2.0-or-later and is taken here under",
            "Apache-2.0.",
            "",
        ]
        for lib in duckdb.libraries:
            for name, text in vendored_texts(libs_dir, lib):
                out += ["-" * 72, f"{lib}: {name}", "-" * 72, "", text, ""]
    out += [RULE, f"LICENSE TEXTS ({len(shared)})", RULE, ""]
    for text, users in sorted(shared.items(), key=lambda kv: (kv[1][0], kv[0])):
        out += ["-" * 72, "Used by: " + ", ".join(users), "-" * 72, "", text, ""]
    return "\n".join(out)


class Drift(Exception):
    """The vendored texts are out of step with the tarball."""


def refresh_duckdb(repo: Path = REPO) -> None:
    """Re-vendor DuckDB's license files from its repository at the tag the
    tarball names. The only step that uses the network."""
    meta = cargo_metadata(repo)
    package = next(p for p in shipped(meta) if p["name"] == BUNDLED_CRATE)
    b = bundled(package)
    assert b is not None
    libs_dir = repo / LICENSES
    with tempfile.TemporaryDirectory() as t:
        clone = Path(t) / "duckdb"
        def git(*args: str) -> str:
            r = subprocess.run(["git", *args], cwd=clone if clone.exists() else t,
                               capture_output=True, text=True)
            if r.returncode != 0:
                raise SystemExit(f"git {' '.join(args)} failed:\n{r.stderr}")
            return r.stdout.strip()
        git("clone", "--quiet", "--depth", "1", "--branch", b.version, "--filter=blob:none",
            "--sparse", UPSTREAM, str(clone))
        git("sparse-checkout", "set", "third_party")
        head = git("rev-parse", "HEAD")
        if not head.startswith(b.source_id):
            raise SystemExit(f"tag {b.version} is {head}, the tarball's source id is {b.source_id}")
        kept = [(lib, HAND_NAME) for lib, _ in HAND_VENDORED] + [(lib, n) for lib, n, _ in HAND_TEXTS]
        keep = {Path(lib) / name: (libs_dir / lib / name).read_bytes() for lib, name in kept}
        if libs_dir.exists():
            shutil.rmtree(libs_dir)
        for lib in b.libraries:
            root = clone / "third_party" / lib
            for f in sorted(root.rglob("*")):
                if f.is_file() and f.name.upper().startswith(BUNDLED_PREFIXES):
                    dest = libs_dir / lib / f.relative_to(root)
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(f, dest)
        for rel, body in keep.items():
            (libs_dir / rel).write_bytes(body)
        (libs_dir / SOURCE_FILE).write_text(f"{b.version} {b.source_id}\n")


def generate(repo: Path = REPO) -> str:
    """The file the lockfile and the vendored texts yield; `Drift` when the
    texts no longer match the tarball."""
    packages = shipped(cargo_metadata(repo))
    libs_dir = repo / LICENSES
    problems = [m for p in packages for m in bundled_problems(p, libs_dir)]
    if problems:
        raise Drift("\n".join(problems))
    return render(packages, libs_dir)


def check(repo: Path = REPO, out: TextIO = sys.stdout) -> int:
    """0 when the committed file is what the lockfile yields, else 1."""
    try:
        want = generate(repo)
    except Drift as e:
        out.write(f"notices: the vendored texts have drifted:\n{e}\n")
        return 1
    path = repo / NOTICES.name
    have = path.read_text() if path.exists() else None
    if have == want:
        out.write("notices: current\n")
        return 0
    out.write(f"notices: {path.name} is out of date; run `uv run notices.py`\n")
    return 1


def main(argv: Sequence[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--check", action="store_true", help="fail if the file is out of date")
    ap.add_argument("--refresh-duckdb", action="store_true",
                    help="re-vendor DuckDB's bundled libraries' texts (uses the network)")
    args = ap.parse_args(argv)
    if args.refresh_duckdb:
        refresh_duckdb()
        print(f"notices: vendored {LICENSES}")
        return 0
    if args.check:
        return check()
    try:
        NOTICES.write_text(generate())
    except Drift as e:
        print(f"notices: the vendored texts have drifted:\n{e}")
        return 1
    print(f"notices: wrote {NOTICES.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
