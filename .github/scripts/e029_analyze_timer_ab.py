#!/usr/bin/env python3
"""E_029 diagnostic validator; never treats a timer measurement as M0 acceptance."""
import json
import os
import pathlib
import re
import statistics
import sys

ORDER = ("control", "window", "window", "control") * 2
SCHEMA = "boardsi-e029-timer-ab-v1"


def summarize(directory: pathlib.Path):
    filenames = sorted(directory.glob("e029-ab-??-*.json"))
    if len(filenames) != len(ORDER):
        raise ValueError(f"Expected {len(ORDER)} native process results; got {len(filenames)}")
    expected_sha = os.environ.get("BOARDSI_E029_HEAD_SHA", "")
    if not re.fullmatch(r"[0-9a-fA-F]{40}", expected_sha):
        raise ValueError("Exact source SHA missing or invalid")
    samples = []
    for idx, (filename, mode) in enumerate(zip(filenames, ORDER), 1):
        if filename.name != f"e029-ab-{idx:02d}-{mode}.json":
            raise ValueError(f"Wrong ABBA order or file: {filename.name}")
        record = json.loads(filename.read_text(encoding="utf-8"))
        if record.get("schema") != SCHEMA or record.get("mode") != mode:
            raise ValueError(f"Unsupported schema or condition: {filename.name}")
        if record.get("source_sha") != expected_sha:
            raise ValueError(f"Non-exact SHA: {filename.name}")
        if record.get("diagnostic_only") is not True or record.get("pass") is not True:
            raise ValueError(f"Missing/failed diagnostic observations: {filename.name}")
        if record.get("native_backend_verified") is not True or record.get("platform") != record.get("expected_platform"):
            raise ValueError(f"Not actually running expected native backend: {filename.name}")
        if record.get("global_1500ms_acceptance") != "NOT_EVALUATED":
            raise ValueError("Diagnostic attempted to claim M0 global-timer acceptance")
        tick = record.get("timer_armed_ms", -1)
        first = record.get("first_precise_ms", -1)
        zero = record.get("first_zero_ms", -1)
        gap = record.get("first_precise_gap_ms", -1)
        if not all(isinstance(v, int) and not isinstance(v, bool) for v in (tick, first, zero, gap)):
            raise ValueError(f"Noninteger native timestamps: {filename.name}")
        if tick < 0 or first < tick or zero < tick or gap != first - tick:
            raise ValueError(f"Invalid timing order/delta: {filename.name}")
        show_begin = record.get("show_begin_ms")
        show_end = record.get("show_end_ms")
        if mode == "control" and (show_begin != -1 or show_end != -1):
            raise ValueError("Control created a recorded native window")
        if mode == "window" and (not isinstance(show_begin, int) or not isinstance(show_end, int)
                                 or show_begin < tick or show_end < show_begin):
            raise ValueError("Experimental window missing native show timing")
        if record.get("timer_callback_count", 0) < 1:
            raise ValueError("No PreciseTimer callback actually fired")
        samples.append(record)

    control = [s["first_precise_gap_ms"] for s in samples if s["mode"] == "control"]
    window = [s["first_precise_gap_ms"] for s in samples if s["mode"] == "window"]
    result = {
        "schema": "boardsi-e029-timer-ab-summary-v1",
        "source_sha": expected_sha,
        "diagnostic_only": True,
        "experiment_executed": True,
        "root_cause_certified": False,
        "global_1500ms_acceptance": "NOT_EVALUATED",
        "sample_count": len(samples),
        "sequence": list(ORDER),
        "platform": samples[0]["platform"],
        "qt": samples[0]["qt"],
        "os": samples[0]["os"],
        "control_gaps_ms": control,
        "window_gaps_ms": window,
        "control_median_ms": statistics.median(control),
        "window_median_ms": statistics.median(window),
        "window_minus_control_median_ms": statistics.median(window) - statistics.median(control),
        "max_observed_first_callback_gap_ms": max(control + window),
        "interpretation": "Correlation only. Runner load and cold-host independence NOT_VERIFIED.",
        "raw_files": [f.name for f in filenames],
    }
    if len({(s["platform"], s["qt"], s["source_sha"], s["architecture"]) for s in samples}) != 1:
        raise ValueError("Samples came from mismatched platform/Qt/SHA/CPU conditions")
    return result


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: e029_analyze_timer_ab.py <build-directory>")
    directory = pathlib.Path(sys.argv[1])
    try:
        result = summarize(directory)
    except (ValueError, KeyError, OSError, json.JSONDecodeError) as err:
        print(f"E029_DIAGNOSTIC_FAIL: {err}", file=sys.stderr)
        raise SystemExit(1)
    output = directory / "e029-ab-summary.json"
    output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(output.read_text(encoding="utf-8"))


if __name__ == "__main__":
    main()
