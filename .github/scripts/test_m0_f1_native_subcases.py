"""Adversarial tests: F1 CI proof is limited, per-case and fail closed."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from m0_f1_native_subcases import (CASE_SCENARIOS, attest_exact_source,
    evaluate, reject_duplicate_json_members)


def reference():
    case_names = set()
    for scenario_names in CASE_SCENARIOS.values():
        case_names.update(scenario_names)
    for n in range(30):
        case_names.add("extra-native-probe-" + str(n))
    data = {
        "qt": "6.8.3",
        "platform": "windows",
        "cpu_arch": "x86_64",
        "pass": True,
        "assertions": 100,
        "cases": [
            {"name": name, "pass": True, "exit_code": 0,
             "exit_status": 0, "error": ""}
            for name in sorted(case_names)
        ],
        "metrics": {
            "normal_ui_count": 150,
            "burst_ui_count": 150,
            "cpu_stress_ui_count": 150,
            "normal_ui_p95_ms": 0.4,
            "burst_ui_p95_ms": 0.5,
            "cpu_stress_ui_p95_ms": 5,
            "normal_ui_max_ms": 20,
            "burst_ui_max_ms": 30,
            "cpu_stress_ui_max_ms": 40,
            "normal_ui_timer_max_gap_ms": 20,
            "burst_ui_timer_max_gap_ms": 20,
            "cpu_stress_ui_timer_max_gap_ms": 50,
            "global_event_loop_gap_limit_ms": 1500,
            "entire_test_timer_max_gap_ms": 912,
            "heartbeat_samples": 6,
            "heartbeat_min_gap_ms": 990,
            "heartbeat_max_gap_ms": 1050,
            "receive_to_widget_update_max_ms": 5,
            "result_samples": 100,
            "progress_samples": 100,
            "cancel_ack_max_ms": 16,
            "cancel_exit_max_ms": 22,
            "partial_read_observations": 3,
            "cpu_stress_threads": 2,
            "render_width_px": 500,
            "render_height_px": 400,
        },
    }
    return data


class NativeScopedEvidenceTests(unittest.TestCase):
    def test_all_six_restricted_subcases_have_explicit_evidence(self):
        got = evaluate(reference(), "windows", "x86_64")
        self.assertEqual(len(got["cases"]), 6)
        self.assertEqual(set(c["id"] for c in got["cases"]), set(CASE_SCENARIOS))
        self.assertTrue(all(c["status"] == "PASS_RESTRICTED_SYNTHETIC_SUBCASE"
                            for c in got["cases"]))
        self.assertEqual(got["formal_M0"], "BLOCKED")
        self.assertEqual(got["formal_F1"], "PARTIAL")
        self.assertTrue(all(c["full_product_T_acceptance"] == "NOT_RUN" for c in got["cases"]))
        self.assertTrue(all(c["independent_case_specific_acceptance_review"] == "NOT_PERFORMED"
                            for c in got["cases"]))

    def test_native_checkout_exact_sha_proof(self):
        head = "a" * 40
        self.assertEqual(attest_exact_source(head, head), "EXACT_COMMIT_SHA")
        with self.assertRaisesRegex(ValueError, "differs from"):
            attest_exact_source("b" * 40, head)
        with self.assertRaisesRegex(ValueError, "Missing/invalid"):
            attest_exact_source(head, "NOT_GITHUB")
        with self.assertRaisesRegex(ValueError, "Invalid checked-out"):
            attest_exact_source("bad", head)

    def test_native_arm64_targets_are_distinct(self):
        for platform in ("windows", "cocoa"):
            sample = reference()
            sample["platform"], sample["cpu_arch"] = platform, "arm64"
            got = evaluate(sample, platform, "arm64")
            self.assertEqual(got["os_qt_backend"], platform)
            self.assertEqual(got["cpu_architecture"], "arm64")

    def test_os_or_cpu_spoof_fails(self):
        for platform, cpu in (("windows", "arm64"), ("cocoa", "x86_64"),
                              ("offscreen", "x86_64")):
            with self.subTest(platform=platform, cpu=cpu):
                with self.assertRaises(ValueError):
                    evaluate(reference(), platform, cpu)

    def test_no_overall_pass(self):
        sample = reference()
        sample["pass"] = False
        with self.assertRaisesRegex(ValueError, "not PASS"):
            evaluate(sample, "windows", "x86_64")

    def test_missing_real_case_fails(self):
        sample = reference()
        sample["cases"] = [c for c in sample["cases"] if c["name"] != "snapshot"]
        with self.assertRaisesRegex(ValueError, "missing actual executed"):
            evaluate(sample, "windows", "x86_64")

    def test_failure_disguised_as_case_record_fails(self):
        sample = reference()
        sample["cases"][0]["pass"] = "true"
        with self.assertRaisesRegex(ValueError, "failed or unverified"):
            evaluate(sample, "windows", "x86_64")

    def test_missing_real_producer_field_fails(self):
        sample = reference()
        sample["cases"][0].pop("exit_status")
        with self.assertRaisesRegex(ValueError, "producer record"):
            evaluate(sample, "windows", "x86_64")

    def test_invalid_producer_exit_status_fails(self):
        sample = reference()
        sample["cases"][0]["exit_status"] = "0"
        with self.assertRaisesRegex(ValueError, "producer record"):
            evaluate(sample, "windows", "x86_64")

    def test_duplicate_nested_json_keys_are_not_normalized(self):
        for data in (
            '{"pass": false, "pass": true}',
            '{"metrics": {"normal_ui_p95_ms": 999, "normal_ui_p95_ms": 1}}',
            '{"cases":[{"name":"snapshot","name":"other"}]}',
        ):
            with self.subTest(data=data), self.assertRaisesRegex(ValueError, "Duplicate JSON field"):
                json.loads(data, object_pairs_hook=reject_duplicate_json_members)

    def test_duplicate_case_fails(self):
        sample = reference()
        sample["cases"].append(dict(sample["cases"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate"):
            evaluate(sample, "windows", "x86_64")

    def test_historical_2930ms_gap_stays_failure(self):
        sample = reference()
        sample["metrics"]["entire_test_timer_max_gap_ms"] = 2930
        with self.assertRaisesRegex(ValueError, "exceeded 1500"):
            evaluate(sample, "windows", "x86_64")

    def test_m0_policy_not_silently_relaxed(self):
        sample = reference()
        sample["metrics"]["global_event_loop_gap_limit_ms"] = 3000
        with self.assertRaisesRegex(ValueError, "Unexpected M0-only"):
            evaluate(sample, "windows", "x86_64")

    def test_other_ui_thresholds_not_relaxed(self):
        for field, value in (("normal_ui_p95_ms", 201),
                             ("cpu_stress_ui_max_ms", 1001),
                             ("burst_ui_timer_max_gap_ms", 1001),
                             ("heartbeat_max_gap_ms", 1501),
                             ("cancel_ack_max_ms", 1001)):
            with self.subTest(field=field):
                sample = reference()
                sample["metrics"][field] = value
                with self.assertRaises(ValueError):
                    evaluate(sample, "windows", "x86_64")

    def test_boolean_and_invalid_numerics_not_treated_as_measurement(self):
        for field, value in (("normal_ui_p95_ms", True),
                             ("cancel_ack_max_ms", float("nan")),
                             ("cpu_stress_threads", -1)):
            sample = reference()
            sample["metrics"][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                evaluate(sample, "windows", "x86_64")

    def test_missing_heartbeat_or_real_raster_fails(self):
        for field in ("heartbeat_samples", "render_width_px", "partial_read_observations"):
            sample = reference()
            sample["metrics"].pop(field)
            with self.subTest(field=field), self.assertRaises(ValueError):
                evaluate(sample, "windows", "x86_64")

    def test_oversized_integer_fails_validation_not_with_overflow(self):
        sample = reference()
        sample["metrics"]["cancel_ack_max_ms"] = 10 ** 400
        with self.assertRaisesRegex(ValueError, "oversized"):
            evaluate(sample, "windows", "x86_64")

    def test_failure_json_preserves_input_hash_even_for_rejected_sizes(self):
        for raw in (b"", b"0" * 4000001, b"[" * 100000 + b"]" * 100000):
            with self.subTest(input_bytes=len(raw)), tempfile.TemporaryDirectory() as d:
                incoming = Path(d) / "qt.json"
                output = Path(d) / "m0-f1.json"
                incoming.write_bytes(raw)
                command = [sys.executable,
                           str(Path(__file__).with_name("m0_f1_native_subcases.py")),
                           str(incoming), str(output), "windows", "x86_64"]
                result = subprocess.run(command, capture_output=True, text=True, timeout=15)
                self.assertEqual(result.returncode, 1, result.stderr[-300:])
                proof = json.loads(output.read_text(encoding="utf-8"))
                self.assertEqual(proof["result"], "FAIL_CLOSED")
                self.assertEqual(proof["qt_input_sha256"], hashlib.sha256(raw).hexdigest())
                self.assertEqual(len(proof["cases"]), 6)
                self.assertTrue(all(x["independent_case_specific_acceptance_review"] == "NOT_PERFORMED"
                                    and x["full_product_T_acceptance"] == "NOT_RUN"
                                    for x in proof["cases"]))
                self.assertIn("failure", proof)

    def test_model_to_worker_selftest_failure_can_never_pass_f1(self):
        """Native Qt metrics PASS is insufficient if following IPC selftest fails."""
        sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
        self.assertEqual(len(sha), 40)
        for outcome, expected_ok in (("success", True), ("failure", False),
                                     ("skipped", False), ("", False)):
            with self.subTest(outcome=outcome), tempfile.TemporaryDirectory() as d:
                incoming = Path(d) / "qt.json"
                output = Path(d) / "proof.json"
                incoming.write_text(json.dumps(reference()), encoding="utf-8")
                env = dict(os.environ)
                env["BOARDSI_SOURCE_HEAD_SHA"] = sha
                env["BOARDSI_NATIVE_QT_EXECUTION_OUTCOME"] = outcome
                env["BOARDSI_F1_ADVERSARIAL_TESTS_OUTCOME"] = "success"
                p = subprocess.run(
                    [sys.executable,
                     str(Path(__file__).with_name("m0_f1_native_subcases.py")),
                     str(incoming), str(output), "windows", "x86_64"],
                    env=env, capture_output=True, text=True, timeout=15,
                )
                evidence = json.loads(output.read_text(encoding="utf-8"))
                self.assertEqual(p.returncode, 0 if expected_ok else 1, p.stderr)
                self.assertEqual(evidence["native_qt_execution_outcome"], outcome)
                self.assertEqual(
                    evidence["qt_input_sha256"],
                    hashlib.sha256(incoming.read_bytes()).hexdigest())
                self.assertEqual(evidence["result"],
                                 "PASS_RESTRICTED_SYNTHETIC_SUBCASES_ONLY"
                                 if expected_ok else "FAIL_CLOSED")
                if not expected_ok:
                    self.assertIn("selftest did not PASS", evidence["failure"])
                    self.assertEqual(len(evidence["cases"]), 6)
                    self.assertTrue(all(x["status"] == "NOT_VERIFIED" for x in evidence["cases"]))
                    self.assertTrue(all(x["full_product_T_acceptance"] == "NOT_RUN"
                                        for x in evidence["cases"]))

    def test_adversarial_unit_test_failure_preserves_nonaccepted_proof(self):
        sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
        for verdict in ("success", "failure", "skipped", ""):
            with self.subTest(verdict=verdict), tempfile.TemporaryDirectory() as d:
                incoming, output = Path(d) / "qt.json", Path(d) / "proof.json"
                incoming.write_text(json.dumps(reference()), encoding="utf-8")
                env = dict(os.environ)
                env["BOARDSI_SOURCE_HEAD_SHA"] = sha
                env["BOARDSI_NATIVE_QT_EXECUTION_OUTCOME"] = "success"
                env["BOARDSI_F1_ADVERSARIAL_TESTS_OUTCOME"] = verdict
                result = subprocess.run(
                    [sys.executable,
                     str(Path(__file__).with_name("m0_f1_native_subcases.py")),
                     str(incoming), str(output), "windows", "x86_64"],
                    env=env, capture_output=True, text=True, timeout=15)
                proof = json.loads(output.read_text(encoding="utf-8"))
                self.assertEqual(proof["f1_adversarial_tests_outcome"], verdict)
                self.assertEqual(proof["qt_input_sha256"],
                                 hashlib.sha256(incoming.read_bytes()).hexdigest())
                self.assertEqual(result.returncode, 0 if verdict == "success" else 1)
                if verdict != "success":
                    self.assertEqual(proof["result"], "FAIL_CLOSED")
                    self.assertIn("adversarial verifier unit tests did not PASS",
                                  proof["failure"])
                    self.assertTrue(all(x["status"] == "NOT_VERIFIED" for x in proof["cases"]))
                    self.assertTrue(all(x["independent_case_specific_acceptance_review"] ==
                                        "NOT_PERFORMED" for x in proof["cases"]))
                else:
                    self.assertEqual(proof["result"], "PASS_RESTRICTED_SYNTHETIC_SUBCASES_ONLY")

    def test_wrong_qt_version_fails(self):
        sample = reference()
        sample["qt"] = "6.8.4"
        with self.assertRaisesRegex(ValueError, "Unreviewed"):
            evaluate(sample, "windows", "x86_64")


if __name__ == "__main__":
    unittest.main()
