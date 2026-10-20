"""F6 source metadata regression tests; all fixtures are fully synthetic."""
import hashlib
import unittest
from unittest import mock

from m0_f6_board_integrity import geometry_inventory, iter_sexpressions, verify


SYNTH = rb'''(kicad_pcb
  (version 20240108)
  (layers
    (0 "F.Cu" signal)
    (1 "In1.Cu" signal)
    (31 "B.Cu" signal)
    (44 "Edge.Cuts" user)
  )
  (net 0 "")
  (net 1 "GND")
  (net 2 "DATA")
  (gr_line (start 10 20) (end 50 20) (layer "Edge.Cuts"))
  (gr_arc (start 50 20) (mid 55 25) (end 50 30) (layer "Edge.Cuts"))
  (gr_line (start 50 30) (end 10 30) (layer "Edge.Cuts"))
  (gr_line (start 10 30) (end 10 20) (layer "Edge.Cuts"))
  (footprint "dummy"
    (at 1 2 90)
    (fp_line (start 0 0) (end 1 1) (layer "Edge.Cuts"))
    (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu")
      (net 2 "DATA"))
  )
  (via (at 11 22) (size 0.6) (drill 0.3) (layers "F.Cu" "B.Cu"))
  (segment (start 1 2) (end 3 4) (width 0.1) (layer "F.Cu") (net 2))
)'''


class TestPublicBoardInventory(unittest.TestCase):
    def test_metadata_not_board_certification(self):
        metrics = geometry_inventory(SYNTH)
        self.assertEqual(metrics["copper_layer_table_count_lexical"], 3)
        self.assertEqual(
            metrics["copper_layer_table_names_lexical"],
            ["F.Cu", "In1.Cu", "B.Cu"]
        )
        self.assertEqual(metrics["observed_unique_named_net_ids_lexical"], 2)
        self.assertEqual(metrics["count_net_assignment_occurrences_NOT_distinct_nets"], 4)
        self.assertEqual(metrics["edge_cuts_graphic_primitives"], 4)
        self.assertEqual(metrics["edge_cuts_types_lexical"], {"gr_arc": 1, "gr_line": 3})
        self.assertEqual(metrics["footprint_local_edge_cuts_primitives_lexical"], 1)
        self.assertEqual(metrics["count_pad_tokens"], 1)
        self.assertEqual(metrics["count_via_tokens"], 1)
        self.assertEqual(metrics["count_segment_tokens"], 1)
        self.assertEqual(metrics["edge_endpoints_extent_mm_DIAGNOSTIC_ONLY"]["delta_x"], 45)
        self.assertFalse(metrics["edge_extent_certified"])
        self.assertFalse(metrics["board_dimensions_mm_verified"])
        self.assertFalse(metrics["board_size_classification_verified"])

    def test_inconsistent_net_names_fail_closed(self):
        # Keep one syntactically valid KiCad root so semantic net-ID
        # conflict rejection, not the global syntax guard, is exercised.
        corrupted = SYNTH[:-1] + b'\n(net 2 "CONFLICT")\n)'
        with self.assertRaisesRegex(ValueError, "Conflicting observed"):
            geometry_inventory(corrupted)

    def test_unterminated_named_net_fails_before_regex(self):
        # The previous regex would skip this malformed conflicting declaration.
        damaged = SYNTH[:-1] + b'\n(net 2 "CONFLICT))'
        with self.assertRaisesRegex(ValueError, "unterminated quoted"):
            geometry_inventory(damaged)

    def test_malformed_global_parentheses_fail_before_inventory(self):
        with self.assertRaisesRegex(ValueError, "unbalanced"):
            geometry_inventory(SYNTH[:-1])
        with self.assertRaisesRegex(ValueError, "multiple roots"):
            geometry_inventory(SYNTH + b"\n(net 3 \"X\")")

    def test_duplicate_copper_table_indices_fail_closed(self):
        corrupted = SYNTH.replace(b'(1 "In1.Cu" signal)', b'(0 "In1.Cu" signal)')
        with self.assertRaisesRegex(ValueError, "Duplicate copper"):
            geometry_inventory(corrupted)

    def test_balanced_scanner_handles_escaped_quote(self):
        a = '(gr_line (name "escaped \\" (text)") (layer "Edge.Cuts"))'
        self.assertEqual(list(iter_sexpressions(a, "(gr_")), [a])
        with self.assertRaisesRegex(ValueError, "unbalanced"):
            list(iter_sexpressions("(gr_line (start 1 2)", "(gr_"))

    def test_license_family_is_not_redistribution_approval(self):
        lic = b"CERN Open Hardware Licence Version 2 - Permissive\n"
        raw = SYNTH
        entry = {
            "name": "RoyalBlue54L-Feather", "path": "source.kicad_pcb",
            "license": "LICENSE", "bytes": len(raw),
            "sha256": hashlib.sha256(raw).hexdigest(),
            "git_blob_sha1": hashlib.sha1(
                b"blob " + str(len(raw)).encode("ascii") + b"\0" + raw
            ).hexdigest(),
            "license_marker": "CERN",
        }
        with mock.patch("m0_f6_board_integrity.download",
                        side_effect=[raw, lic]):
            result = verify(entry)
        self.assertTrue(result["integrity_pass"])
        self.assertEqual(result["redistribution_legal_clearance"], "NOT_VERIFIED")
        self.assertFalse(result["redistribution_approved"])

    def test_wrong_license_family_rejected(self):
        raw = SYNTH
        entry = {
            "name": "TinyTapeout-demo", "path": "source.kicad_pcb",
            "license": "LICENSE", "bytes": len(raw),
            "sha256": hashlib.sha256(raw).hexdigest(),
            "git_blob_sha1": hashlib.sha1(
                b"blob " + str(len(raw)).encode("ascii") + b"\0" + raw
            ).hexdigest(),
            "license_marker": "Apache",
        }
        with mock.patch("m0_f6_board_integrity.download",
                        side_effect=[raw, b"Apache unrelated sample"]):
            with self.assertRaisesRegex(ValueError, "license family"):
                verify(entry)


if __name__ == "__main__":
    unittest.main()
