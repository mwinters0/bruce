#!/usr/bin/env python3
"""Unit tests for shot.py, run with `uv run python -m unittest test_shot`.

The browser is not run here: `mise run shot` is the check that it renders.
`ThisRepo` holds the real page to the print copy.
"""

from __future__ import annotations

import os
import socket
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import shot

PAGE = """<style>
@media (max-width: 40rem) { body { font-size: 14px; } }
@media  print {
  details { break-inside: avoid-page; }
}
</style>
<details id="a"><summary>A</summary></details>
<details id="b" open><summary>B</summary></details>
<details>
<summary>C</summary></details>
"""


class PrintCopy(unittest.TestCase):
    def test_the_print_rules_apply_on_screen_and_the_others_are_left(self):
        copy = shot.print_copy(PAGE)
        self.assertIn("@media all {", copy)
        self.assertNotIn("@media  print", copy)
        self.assertIn("@media (max-width: 40rem)", copy)

    def test_every_details_is_open_once(self):
        copy = shot.print_copy(PAGE)
        self.assertIn('<details id="a" open>', copy)
        self.assertIn('<details id="b" open>', copy)
        self.assertNotIn("open open", copy)
        self.assertIn("<details open>", copy)

    def test_a_page_with_no_print_rule_is_refused(self):
        with self.assertRaisesRegex(shot.ShotError, "no `@media print`"):
            shot.print_copy("<details></details>")

    def test_a_print_prefixed_name_is_not_a_print_rule(self):
        with self.assertRaises(shot.ShotError):
            shot.print_copy("@media printer { a {} }")


class Runtime(unittest.TestCase):
    def test_the_flag_beats_the_environment_and_is_split_as_a_command(self):
        env = {shot.RUNTIME_ENV: "podman"}
        self.assertEqual(
            shot.runtime_command("sudo nerdctl", env, lambda r: False), ["sudo", "nerdctl"]
        )

    def test_the_environment_is_taken_as_given(self):
        env = {shot.RUNTIME_ENV: "sudo nerdctl"}
        self.assertEqual(shot.runtime_command(None, env, lambda r: False), ["sudo", "nerdctl"])

    def test_the_first_runtime_that_answers_is_used(self):
        self.assertEqual(shot.runtime_command(None, {}, lambda r: r == "docker"), ["docker"])
        self.assertEqual(shot.runtime_command(None, {}, lambda r: True), ["podman"])

    def test_none_answering_names_the_setting(self):
        with self.assertRaisesRegex(shot.ShotError, shot.RUNTIME_ENV):
            shot.runtime_command(None, {}, lambda r: False)


class Container(unittest.TestCase):
    def test_the_image_is_pinned_by_version_and_digest(self):
        self.assertRegex(shot.IMAGE, r":\d+\.\d+\.\d+\.\d+@sha256:[0-9a-f]{64}$")

    def test_the_port_is_local_only_and_the_mounts_read_only(self):
        with tempfile.TemporaryDirectory() as d:
            command = shot.container_command(
                ["sudo", "nerdctl"], "n", 4242, Path(d), Path(d), Path(d)
            )
        self.assertEqual(command[:3], ["sudo", "nerdctl", "run"])
        self.assertNotIn("--rm", command)
        self.assertIn("127.0.0.1:4242:9222", command)
        self.assertIn("FONTCONFIG_FILE=/work/fonts.conf", command)
        mounts = [c for c in command if c.endswith(":ro")]
        self.assertEqual(len(mounts), 3)
        self.assertEqual(command[-1], shot.IMAGE)

    def test_the_widths_are_a_desktop_a_phone_and_a4_and_only_the_phone_is_mobile(self):
        self.assertEqual(
            list(shot.WIDTHS),
            [("1280", 1280, False), ("390", 390, True), ("print", 794, False)],
        )


class Frames(unittest.TestCase):
    def roundtrip(self, payload: bytes) -> str:
        a, b = socket.socketpair()
        with a, b:
            a.sendall(shot.encode_frame(payload))
            return shot.WebSocket(b).receive()

    def test_each_length_form_reads_back(self):
        for n in (0, 125, 126, 70000):
            text = "x" * n
            self.assertEqual(self.roundtrip(text.encode()), text)

    def test_a_masked_frame_is_masked_with_its_key(self):
        frame = shot.encode_frame(b"hi", b"\x01\x02\x03\x04")
        self.assertEqual(frame, bytes([0x81, 0x82, 1, 2, 3, 4, ord("h") ^ 1, ord("i") ^ 2]))

    def test_fragments_are_joined(self):
        a, b = socket.socketpair()
        with a, b:
            a.sendall(bytes([0x01, 2]) + b"he" + bytes([0x80, 3]) + b"llo")
            self.assertEqual(shot.WebSocket(b).receive(), "hello")

    def test_a_close_is_an_error(self):
        a, b = socket.socketpair()
        with a, b:
            a.sendall(bytes([0x88, 0]))
            with self.assertRaisesRegex(shot.ShotError, "closed"):
                shot.WebSocket(b).receive()


class Fonts(unittest.TestCase):
    def test_the_pinned_font_stands_behind_every_family(self):
        conf = shot.fonts_conf("/fonts", "/c")
        self.assertIn("<dir>/fonts</dir>", conf)
        self.assertIn("<cachedir>/c</cachedir>", conf)
        for family in ("ui-monospace", "monospace", "serif"):
            self.assertIn(f"<string>{family}</string>", conf)
        self.assertIn('mode="append_last"', conf)


class Run(unittest.TestCase):
    def test_no_runtime_answering_exits_one(self):
        with tempfile.TemporaryDirectory() as d:
            with mock.patch.object(shot, "answers", lambda r: False), mock.patch.dict(
                os.environ, {}, clear=True
            ):
                code = shot.run(Path(d) / "p.html", Path(d) / "out", 100, None, Path(d))
        self.assertEqual(code, 1)

    def test_a_missing_font_directory_names_it_and_exits_one(self):
        with tempfile.TemporaryDirectory() as d:
            code = shot.run(Path(d) / "p.html", Path(d) / "out", 100, "true", Path(d) / "nope")
        self.assertEqual(code, 1)

    def test_a_missing_page_is_a_shot_error(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaisesRegex(shot.ShotError, "p.html"):
                shot.render(Path(d) / "p.html", Path(d) / "out", 100, ["true"], Path(d))


class ThisRepo(unittest.TestCase):
    def test_the_digests_golden_file_has_print_rules_and_disclosures_to_apply(self):
        html = (shot.REPO / shot.PAGE).read_text(encoding="utf-8")
        copy = shot.print_copy(html)
        self.assertNotIn("@media print", copy)
        self.assertEqual(copy.count("<details"), copy.count(" open>"))


if __name__ == "__main__":
    unittest.main()
