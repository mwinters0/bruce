#!/usr/bin/env python3
"""Unit tests for shot.py, run with `uv run python -m unittest test_shot`.

Servo is not run here: `mise run shot` is the check that it renders.
`ThisRepo` holds the real page to the print copy.
"""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

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


class Command(unittest.TestCase):
    def test_the_window_and_the_file_url_are_named(self):
        with tempfile.TemporaryDirectory() as d:
            page = Path(d) / "p q.html"
            page.write_text("x", encoding="utf-8")
            command = shot.render_command(Path("/bin/servoshell"), page, Path("o.png"), 390, 3200)
        self.assertEqual(command[:3], ["/bin/servoshell", "-z", "-x"])
        self.assertIn("--window-size=390x3200", command)
        self.assertTrue(command[-1].startswith("file:///"))
        self.assertTrue(command[-1].endswith("p%20q.html"))

    def test_the_widths_are_a_desktop_a_phone_and_a4(self):
        self.assertEqual(dict(shot.WIDTHS), {"1280": 1280, "390": 390, "print": 794})


class Fonts(unittest.TestCase):
    def test_the_pinned_font_stands_behind_every_family(self):
        conf = shot.fonts_conf(Path("/f/ttf"), Path("/c"))
        self.assertIn("<dir>/f/ttf</dir>", conf)
        self.assertIn("<cachedir>/c</cachedir>", conf)
        for family in ("ui-monospace", "monospace", "serif"):
            self.assertIn(f"<string>{family}</string>", conf)
        self.assertIn('mode="append_last"', conf)


class Run(unittest.TestCase):
    def test_a_missing_tool_names_it_and_exits_one(self):
        with tempfile.TemporaryDirectory() as d:
            code = shot.run(
                Path(d) / "p.html", Path(d) / "out", 100, Path(d) / "no-servoshell", Path(d)
            )
        self.assertEqual(code, 1)

    def test_a_missing_page_is_a_shot_error(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaisesRegex(shot.ShotError, "p.html"):
                shot.render(Path(d) / "p.html", Path(d) / "out", 100, Path(d), Path(d))


class ThisRepo(unittest.TestCase):
    def test_the_digests_golden_file_has_print_rules_and_disclosures_to_apply(self):
        html = (shot.REPO / shot.PAGE).read_text(encoding="utf-8")
        copy = shot.print_copy(html)
        self.assertNotIn("@media print", copy)
        self.assertEqual(copy.count("<details"), copy.count(" open>"))


if __name__ == "__main__":
    unittest.main()
