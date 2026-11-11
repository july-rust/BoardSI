//! E_003: adversarial on-wire IPC v1 compatibility; no protocol change.
use boardsi_ipc::{encode, read_message, Body, Message, Scope, Snapshot, MAX_FRAME};
use std::io::{Cursor, ErrorKind};

fn frame(payload: &[u8]) -> Vec<u8> {
    let mut bytes = (payload.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(payload);
    bytes
}

fn message(seq: u64, body: Body) -> Message {
    let snapshot = Snapshot {
        points: 1,
        delay_ms: 0,
    };
    Message::new("e003/run", &snapshot.id(), seq, body)
}

#[test]
fn malformed_wire_frames_fail_closed_without_implicit_fallback() {
    let valid = encode(&message(1, Body::Started)).unwrap();
    let json = String::from_utf8(valid[4..].to_vec()).unwrap();
    let invalid = [
        frame(json.replace("\"version\":1", "\"version\":2").as_bytes()),
        frame(json.replacen('{', "{\"unexpected\":true,", 1).as_bytes()),
        frame(
            json.replace(
                "\"type\":\"Started\"",
                "\"type\":\"Started\",\"unexpected\":true",
            )
            .as_bytes(),
        ),
        frame(
            json.replace("\"kind\":\"response\"", "\"kind\":\"event\"")
                .as_bytes(),
        ),
        frame(&[0xff]),
        vec![0, 0],
        vec![0, 0, 0, 8, b'{', b'}'],
        vec![0, 0, 0, 0],
        (MAX_FRAME as u32 + 1).to_be_bytes().to_vec(),
    ];
    for (case, bytes) in invalid.into_iter().enumerate() {
        let result = read_message(&mut Cursor::new(bytes));
        let err = result.expect_err("invalid wire input must never decode");
        assert!(
            matches!(
                err.kind(),
                ErrorKind::InvalidData | ErrorKind::UnexpectedEof
            ),
            "case {case}: unexpected error {err}"
        );
    }
    // An exact v1 frame must remain byte-for-byte accepted.
    assert_eq!(
        read_message(&mut Cursor::new(valid)).unwrap(),
        Some(message(1, Body::Started))
    );
}

#[test]
fn rejected_version_and_scope_do_not_advance_sequence_cursor() {
    let snap = Snapshot {
        points: 1,
        delay_ms: 0,
    };
    let mut scope = Scope::new("e003/run", &snap.id());
    scope.accept(&message(1, Body::Started)).unwrap();
    let mut unsupported = message(2, Body::Heartbeat);
    unsupported.version = 2;
    assert_eq!(scope.accept(&unsupported), Err("UnsupportedVersion"));
    let mut foreign = message(2, Body::Heartbeat);
    foreign.run_id = "previous-run".into();
    assert_eq!(scope.accept(&foreign), Err("RunMismatch"));
    let mut wrong_snapshot = message(2, Body::Heartbeat);
    wrong_snapshot.snapshot_id = "stale-snapshot".into();
    assert_eq!(scope.accept(&wrong_snapshot), Err("SnapshotMismatch"));
    assert_eq!(
        scope.accept(&message(3, Body::Heartbeat)),
        Err("SequenceMismatch")
    );
    scope.accept(&message(2, Body::Heartbeat)).unwrap();
    scope.accept(&message(3, Body::Heartbeat)).unwrap();
}
