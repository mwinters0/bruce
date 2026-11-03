#!/usr/bin/env python3
"""Unit tests for deployment.py, run with `uv run python -m unittest test_deployment`.

The guide's real scripts run here against a stub `bruce` that records its
command lines and exits as a test tells it to, so the job's branches (a schema
change, a failing digest, retention) are exercised without a crawl;
`e2e.py` runs them against the real binary and the fixture site.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path

import deployment

#: A `bruce` that logs its command line to `$STUB_LOG` and answers each
#: subcommand from `$STUB_<SUB>` (`report` as `DIGEST`): an exit status, or a
#: space-separated list of them, one a call and the last repeating, kept in
#: `$STUB_STATE/` for a run's calls. It writes the files the real one would.
STUB = r"""#!/usr/bin/env bash
echo "$*" >> "$STUB_LOG"
sub=$1; [ "$sub" = report ] && sub=digest
var=STUB_$(echo "$sub" | tr a-z- A-Z_)
if [ -f "$STUB_STATE/$var" ]; then seq=$(cat "$STUB_STATE/$var"); else seq=${!var:-0}; fi
status=${seq%% *}
rest=${seq#"$status"}; rest=${rest# }
[ -z "$rest" ] && rest=$status
echo "$rest" > "$STUB_STATE/$var"
out=; name=bruce; db=; since=; view=default
while [ $# -gt 0 ]; do
  case $1 in
    --out) out=$2; shift ;;
    --name) name=$2; shift ;;
    --db) db=$2; shift ;;
    --since) since=$2 ;;
    --view) view=$2 ;;
  esac
  shift
done
if [ "$sub" = digest ] && [ -n "$since" ] && [ -n "${STUB_DIGEST_SINCE:-}" ]; then exit "$STUB_DIGEST_SINCE"; fi
[ "$status" -eq 0 ] || exit "$status"
case $sub in
  crawl) echo "crawl $(date +%s%N)" > "$db" ;;
  export-baseline) echo "baseline of $(cat "$db")" > "$out" ;;
  digest)
    echo "{\"since\": \"$since\"}" > "$out/$name-$STUB_DATE.json"
    if [ "$view" = default ]; then
      echo "<html>$STUB_DATE</html>" > "$out/$name-$STUB_DATE.html"
    else
      echo "$view $STUB_DATE" > "$out/$name-$STUB_DATE-${view##*/}"
    fi ;;
esac
"""


class Job(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.guide = deployment.extract(deployment.REPO / "docs/manual/deploy.md", self.tmp / "guide")
        self.bin = self.tmp / "bin"
        self.bin.mkdir()
        (self.bin / "bruce").write_text(STUB)
        (self.bin / "bruce").chmod(0o755)
        self.archive = self.tmp / "release.tar.gz"
        self.archive.write_bytes(b"an archive")
        self.work = self.tmp / "work"
        self.log = self.tmp / "log"
        self.state = self.tmp / "state"
        self.pub = self.work / "pub"

    def run_night(self, date: str = "2026-11-01", **env: str) -> subprocess.CompletedProcess[str]:
        shutil.rmtree(self.state, ignore_errors=True)
        self.state.mkdir()
        config = self.tmp / "bruce.toml"
        config.write_text(env.pop("CONFIG_TEXT", "# the config\n"))
        return deployment.night(
            self.guide, self.work, [self.bin],
            {
                "ARCHIVE": str(self.archive), "CONFIG": str(config), "DB": "bruce.duckdb", "PUB": "pub",
                "STUB_LOG": str(self.log), "STUB_STATE": str(self.state), "STUB_DATE": date, **env,
            },
        )

    def calls(self) -> list[str]:
        return self.log.read_text().splitlines() if self.log.exists() else []

    def test_the_first_night_publishes_a_directory_that_meets_the_contract(self) -> None:
        done = self.run_night()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(deployment.check_published(self.pub), [])
        self.assertEqual(json.loads((self.pub / "digest.json").read_text()), {"since": ""})
        self.assertEqual(sorted(p.name for p in (self.pub / "digests").iterdir()), [
            "bruce-2026-11-01.html", "bruce-2026-11-01.json",
        ])
        self.assertEqual(
            [c.split()[0] for c in self.calls()], ["crawl", "export-baseline", "report", "prune"],
        )
        self.assertFalse(any("--since" in c for c in self.calls()))
        self.assertIn("prune --db bruce.duckdb --keep 30", self.calls()[-1])
        self.assertEqual([p.name for p in self.work.iterdir() if p.name != "bruce.duckdb"], ["pub"])

    def test_the_next_night_compares_with_the_baseline_it_replaces_and_keeps_last_nights_set(self) -> None:
        self.run_night("2026-11-01")
        first = deployment.sums(self.pub)
        old = {n for n in first if not n.startswith("digest")}
        done = self.run_night("2026-11-02")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("--since pub/baseline.duckdb.gz", [c for c in self.calls() if c.startswith("report")][1])
        self.assertEqual(json.loads((self.pub / "digest.json").read_text()), {"since": "pub/baseline.duckdb.gz"})
        self.assertEqual(deployment.check_published(self.pub), [])
        for name in old:
            self.assertTrue((self.pub / name).is_file(), f"{name} did not stay one more night")
        self.assertEqual(len(list((self.pub / "digests").glob("*.json"))), 2)
        self.assertNotEqual(deployment.sums(self.pub), first)

    def test_a_third_night_drops_the_first_nights_set(self) -> None:
        names = []
        for i in range(3):
            self.run_night(f"2026-11-0{i + 1}", CONFIG_TEXT=f"# config {i}\n")
            names.append({n for n in deployment.sums(self.pub) if n.startswith("config-")})
        self.assertEqual(len({n for s in names for n in s}), 3)
        present = {p.name for p in self.pub.glob("config-*")}
        self.assertEqual(present, names[1] | names[2])

    def test_a_database_of_another_schema_is_moved_aside_and_the_crawl_starts_fresh(self) -> None:
        self.work.mkdir()
        (self.work / "bruce.duckdb").write_text("an old database")
        done = self.run_night(STUB_CRAWL="3 0")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual((self.work / "bruce.duckdb.archive").read_text(), "an old database")
        self.assertTrue((self.work / "bruce.duckdb").read_text().startswith("crawl "))
        self.assertEqual([c.split()[0] for c in self.calls()].count("crawl"), 2)
        self.assertEqual(deployment.check_published(self.pub), [])

    def test_another_failing_crawl_stops_the_job_with_the_published_directory_as_it_was(self) -> None:
        self.run_night("2026-11-01")
        before = (self.pub / "SHA256SUMS").read_text()
        done = self.run_night("2026-11-02", STUB_CRAWL="1", DELIVER="/bin/false")
        self.assertEqual(done.returncode, 1)
        self.assertEqual((self.pub / "SHA256SUMS").read_text(), before)
        self.assertNotIn("export-baseline", self.calls()[-1])

    def test_every_digest_is_written_through_the_view_the_job_names_and_the_built_in_one_by_default(self) -> None:
        self.run_night("2026-11-01")
        self.run_night("2026-11-02", VIEW="/etc/bruce/fedora.html", STUB_DIGEST_SINCE="3")
        digests = [c for c in self.calls() if c.startswith("report")]
        self.assertEqual(len(digests), 3)
        self.assertIn("--view default", digests[0])
        for call in digests[1:]:
            self.assertIn("--view /etc/bruce/fedora.html", call)

    def test_a_view_in_a_file_is_published_as_the_digest_of_its_format_whatever_its_name(self) -> None:
        for date in ("2026-11-01", "2026-11-02"):
            done = self.run_night(date, VIEW="/etc/bruce/fedora.html")
            self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(deployment.check_published(self.pub), [])
        self.assertEqual((self.pub / "digest.html").read_text(), "/etc/bruce/fedora.html 2026-11-02\n")
        self.assertEqual(sorted(p.name for p in (self.pub / "digests").iterdir()), [
            "bruce-2026-11-01.html", "bruce-2026-11-01.json", "bruce-2026-11-02.html", "bruce-2026-11-02.json",
        ])
        self.assertEqual([p.name for p in self.pub.glob("digest-*")], [])

    def test_a_view_the_job_cannot_name_is_refused_before_the_crawl(self) -> None:
        for view in ("views/fedora", "fedora.json", "/etc/bruce/Links.JSON"):
            with self.subTest(view):
                done = self.run_night(VIEW=view)
                self.assertEqual(done.returncode, 1)
                self.assertIn("extension", done.stderr)
                self.assertEqual(self.calls(), [])
                self.assertFalse(self.pub.exists())

    def test_a_digest_published_under_the_views_file_name_is_not_the_contract(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            pub = Path(d)
            (pub / "digest-fedora.html").write_text("x")
            (pub / "digests").mkdir()
            (pub / "digests" / "bruce-2026-11-01-fedora.html").write_text("x")
            (pub / "SHA256SUMS").write_text("")
            problems = "\n".join(deployment.check_published(pub))
            self.assertIn("no digest.html", problems)
            self.assertIn("digest-fedora.html: a digest is published as digest.<ext>", problems)
            self.assertIn("digests/bruce-2026-11-01-fedora.html is not <stem>-<date>.<ext>", problems)

    def test_a_baseline_of_another_schema_writes_the_digest_uncompared(self) -> None:
        self.run_night("2026-11-01")
        done = self.run_night("2026-11-02", STUB_DIGEST_SINCE="3")
        self.assertEqual(done.returncode, 0, done.stderr)
        digests = [c for c in self.calls() if c.startswith("report")]
        self.assertEqual(len(digests), 3)
        self.assertNotIn("--since", digests[2])
        self.assertEqual(json.loads((self.pub / "digest.json").read_text()), {"since": ""})

    def test_a_failing_digest_still_publishes_the_baseline_and_the_job_fails(self) -> None:
        self.run_night("2026-11-01")
        latest = (self.pub / "digest.json").read_text()
        done = self.run_night("2026-11-02", STUB_DIGEST="1")
        self.assertEqual(done.returncode, 1, done.stderr)
        self.assertEqual(deployment.check_published(self.pub), [])
        self.assertEqual((self.pub / "digest.json").read_text(), latest)
        self.assertEqual(len(list((self.pub / "digests").glob("*.json"))), 1)
        self.assertEqual(
            sum(1 for c in deployment.sums(self.pub) if c.startswith("baseline-")), 1,
        )

    def test_the_digests_are_trimmed_by_nights_views_and_json_apart(self) -> None:
        for day in range(1, 6):
            done = self.run_night(f"2026-11-0{day}", KEEP_VIEWS="2", KEEP_JSON="4", VIEW="views/links.csv")
            self.assertEqual(done.returncode, 0, done.stderr)
        kept = sorted(p.name for p in (self.pub / "digests").iterdir())
        self.assertEqual(kept, [
            "bruce-2026-11-02.json", "bruce-2026-11-03.json", "bruce-2026-11-04.csv",
            "bruce-2026-11-04.json", "bruce-2026-11-05.csv", "bruce-2026-11-05.json",
        ])
        self.assertEqual(deployment.check_published(self.pub, "csv"), [])

    def test_no_history_is_kept_and_nothing_pruned_with_an_empty_keep(self) -> None:
        self.run_night(KEEP="")
        self.assertNotIn("prune", [c.split()[0] for c in self.calls()])

    def test_a_prune_that_fails_fails_the_job_after_publishing(self) -> None:
        done = self.run_night(STUB_PRUNE="1")
        self.assertEqual(done.returncode, 1)
        self.assertEqual(deployment.check_published(self.pub), [])

    def test_the_deliver_command_is_given_the_published_directory(self) -> None:
        deliver = self.tmp / "deliver.sh"
        deliver.write_text('#!/bin/sh\nls "$1/SHA256SUMS" > "$1/../delivered"\n')
        deliver.chmod(deliver.stat().st_mode | stat.S_IXUSR)
        done = self.run_night(DELIVER=str(deliver))
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual((self.work / "delivered").read_text().strip(), "pub/SHA256SUMS")

    def test_a_deliver_command_that_fails_fails_the_job(self) -> None:
        self.assertEqual(self.run_night(DELIVER="/bin/false").returncode, 1)

    def test_a_staging_failure_leaves_the_directory_and_fails_the_job(self) -> None:
        self.run_night("2026-11-01")
        before = (self.pub / "SHA256SUMS").read_text()
        done = self.run_night("2026-11-02", ARCHIVE=str(self.tmp / "absent.tar.gz"))
        self.assertNotEqual(done.returncode, 0)
        self.assertEqual((self.pub / "SHA256SUMS").read_text(), before)
        self.assertEqual([p.name for p in self.work.glob("pub.*")], [])

    def test_a_job_without_an_archive_says_what_it_needs(self) -> None:
        env = {k: v for k, v in os.environ.items() if k != "ARCHIVE"}
        done = subprocess.run(
            ["bash", str(self.guide / deployment.NIGHTLY)], cwd=self.tmp, env=env, capture_output=True, text=True,
        )
        self.assertEqual(done.returncode, 1)
        self.assertIn("ARCHIVE", done.stderr)


class Pull(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.guide = deployment.extract(deployment.REPO / "docs/manual/deploy.md", self.tmp / "guide")
        self.pub = self.tmp / "pub"
        (self.pub / "digests").mkdir(parents=True)
        self.dest = self.tmp / "served"
        self.server = deployment.serve_directory(self.pub, 0)
        self.addCleanup(self.server.server_close)
        self.addCleanup(self.server.shutdown)
        self.url = f"http://127.0.0.1:{self.server.server_address[1]}"

    def publish(self, **files: str) -> None:
        for name, text in files.items():
            (self.pub / name.replace("__", "/")).write_text(text)
        lines = [
            f"{deployment.sha256(self.pub / n)}  {n}" for n in sorted(
                p.relative_to(self.pub).as_posix() for p in self.pub.rglob("*") if p.is_file() and p.name != "SHA256SUMS"
            )
        ]
        (self.pub / "SHA256SUMS").write_text("\n".join(lines) + "\n")

    def pull(self) -> subprocess.CompletedProcess[str]:
        return deployment.pull(self.guide, self.url, self.dest, self.tmp / "work")

    def test_the_first_pull_fetches_the_set_and_the_next_finds_nothing_new(self) -> None:
        self.publish(a="one", digests__d="two")
        done = self.pull()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual((self.dest / "digests" / "d").read_text(), "two")
        self.assertEqual((self.dest / "SHA256SUMS").read_text(), (self.pub / "SHA256SUMS").read_text())
        before = (self.dest / "a").stat().st_ino
        self.assertEqual(self.pull().returncode, 0)
        self.assertEqual((self.dest / "a").stat().st_ino, before)

    def test_a_changed_set_replaces_the_directory_whole(self) -> None:
        self.publish(a="one", b="two")
        self.pull()
        (self.pub / "b").unlink()
        self.publish(a="one", c="three")
        done = self.pull()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(sorted(p.name for p in self.dest.iterdir()), ["SHA256SUMS", "a", "c"])
        self.assertEqual([p.name for p in self.tmp.glob("served.*")], [])

    def test_a_set_that_fails_its_sums_leaves_the_directory_served(self) -> None:
        self.publish(a="one")
        self.pull()
        self.publish(a="one", b="two")
        (self.pub / "b").write_text("stale")
        done = self.pull()
        self.assertEqual(done.returncode, 1)
        self.assertIn("fails its sums", done.stderr)
        self.assertEqual(sorted(p.name for p in self.dest.iterdir()), ["SHA256SUMS", "a"])
        self.assertEqual([p.name for p in self.tmp.glob("served.*")], [])

    def test_a_listed_file_that_is_missing_fails_the_pull(self) -> None:
        self.publish(a="one", b="two")
        (self.pub / "b").unlink()
        self.assertEqual(self.pull().returncode, 1)
        self.assertFalse(self.dest.exists())

    def test_a_name_outside_the_directory_is_refused(self) -> None:
        (self.pub / "SHA256SUMS").write_text(f"{'0' * 64}  ../escape\n")
        done = self.pull()
        self.assertEqual(done.returncode, 1)
        self.assertIn("not inside", done.stderr)
        self.assertFalse((self.tmp / "escape").exists())

    def test_no_sums_to_fetch_fails_the_pull(self) -> None:
        self.assertEqual(self.pull().returncode, 1)


class CiWorkflow(unittest.TestCase):
    """The guide's CI workflow, its fetch and check steps run on their own: the build is
    Antora's and `e2e.py` runs it."""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.guide = deployment.extract(deployment.REPO / "docs/manual/deploy.md", self.tmp / "guide")
        self.pub = self.tmp / "pub"
        self.pub.mkdir()
        self.ws = self.tmp / "ws"

    def publish(self, baselines: int = 1, bad: str | None = None) -> str:
        """A published directory of the contract's three files by content; returns its URL."""
        binary = self.tmp / "bruce"
        binary.write_text("#!/bin/sh\n")
        deployment.release_archive(binary, self.pub / "bruce.tar.gz")
        lines = []
        names = [
            ("bruce-{}.tar.gz", (self.pub / "bruce.tar.gz").read_bytes()),
            ("config-{}.toml", b"# the config\n"),
            *[("baseline-{}.duckdb.gz", f"baseline {i}".encode()) for i in range(baselines)],
        ]
        for pattern, body in names:
            sha = hashlib.sha256(body).hexdigest()
            name = pattern.format(sha[:16])
            (self.pub / name).write_bytes(body)
            lines.append(f"{sha}  {name}\n")
            if bad and pattern.startswith(bad):
                (self.pub / name).write_bytes(body + b"!")
        (self.pub / "SHA256SUMS").write_text("".join(lines))
        server = deployment.serve_directory(self.pub, 0)
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        return f"http://127.0.0.1:{server.server_address[1]}"

    def fetch(self, url: str) -> subprocess.CompletedProcess[str]:
        shutil.rmtree(self.ws, ignore_errors=True)
        return deployment.ci_job(self.guide, self.ws, {"PUB_URL": url}, only=[deployment.FETCH_STEP])

    def test_the_env_is_the_workflows_own_and_a_comment_is_skipped(self) -> None:
        env = deployment.workflow_env("jobs:\n  j:\n    env:\n      # why\n      A: x y\n      B: 1\n    steps:\n")
        self.assertEqual(env, {"A": "x y", "B": "1"})
        guide = deployment.workflow_env(deployment.workflow_text(self.guide))
        for key in ("PUB_URL", "PREFIX", "PLAYBOOK", "SITE_DIR", "ANTORA_VERSION", "ANTORA_DIR"):
            self.assertIn(key, guide)

    def test_the_fetch_takes_the_three_files_by_content_and_unpacks_the_binary(self) -> None:
        done = self.fetch(self.publish())
        self.assertEqual(done.returncode, 0, done.stderr)
        for name in ("bruce.toml", "baseline.duckdb.gz", "bin/bruce"):
            self.assertTrue((self.ws / "bruce-ci" / name).is_file(), name)

    def test_a_file_failing_its_sum_fails_the_job_as_could_not_run(self) -> None:
        for kind in ("bruce-", "config-", "baseline-"):
            with self.subTest(kind):
                done = self.fetch(self.publish(bad=kind))
                self.assertEqual(done.returncode, 2, done.stderr)
                self.assertIn("fails its sum", done.stderr)
                self.assertIn("says nothing of the change", done.stderr)

    def test_no_directory_and_an_ambiguous_one_fail_the_job_as_could_not_run(self) -> None:
        done = self.fetch(self.publish() + "/nothing")
        self.assertEqual(done.returncode, 2, done.stderr)
        self.assertIn("no SHA256SUMS", done.stderr)
        self.pub.joinpath("SHA256SUMS").unlink()
        done = self.fetch(self.publish(baselines=2))
        self.assertEqual(done.returncode, 2, done.stderr)
        self.assertIn("no single baseline", done.stderr)

    def check(self, status: int) -> subprocess.CompletedProcess[str]:
        shutil.rmtree(self.ws, ignore_errors=True)
        stub = self.ws / "bruce-ci" / "bin" / "bruce"
        stub.parent.mkdir(parents=True)
        stub.write_text(f"#!/bin/sh\necho \"$*\"\nexit {status}\n")
        stub.chmod(0o755)
        return deployment.ci_job(self.guide, self.ws, {"SITE_DIR": "site", "PREFIX": "https://x/"}, only=["Check the build"])

    def test_the_checks_exit_status_is_the_jobs_and_says_whose_failure_it_is(self) -> None:
        for status, word in ((0, "breaks no link"), (1, "breaks the links listed above"), (2, "the job failed, not the change"),
                             (3, "the job failed, not the change")):
            with self.subTest(status):
                done = self.check(status)
                self.assertEqual(done.returncode, status, done.stderr)
                self.assertIn(word, done.stdout + done.stderr)

    def test_the_check_names_the_baseline_and_config_the_fetch_leaves(self) -> None:
        done = self.check(0)
        self.assertIn("--baseline bruce-ci/baseline.duckdb.gz --config bruce-ci/bruce.toml", done.stdout)
        self.assertIn("check-dir site --prefix https://x/", done.stdout)

    def test_the_guides_workflow_uses_no_action_and_its_steps_are_the_ones_e2e_skips_and_runs(self) -> None:
        text = deployment.workflow_text(self.guide)
        self.assertEqual(deployment.workflows.uses(text), [])
        names = [s.name for s in deployment.workflows.steps(text)]
        for step in (*deployment.INSTALL_STEPS, deployment.FETCH_STEP, "Build the site", "Check the build"):
            self.assertIn(step, names)


class Published(unittest.TestCase):
    def test_a_directory_without_its_sums_is_not_one(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            self.assertIn("no SHA256SUMS", deployment.check_published(Path(d))[0])


if __name__ == "__main__":
    unittest.main()
