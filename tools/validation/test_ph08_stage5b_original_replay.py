"""Public synthetic-only tests. Never run or upload a private original HMM ZIP."""
import json
import tempfile
from pathlib import Path
import unittest

from ph08_stage5b_original_replay import (
    compare_qualification, fail, replay, STAGE5B_SHA256
)


def synthetic_record():
    return {
        "Gate_Q": "FAIL", "certified": False, "physical_solves": 0,
        "rows": [{"linear": {"seconds": 1.1, "residual": 0.25},
                  "bulk": {"gradient": [0.125, 0.50]}}],
        "historic_mms_failures_retained": [["uniform","quadratic"]],
    }


class ReplayGuard(unittest.TestCase):
    def test_only_timing_is_excluded(self):
        a=synthetic_record();b=synthetic_record()
        b["rows"][0]["linear"]["seconds"]=1.75
        result=compare_qualification(a,b)
        self.assertEqual(result, {"numeric_scalars_compared": 4,
                                  "timing_scalars_excluded":1,"mismatches":0})

    def test_gradient_mutation_is_rejected(self):
        a=synthetic_record();b=synthetic_record()
        b["rows"][0]["bulk"]["gradient"][1]+=0.0001
        self.assertEqual(compare_qualification(a,b)["mismatches"],1)

    def test_historical_failure_cannot_be_cleared(self):
        a=synthetic_record();b=synthetic_record()
        b["historic_mms_failures_retained"]=[]
        self.assertNotEqual(compare_qualification(a,b)["mismatches"],0)

    def test_gateq_fail_to_pass_detected(self):
        a=synthetic_record();b=synthetic_record()
        b["Gate_Q"]="PASS"
        self.assertNotEqual(compare_qualification(a,b)["mismatches"],0)

    def test_false_certified_boolean_detected(self):
        a=synthetic_record();b=synthetic_record()
        b["certified"]=True
        self.assertNotEqual(compare_qualification(a,b)["mismatches"],0)

    def test_type_mutation_not_accepted(self):
        a=synthetic_record();b=synthetic_record()
        b["physical_solves"]=False
        self.assertNotEqual(compare_qualification(a,b)["mismatches"],0)

    def test_extra_key_detected(self):
        a=synthetic_record();b=synthetic_record()
        b["unexpected"]="PASS"
        self.assertNotEqual(compare_qualification(a,b)["mismatches"],0)

    def test_other_seconds_not_exempt(self):
        a=synthetic_record();b=synthetic_record()
        a["measurement"]={"seconds":20.0}
        b["measurement"]={"seconds":21.0}
        self.assertEqual(compare_qualification(a,b)["mismatches"],1)

    def test_wrong_row_timing_path_not_exempt(self):
        a=synthetic_record();b=synthetic_record()
        a["metadata"]={"linear":{"seconds":1.0}}
        b["metadata"]={"linear":{"seconds":2.0}}
        self.assertEqual(compare_qualification(a,b)["mismatches"],1)

    def test_array_mutation_detected(self):
        a=synthetic_record();b=synthetic_record()
        b["rows"][0]["bulk"]["gradient"].append(0.6)
        self.assertNotEqual(compare_qualification(a,b)["mismatches"],0)

    def test_missing_original_fail_closed(self):
        with tempfile.TemporaryDirectory() as d:
            result=replay(Path(d)/"never-uploaded-private.zip")
        self.assertEqual(result["status"],"REPLAY_NOT_VERIFIED")
        self.assertEqual(result["reason_code"],"MISSING_PRIVATE_ARCHIVE")
        self.assertEqual(result["physical_gate_q"],"FAIL")
        self.assertEqual(result["rust_fem"],"NO_GO")

    def test_synthetic_archive_never_admitted_as_original(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/"synthetic.zip";p.write_bytes(b"synthetic")
            result=replay(p)
        self.assertEqual(result["reason_code"],"ORIGINAL_ARCHIVE_SHA256_MISMATCH")

    def test_failure_records_never_claim_pass(self):
        for code in ("UNINITIALIZED","PRIVATE_REPLAY_TIMEOUT",
                     "ARCHIVE_SCHEMA_OR_IO_FAILURE"):
            result=fail(code)
            self.assertEqual(result["archived_gate_q"],"FAIL")
            self.assertEqual(result["physical_error_bound"],"BOUND_NOT_CERTIFIED")
            self.assertEqual(result["contract"],"NOT_FROZEN")

    def test_historical_sha_pin_unchanged(self):
        self.assertEqual(STAGE5B_SHA256,
          "f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36")


if __name__=="__main__":
    unittest.main(verbosity=2)
