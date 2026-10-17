"""Public synthetic-only tests. Never run or upload a private original HMM ZIP."""
import json
import tempfile
from pathlib import Path
import unittest
from unittest import mock
import resource

from ph08_stage5b_original_replay import (
    compare_qualification, fail, replay, STAGE5B_SHA256, _approved_runtime, _limits, _kill_replay_group, require_qualification_object
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
            # Isolate the SHA admission control from the separately tested
            # private-only numerical-package version admission gate.
            with mock.patch("ph08_stage5b_original_replay._approved_runtime",
                            return_value=(True, {"python": "test-pinned"})):
                result=replay(p)
        self.assertEqual(result["reason_code"],"ORIGINAL_ARCHIVE_SHA256_MISMATCH")

    def test_failure_records_never_claim_pass(self):
        for code in ("UNINITIALIZED","PRIVATE_REPLAY_TIMEOUT",
                     "ARCHIVE_SCHEMA_OR_IO_FAILURE"):
            result=fail(code)
            self.assertEqual(result["archived_gate_q"],"FAIL")
            self.assertEqual(result["physical_error_bound"],"BOUND_NOT_CERTIFIED")
            self.assertEqual(result["contract"],"NOT_FROZEN")

    def test_runtime_wrong_package_versions_not_approved(self):
        with mock.patch("ph08_stage5b_original_replay.importlib.metadata.version",
                        side_effect=lambda p: "9.9.9"):
            ok, observed = _approved_runtime()
        self.assertFalse(ok)
        self.assertEqual(observed["numpy"], "9.9.9")

    def test_runtime_missing_package_not_approved(self):
        from importlib.metadata import PackageNotFoundError
        with mock.patch("ph08_stage5b_original_replay.importlib.metadata.version",
                        side_effect=PackageNotFoundError("numpy")):
            ok, observed = _approved_runtime()
        self.assertFalse(ok)
        self.assertEqual(observed["numpy"], "NOT_INSTALLED")

    def test_unreadable_package_metadata_returns_unapproved_runtime(self):
        for error in (UnicodeDecodeError("utf-8", b"\\xff", 0, 1, "invalid"),
                      PermissionError("metadata unavailable"),
                      ValueError("malformed version metadata")):
            with self.subTest(kind=type(error).__name__):
                with mock.patch(
                    "ph08_stage5b_original_replay.importlib.metadata.version",
                    side_effect=error
                ):
                    ok, runtime = _approved_runtime()
                self.assertFalse(ok)
                self.assertEqual(runtime["numpy"], "METADATA_NOT_VERIFIED")
                self.assertEqual(runtime["scipy"], "METADATA_NOT_VERIFIED")

    def test_resource_caps_never_exceed_inherited_hard_limits(self):
        hard_caps = {resource.RLIMIT_CPU: 30,
                     resource.RLIMIT_AS: 1024 * 1024 * 1024,
                     resource.RLIMIT_FSIZE: 1024}
        with mock.patch("ph08_stage5b_original_replay.resource.getrlimit",
                        side_effect=lambda kind: (hard_caps[kind], hard_caps[kind])):
            with mock.patch("ph08_stage5b_original_replay.resource.setrlimit") as setter:
                _limits()
        self.assertEqual(setter.call_count, 3)
        for call in setter.call_args_list:
            kind, pair = call.args
            self.assertEqual(pair, (hard_caps[kind], hard_caps[kind]))

    def test_process_group_cleanup_calls_kill(self):
        with mock.patch("ph08_stage5b_original_replay.os.killpg") as kill:
            _kill_replay_group(12345)
        kill.assert_called_once()

    def test_process_group_already_dead_is_safe(self):
        with mock.patch("ph08_stage5b_original_replay.os.killpg",
                        side_effect=ProcessLookupError):
            _kill_replay_group(12345)

    def test_archive_mismatch_avoids_source_execution(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/"synthetic.zip"
            p.write_bytes(b"nonprivate")
            with mock.patch("ph08_stage5b_original_replay.subprocess.Popen") as run:
                with mock.patch("ph08_stage5b_original_replay._approved_runtime",
                                return_value=(True, {})):
                    result = replay(p)
            self.assertEqual(result["reason_code"], "ORIGINAL_ARCHIVE_SHA256_MISMATCH")
            run.assert_not_called()

    def test_nonobject_generated_json_fails_closed(self):
        for value in ([], [1], 2, 0.5, "valid JSON", None, True):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    require_qualification_object(value)

    def test_object_qualification_json_admitted_for_deeper_validation(self):
        record = synthetic_record()
        self.assertIs(require_qualification_object(record), record)

    def test_historical_sha_pin_unchanged(self):
        self.assertEqual(STAGE5B_SHA256,
          "f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36")


if __name__=="__main__":
    unittest.main(verbosity=2)
