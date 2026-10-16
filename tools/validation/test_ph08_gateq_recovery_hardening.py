"""Actual verdict-negative controls; synthetic tests never qualify HMM."""
import hashlib
import unittest

import numpy as np

from ph08_gateq_independent import samples
from ph08_gateq_recovery_hardening import (
    SCOPE_SYNTHETIC, SCOPE_OPERATOR, rational_operator,
    gradient_transfer, transfer_error, observed_diagnostics, verdict,
)

class HardeningTests(unittest.TestCase):
    def setUp(self):
        A, y, h, _ = samples("uniform", "quadratic", 8)
        self.A = np.asarray(A, dtype=np.float64)
        self.y = np.asarray(y, dtype=np.float64)
        self.D = np.zeros((2, 6))
        self.D[0, 1] = 8.
        self.D[1, 2] = 8.
        self.c = np.linalg.lstsq(self.A, self.y, rcond=None)[0]
        self.ref = np.asarray([float(x) for x in
                               rational_operator(self.A, self.y, role="design")])
        self.digest = hashlib.sha256(self.A.tobytes()+self.y.tobytes()).hexdigest()

    def judge(self, observed=None, **changes):
        args = dict(matrix_role="design", source_digest=self.digest,
                    expected_source_digest=self.digest, same_problem=True,
                    gradient_map=self.D, tolerance=1e-8, coefficient_tolerance=1e-8,
                    tolerance_preregistered=True, scope=SCOPE_SYNTHETIC)
        args.update(changes)
        return verdict(self.c if observed is None else observed,
                       self.ref, **args)

    def test_valid_synthetic_scope_only(self):
        result = self.judge()
        self.assertEqual(result["status"], "VERIFIED")
        self.assertEqual(result["scope"], SCOPE_SYNTHETIC)
        self.assertEqual(result["historical_gate_q"], "FAIL")
        self.assertEqual(result["rust_fem"], "NO-GO")

    def test_source_hash_mismatch_is_not_verified(self):
        self.assertEqual(self.judge(source_digest="0"*64)["status"], "NOT VERIFIED")

    def test_semantic_mismatch_is_not_verified(self):
        for extra in (dict(matrix_role=None),dict(same_problem=False),
                      dict(gradient_map=None),dict(tolerance_preregistered=False),
                      dict(tolerance=None),dict(source_digest=None),
                      dict(scope="unregistered")):
            with self.subTest(extra=extra):
                self.assertNotEqual(self.judge(**extra)["status"], "VERIFIED")

    def test_gradient_sign_flip_fails(self):
        v=self.c.copy();v[1]=-v[1]
        self.assertEqual(self.judge(v)["status"], "FAIL")

    def test_gradient_constant_shift_fails(self):
        v=self.c.copy();v[2]+=1e-4
        self.assertEqual(self.judge(v)["status"], "FAIL")

    def test_gradient_scale_fails(self):
        v=self.c.copy();v[1:3]*=10
        self.assertEqual(self.judge(v)["status"], "FAIL")

    def test_one_coefficient_corrupted_fails(self):
        v=self.c.copy();v[1]+=1e-5
        self.assertEqual(self.judge(v)["status"], "FAIL")

    def test_nan_inf_fails(self):
        for bad in (float("nan"),float("inf"),-float("inf")):
            with self.subTest(bad=bad):
                v=self.c.copy();v[1]=bad
                self.assertEqual(self.judge(v)["status"], "FAIL")

    def test_constant_coefficient_mutation_fails_despite_invariant_gradient(self):
        v=self.c.copy();v[0]+=1.0
        self.assertEqual(self.judge(v)["status"], "FAIL")

    def test_missing_sample_cannot_pass_diagnostics(self):
        # A removed row and corresponding y can still be a valid LS problem;
        # its semantic/provenance fingerprint must therefore block verdict.
        altered=self.A[:-1]
        forged_digest=hashlib.sha256(altered.tobytes()+self.y[:-1].tobytes()).hexdigest()
        self.assertNotEqual(self.judge(source_digest=forged_digest)["status"],"VERIFIED")

    def test_sample_mutation_with_old_digest_is_not_verified(self):
        corrupt=self.y.copy();corrupt[0]+=0.1
        new_digest=hashlib.sha256(self.A.tobytes()+corrupt.tobytes()).hexdigest()
        self.assertEqual(self.judge(source_digest=new_digest)["status"],"NOT VERIFIED")

    def test_weights_require_explicit_positive_convention(self):
        with self.assertRaises(ValueError):
            rational_operator(self.A,self.y,role="weighted_design")
        for w in (np.zeros(len(self.y)), -np.ones(len(self.y)),np.ones(2)):
            with self.subTest(weight=w):
                with self.assertRaises(ValueError):
                    rational_operator(self.A,self.y,role="weighted_design",weights=w)

    def test_design_vs_normal_matrix_not_double_squared(self):
        H=self.A.T@self.A
        rhs=self.A.T@self.y
        normal=rational_operator(H,rhs,role="normal_equation")
        exact_normal=np.linalg.solve(H,rhs)
        self.assertLess(max(abs(float(normal[i])-exact_normal[i]) for i in range(6)),1e-10)
        self.assertEqual(rational_operator(H,rhs,role="normal_equation"),
                           rational_operator(H,rhs,role="design"))
        d=observed_diagnostics(self.A,self.y,self.c,role="design",method="lstsq")
        n=observed_diagnostics(H,rhs,exact_normal,role="normal_equation",method="solve")
        self.assertIsNotNone(d["condition_design_2"])
        self.assertIsNone(n["condition_design_2"])
        self.assertGreater(n["condition_supplied_system_2"],d["condition_design_2"])

    def test_nonsymmetric_system_is_not_design(self):
        H=np.eye(6);H[0,1]=0.5
        b=np.arange(6.)
        x=rational_operator(H,b,role="linear_system")
        self.assertAlmostEqual(float(x[0]),-0.5)
        with self.assertRaises(ValueError):
            rational_operator(H,b,role="normal_equation")

    def test_wrong_algorithm_role_rejected(self):
        with self.assertRaises(ValueError):
            observed_diagnostics(self.A,self.y,self.c,role="design",method="solve")
        with self.assertRaises(ValueError):
            observed_diagnostics(np.eye(6),np.ones(6),np.ones(6),
                                 role="linear_system",method="lstsq")

    def test_exact_constant_offset_invariance_of_gradient(self):
        T,K=gradient_transfer(self.A,self.D)
        self.assertEqual(len(T),2)
        self.assertTrue(K>0)
        for row in T:
            self.assertEqual(sum(row),0)
        delta=transfer_error(self.A,self.D,[0.125]*len(self.y))
        self.assertEqual(delta["gradient_error"],["0","0"])

    def test_error_transfer_exactness_and_enclosure(self):
        perturb=[0.0]*len(self.y)
        perturb[0]=0.1
        out=transfer_error(self.A,self.D,perturb)
        direct=np.asarray([float(t) for t in rational_operator(
            self.A,self.y+np.asarray(perturb),role="design")])
        # y+delta is rounded in this direct numerical check; exact transfer
        # instead treats separately rounded baseline y and rounded delta.
        self.assertEqual(len(out["gradient_error"]),2)
        self.assertGreater(__import__("fractions").Fraction(out["gradient_error_inf_bound_exact"]),0)
        T,_=gradient_transfer(self.A,self.D)
        expected=[sum(row[k]*__import__("fractions").Fraction.from_float(perturb[k])
                      for k in range(len(perturb))) for row in T]
        self.assertEqual([str(t) for t in expected],out["gradient_error"])
        self.assertEqual(direct.shape,(6,))

    def test_rank_deficient_is_rejected(self):
        bad=np.ones((9,6))
        with self.assertRaises(ValueError):
            rational_operator(bad,np.ones(9),role="design")
        with self.assertRaises(ValueError):
            gradient_transfer(bad,self.D)

    def test_binary64_nonfinite_matrix_fails(self):
        bad=self.A.copy();bad[0,0]=float("nan")
        with self.assertRaises(ValueError):
            rational_operator(bad,self.y,role="design")

    def test_100_digit_reference_is_not_a_certified_bound(self):
        self.assertEqual(observed_diagnostics(
            self.A,self.y,self.c,role="design",method="lstsq"
        )["certified_float_bound"],"BOUND_NOT_CERTIFIED")

    def test_original_cannot_pass_without_bound(self):
        self.assertEqual(self.judge(scope=SCOPE_OPERATOR,float_bound_certified=True)["status"],"NOT VERIFIED")

if __name__=="__main__":
    unittest.main()
