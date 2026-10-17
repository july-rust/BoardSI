#!/usr/bin/env python3
"""Public synthetic PH-08 two-operator perturbation tests. Never original HMM."""
import unittest
from fractions import Fraction as Q
from itertools import repeat
from pathlib import Path
from ph08_recovery_operator_perturbation import analyze_two_operator_budget

def fixture():
    xy = [(0, 0), (1, 0), (0, 1), (-1, 0),
          (0, -1), (1, 1), (-1, -1), (1, -1)]
    ar = [[Q(1), Q(u), Q(v), Q(u*u), Q(u*v), Q(v*v)]
          for u, v in xy]
    dr = [[Q(0), Q(1,2), Q(0), Q(1,2), Q(1,8), Q(0)],
          [Q(0), Q(0), Q(1,4), Q(0), Q(1,8), Q(1,8)]]
    coeff = [Q(c) for c in (32, 2, 3, 4, 5, 6)]
    yr = [sum((a*b for a,b in zip(row, coeff)), Q(0)) for row in ar]
    gr = [sum((a*b for a,b in zip(row, coeff)), Q(0)) for row in dr]
    ac = [[float(x) for x in row] for row in ar]
    dc = [[float(x) for x in row] for row in dr]
    return ac, dc, ar, dr, yr, gr, coeff

def run(**changes):
    a, d, ar, dr, yr, gr, cr = fixture()
    inputs = dict(captured_A=a, captured_D=d, reference_A=ar,
                  reference_D=dr, numeric_samples=[float(v) for v in yr],
                  reference_samples=yr, reference_gradient=gr,
                  observed_gradient=[float(v) for v in gr],
                  reference_quadratic_coefficients=cr)
    inputs.update(changes)
    return analyze_two_operator_budget(**inputs)

class TwoOperatorBudgetTests(unittest.TestCase):
    def test_exact_quadratic_zero_error(self):
        r = run()
        self.assertTrue(r["identity_verified_exactly"])
        self.assertTrue(r["reference_quadratic_reproduced"])
        self.assertTrue(r["both_constant_shift_null_exact"])
        for key in ("arithmetic_error_inf_exact", "sample_error_inf_exact",
                    "operator_perturbation_error_inf_exact",
                    "reference_polynomial_defect_inf_exact",
                    "observed_total_error_inf_exact"):
            self.assertEqual(r[key], "0/1")

    def test_captured_A_perturbation_not_disguised_as_sample_error(self):
        a, _, _, _, _, _, _ = fixture()
        a[0][1] += 1/256
        r = run(captured_A=a)
        self.assertGreater(Q(r["operator_perturbation_error_inf_exact"]), 0)
        self.assertEqual(r["sample_error_inf_exact"], "0/1")
        self.assertEqual(r["observed_total_error_inf_exact"], "0/1")

    def test_captured_D_perturbation_detected(self):
        _, d, _, _, _, _, _ = fixture()
        d[0][3] += 1/64
        r = run(captured_D=d)
        self.assertGreater(Q(r["operator_perturbation_error_inf_exact"]), 0)
        self.assertGreater(Q(r["operator_difference_norm_inf_exact"]), 0)

    def test_nonuniform_sample_error_not_hidden(self):
        _, _, _, _, y, _, _ = fixture()
        yn = [float(v) for v in y]
        yn[2] += 1/128
        r = run(numeric_samples=yn)
        self.assertGreater(Q(r["sample_error_inf_exact"]), 0)
        self.assertEqual(r["operator_perturbation_error_inf_exact"], "0/1")

    def test_exact_dyadic_common_offset_is_null(self):
        _, _, _, _, y, _, _ = fixture()
        yn = [float(v-Q(1,64)) for v in y]
        r = run(numeric_samples=yn)
        self.assertEqual(r["sample_error_inf_exact"], "0/1")
        self.assertTrue(r["both_constant_shift_null_exact"])

    def test_h_squared_over_six_not_a_floating_point_proof(self):
        # h=1/8, h^2/6=1/384. Synthetic, not original HMM theorem.
        _, _, _, _, y, _, _ = fixture()
        yn = [float(v-Q(1,384)) for v in y]
        r = run(numeric_samples=yn)
        self.assertTrue(r["both_constant_shift_null_exact"])
        self.assertEqual(r["original_HMM_uniform_h2_over_6_theorem"], "NOT_DERIVED")
        self.assertEqual(r["physical_error_bound"], "BOUND_NOT_CERTIFIED")

    def test_midrange_operator_bound_not_worse_than_naive(self):
        a, _, _, _, _, _, _ = fixture()
        a[0][1] += 1/256
        r = run(captured_A=a)
        self.assertGreater(Q(r["operator_perturbation_error_inf_exact"]), 0)
        self.assertLessEqual(Q(r["centered_operator_upper_bound_exact"]),
                             Q(r["operator_upper_bound_exact"]))
        self.assertLessEqual(Q(r["operator_perturbation_error_inf_exact"]),
                             Q(r["centered_operator_upper_bound_exact"]))

    def test_constant_reference_shift_invariance(self):
        a, _, _, _, yr, _, c = fixture()
        a[0][1] += 1/256
        r0 = run(captured_A=a)
        shift = Q(256)
        yy = [v+shift for v in yr]
        cc = c[:]
        cc[0] += shift
        r1 = run(captured_A=a, reference_samples=yy,
                 numeric_samples=[float(v) for v in yy],
                 reference_quadratic_coefficients=cc)
        self.assertEqual(r0["operator_components_exact"], r1["operator_components_exact"])
        self.assertEqual(r0["centered_operator_upper_bound_exact"],
                         r1["centered_operator_upper_bound_exact"])
        self.assertTrue(r1["reference_quadratic_reproduced"])

    def test_sign_and_axis_mutations_not_physical_pass(self):
        _, _, _, _, _, gr, _ = fixture()
        for mutated in ([-float(v) for v in gr],
                        [float(gr[1]), float(gr[0])]):
            with self.subTest(g=mutated):
                r = run(observed_gradient=mutated)
                self.assertGreater(Q(r["observed_total_error_inf_exact"]), 0)
                self.assertEqual(r["physical_gate_q"], "FAIL")

    def test_reference_polynomial_defect_caught(self):
        _, _, _, _, _, gr, _ = fixture()
        r = run(reference_gradient=[gr[0]+Q(1,256), gr[1]])
        self.assertEqual(r["reference_polynomial_defect_inf_exact"], "1/256")
        self.assertFalse(r["reference_quadratic_reproduced"])

    def test_rank_deficiency_both_paths(self):
        ac, _, ar, _, _, _, _ = fixture()
        with self.assertRaises(ValueError):
            run(captured_A=[[1.,0.,0.,0.,0.,0.]]*len(ac))
        with self.assertRaises(ValueError):
            run(reference_A=[[Q(1),Q(0),Q(0),Q(0),Q(0),Q(0)]]*len(ar))

    def test_nonfinite_and_wrong_reference_types_rejected(self):
        ac, _, ar, dr, _, _, _ = fixture()
        ac[0][0] = float("inf")
        with self.assertRaises(ValueError):
            run(captured_A=ac)
        ar[0][0] = 1.0
        with self.assertRaises(ValueError):
            run(reference_A=ar)
        dr[1][3] = Q(2**4100)
        with self.assertRaises(ValueError):
            run(reference_D=dr)

    def test_unbounded_iterables_and_shapes_rejected(self):
        a, d, ar, dr, _, _, _ = fixture()
        for changed in (dict(captured_A=repeat(a[0])),
                        dict(reference_A=repeat(ar[0])),
                        dict(captured_D=repeat(d[0])),
                        dict(reference_D=[dr[0]]),
                        dict(reference_samples=repeat(Q(1))),
                        dict(captured_A=[repeat(1.)]+a[1:])):
            with self.subTest(keys=list(changed)), self.assertRaises(ValueError):
                run(**changed)

    def test_distinct_large_denominators_hit_aggregate_budget(self):
        ac, _, ar, _, yr, _, _ = fixture()
        aa = [r[:] for r in ac] * 2
        refs = [r[:] for r in ar] * 2
        for i, row in enumerate(refs):
            row[0] = Q(1, 2**(512+i))
        y2 = yr + yr
        with self.assertRaisesRegex(ValueError, "aggregate exact input"):
            run(captured_A=aa, reference_A=refs,
                numeric_samples=[float(v) for v in y2],
                reference_samples=y2)

    def test_optional_workflow_tracks_shared_exact_helper(self):
        workflow = (Path(__file__).resolve().parents[2] /
                    ".github/workflows/m2-ph08-two-operator.yml").read_text()
        self.assertIn('"tools/validation/ph08_gateq_exact_captured.py"', workflow)

    def test_explicit_no_original_physics_certification(self):
        r = run()
        self.assertEqual(r["reference_provenance"],
                         "CALLER_SUPPLIED_NOT_INDEPENDENTLY_AUTHENTICATED")
        self.assertEqual(r["actual_original_A_D_assembly"], "NOT_VERIFIED")
        self.assertEqual(r["whole_cell_quantified_bounds"], "BOUND_NOT_CERTIFIED")
        self.assertEqual(r["historical_gate_q"], "FAIL")
        self.assertEqual(r["AC_P1"], "PENDING")
        self.assertEqual(r["contract"], "NOT_FROZEN")
        self.assertEqual(r["rust_fem"], "NO_GO")
        self.assertEqual(r["Gate_M"], "NOT_VERIFIED")

if __name__ == "__main__":
    unittest.main()
