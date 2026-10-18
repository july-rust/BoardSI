#!/usr/bin/env python3
"""PUBLIC SYNTHETIC exact-identity tests. These never execute original HMM."""
import unittest
from fractions import Fraction as Q

from ph08_uniform_quadratic_identity import MAX_N, verify_square


class UniformQuadraticIdentityTests(unittest.TestCase):
    def test_exact_interior_boundary_and_gradient_identities(self):
        for n in (2, 4, 8, 16, 32):
            with self.subTest(n=n):
                result = verify_square(n)
                self.assertEqual(result["cell_count"], n*n)
                self.assertEqual(result["interior_face_count"], 2*n*(n-1))
                self.assertEqual(result["expected_cell_bias"], -Q(1, 6*n*n))
                self.assertEqual(result["expected_face_bias"], Q(1, 12*n*n))
                for field in ("cell_equation_residual_inf",
                              "interior_face_residual_inf",
                              "boundary_mean_residual_inf",
                              "quadratic_gradient_error_inf",
                              "cell_shift_error_inf",
                              "face_shift_error_inf"):
                    self.assertEqual(result[field], 0, field)

    def test_no_cell_shift_rejected_by_local_equations(self):
        r = verify_square(8, cell_bias=Q(0))
        self.assertNotEqual(r["cell_equation_residual_inf"], 0)
        self.assertNotEqual(r["cell_shift_error_inf"], 0)

    def test_wrong_signed_cell_shift_rejected(self):
        r = verify_square(8, cell_bias=Q(1, 384))
        self.assertGreater(r["cell_equation_residual_inf"], 0)

    def test_midpoint_faces_violate_simpson_boundary(self):
        r = verify_square(8, face_bias=Q(0))
        self.assertGreater(r["boundary_mean_residual_inf"], 0)
        self.assertGreater(r["face_shift_error_inf"], 0)

    def test_wrong_source_detected(self):
        r = verify_square(8, source=Q(4))
        self.assertGreater(r["cell_equation_residual_inf"], 0)

    def test_wrong_boundary_oracle_detected(self):
        r = verify_square(8, midpoint_boundary=True)
        self.assertGreater(r["boundary_mean_residual_inf"], 0)

    def test_bounded_input_and_fraction_only(self):
        for bad in (True, False, 0, 1, MAX_N+1, 2.0, "2"):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                verify_square(bad)
        with self.assertRaises(ValueError):
            verify_square(2, cell_bias=0.0)
        with self.assertRaises(ValueError):
            verify_square(2, face_bias=0)
        with self.assertRaises(ValueError):
            verify_square(2, source=-4)

    def test_scope_fail_closed(self):
        r = verify_square(8)
        self.assertEqual(r["scope"], "IDEAL_EXACT_UNIFORM_SQUARE_SYNTHETIC_ONLY")
        self.assertEqual(r["historical_gate_q"], "FAIL")
        self.assertEqual(r["original_float_replay"], "NOT_RUN")
        self.assertEqual(r["original_HMM_full_domain_error_bound"],
                         "BOUND_NOT_CERTIFIED")
        self.assertEqual(r["contract"], "NOT_FROZEN")
        self.assertEqual(r["rust_fem"], "NO_GO")


if __name__ == "__main__":
    unittest.main()
