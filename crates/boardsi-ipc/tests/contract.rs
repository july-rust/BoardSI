use boardsi_ipc::*;
use std::io::{self, Cursor, Read};
use std::time::{Duration, Instant};
fn snapshot() -> Snapshot {
    Snapshot {
        points: 2,
        delay_ms: 10,
    }
}
fn msg(seq: u64, body: Body) -> Message {
    Message::new("session-a/run-1", &snapshot().id(), seq, body)
}
#[test]
fn roundtrip_all_message_types() {
    let bodies = vec![
        Body::Start {
            snapshot: snapshot(),
        },
        Body::Cancel,
        Body::Started,
        Body::Progress { done: 1, total: 2 },
        Body::Result {
            point: 0,
            frequency_hz: 0.0,
            s11: [0.0, 0.0],
            s21: [1.0, 0.0],
            synthetic: true,
        },
        Body::Completed,
        Body::Failed {
            code: "test".into(),
            context: "합성".into(),
        },
        Body::CancelAcknowledged,
        Body::Cancelled,
        Body::Heartbeat,
    ];
    for body in bodies {
        let m = msg(1, body);
        assert_eq!(
            read_message(&mut Cursor::new(encode(&m).unwrap())).unwrap(),
            Some(m)
        );
    }
}
struct ByteReader(Cursor<Vec<u8>>);
impl Read for ByteReader {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        let n = b.len().min(1);
        self.0.read(&mut b[..n])
    }
}
#[test]
fn partial_and_multiple_frames() {
    let m = msg(1, Body::Started);
    let mut bytes = encode(&m).unwrap();
    bytes.extend(encode(&msg(2, Body::Heartbeat)).unwrap());
    let mut r = ByteReader(Cursor::new(bytes));
    assert_eq!(read_message(&mut r).unwrap(), Some(m));
    assert_eq!(read_message(&mut r).unwrap().unwrap().seq, 2);
    assert!(read_message(&mut r).unwrap().is_none());
}
#[test]
fn truncated_invalid_and_oversized_frames() {
    for bytes in [
        vec![0],
        vec![0, 0, 0, 2, b'{'],
        vec![0, 0, 0, 1, b'x'],
        vec![0, 0, 0, 0],
        (MAX_FRAME as u32 + 1).to_be_bytes().to_vec(),
    ] {
        assert!(read_message(&mut Cursor::new(bytes)).is_err());
    }
    let mut m = msg(
        1,
        Body::Failed {
            code: "x".into(),
            context: "x".repeat(MAX_FRAME),
        },
    );
    assert!(encode(&m).is_err());
    m.body = Body::Heartbeat;
    m.kind = "request".into();
    assert!(encode(&m).is_err());
}
#[test]
fn version_unknown_field_and_nonfinite_rejected() {
    let m = msg(1, Body::Started);
    let frame = encode(&m).unwrap();
    let json = String::from_utf8(frame[4..].to_vec()).unwrap();
    for invalid in [
        json.replace("\"version\":1", "\"version\":2"),
        json.replacen('{', "{\"extra\":true,", 1),
    ] {
        let mut bytes = (invalid.len() as u32).to_be_bytes().to_vec();
        bytes.extend(invalid.bytes());
        assert!(read_message(&mut Cursor::new(bytes)).is_err());
    }
    assert!(encode(&msg(
        1,
        Body::Result {
            point: 0,
            frequency_hz: f64::NAN,
            s11: [0.0, 0.0],
            s21: [1.0, 0.0],
            synthetic: true
        }
    ))
    .is_err());
}
#[test]
fn duplicate_reverse_gap_and_foreign_scope() {
    let mut s = Scope::new("session-a/run-1", &snapshot().id());
    s.accept(&msg(1, Body::Started)).unwrap();
    for n in [1, 0, 3] {
        assert!(s.accept(&msg(n, Body::Heartbeat)).is_err());
    }
    let mut foreign = msg(2, Body::Heartbeat);
    foreign.run_id = "old-session/run-1".into();
    assert_eq!(s.accept(&foreign), Err("RunMismatch"));
    foreign.run_id = "session-a/run-1".into();
    foreign.snapshot_id = "changed".into();
    assert_eq!(s.accept(&foreign), Err("SnapshotMismatch"));
    s.accept(&msg(2, Body::Heartbeat)).unwrap();
}
#[test]
fn snapshot_changed_or_out_of_bounds_rejected() {
    let mut m = msg(
        1,
        Body::Start {
            snapshot: snapshot(),
        },
    );
    m.snapshot_id = "fake".into();
    assert!(m.validate().is_err());
    for points in [0, 1001] {
        assert!(Snapshot {
            points,
            delay_ms: 1
        }
        .validate()
        .is_err());
    }
    assert!(Snapshot {
        points: 1,
        delay_ms: 1001
    }
    .validate()
    .is_err());
    assert_ne!(
        snapshot().id(),
        Snapshot {
            points: 2,
            delay_ms: 11
        }
        .id()
    );
}
#[test]
fn incomplete_completion_and_restart_rejected() {
    let now = Instant::now();
    let mut g = RunGuard::new("session-a/run-1", &snapshot(), now);
    g.accept(msg(1, Body::Started), now).unwrap();
    assert!(g.accept(msg(2, Body::Completed), now).is_err());
    let mut restart = RunGuard::new("session-b/run-1", &snapshot(), now);
    assert!(restart.accept(msg(1, Body::Started), now).is_err());
    assert_eq!(restart.state, State::Awaiting);
    assert!(restart.results.is_empty());
}
#[test]
fn cancel_ack_is_not_terminal_and_failed_clears_partial() {
    let now = Instant::now();
    let mut g = RunGuard::new("session-a/run-1", &snapshot(), now);
    g.accept(msg(1, Body::Started), now).unwrap();
    g.cancel_requested(now).unwrap();
    g.accept(msg(2, Body::CancelAcknowledged), now).unwrap();
    assert!(!g.state.terminal());
    assert!(g.accept(msg(3, Body::Completed), now).is_err());
    g.accept(msg(3, Body::Cancelled), now).unwrap();
    assert_eq!(g.state, State::Cancelled);
    assert!(g.accept(msg(4, Body::Heartbeat), now).is_err());
}
#[test]
fn warning_clock_and_force_option_exact_boundaries() {
    let now = Instant::now();
    let mut g = RunGuard::new("session-a/run-1", &snapshot(), now);
    g.accept(msg(1, Body::Started), now).unwrap();
    g.cancel_requested(now).unwrap();
    assert_eq!(
        g.warnings(now + Duration::from_millis(4999)),
        (false, false, false)
    );
    assert_eq!(
        g.warnings(now + Duration::from_secs(5)),
        (true, true, false)
    );
    g.accept(
        msg(2, Body::Progress { done: 0, total: 2 }),
        now + Duration::from_secs(6),
    )
    .unwrap();
    assert_eq!(
        g.warnings(now + Duration::from_secs(10)),
        (false, true, true)
    );
    g.transport_failed();
    assert_eq!(g.state, State::Failed);
    assert_eq!(
        g.warnings(now + Duration::from_secs(20)),
        (false, false, false)
    );
}
#[test]
fn cancelled_terminal_keeps_force_option_until_os_exit() {
    // Scoped Cancelled is a protocol event, not proof that the child has exited.
    // Clients must still be able to force-stop a worker stuck during shutdown.
    let now = Instant::now();
    let mut g = RunGuard::new("session-a/run-1", &snapshot(), now);
    g.accept(msg(1, Body::Started), now).unwrap();
    g.cancel_requested(now).unwrap();
    g.accept(msg(2, Body::CancelAcknowledged), now).unwrap();
    g.accept(msg(3, Body::Cancelled), now).unwrap();
    assert_eq!(g.state, State::Cancelled);
    assert_eq!(
        g.warnings(now + Duration::from_millis(9999)),
        (false, false, false)
    );
    assert_eq!(
        g.warnings(now + Duration::from_secs(10)),
        (false, false, true)
    );
    // An actual transport failure is not a protocol-level Cancelled status.
    g.transport_failed();
    assert_eq!(
        g.warnings(now + Duration::from_secs(20)),
        (false, false, false)
    );
}
#[test]
fn completed_then_abnormal_exit_invalidates_results() {
    let now = Instant::now();
    let mut g = RunGuard::new("session-a/run-1", &snapshot(), now);
    g.accept(msg(1, Body::Started), now).unwrap();
    for i in 0..2 {
        g.accept(
            msg(
                2 + u64::from(i),
                Body::Result {
                    point: i,
                    frequency_hz: f64::from(i) * 1e6,
                    s11: [0.0, 0.0],
                    s21: [1.0, 0.0],
                    synthetic: true,
                },
            ),
            now,
        )
        .unwrap();
    }
    g.accept(msg(4, Body::Completed), now).unwrap();
    assert_eq!(g.results.len(), 2);
    g.transport_failed();
    assert!(g.results.is_empty());
    assert_eq!(g.state, State::Failed);
}
