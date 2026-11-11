//! E_005: a cancelled run never accepts calculation data after cancellation ACK.
use boardsi_ipc::{Body, Message, RunGuard, Snapshot, State};
use std::time::{Duration, Instant};

fn message(run: &str, snapshot: &Snapshot, seq: u64, body: Body) -> Message {
    Message::new(run, &snapshot.id(), seq, body)
}

fn result(point: u32) -> Body {
    Body::Result {
        point,
        frequency_hz: f64::from(point) * 1e6,
        s11: [0.0, 0.0],
        s21: [1.0, 0.0],
        synthetic: true,
    }
}

#[test]
fn ack_rejects_late_result_and_progress_without_advancing_cursor_or_clock() {
    let snapshot = Snapshot { points: 2, delay_ms: 50 };
    let start = Instant::now();
    let mut guard = RunGuard::new("run-a", &snapshot, start);
    guard.accept(message("run-a", &snapshot, 1, Body::Started), start).unwrap();
    guard.cancel_requested(start + Duration::from_millis(1)).unwrap();
    // A genuinely in-flight result before the ACK remains provisional, then is discarded.
    guard.accept(message("run-a", &snapshot, 2, result(0)), start).unwrap();
    guard.accept(message("run-a", &snapshot, 3, Body::CancelAcknowledged), start).unwrap();
    assert_eq!(guard.state, State::CancelAcknowledged);
    assert_eq!(guard.results.len(), 1);
    let accepted_clock = guard.last_rx;
    let late_at = start + Duration::from_secs(1);
    for body in [result(1), Body::Progress { done: 1, total: 2 }] {
        assert_eq!(
            guard.accept(message("run-a", &snapshot, 4, body), late_at),
            Err("InvalidStateOrPayload")
        );
        assert_eq!(guard.last_rx, accepted_clock, "rejected data refreshed liveness");
        assert_eq!(guard.results.len(), 1, "rejected data was retained");
        assert_eq!(guard.state, State::CancelAcknowledged);
    }
    // A rejected message must not consume the valid sequence number.
    guard.accept(message("run-a", &snapshot, 4, Body::Heartbeat), late_at).unwrap();
    guard.accept(message("run-a", &snapshot, 5, Body::Cancelled), late_at).unwrap();
    assert_eq!(guard.state, State::Cancelled);
    assert!(guard.results.is_empty(), "cancelled run retained provisional data");
    assert_eq!(
        guard.accept(message("run-a", &snapshot, 6, result(1)), late_at),
        Err("AlreadyTerminal")
    );
}

#[test]
fn prior_run_response_does_not_change_new_run_state_or_sequence() {
    let snapshot = Snapshot { points: 1, delay_ms: 50 };
    let now = Instant::now();
    let mut current = RunGuard::new("new-run", &snapshot, now);
    let previous = message("previous-run", &snapshot, 1, Body::Started);
    assert_eq!(current.accept(previous, now), Err("RunMismatch"));
    assert_eq!(current.state, State::Awaiting);
    assert_eq!(current.last_rx, now);
    current.accept(message("new-run", &snapshot, 1, Body::Started), now).unwrap();
    current.cancel_requested(now).unwrap();
    current.accept(message("new-run", &snapshot, 2, Body::CancelAcknowledged), now).unwrap();
    current.accept(message("new-run", &snapshot, 3, Body::Cancelled), now).unwrap();
    assert_eq!(current.state, State::Cancelled);
}
