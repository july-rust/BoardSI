#!/usr/bin/env python3
"""Independent synthetic regression for bounded four-term PH-08 decomposition."""
import unittest
from fractions import Fraction as F

from ph08_uniform_gradient_error_budget import analyze_uniform_synthetic


def fraction(value):
    return F(value)


class UniformGradientBudgetTests(unittest.TestCase):
    def test_all_three_grids_exact_identity_and_reference(self):
        for n in (8, 16, 32):
            with self.subTest(n=n):
                r = analyze_uniform_synthetic(n)
                d = r["exact_decomposition"]
                self.assertTrue(d["identity_verified_exactly"])
                self.assertTrue(d["reference_quadratic_reproduced"])
                self.assertTrue(d["both_constant_shift_null_exact"])
                self.assertEqual(fraction(d["reference_polynomial_defect_inf_exact"]), 0)
                self.assertEqual(fraction(d["operator_perturbation_error_inf_exact"]), 0)
                self.assertLessEqual(
                    fraction(d["observed_total_error_inf_exact"]),
                    fraction(d["triangle_upper_bound_exact"]))
                self.assertEqual(r["scope"], "PUBLIC_SYNTHETIC_UNIFORM_QUADRATIC_ONLY")
                self.assertFalse(r["original_HMM_source_executed"])
                self.assertEqual(r["historical_stage4_gate_q"], "FAIL")
                self.assertEqual(r["full_floating_forward_bound"], "BOUND_NOT_CERTIFIED")
                self.assertEqual(r["contract"], "NOT_FROZEN")

    def test_design_ulp_separately_isolated(self):
        d = analyze_uniform_synthetic(8, design_ulp=True)["exact_decomposition"]
        self.assertTrue(d["identity_verified_exactly"])
        self.assertGreater(fraction(d["operator_difference_norm_inf_exact"]), 0)
        self.assertTrue(d["reference_quadratic_reproduced"])
        self.assertEqual(fraction(d["reference_polynomial_defect_inf_exact"]), 0)

    def test_derivative_ulp_separately_isolated(self):
        d = analyze_uniform_synthetic(8, derivative_ulp=True)["exact_decomposition"]
        self.assertTrue(d["identity_verified_exactly"])
        self.assertGreater(fraction(d["operator_difference_norm_inf_exact"]), 0)

    def test_sample_ulp_perturbation(self):
        a = analyze_uniform_synthetic(8)["exact_decomposition"]
        b = analyze_uniform_synthetic(8, sample_ulp=True)["exact_decomposition"]
        self.assertTrue(b["identity_verified_exactly"])
        self.assertNotEqual(a["sample_error_inf_exact"], b["sample_error_inf_exact"]) if (
            "sample_error_inf_exact" in a
        ) else self.assertNotEqual(a["sample_components_exact"], b["sample_components_exact"])

    def test_multi_perturbation_triangle_and_centered_bounds(self):
        d = analyze_uniform_synthetic(
            16, design_ulp=True, derivative_ulp=True, sample_ulp=True
        )["exact_decomposition"]
        self.assertTrue(d["identity_verified_exactly"])
        self.assertTrue(d["both_constant_shift_null_exact"])
        self.assertLessEqual(
            fraction(d["operator_perturbation_error_inf_exact"]),
            fraction(d["centered_operator_upper_bound_exact"]))
        self.assertLessEqual(
            fraction(d["observed_total_error_inf_exact"]),
            fraction(d["triangle_upper_bound_exact"]))

    def test_reject_noninteger_or_unbounded_mesh(self):
        for value in (True, 8.0, 0, 2, 64, -8, "8"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                analyze_uniform_synthetic(value)

    def test_reject_nonboolean_switch(self):
        with self.assertRaises(ValueError):
            analyze_uniform_synthetic(8, sample_ulp=1)

    def test_no_physical_acceptance_on_perturbed_cases(self):
        for kwargs in ({}, {"design_ulp": True}, {"sample_ulp": True}):
            r = analyze_uniform_synthetic(8, **kwargs)
            self.assertEqual(r["historical_stage5b_gate_q"], "FAIL")
            self.assertEqual(r["Gate_M"], "NOT_VERIFIED")
            self.assertEqual(r["rust_fem"], "NO_GO")
            self.assertEqual(r["original_global_sparse_assembly"], "NOT_RUN")


if __name__ == "__main__":
    unittest.main()
