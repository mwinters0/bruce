#!/usr/bin/env python3
"""The third-party notices a release carries, generated from `Cargo.lock`.

`THIRD-PARTY-NOTICES.txt` at the repository root names every crate linked
into the release binary with its license expression, and prints each license
text the crates ship. The set is the **normal-dependency closure** of `bruce`
for `x86_64-unknown-linux-gnu`, the release's target, read from `cargo
metadata --locked --offline`: a dev-dependency, a build-dependency and a
crate only another platform pulls in are not in the binary, so they are not
listed. A license text is each top-level `LICENSE*`, `LICENSE*`, `COPYING*`,
`NOTICE*` or `UNLICENSE*` file of the crate's source; crates sharing one text
share one entry. A crate that ships none is listed with its expression and its
repository, and says so.

DuckDB is bundled (D23): `libduckdb-sys` compiles a C++ source tree that holds
libraries of its own under `third_party/`. They are listed by name, read from
the archive; their texts are not in the crate, and the file says so.

The file is committed, so the release workflow copies it and a reader sees it
in the tree. `--check` fails when it differs from what the lockfile now
yields, and `ThisRepo` in `test_notices.py` runs that check in the scripts'
`unittest`, so a dependency change that does not regenerate it fails
`mise run check`.

Usage:

    cd scripts
    uv run notices.py            # rewrite THIRD-PARTY-NOTICES.txt
    uv run notices.py --check    # fail if it is out of date
    uv run python -m unittest test_notices
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tarfile
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
BUNDLED_THIRD_PARTY = "duckdb/third_party/"

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
    """The packages linked into the binary: the root's normal-dependency
    closure, the root excluded, sorted by name and version."""
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


def bundled_libraries(package: dict[str, Any]) -> list[str]:
    """The directories under the bundled DuckDB's `third_party/`, or none for
    any other crate."""
    if package["name"] != BUNDLED_CRATE:
        return []
    archive = Path(package["manifest_path"]).parent / BUNDLED_ARCHIVE
    with tarfile.open(archive) as tar:
        names = {
            m.name[len(BUNDLED_THIRD_PARTY):].split("/")[0]
            for m in tar.getmembers()
            if m.name.startswith(BUNDLED_THIRD_PARTY)
        }
    return sorted(n for n in names if n)


def render(packages: Sequence[dict[str, Any]]) -> str:
    """The file's text, a function of the packages and their source alone."""
    shared: dict[str, list[str]] = {}
    crates: list[str] = []
    bundled: list[str] = []
    for p in packages:
        label = f"{p['name']} {p['version']}"
        expr = p.get("license") or "no license expression"
        repo = p.get("repository") or "no repository"
        texts = license_texts(p)
        note = "" if texts else "; the crate ships no license text"
        crates.append(f"{label}: {expr} ({repo}){note}")
        for t in texts:
            shared.setdefault(t, []).append(label)
        bundled += bundled_libraries(p)
    out = [
        "THIRD-PARTY NOTICES",
        "",
        "bruce is licensed MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE). The",
        "release binary links the crates below, each under the license named.",
        "Generated from Cargo.lock by scripts/notices.py; do not edit by hand.",
        "",
        RULE,
        f"CRATES ({len(crates)})",
        RULE,
        "",
        *crates,
        "",
    ]
    if bundled:
        out += [
            RULE,
            "LIBRARIES BUNDLED IN DUCKDB",
            RULE,
            "",
            "The DuckDB source compiled into the binary (libduckdb-sys, MIT) holds",
            "these libraries under its third_party directory. Their license texts",
            "are not in the crate and are not reproduced here; they are in DuckDB's",
            "source repository, https://github.com/duckdb/duckdb, third_party/<name>.",
            "",
            *bundled,
            "",
        ]
    out += [RULE, f"LICENSE TEXTS ({len(shared)})", RULE, ""]
    for text, users in sorted(shared.items(), key=lambda kv: (kv[1][0], kv[0])):
        out += ["-" * 72, "Used by: " + ", ".join(users), "-" * 72, "", text, ""]
    return "\n".join(out)


def generate(repo: Path = REPO) -> str:
    return render(shipped(cargo_metadata(repo)))


def check(repo: Path = REPO, out: TextIO = sys.stdout) -> int:
    """0 when the committed file is what the lockfile yields, else 1."""
    want = generate(repo)
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
    args = ap.parse_args(argv)
    if args.check:
        return check()
    NOTICES.write_text(generate())
    print(f"notices: wrote {NOTICES.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
