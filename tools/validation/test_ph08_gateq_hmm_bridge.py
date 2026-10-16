"""Synthetic-only reproducibility and adversarial PH-08 bridge checks."""
import unittest
import numpy as np
from ph08_gateq_hmm_bridge import diagnose,synthetic_report,rational_fit
from ph08_gateq_independent import samples,SHAPES,SIZES

class Tests(unittest.TestCase):
    def inputs(self,kind="quadratic"):
        A,y,_,_=samples("uniform",kind,8)
        return np.asarray(A,float),np.asarray(y,float)
    def test_all_18_cases(self):
        r=synthetic_report()
        self.assertEqual(r["cases"],18)
        self.assertEqual(r["original_hmm"],"NOT RUN")
        self.assertEqual(r["historical_gate_q"],"FAIL")
        self.assertTrue(all(q["rank"]==6 for q in r["rows"]))
    def test_all_shapes_quad(self):
        for grid in SHAPES:
            for n in SIZES:
                A,y,_,_=samples(grid,"quadratic",n)
                with self.subTest(grid=grid,n=n):
                    self.assertGreater(diagnose(A,y)["singular_min"],0)
    def test_all_shapes_cubic(self):
        for grid in SHAPES:
            for n in SIZES:
                A,y,_,_=samples(grid,"cubic",n)
                with self.subTest(grid=grid,n=n):
                    self.assertEqual(diagnose(A,y)["certified_float_bound"],"BOUND_NOT_CERTIFIED")
    def test_exact_reproduces_six_coefficients(self):
        A,y=self.inputs()
        self.assertEqual(len(rational_fit(A,y)),6)
    def test_constant_gradient_mutation_detected(self):
        A,y=self.inputs(); c=np.linalg.lstsq(A,y,rcond=None)[0]
        c[1]+=1e-3
        self.assertGreater(diagnose(A,y,c)["coefficient_diff_vs_exact_binary64_inputs"],1e-4)
    def test_flip_sign_detected(self):
        A,y=self.inputs(); c=np.linalg.lstsq(A,y,rcond=None)[0]
        c[1]*=-1
        self.assertGreater(diagnose(A,y,c)["coefficient_diff_vs_exact_binary64_inputs"],0.01)
    def test_wrong_scale_detected(self):
        A,y=self.inputs();c=np.linalg.lstsq(A,y,rcond=None)[0];c[1:3]*=8
        self.assertGreater(diagnose(A,y,c)["coefficient_diff_vs_exact_binary64_inputs"],0.1)
    def test_sample_corruption_detected_by_residual(self):
        A,y=self.inputs();bad=y.copy();bad[0]+=0.1
        self.assertGreater(diagnose(A,bad)["residual_norm2"],1e-3)
    def test_nan(self):
        A,y=self.inputs();A[0,0]=float("nan")
        with self.assertRaises(ValueError):diagnose(A,y)
    def test_inf(self):
        A,y=self.inputs();y[0]=float("inf")
        with self.assertRaises(ValueError):diagnose(A,y)
    def test_singular(self):
        with self.assertRaises(ValueError):diagnose(np.ones((9,6)),np.ones(9))
    def test_wrong_count(self):
        with self.assertRaises(ValueError):diagnose(np.eye(6),np.ones(5))
    def test_bad_coeff(self):
        A,y=self.inputs()
        with self.assertRaises(ValueError):diagnose(A,y,[0]*5)
    def test_no_false_certification(self):
        r=synthetic_report()
        self.assertEqual(r["rust_fem"],"NO-GO")
        self.assertEqual(r["certified_float_bound"],"BOUND_NOT_CERTIFIED")
if __name__=="__main__":unittest.main()
