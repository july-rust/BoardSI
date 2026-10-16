#!/usr/bin/env python3
"""Fail-closed positive/negative tests for the synthetic PH-08 Gate Q oracle."""
from __future__ import annotations

from fractions import Fraction as F
import hashlib
import math
import sys
import tempfile
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import ph08_gateq_independent as q


class PolynomialOracleTests(unittest.TestCase):
    def test_quadratic_exact_reproduction_each_shape_and_resolution(self):
        for shape in q.SHAPES:
            for n in q.SIZES:
                with self.subTest(shape=shape, n=n):
                    c = q.exact_case(shape, "quadratic", n)
                    self.assertEqual(c["gradient"], c["theory"])
                    self.assertEqual(c["squared_error"], F(0))
                    self.assertEqual(c["residual"], [F(0)]*9)

    def test_cubic_exact_gradient_truncation_reduction_each_shape(self):
        for shape in q.SHAPES:
            errs = [q.exact_case(shape, "cubic", n)["squared_error"]
                    for n in q.SIZES]
            self.assertTrue(all(v > 0 for v in errs), shape)
            # Exact, synthetic stencils have invariant normalized templates.
            self.assertEqual(errs[0]/errs[1], F(16), shape)
            self.assertEqual(errs[1]/errs[2], F(16), shape)

    def test_constant_potential_offset_cannot_change_exact_gradient(self):
        for shape in q.SHAPES:
            for n in q.SIZES:
                with self.subTest(shape=shape, n=n):
                    h=F(1,n)
                    original=q.exact_case(shape,"quadratic",n)
                    shifted=q.exact_case(shape,"quadratic",n,-h*h/6)
                    self.assertEqual(shifted["gradient"],original["gradient"])
                    self.assertEqual(shifted["coeff"][0]-original["coeff"][0],-h*h/6)
                    self.assertEqual(shifted["residual"],[F(0)]*9)

    def test_high_precision_reference_matches_exact_rational(self):
        for shape in q.SHAPES:
            for kind in ("quadratic", "cubic"):
                for n in q.SIZES:
                    with self.subTest(shape=shape, kind=kind, n=n):
                        hp=q.high_precision_case(shape,kind,n)
                        self.assertTrue(hp["within_1e_minus_65"],hp)

    def test_float64_diagnostics_are_finite_and_full_rank(self):
        for shape in q.SHAPES:
            for kind in ("quadratic", "cubic"):
                for n in q.SIZES:
                    with self.subTest(shape=shape,kind=kind,n=n):
                        got=q.double_case(shape,kind,n)
                        self.assertEqual(got["rank"],6)
                        for key in ("error_rms","condition_2","residual_max"):
                            self.assertTrue(math.isfinite(got[key]),key)
                        self.assertGreater(got["condition_2"],0)

    def test_detect_constant_gradient_mutation(self):
        c=q.exact_case("uniform","quadratic",16)
        corrupted=(c["gradient"][0]+F(1,1000),c["gradient"][1])
        self.assertGreater(q.squared_error(corrupted,c["theory"]),0)

    def test_detect_gradient_sign_mutation(self):
        c=q.exact_case("skew","quadratic",8)
        corrupted=(-c["gradient"][0],c["gradient"][1])
        self.assertGreater(q.squared_error(corrupted,c["theory"]),0)

    def test_detect_gradient_factor_mutation(self):
        c=q.exact_case("graded","quadratic",32)
        corrupted=tuple(F(2)*v for v in c["gradient"])
        self.assertGreater(q.squared_error(corrupted,c["theory"]),0)

    def test_detect_basis_scaling_mutation(self):
        c=q.exact_case("uniform","quadratic",16)
        wrong=q.recovered_gradient(c["coeff"],2*c["h"])
        self.assertGreater(q.squared_error(wrong,c["theory"]),0)

    def test_detect_single_sample_tampering(self):
        A,y,h,_=q.samples("skew","quadratic",16)
        y[0]+=F(1,1000)
        coeff,residual=q.exact_fit(A,y)
        self.assertTrue(any(v!=0 for v in residual))
        self.assertTrue(q.squared_error(q.recovered_gradient(coeff,h),
                                        q.exact_gradient("quadratic"))>0)

    def test_nonfinite_values_are_rejected(self):
        for bad in (math.nan,math.inf,-math.inf):
            with self.subTest(value=str(bad)):
                with self.assertRaises(ValueError):
                    q.validate_float_vector([0.0]*8+[bad])

    def test_missing_and_extra_samples_are_rejected(self):
        for values in ([0.0]*8,[0.0]*10,[]):
            with self.assertRaises(ValueError):
                q.validate_float_vector(values)
        A,y,_,_=q.samples("uniform","quadratic",16)
        with self.assertRaises(ValueError):
            q.exact_fit(A[:-1],y[:-1])

    def test_missing_and_tampered_attestation_fail_closed(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/"evidence.json"
            data=b'{"synthetic":true}\n'
            digest=hashlib.sha256(data).hexdigest()
            with self.assertRaises(FileNotFoundError):
                q.sha256_attestation(p,digest)
            p.write_bytes(data)
            self.assertEqual(q.sha256_attestation(p,digest),digest)
            p.write_bytes(data+b"x")
            with self.assertRaises(ValueError):
                q.sha256_attestation(p,digest)
            with self.assertRaises(ValueError):
                q.sha256_attestation(p,"not-sha256")

    def test_nan_cannot_serialize_as_evidence(self):
        import json
        with self.assertRaises(ValueError):
            json.dumps({"error": math.nan},allow_nan=False)

    def test_unsupported_grid_polynomial_and_precision_rejected(self):
        with self.assertRaises(ValueError):
            q.samples("unknown","quadratic",16)
        with self.assertRaises(ValueError):
            q.samples("uniform","unknown",16)
        with self.assertRaises(ValueError):
            q.samples("uniform","cubic",0)
        with self.assertRaises(ValueError):
            q.high_precision_case("uniform","quadratic",8,dps=30)

    def test_report_never_claims_physical_approval(self):
        output=q.report()
        self.assertEqual(len(output["rows"]),18)
        self.assertEqual(len(output["cubic_reductions"]),3)
        self.assertEqual(output["historical_stage4_gate_q"],"FAIL")
        self.assertEqual(output["AC_P1"],"PENDING")
        self.assertEqual(output["stage5b"],"NOT RUN")
        self.assertEqual(output["rust_fem"],"NO-GO")
        self.assertEqual(output["qualification"],"PROPOSED / NOT FROZEN")
        for row in output["rows"]:
            self.assertTrue(math.isfinite(row["condition_2_float64"]))


from test_ph08_gateq_hmm_bridge import Tests as OriginalHMMBridgeSyntheticTests
from test_ph08_gateq_recovery_hardening import HardeningTests as RecoveryHardeningSyntheticTests
from test_ph08_gateq_recovery_hardening import SyntheticInstrumentationTests as RecoveryInstrumentationSyntheticTests

if __name__ == "__main__":
    unittest.main()
