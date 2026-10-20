"""Synthetic-only regression tests for conservative F6 closed-outline qualification."""
import unittest

from m0_f6_outline_qualification import qualify_line_outline
from m0_f6_board_integrity import geometry_inventory, iter_sexpressions


def line(a, b):
    return f'(gr_line (start {a[0]} {a[1]}) (end {b[0]} {b[1]}) (layer "Edge.Cuts"))'


def rectangle(x0=10, y0=20, x1=50, y1=30):
    p = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
    return [line(p[i], p[(i + 1) % 4]) for i in range(4)]


def board(primitives, other=""):
    return ("(kicad_pcb (version 20240108) (layers (0 \"F.Cu\" signal)) "
            + " ".join(primitives) + " " + other + ")").encode()


class TestStrictLineOnlyOutline(unittest.TestCase):
    def test_single_closed_rectangle_exact_mm(self):
        result = geometry_inventory(board(rectangle()))
        q = result["strict_line_only_outline_qualification"]
        self.assertEqual(q["status"], "VERIFIED_LINE_ONLY")
        self.assertEqual(q["dimensions_mm"]["width"], 40)
        self.assertEqual(q["dimensions_mm"]["height"], 10)
        self.assertEqual(q["shape_area_mm2"], 400)
        self.assertFalse(result["board_dimensions_mm_verified"])
        self.assertFalse(result["board_size_classification_verified"])

    def test_open_path_rejected(self):
        q = qualify_line_outline(rectangle()[:-1], 0)
        self.assertEqual(q["status"], "NOT_VERIFIED")

    def test_disconnected_two_closed_loops_rejected(self):
        q = qualify_line_outline(rectangle() + rectangle(60, 20, 65, 30), 0)
        self.assertEqual(q["status"], "NOT_VERIFIED")

    def test_arc_extrema_never_inferred_from_endpoints(self):
        p = rectangle()
        p[0] = ('(gr_arc (start 10 20) (mid 30 10) '
                '(end 50 20) (layer "Edge.Cuts"))')
        q = qualify_line_outline(p, 0)
        self.assertEqual(q["status"], "NOT_VERIFIED")
        self.assertIsNone(q["dimensions_mm"])

    def test_footprint_local_outline_rejected(self):
        q = qualify_line_outline(rectangle(), 1)
        self.assertEqual(q["status"], "NOT_VERIFIED")

    def test_duplicate_edge_and_zero_length_rejected(self):
        self.assertEqual(
            qualify_line_outline(rectangle() + [rectangle()[0]], 0)["status"],
            "NOT_VERIFIED")
        self.assertEqual(
            qualify_line_outline(rectangle() + [line((1, 1), (1, 1))], 0)["status"],
            "NOT_VERIFIED")

    def test_self_crossing_bowtie_rejected(self):
        p = [(0, 0), (10, 10), (0, 10), (10, 0)]
        q = qualify_line_outline([line(p[i], p[(i + 1) % 4]) for i in range(4)], 0)
        self.assertEqual(q["status"], "NOT_VERIFIED")

    def test_quoted_fake_primitives_are_not_counted(self):
        fake = '(gr_line (start 0 0) (end 999 999) (layer \\"Edge.Cuts\\"))'
        payload = board(rectangle(), '(gr_text "' + fake.replace('"', '\\"') + '")')
        result = geometry_inventory(payload)
        self.assertEqual(result["edge_cuts_graphic_primitives"], 4)
        self.assertEqual(result["strict_line_only_outline_qualification"]["status"],
                         "VERIFIED_LINE_ONLY")

    def test_quoted_scanner_and_global_balance(self):
        src = '(kicad_pcb (gr_text "(gr_line (start 0 0))") (gr_line (start 1 1)))'
        self.assertEqual(len(list(iter_sexpressions(src, "(gr_line"))), 1)
        with self.assertRaisesRegex(ValueError, "unbalanced"):
            list(iter_sexpressions("(gr_line (start 1 1)", "(gr_line"))


if __name__ == "__main__":
    unittest.main()
