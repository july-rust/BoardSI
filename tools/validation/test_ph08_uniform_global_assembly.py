#!/usr/bin/env python3
"""Public-only global HMM synthetic regressions; never an original Gate Q PASS."""
import unittest
from fractions import Fraction as Q

from ph08_uniform_global_assembly import assemble, analyze_global_synthetic


class UniformGlobalAssemblyTests(unittest.TestCase):
    def test_complete_small_global_system_exact_candidate(self):
        for n in (2, 3, 4, 8):
            with self.subTest(n=n):
                r = analyze_global_synthetic(n)
                self.assertEqual(r["free_unknown_count"], 3*n*n - 2*n)
                self.assertEqual(r["exact_ideal_discrete_residual_inf"], "0/1")
                self.assertTrue(r["ideal_candidate_solves_assembled_system"])
                self.assertEqual(r["cell_count"], n*n)
                self.assertEqual(r["interior_face_count"], 2*n*(n-1))
                self.assertLess(r["float64_global_solve_gradient_rms_observed"], 1e-10)
                self.assertEqual(r["scope"], "PUBLIC_SYNTHETIC_GLOBAL_ASSEMBLY_ONLY")
                self.assertEqual(r["actual_original_hmm_full_domain"], "NOT_RUN")
                self.assertFalse(r["original_recovery_executed"])
                self.assertEqual(r["original_whole_rms_bound"], "BOUND_NOT_CERTIFIED")
                self.assertEqual(r["historical_stage4_gate_q"], "FAIL")
                self.assertEqual(r["historical_stage5b_gate_q"], "FAIL")
                self.assertEqual(r["TR"], "PROPOSED_NOT_FROZEN")
                self.assertEqual(r["contract"], "NOT_FROZEN")
                self.assertEqual(r["rust_fem"], "NO_GO")
                self.assertEqual(r["Gate_M"], "NOT_VERIFIED")

    def test_assembly_symmetric_with_positive_diagonal(self):
        for n in (2, 3, 8):
            keys, k, rhs, boundary = assemble(n)
            self.assertEqual(len(keys), len(k))
            self.assertEqual(len(rhs), len(k))
            self.assertTrue(boundary)
            for i, row in enumerate(k):
                self.assertGreater(row.get(i, Q(0)), 0)
                for j, value in row.items():
                    self.assertEqual(value, k[j].get(i, Q(0)))

    def test_exact_conditional_forward_enclosure_on_small_system(self):
        r = analyze_global_synthetic(2)
        e = r["conditional_exact_forward_enclosure"]
        self.assertIsNotNone(e)
        self.assertTrue(e["enclosure_verified"])
        self.assertEqual(e["scope"], "CONDITIONAL_EXACT_SYNTHETIC_K_B_ONLY")
        self.assertLessEqual(Q(e["observed_forward_inf_exact"]),
                             Q(e["forward_bound_inf_exact"]))
        self.assertEqual(r["physical_forward_bound"], "BOUND_NOT_CERTIFIED")
        self.assertIsNone(analyze_global_synthetic(3)[
            "conditional_exact_forward_enclosure"])

    def test_wrong_source_breaks_original_ideal_balance(self):
        r = analyze_global_synthetic(2, mutation="wrong_source")
        self.assertGreater(Q(r["exact_ideal_discrete_residual_inf"]), 0)
        self.assertFalse(r["ideal_candidate_solves_assembled_system"])
        self.assertIsNone(r["conditional_exact_forward_enclosure"])

    def test_midpoint_boundary_breaks_simpson_identity(self):
        r = analyze_global_synthetic(2, mutation="midpoint_boundary")
        self.assertGreater(Q(r["exact_ideal_discrete_residual_inf"]), 0)
        self.assertFalse(r["ideal_candidate_solves_assembled_system"])
        self.assertIsNone(r["conditional_exact_forward_enclosure"])

    def test_observed_solver_ulp_changes_exact_residual(self):
        base = analyze_global_synthetic(2)
        mutated = analyze_global_synthetic(2, mutation="solution_ulp")
        self.assertNotEqual(base["captured_solution_component_0_exact"],
                            mutated["captured_solution_component_0_exact"])
        self.assertTrue(mutated["ideal_candidate_solves_assembled_system"])
        self.assertTrue(mutated["conditional_exact_forward_enclosure"][
            "enclosure_verified"])

    def test_fail_closed_invalid_input_and_metadata(self):
        for bad in (0, 1, 16, True, 2.0, "2"):
            with self.subTest(value=bad), self.assertRaises(ValueError):
                analyze_global_synthetic(bad)
        for bad in ("", "original_hmm", False, None, 1):
            with self.subTest(mutation=bad), self.assertRaises(ValueError):
                analyze_global_synthetic(2, mutation=bad)
        with self.assertRaises(ValueError):
            assemble(2, source=-4)
        with self.assertRaises(ValueError):
            assemble(2, midpoint_boundary=1)


if __name__ == "__main__":
    unittest.main()
