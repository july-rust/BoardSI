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
    # F012 shares the same actual Main Qt fixture and exact-head receipt.
    records = data["f012_inspection"]
    assert records, data
    if records:
        counts = [128, 129, 1000, 10000, 10001, 100000]
        assert [(c["count"], c["offset"]) for c in records] == [
            (count, offset) for count in counts for offset in (0, count - 1, 0)
        ], records
        for c in records:
            assert c["mode"] == "persistent-read-only-worker" and c["physical_si"] == "NOT_RUN", c
            assert c["session_pid"] > 0, c
            assert 0 < c["response_bytes"] < 60000 and 1 <= c["rows"] <= 128, c
            assert c["rows"] == 1 if c["offset"] == c["count"] - 1 else True, c
            assert 0 <= c["page_ms"] <= (20000 if c["count"] <= 10000 else 60000), c
            assert 0 <= c["max_timer_gap_ms"] <= 500, c
            assert 0 < c["qt_peak_or_sampled_rss_bytes"] <= 1073741824, c
            assert -1 <= c["worker_sampled_peak_rss_bytes"] <= 1073741824, c
            if c["count"] >= 10000:
                assert c["worker_sampled_peak_rss_bytes"] > 0, c
        for i in range(0, len(records), 3):
            assert len({c["session_pid"] for c in records[i:i + 3]}) == 1, records
        assert data["f012_lifecycle"] == {
            "cancel": True, "timeout": True, "crash": True,
            "close_reopen": True, "unchanged_sha": True,
        }, data
        print("F013 persistent-worker regression and actual lifecycle measurements verified")
    print("F011 exact-head durable qualification receipt verified")


if __name__ == "__main__":
    main()
