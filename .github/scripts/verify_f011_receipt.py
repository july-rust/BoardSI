"""Verify the actual F011 GUI-generated receipt, never a console-only PASS."""
import json
import sys
from pathlib import Path


def main() -> None:
    data = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    assert data["schema"] == "boardsi-f011-qualification-v1", data
    assert data["pass"] is True and data["physical_si"] == "NOT_RUN", data
    assert data["source_sha"] == sys.argv[2], data
    cases = data["cases"]
    expected = [
        (128, False), (129, False), (1000, False),
        (10000, False), (10000, True), (10001, False), (100000, False),
    ]
    assert [(c["count"], c["mixed"]) for c in cases] == expected, data
    for c in cases:
        assert c["rendered"] == (c["count"] <= 10000), c
        assert c["source_absent"] is True and c["physical_si"] == "NOT_RUN", c
        assert 0 < c["stream_bytes"] <= 16 * 1024 * 1024, c
        assert c["batch_count"] > 0 if c["rendered"] else c["batch_count"] == 0, c
        assert c["open_ms"] >= 0 and c["max_timer_gap_ms"] >= 0, c
        assert c["qt_process_peak_rss_bytes"] > 0, c
    pairs = data["transport_ab"]
    assert len(pairs) == 3 and all(p["exact_receipt_equal"] for p in pairs), data
    assert all(p["legacy_bytes"] > 0 and p["framed_bytes"] > 0 for p in pairs), data
    closed = data["close_reopen"]
    assert [(c["mixed"], c["repeat"]) for c in closed] == [
        (False, 1), (False, 2), (True, 1), (True, 2)
    ], data
    assert all(c["closed_scene_items"] == 0 for c in closed), data
    print("F011 exact-head durable qualification receipt verified")


if __name__ == "__main__":
    main()
