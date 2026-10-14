use boardsi_ipc::{Body, Message, RunGuard, Snapshot, State};
use std::time::{Duration, Instant};

#[test]
fn completion_wins_only_with_all_results_before_cancel_ack() {
    let snapshot = Snapshot {
        points: 1,
        delay_ms: 0,
    };
    let now = Instant::now();
    let message = |seq, body| Message::new("race", &snapshot.id(), seq, body);
    for acknowledged in [false, true] {
        let mut guard = RunGuard::new("race", &snapshot, now);
        guard.accept(message(1, Body::Started), now).unwrap();
        guard.cancel_requested(now).unwrap();
        // Incomplete completion cannot consume the next sequence cursor.
        assert!(guard.accept(message(2, Body::Completed), now).is_err());
        let mut seq = 2;
        if acknowledged {
            guard
                .accept(message(seq, Body::CancelAcknowledged), now)
                .unwrap();
            seq += 1;
        }
        guard
            .accept(
                message(
                    seq,
                    Body::Result {
                        point: 0,
                        frequency_hz: 0.0,
                        s11: [0.0, 0.0],
                        s21: [1.0, 0.0],
                        synthetic: true,
                    },
                ),
                now,
            )
            .unwrap();
        seq += 1;
        if acknowledged {
            assert!(guard.accept(message(seq, Body::Completed), now).is_err());
            guard.accept(message(seq, Body::Cancelled), now).unwrap();
            assert_eq!(guard.state, State::Cancelled);
            assert!(guard.results.is_empty());
        } else {
            guard.accept(message(seq, Body::Completed), now).unwrap();
            assert_eq!(guard.state, State::Completed);
            assert_eq!(guard.results.len(), 1);
            assert!(guard
                .accept(message(seq + 1, Body::CancelAcknowledged), now)
                .is_err());
            assert!(!guard.warnings(now + Duration::from_millis(9999)).2);
            assert!(guard.warnings(now + Duration::from_secs(10)).2);
            guard.transport_failed();
            assert!(guard.results.is_empty());
            assert!(!guard.warnings(now + Duration::from_secs(20)).2);
        }
    }
}
