#!/usr/bin/env python3
"""Negative synthetic controls for the declared original Recovery layout."""
import sys
from pathlib import Path
import unittest
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parent))
from ph08_gateq_operator_semantics import recovery_matrices,sample_digest,judge_capture


class OriginalLayoutSyntheticTests(unittest.TestCase):
    def setUp(self):
        self.ids=list(range(9));self.o=np.array([.2,.4])
        self.c=np.array([[.2+i*.13,.4+j*.11] for j in (-1,0,1) for i in (-1,0,1)])
        self.f=np.array([[-.2,-.1],[.8,-.1],[.8,.9],[-.2,.9]])
        self.s=np.max(np.abs(self.f-self.o),axis=0);self.p=np.array([.24,.51])
        self.y=np.array([x*x+z*z+.3 for x,z in self.c])
        self.mat=["dielectric"]*9
        self.A,self.vals,self.D=recovery_matrices(self.ids,self.c,self.y,self.mat,
                                                     4,self.o,self.f,self.s,self.p)
        self.coef=np.linalg.lstsq(self.A,self.vals,rcond=None)[0]
        self.digest=sample_digest(self.ids,self.A,self.vals,self.D)

    def judge(self,**kw):
        q=dict(ids=self.ids,centers=self.c,values=self.y,materials=self.mat,
               target=4,origin=self.o,face_nodes=self.f,scale=self.s,point=self.p,
               captured_A=self.A,captured_y=self.vals,captured_D=self.D,
               observed_coefficients=self.coef,expected_digest=self.digest,
               source_attested=True,solver_entrypoint="numpy.linalg.lstsq")
        q.update(kw);return judge_capture(**q)

    def test_valid_synthetic_layout_still_not_physical_pass(self):
        o=self.judge()
        self.assertEqual(self.A.shape,(9,6));self.assertEqual(self.D.shape,(2,6))
        self.assertNotEqual(self.D[0,3],0);self.assertNotEqual(self.D[1,5],0)
        self.assertEqual(o["status"],"NOT VERIFIED");self.assertEqual(o["historical_gate_q"],"FAIL")
        self.assertEqual(o["certified_float_bound"],"BOUND_NOT_CERTIFIED")
        self.assertTrue(o["constant_shift_null"])

    def test_missing_source_provenance_or_actual_solver_cannot_pass(self):
        for kw in (dict(source_attested=False),dict(expected_digest=None),
                   dict(expected_digest="0"*64),dict(solver_entrypoint=None),
                   dict(solver_entrypoint="scipy.linalg.lstsq")):
            with self.subTest(kw=kw):
                self.assertEqual(self.judge(**kw)["status"],"NOT VERIFIED")

    def test_sample_order_mutation_is_fail(self):
        c=self.c.copy();c[[0,1]]=c[[1,0]]
        self.assertEqual(self.judge(centers=c)["status"],"FAIL")
        self.assertEqual(self.judge(ids=[1,0]+self.ids[2:])["status"],"FAIL")

    def test_missing_or_duplicate_cell_is_fail(self):
        self.assertEqual(self.judge(ids=self.ids[:-1])["status"],"FAIL")
        self.assertEqual(self.judge(ids=self.ids[:-1]+[7])["status"],"FAIL")

    def test_sample_potential_or_basis_mutation_is_fail(self):
        y=self.y.copy();y[0]+=.001
        self.assertEqual(self.judge(values=y)["status"],"FAIL")
        A=self.A.copy();A[0,2]+=.1
        self.assertEqual(self.judge(captured_A=A)["status"],"FAIL")

    def test_gradient_sign_scale_basis_reversal_or_origin_is_fail(self):
        for D in (-self.D,self.D*2,self.D[:,::-1]):
            with self.subTest(D=D.tolist()):
                self.assertEqual(self.judge(captured_D=D)["status"],"FAIL")
        self.assertEqual(self.judge(origin=self.o+.001)["status"],"FAIL")
        self.assertEqual(self.judge(scale=self.s*1000)["status"],"FAIL")

    def test_face_extent_and_material_mutation_is_fail(self):
        f=self.f.copy();f[0,0]-=.11
        self.assertEqual(self.judge(face_nodes=f)["status"],"FAIL")
        mat=self.mat.copy();mat[0]="air"
        self.assertEqual(self.judge(materials=mat)["status"],"FAIL")

    def test_no_unattested_weights_even_uniform(self):
        self.assertEqual(self.judge(weights=np.ones(9))["status"],"FAIL")
        self.assertEqual(self.judge(weights=np.arange(1,10))["status"],"FAIL")

    def test_nan_inf_rank_deficiency_is_fail(self):
        for v in (np.nan,np.inf,-np.inf):
            y=self.vals.copy();y[0]=v
            self.assertEqual(self.judge(captured_y=y)["status"],"FAIL")
        c=self.c.copy();c[:]=c[0]
        self.assertEqual(self.judge(centers=c)["status"],"FAIL")

    def test_constant_coefficient_corruption_not_physical_pass(self):
        coeff=self.coef.copy();coeff[0]+=.5
        o=self.judge(observed_coefficients=coeff)
        self.assertEqual(o["status"],"NOT VERIFIED")
        self.assertGreater(o["observed_coefficients_error_inf"],.49)
        self.assertEqual(o["rust_fem"],"NO-GO")

    def test_fingerprint_uses_sample_order_potentials_and_gradient(self):
        swapped=self.A.copy();swapped[[0,1]]=swapped[[1,0]]
        self.assertNotEqual(sample_digest(self.ids,swapped,self.vals,self.D),self.digest)
        self.assertNotEqual(sample_digest(self.ids,self.A,self.vals+.01,self.D),self.digest)
        self.assertNotEqual(sample_digest(self.ids,self.A,self.vals,self.D*2),self.digest)


if __name__=="__main__":unittest.main()
