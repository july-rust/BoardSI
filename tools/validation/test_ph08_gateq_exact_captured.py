#!/usr/bin/env python3
"""GitHub Actions regression cases for strictly scoped PH-08 captured arithmetic."""
import math
import json
import tempfile
import sys
import unittest
from pathlib import Path
from fractions import Fraction as F
sys.path.insert(0, str(Path(__file__).resolve().parent))
import numpy as np
import mpmath as mp
from ph08_gateq_exact_captured import (
    binary64, exact_captured_system, prereg_hash, judge_tr, historical_gate_q,
    judge_tr_registered, REGISTERED_SYNTHETIC_POLICY_SHA256, MAX_CAPTURE_ROWS,
)


class CapturedBoundAndTRTests(unittest.TestCase):
    def setUp(self):
        points = [(float(x)/8, float(y)/8) for y in (-1, 0, 1)
                  for x in (-1, 0, 1)]
        self.A = [[1.0, u, v, u*u, u*v, v*v] for u,v in points]
        self.y = [2.0 + .75*u - .125*v + .5*u*u + .25*u*v
                  - .375*v*v for u,v in points]
        self.D = [[0.0, 1.0, 0.0, .25, .375, 0.0],
                  [0.0, 0.0, 1.0, 0.0, .125, .75]]
        self.c = np.linalg.lstsq(np.array(self.A), np.array(self.y), rcond=None)[0].tolist()
        self.g = (np.array(self.D) @ np.array(self.c)).tolist()
        self.config = {"version": 1, "t_min_reduction": 1.8,
                       "cases": {"original_uniform_quadratic": {"regime": "T"},
                                 "original_skew_cubic": {"regime": "T"},
                                 "proposed_roundoff_demo": {"regime": "R"}}}
        self.pin = prereg_hash(self.config)

    def result(self, **changes):
        k = dict(A=self.A, y=self.y, D=self.D,
                 observed_c=self.c, observed_g=self.g)
        k.update(changes)
        return exact_captured_system(**k)

    def test_original_history_is_fail(self):
        h=historical_gate_q()
        self.assertEqual(h["status"], "FAIL")
        self.assertEqual(h["ratios"], (0.953506146487913, 0.45916314091601546))

    def test_reference_exact_fractions_not_prematurely_rounded(self):
        o=self.result()
        self.assertEqual(o["scope"], "EXACT_CAPTURED_BINARY64_ALGEBRA_ONLY")
        self.assertEqual(o["rank"], 6)
        self.assertEqual(o["physical_error_bound"], "BOUND_NOT_CERTIFIED")
        self.assertEqual(o["physical_gate_q"], "FAIL")
        self.assertTrue(o["constant_shift_null_exact"])
        self.assertGreater(F(o["coefficient_forward_error_inf_exact"]), 0)
        self.assertGreaterEqual(F(o["gradient_forward_error_inf_exact"]), 0)

    def test_subulp_rational_difference_not_erased(self):
        o=self.result()
        self.assertGreater(F(o["coefficient_forward_error_inf_exact"]), 0)
        self.assertNotEqual(o["coefficient_forward_error_inf_exact"], "0/1")

    def test_reference_gradient_uses_off_origin_D(self):
        o=self.result()
        self.assertNotEqual(F(o["exact_gradient"][0]), F(o["exact_coefficients"][1]))
        self.assertNotEqual(F(o["exact_gradient"][1]), F(o["exact_coefficients"][2]))
        changed=self.D.copy()
        changed=[row[:] for row in changed]
        changed[0][3] *= 2
        self.assertNotEqual(o["exact_gradient"], self.result(D=changed)["exact_gradient"])

    def test_numpy_lstsq_against_independent_120_digit_reference(self):
        o=self.result()
        mp.mp.dps=120
        def M(x): 
            q=binary64(x)
            return mp.mpf(q.numerator)/mp.mpf(q.denominator)
        a=mp.matrix([[M(v) for v in row] for row in self.A])
        y=mp.matrix([M(v) for v in self.y])
        c=mp.lu_solve(a.T*a, a.T*y)
        for got, expect in zip(o["exact_coefficients"], c):
            q=F(got)
            delta=abs(mp.mpf(q.numerator)/mp.mpf(q.denominator)-expect)
            self.assertLess(delta,mp.mpf("1e-95"))

    def test_primal_vs_system_residual_separate(self):
        o=self.result()
        self.assertGreaterEqual(F(o["primal_residual_inf_exact"]), 0)
        self.assertGreaterEqual(F(o["normal_equations_residual_inf_exact"]), 0)
        self.assertGreaterEqual(F(o["normal_equations_backward_error_exact"]), 0)

    def test_constant_potential_shifts_affect_constant_coeff_only(self):
        shifted=[v+1.0 for v in self.y]
        a=self.result()
        b=self.result(y=shifted)
        # Since y are binary64, this comparison is a diagnostic, not a physical identity
        self.assertTrue(a["constant_shift_null_exact"])
        self.assertTrue(b["constant_shift_null_exact"])

    def test_rank_deficiency_fails_not_passes(self):
        with self.assertRaises(ValueError):
            self.result(A=[[1.0]*6 for _ in range(9)])

    def test_nan_inf_or_wrong_numeric_kind_fails(self):
        for bad in (float("nan"),float("inf"),-float("inf"),1,"1",True):
            with self.subTest(bad=str(bad)), self.assertRaises(ValueError):
                self.result(y=[bad]+self.y[1:])

    def test_missing_wrong_dimensions_fails(self):
        for change in (dict(y=self.y[:-1]), dict(D=self.D[:1]),
                       dict(A=[row[:5] for row in self.A]),
                       dict(observed_c=self.c[:5]),dict(observed_g=self.g[:1])):
            with self.subTest(change=list(change)), self.assertRaises(ValueError):
                self.result(**change)

    def test_t_historical_fail_never_upgraded(self):
        r=judge_tr(self.config,self.pin,"original_uniform_quadratic",
                   reductions=[0.953506146487913, 0.45916314091601546])
        self.assertEqual(r["status"],"FAIL")
        self.assertEqual(r["physical_gate_q"],"FAIL")

    def test_t_only_pass_not_physical_pass(self):
        r=judge_tr(self.config,self.pin,"original_skew_cubic", reductions=[2.0,2.1])
        self.assertEqual(r["status"],"PASS")
        self.assertEqual(r["scope"],"T_RATE_ONLY_NOT_PHYSICAL_APPROVAL")
        self.assertEqual(r["physical_gate_q"],"FAIL")

    def test_hash_mutation_blocks_ex_post_rule_edits(self):
        k=dict(self.config,cases=dict(self.config["cases"]))
        k["cases"]["original_uniform_quadratic"]={"regime":"R"}
        r=judge_tr(k,self.pin,"original_uniform_quadratic")
        self.assertEqual(r["status"],"FAIL")

    def test_unregistered_case_and_bad_reductions_fail(self):
        self.assertEqual(judge_tr(self.config,self.pin,"unknown")["status"],"FAIL")
        for values in ([float("nan"),2.0], [float("inf"),2.0], [1.9],
                       ["2.0",2.0], [-1.0,2.0]):
            with self.subTest(v=values):
                self.assertEqual(judge_tr(self.config,self.pin,"original_skew_cubic",values)["status"],"FAIL")

    def test_R_without_certificates_indeterminate(self):
        r=judge_tr(self.config,self.pin,"proposed_roundoff_demo")
        self.assertEqual(r["status"],"INDETERMINATE")
        self.assertEqual(r["float_physical_bound"],"BOUND_NOT_CERTIFIED")

    def test_R_reduction_leak_is_fail(self):
        r=judge_tr(self.config,self.pin,"proposed_roundoff_demo",reductions=[2.0,1.0])
        self.assertEqual(r["status"],"FAIL")

    def test_R_self_asserted_certificates_still_not_verification(self):
        evidence={k:True for k in (
            "original_operator_verified","exact_reference_verified",
            "binary64_assembly_bound_certified","solver_bound_certified",
            "input_error_bound_certified","gradient_transfer_bound_certified",
            "physical_gradient_bound_certified","negative_controls_verified")}
        r=judge_tr(self.config,self.pin,"proposed_roundoff_demo",r_inputs=evidence)
        self.assertEqual(r["status"],"NOT VERIFIED")
        self.assertEqual(r["physical_gate_q"],"FAIL")


    def test_independent_high_precision_qr_direct_rectangular_ls(self):
        # QR works on the tall captured A directly; not a second LU solve
        # of the same normal equations. This remains a numerical crosscheck.
        mp.mp.dps = 120
        def m(v):
            q = binary64(v)
            return mp.mpf(q.numerator) / mp.mpf(q.denominator)
        mat = mp.matrix([[m(v) for v in row] for row in self.A])
        target = mp.matrix([m(v) for v in self.y])
        qr_c, qr_residual = mp.qr_solve(mat, target)
        exact = self.result()
        for expected, got in zip(exact["exact_coefficients"], qr_c):
            q = F(expected)
            difference = abs(mp.mpf(q.numerator)/mp.mpf(q.denominator) - got)
            self.assertLess(difference, mp.mpf("1e-95"))
        self.assertTrue(mp.isfinite(qr_residual))

    def test_constant_binary64_shift_direct_exact_gradient_invariance(self):
        initial = self.result()
        shifted = self.result(y=[v + 1.0 for v in self.y])
        self.assertEqual(initial["exact_gradient"], shifted["exact_gradient"])
        self.assertEqual(initial["constant_shift_gradient_delta_inf_exact"], "0/1")
        self.assertEqual(shifted["constant_shift_gradient_delta_inf_exact"], "0/1")
        self.assertNotEqual(initial["exact_coefficients"][0],
                            shifted["exact_coefficients"][0])

    def test_nonconstant_shift_is_not_annihilated(self):
        original = self.result()
        changed = self.y[:]
        changed[0] += 0.125
        new = self.result(y=changed)
        self.assertNotEqual(original["exact_gradient"], new["exact_gradient"])

    def test_g_observed_is_independent_not_recomputed_inside_reference(self):
        reference = self.result()
        altered = self.result(observed_g=[self.g[0] + 0.125, self.g[1]])
        self.assertGreater(F(altered["gradient_forward_error_inf_exact"]),
                           F(reference["gradient_forward_error_inf_exact"]))

    def test_conditional_capture_scope_and_backward_error_are_separate(self):
        result = self.result()
        self.assertEqual(result["least_squares_backward_error_bound"], "NOT_CERTIFIED")
        self.assertEqual(result["normal_equations_residual_interpretation"],
                         "BACKWARD_RESIDUAL_OF_NORMAL_SYSTEM_ONLY")
        self.assertEqual(result["A_D_input_perturbation_bound"], "NOT_CERTIFIED")
        self.assertEqual(result["gradient_evaluation_position"],
                         "CAPTURED_IN_D_NOT_INDEPENDENTLY_ATTESTED")
        self.assertIn("E = -D*c", result["gradient_definition"])

    def test_near_singular_full_rank_is_diagnostic_not_certified(self):
        # Make the last column almost dependent on the fourth, while the
        # fixed exact binary64 matrix retains rank six.
        near = [row[:] for row in self.A]
        for row in near:
            row[5] = row[3] + (2.0**-32) * row[5]
        result = self.result(A=near)
        self.assertEqual(result["rank"], 6)
        self.assertEqual(result["operator_cond2_bound"], "NOT_CERTIFIED")
        self.assertEqual(result["physical_error_bound"], "BOUND_NOT_CERTIFIED")

    def test_resource_ceiling_rejects_unbounded_capture(self):
        with self.assertRaises(ValueError):
            self.result(A=[self.A[0]]*(MAX_CAPTURE_ROWS+1),
                        y=[self.y[0]]*(MAX_CAPTURE_ROWS+1))

    def test_pinned_synthetic_fixture_T_never_certifies_original(self):
        result = judge_tr_registered("synthetic_uniform_quadratic_T",
                                     reductions=[2.0, 2.1])
        self.assertEqual(result["status"], "PASS")
        self.assertEqual(result["rate_check_status"], "PASS")
        self.assertEqual(result["certification_status"], "NOT VERIFIED")
        self.assertEqual(result["physical_gate_q"], "FAIL")
        self.assertEqual(result["historical_gate_q"], "FAIL")
        self.assertEqual(result["registered_policy_sha256"],
                         REGISTERED_SYNTHETIC_POLICY_SHA256)
        self.assertEqual(result["registration_status"],
                         "PROPOSED_ONLY_NOT_ORIGINAL_PREREGISTERED")

    def test_caller_recomputed_hash_cannot_rewrite_independent_pin(self):
        path = (Path(__file__).resolve().parents[2]
                / "docs/validation/ph08-gateq-tr-synthetic-prereg-v1.json")
        config = json.loads(path.read_text())
        changed = dict(config, cases=dict(config["cases"]))
        changed["cases"]["synthetic_uniform_quadratic_T"] = dict(
            changed["cases"]["synthetic_uniform_quadratic_T"], regime="R")
        self.assertNotEqual(prereg_hash(changed), REGISTERED_SYNTHETIC_POLICY_SHA256)
        # Legacy caller-controlled hash can be rewritten, hence is not an
        # external preregistration. The pinned path must refuse it.
        with tempfile.TemporaryDirectory() as d:
            altered = Path(d) / "policy.json"
            altered.write_text(json.dumps(changed))
            outcome = judge_tr_registered("synthetic_uniform_quadratic_T",
                                          policy_path=altered)
        self.assertEqual(outcome["status"], "FAIL")
        self.assertEqual(outcome["certification_status"], "NOT VERIFIED")

    def test_pinned_policy_missing_and_bad_json_fail_closed(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "policy.json"
            self.assertEqual(judge_tr_registered("synthetic_uniform_quadratic_T",
                                                 policy_path=path)["status"], "FAIL")
            path.write_text("{bad")
            self.assertEqual(judge_tr_registered("synthetic_uniform_quadratic_T",
                                                 policy_path=path)["status"], "FAIL")

    def test_original_and_unknown_case_not_registered_for_evaluation(self):
        for name in ("original_uniform_quadratic", "original_skew_cubic",
                     "original_stage5b", "unregistered"):
            with self.subTest(case=name):
                result = judge_tr_registered(name, reductions=[3.0, 3.0])
                self.assertEqual(result["status"], "FAIL")
                self.assertEqual(result["physical_gate_q"], "FAIL")

    def test_pinned_R_all_self_declarations_never_pass(self):
        claimed = {k: True for k in (
            "original_operator_verified", "exact_reference_verified",
            "binary64_assembly_bound_certified", "solver_bound_certified",
            "input_error_bound_certified", "gradient_transfer_bound_certified",
            "physical_gradient_bound_certified", "negative_controls_verified")}
        result = judge_tr_registered("synthetic_roundoff_R", r_inputs=claimed)
        self.assertEqual(result["status"], "NOT VERIFIED")
        self.assertEqual(result["certification_status"], "NOT VERIFIED")
        self.assertEqual(result["physical_gate_q"], "FAIL")
        self.assertEqual(judge_tr_registered("synthetic_roundoff_R",
                                             reductions=[1., 1.])["status"], "FAIL")

    def test_pinned_T_cannot_transition_after_rate_measurement(self):
        # The case keeps T even when it fails the threshold.
        result = judge_tr_registered("synthetic_uniform_quadratic_T",
                                     reductions=[0.953506146487913, 0.45916314091601546])
        self.assertEqual(result["regime"], "T")
        self.assertEqual(result["rate_check_status"], "FAIL")
        self.assertEqual(result["historical_gate_q"], "FAIL")


    def test_optional_workflow_path_filter_lists_every_capture_input(self):
        workflow = (Path(__file__).resolve().parents[2]
                    / ".github/workflows/m2-ph08-gateq-independent.yml").read_text()
        for path in (
            "tools/validation/ph08_gateq_exact_captured.py",
            "tools/validation/test_ph08_gateq_exact_captured.py",
            "tools/validation/test_ph08_gateq_independent.py",
            "docs/validation/ph08-gateq-tr-synthetic-prereg-v1.json",
            "docs/validation/m2-g2-ph08-gateq-tr-exact-captured-proposed.md",
        ):
            with self.subTest(path=path):
                self.assertIn('      - "' + path + '"', workflow)



if __name__=="__main__": unittest.main()
