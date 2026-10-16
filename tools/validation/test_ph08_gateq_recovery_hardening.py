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


class SyntheticInstrumentationTests(unittest.TestCase):
    """Dry-run PRIVATE_SCRIPT on OPEN synthetic stand-in modules, not original HMM."""
    def execute(self, aliased=False, incidental_solves=0):
        import json
        from pathlib import Path
        import subprocess
        import sys
        import tempfile
        from ph08_gateq_hmm_bridge import PRIVATE_SCRIPT
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)
            (root/"m2_ph08_hmm_reference.py").write_text(
                "def solve():\n    return 'synthetic'\n",encoding="utf-8")
            origin=("from numpy.linalg import lstsq as alias\n" if aliased else
                    "import numpy as np\n")
            call=("alias(A,b,rcond=None)[0]" if aliased else
                  "np.linalg.lstsq(A,b,rcond=None)[0]")
            (root/"m2_ph08_stage3.py").write_text(
                origin+
                "class Recovery:\n    @staticmethod\n    def polynomial():\n"
                "        import numpy as np\n"
                "        A=np.array([[1.,float(i),float(j),i*i,i*j,j*j]\n"
                "             for i in (-1,0,1) for j in (-1,0,1)])\n"
                "        b=A@np.array([1.,2.,3.,4.,5.,6.])\n"
                f"        for _ in range({incidental_solves}):\n"
                "            np.linalg.solve(np.eye(2),np.ones(2))\n"
                f"        return {call}\n",
                encoding="utf-8")
            (root/"m2_ph08_stage4_qualification.py").write_text(
                "from m2_ph08_stage3 import Recovery\n"
                "def mms_suite():\n"
                "    return {'rows':[{'test':'synthetic',"
                " 'coeff':[float(x) for x in Recovery.polynomial()]}]}\n",
                encoding="utf-8")
            outputs={}
            for mode in ("baseline","instrumented"):
                proc=subprocess.run(
                    [sys.executable,"-c",PRIVATE_SCRIPT,str(root),
                     str(Path(__file__).resolve().parent),mode],
                    cwd=root,text=True,capture_output=True,timeout=30,check=False)
                self.assertEqual(proc.returncode,0,msg=proc.stderr)
                outputs[mode]=json.loads(proc.stdout)
            self.assertEqual(outputs["baseline"]["mms"],
                             outputs["instrumented"]["mms"])
            return outputs["instrumented"]["stats"]

    def test_actual_synthetic_solver_capture_is_unclassified(self):
        stats=self.execute(aliased=False)
        self.assertEqual(stats["recovery_calls"],1)
        self.assertEqual(stats["solver_calls"],1)
        self.assertEqual(len(stats["operators"]),1)
        self.assertEqual(stats["operators"][0]["operator_role"],"UNCLASSIFIED")
        self.assertEqual(stats["operators"][0]["algebraic_role_hypothesis"],"design")

    def test_preimported_solver_alias_is_not_falsely_claimed(self):
        stats=self.execute(aliased=True)
        self.assertEqual(stats["recovery_calls"],1)
        self.assertEqual(stats["solver_calls"],0)
        self.assertEqual(stats["operators"],[])
        self.assertEqual(stats["qualifying_solver_calls"],0)
        self.assertEqual(stats["aliases_and_custom_solver"],"NOT VERIFIED")

    def test_many_incidental_solves_do_not_fill_24_record_cap(self):
        stats=self.execute(aliased=False,incidental_solves=30)
        self.assertEqual(stats["solver_calls"],31)
        self.assertEqual(stats["nonqualifying_solver_calls"],30)
        self.assertEqual(stats["qualifying_solver_calls"],1)
        self.assertEqual(len(stats["operators"]),1)
        self.assertNotIn("SCIPY_NOT_INSTALLED",stats["errors"])

    def test_incidental_solve_does_not_mask_uncaptured_alias(self):
        stats=self.execute(aliased=True,incidental_solves=30)
        self.assertEqual(stats["recovery_calls"],1)
        self.assertEqual(stats["solver_calls"],30)
        self.assertEqual(stats["nonqualifying_solver_calls"],30)
        self.assertEqual(stats["qualifying_solver_calls"],0)
        self.assertEqual(stats["operators"],[])


if __name__=="__main__":
    unittest.main()
