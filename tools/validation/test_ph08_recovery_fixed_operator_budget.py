#!/usr/bin/env python3
"""Public synthetic tests only; NOT an original-HMM or physical certification."""
import unittest
from fractions import Fraction as Q

from ph08_recovery_fixed_operator_budget import analyze_fixed_recovery


def fixture():
    pts = [(0, 0), (1, 0), (0, 1), (-1, 0),
           (0, -1), (1, 1), (-1, -1), (1, -1)]
    a = [[float(z) for z in (1, u, v, u*u, u*v, v*v)]
         for u, v in pts]
    # Off-origin physical-gradient D at (u,v)=(1/2,1/4),
    # with physical scales sx=2 and sz=4.
    d = [[0., .5, 0., .5, .125, 0.],
         [0., 0., .25, 0., .125, .125]]
    coeff = [Q(j) for j in (1, 2, 3, 4, 5, 6)]
    y = [sum((Q(int(z)) * c for z, c in zip(row, coeff)), Q(0))
         for row in a]
    g = [sum((Q(z) * c for z, c in zip(row, coeff)), Q(0))
         for row in d]
    return a, d, y, g, coeff


def run_case(yn=None, gr=None, a=None, d=None, yr=None, gp=None, coeff=None):
    aa, dd, yy, gg, cc = fixture()
    return analyze_fixed_recovery(
        aa if a is None else a, dd if d is None else d,
        [float(v) for v in yy] if yn is None else yn,
        yy if yr is None else yr,
        gg if gp is None else gp,
        [float(v) for v in gg] if gr is None else gr,
        cc if coeff is None else coeff,
    )


class FixedOperatorTests(unittest.TestCase):
    def test_exact_quadratic_off_origin(self):
        r = run_case()
        self.assertTrue(r["quadratic_reproduced_for_supplied_exact_reference"])
        self.assertTrue(r["identity_verified_exactly"])
        self.assertTrue(r["constant_shift_null_exact"])
        for k in ("polynomial_reconstruction_defect_inf_exact",
                  "arithmetic_output_error_inf_exact",
                  "sample_error_inf_exact", "observed_total_error_inf_exact"):
            self.assertEqual(r[k], "0/1")

    def test_sample_perturbation_is_not_silent(self):
        _, _, y, _, _ = fixture()
        yn = [float(v) for v in y]
        yn[2] += 1. / 1024
        r = run_case(yn=yn)
        self.assertNotEqual(r["sample_error_inf_exact"], "0/1")
        self.assertNotEqual(r["arithmetic_output_error_inf_exact"], "0/1")
        self.assertEqual(r["observed_total_error_inf_exact"], "0/1")
        self.assertEqual(r["physical_error_bound"], "BOUND_NOT_CERTIFIED")

    def test_exact_constant_shift_is_annihilated(self):
        _, _, y, _, _ = fixture()
        r = run_case(yn=[float(v + 2) for v in y])
        self.assertTrue(r["constant_shift_null_exact"])
        self.assertEqual(r["sample_error_inf_exact"], "0/1")
        self.assertNotEqual(r["sample_input_difference_inf_exact"], "0/1")

    def test_observed_output_error_is_separate(self):
        _, _, _, g, _ = fixture()
        gr = [float(g[0] + Q(1, 1024)), float(g[1])]
        r = run_case(gr=gr)
        self.assertEqual(r["observed_total_error_inf_exact"], "1/1024")
        self.assertEqual(r["arithmetic_output_error_inf_exact"], "1/1024")
        self.assertEqual(r["physical_gate_q"], "FAIL")

    def test_reference_data_not_implicitly_cast_from_float(self):
        _, _, y, _, _ = fixture()
        y[0] = float(y[0])
        with self.assertRaisesRegex(ValueError, "Fraction"):
            run_case(yr=y)

    def test_bad_gradient_reference_type(self):
        _, _, _, g, _ = fixture()
        with self.assertRaises(ValueError):
            run_case(gp=[float(g[0]), g[1]])

    def test_no_false_polynomial_reproduction(self):
        a, d, y, g, c = fixture()
        bad_g = [g[0] + Q(1, 1024), g[1]]
        r = run_case(gp=bad_g)
        self.assertFalse(r["quadratic_reproduced_for_supplied_exact_reference"])
        self.assertEqual(r["polynomial_reconstruction_defect_inf_exact"], "1/1024")

    def test_rank_deficiency_rejected(self):
        a, _, _, _, _ = fixture()
        a = [[1., 0., 0., 0., 0., 0.] for _ in a]
        with self.assertRaises(ValueError):
            run_case(a=a)

    def test_nan_in_design_refused(self):
        a, _, _, _, _ = fixture()
        a[0][0] = float("nan")
        with self.assertRaises(ValueError):
            run_case(a=a)

    def test_infinite_observed_gradient_refused(self):
        with self.assertRaises(ValueError):
            run_case(gr=[float("inf"), 0.])

    def test_truncated_design_refused(self):
        a, _, _, _, _ = fixture()
        with self.assertRaises(ValueError):
            run_case(a=a[:5])

    def test_oversize_reference_rejected(self):
        _, _, y, _, _ = fixture()
        y[0] = Q(2**4100)
        with self.assertRaises(ValueError):
            run_case(yr=y)

    def test_no_physical_acceptance_from_tight_triangle(self):
        r = run_case()
        self.assertEqual(r["physical_gate_q"], "FAIL")
        self.assertEqual(r["contract"], "NOT_FROZEN")
        self.assertEqual(r["rust_fem"], "NO_GO")
        self.assertEqual(r["Gate_M"], "NOT_VERIFIED")
        self.assertEqual(r["reference_provenance"],
                         "CALLER_SUPPLIED_NOT_INDEPENDENTLY_AUTHENTICATED")


if __name__ == "__main__":
    unittest.main()
