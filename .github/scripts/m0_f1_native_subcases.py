#!/usr/bin/env python3
"""Fail-closed M0 *synthetic* case-specific Qt evidence validator.

This consumes the actual native QApplication/OS Rust Worker results. It does
NOT implement, imply or approve any full T01..T54 product acceptance, actual
PCB/CAD/SI, distribution clearance or independent human case signoff.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys

CONTRACT = "boardsi.m0.f1.native-subcases.v1"
TARGETS = {("windows", "x86_64"), ("windows", "arm64"), ("cocoa", "arm64")}
CASE_SCENARIOS = {
    "T10": (
        "real-rust-normal", "real-rust-5-rapid-start-cancel-restarts",
        "real-rust-crash", "real-worker-with-cpu-saturated-background-load",
    ),
    "T13": ("real-rust-normal",),
    "T32": (
        "real-rust-crash", "real-rust-5-rapid-start-cancel-restarts",
        "unresponsive", "cancel-hang", "completed-hang",
        "heartbeat-missing-with-progress-traffic", "close-active-worker",
    ),
    "T34": (
        "real-rust-normal", "20000-heartbeats-plus-200-result-progress-frames",
        "real-worker-with-cpu-saturated-background-load",
    ),
    "T46": (
        "duplicate", "reverse", "scope", "snapshot", "frame",
        "json", "truncated", "completion-after-ack",
        "completion-incomplete", "closed-stdin-preserves-scoped-failure",
        "closed-qt-write-channel-preserves-scoped-failure",
    ),
    "T50": (
        "real-rust-normal", "real-rust-5-rapid-start-cancel-restarts",
        "real-worker-with-cpu-saturated-background-load",
        "cancel-hang", "heartbeat-missing-with-progress-traffic",
    ),
}
EVIDENCE_METRICS = {
    "T10": ("cancel_ack_max_ms", "cancel_exit_max_ms", "heartbeat_samples"),
    "T13": ("result_samples", "progress_samples"),
    "T32": ("cancel_ack_max_ms", "cancel_exit_max_ms"),
    "T34": (
        "normal_ui_count", "burst_ui_count", "cpu_stress_ui_count",
        "normal_ui_p95_ms", "burst_ui_p95_ms", "cpu_stress_ui_p95_ms",
    ),
    "T46": ("partial_read_observations",),
    "T50": (
        "global_event_loop_gap_limit_ms", "entire_test_timer_max_gap_ms",
        "normal_ui_p95_ms", "cpu_stress_ui_p95_ms",
        "heartbeat_min_gap_ms", "heartbeat_max_gap_ms",
        "receive_to_widget_update_max_ms", "cancel_ack_max_ms",
    ),
}

def requirement(test: bool, message: str) -> None:
    if not test:
        raise ValueError(message)


def reject_duplicate_json_members(pairs):
    record = {}
    for key, value in pairs:
        requirement(key not in record, "Duplicate JSON field: " + key)
        record[key] = value
    return record


def number(metrics: dict, key: str, minimum: float = 0.0) -> float:
    value = metrics.get(key)
    requirement(type(value) in (int, float),
                "Invalid/missing numeric metric: " + key)
    try:
        candidate = float(value)
    except (OverflowError, ValueError):
        raise ValueError("Invalid/oversized numeric metric: " + key) from None
    requirement(math.isfinite(candidate) and candidate >= minimum,
                "Nonfinite/negative numeric metric: " + key)
    return candidate


def attest_exact_source(checkout_sha: str, claimed_source_sha: str) -> str:
    """Never promote merge-ref output under a claimed different source HEAD."""
    requirement(bool(re.fullmatch(r"[0-9a-f]{40}", checkout_sha)),
                "Invalid checked-out SHA")
    requirement(bool(re.fullmatch(r"[0-9a-f]{40}", claimed_source_sha)),
                "Missing/invalid claimed GitHub source SHA")
    requirement(checkout_sha == claimed_source_sha,
                "Native Qt checkout SHA differs from claimed source HEAD")
    return "EXACT_COMMIT_SHA"


def evaluate(report: dict, platform: str, architecture: str) -> dict:
    """Return subcase PASS only if the real native case array/measurements pass."""
    requirement((platform, architecture) in TARGETS, "Unsupported M0 native target")
    requirement(isinstance(report, dict), "Invalid Qt runtime report")
    requirement(report.get("pass") is True, "Qt runtime overall result is not PASS")
    requirement(report.get("platform") == platform, "Actual Qt platform mismatch")
    requirement(report.get("cpu_arch") == architecture, "Actual native CPU mismatch")
    requirement(report.get("qt") == "6.8.3", "Unreviewed Qt prototype version")
    requirement(type(report.get("assertions")) is int and
                report["assertions"] >= 1, "Missing actual runtime assertions")
    raw_cases = report.get("cases")
    requirement(isinstance(raw_cases, list) and len(raw_cases) >= 25,
                "Missing native test case coverage")
    seen = set()
    for item in raw_cases:
        requirement(isinstance(item, dict) and
                    set(item) == {"name", "pass", "exit_code", "exit_status", "error"} and
                    isinstance(item.get("name"), str) and
                    bool(item["name"]) and item["name"] not in seen and
                    type(item.get("exit_code")) is int and
                    -2147483648 <= item["exit_code"] <= 2147483647 and
                    type(item.get("exit_status")) is int and
                    item["exit_status"] in (0, 1) and
                    isinstance(item.get("error"), str) and
                    len(item["error"]) <= 4096,
                    "Malformed or duplicate Qt scenario producer record")
        seen.add(item["name"])
        requirement(item.get("pass") is True,
                    "Native scenario failed or unverified: " + item["name"])
    actual = report.get("metrics")
    requirement(isinstance(actual, dict), "Qt metrics are missing")
    for test_id, scenarios in CASE_SCENARIOS.items():
        missing = set(scenarios) - seen
        requirement(not missing,
                    "M0 " + test_id + " missing actual executed cases: " + ",".join(sorted(missing)))
    # Retain original Qt test guarantees, never relax p95/response/IPC/heartbeat.
    for name in ("normal_ui", "burst_ui", "cpu_stress_ui"):
        requirement(number(actual, name + "_count") == 150,
                    "Native queued GUI sample count changed: " + name)
        requirement(number(actual, name + "_p95_ms") <= 200,
                    "Qt UI p95 exceeded existing target: " + name)
        requirement(number(actual, name + "_max_ms") <= 1000,
                    "Qt UI max exceeded existing target: " + name)
        requirement(number(actual, name + "_timer_max_gap_ms") <= 1000,
                    "Qt per-scenario timer gap exceeded existing target: " + name)
    requirement(number(actual, "global_event_loop_gap_limit_ms") == 1500,
                "Unexpected M0-only global event-loop policy")
    requirement(number(actual, "entire_test_timer_max_gap_ms") <= 1500,
                "M0 global event-loop timer exceeded 1500 ms")
    requirement(number(actual, "heartbeat_samples") >= 4,
                "Insufficient heartbeat observations")
    requirement(number(actual, "heartbeat_min_gap_ms") >= 800,
                "Heartbeat too fast; policy changed")
    requirement(number(actual, "heartbeat_max_gap_ms") <= 1500,
                "Heartbeat deadline exceeded")
    requirement(number(actual, "receive_to_widget_update_max_ms") <= 1000,
                "Widget update latency exceeded original target")
    requirement(number(actual, "result_samples") == 100 and
                number(actual, "progress_samples") == 100,
                "Worker result/progress samples incomplete")
    requirement(number(actual, "cancel_ack_max_ms") <= 1000,
                "Cancel acknowledgement exceeds original target")
    requirement(number(actual, "cancel_exit_max_ms") <= 3000,
                "Worker not reaped within existing test deadline")
    requirement(number(actual, "partial_read_observations", 1) >= 1,
                "Missing actual partial frame reads")
    requirement(number(actual, "cpu_stress_threads") >= 2,
                "CPU-saturated scenario did not exercise concurrency")
    requirement(number(actual, "render_width_px", 1) >= 1 and
                number(actual, "render_height_px", 1) >= 1,
                "No real Qt widget raster rendered")
    results = []
    for test_id, scenarios in CASE_SCENARIOS.items():
        evidence = {key: actual[key] for key in EVIDENCE_METRICS[test_id]}
        results.append({
            "id": test_id,
            "status": "PASS_RESTRICTED_SYNTHETIC_SUBCASE",
            "scenarios_executed": list(scenarios),
            "metrics_observed": evidence,
            "independent_case_specific_acceptance_review": "NOT_PERFORMED",
            "full_product_T_acceptance": "NOT_RUN",
        })
    return {
        "schema": CONTRACT,
        "scope": "M0 native Qt Widgets + independent Rust Worker synthetic technology only",
        "os_qt_backend": platform,
        "cpu_architecture": architecture,
        "qt_version": report["qt"],
        "scenario_count_all_passed": len(raw_cases),
        "runtime_assertions": report["assertions"],
        "result": "PASS_RESTRICTED_SYNTHETIC_SUBCASES_ONLY",
        "cases": results,
        "outside_M0_or_not_proven": ["T11 GPU equivalence", "T35 actual SSD spill/physical PC",
                                  "T39 common PCB/S-parameters", "T54 durable checkpoint recovery",
                                  "real CAD import/Artwork/SI", "Qt clean OS distribution/legal review",
                                  "independent owner case signoff"],
        "formal_M0": "BLOCKED",
        "formal_F1": "PARTIAL",
    }


def cli() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("qt_json", type=Path)
    parser.add_argument("output_json", type=Path)
    parser.add_argument("qt_platform", choices=["windows", "cocoa"])
    parser.add_argument("cpu_arch", choices=["x86_64", "arm64"])
    args = parser.parse_args()
    data = {
        "schema": CONTRACT,
        "result": "FAIL_CLOSED",
        "formal_M0": "BLOCKED",
        "formal_F1": "PARTIAL",
        "source_run_id": os.environ.get("GITHUB_RUN_ID", "NOT_GITHUB"),
        "source_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", "NOT_GITHUB"),
        "github_source_sha": os.environ.get("BOARDSI_SOURCE_HEAD_SHA", "NOT_GITHUB"),
        "checkout_sha": "NOT_VERIFIED",
        "source_identity_proof": "NOT_VERIFIED",
        "native_qt_execution_outcome": os.environ.get("BOARDSI_NATIVE_QT_EXECUTION_OUTCOME", "NOT_VERIFIED"),
        "f1_adversarial_tests_outcome": os.environ.get("BOARDSI_F1_ADVERSARIAL_TESTS_OUTCOME", "NOT_VERIFIED"),
        "qt_input_sha256": "NOT_COMPUTED",
        "cases": [{
            "id": test_id,
            "status": "NOT_VERIFIED",
            "scenarios_executed": [],
            "independent_case_specific_acceptance_review": "NOT_PERFORMED",
            "full_product_T_acceptance": "NOT_RUN",
        } for test_id in CASE_SCENARIOS],
        "outside_M0_or_not_proven": [
            "later phase full T acceptance", "independent owner case signoff",
            "clean OS legal/dependency qualification", "8 GiB physical reference PC",
        ],
    }
    try:
        sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
        requirement(bool(re.fullmatch(r"[0-9a-f]{40}", sha)),
                    "Unable to attest checked-out source SHA")
        data["checkout_sha"] = sha
        raw = args.qt_json.read_bytes()
        # Input already exists, so preserve its hash even if size/shape invalid.
        data["qt_input_sha256"] = hashlib.sha256(raw).hexdigest()
        requirement(len(raw) <= 4_000_000 and bool(raw), "Empty or unbounded native report")
        source = json.loads(raw.decode("utf-8"),
                            object_pairs_hook=reject_duplicate_json_members)
        summary = evaluate(source, args.qt_platform, args.cpu_arch)
        # Native Qt checkout is explicitly pinned to the PR source SHA.
        # An Actions merge-ref checkout is a different commit and MUST fail.
        proof = attest_exact_source(sha, data["github_source_sha"])
        # A PASS for the runtime harness alone cannot mask failure in the
        # immediately following Qt-model -> real Rust Worker --selftest.
        # This records FAIL_CLOSED even if the runtime metrics themselves PASS.
        requirement(data["native_qt_execution_outcome"] == "success",
                    "Native Qt backend or model-to-Worker selftest did not PASS")
        requirement(data["f1_adversarial_tests_outcome"] == "success",
                    "F1 adversarial verifier unit tests did not PASS")
        data.update(summary)
        data["source_identity_proof"] = proof
    except (OSError, UnicodeError, ValueError, RecursionError,
            subprocess.CalledProcessError) as exc:
        data["failure"] = f"{type(exc).__name__}: {exc}"
    args.output_json.parent.mkdir(parents=True, exist_ok=True)
    args.output_json.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n",
                                encoding="utf-8")
    print(json.dumps(data, indent=2, sort_keys=True))
    return 0 if data["result"] == "PASS_RESTRICTED_SYNTHETIC_SUBCASES_ONLY" else 1


if __name__ == "__main__":
    sys.exit(cli())
