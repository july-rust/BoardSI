use boardsi_ipc::{Body, Snapshot, State};
use boardsi_worker::Client;
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};
fn spawn(fault: Option<&str>, points: u32, delay_ms: u32) -> Client {
    Client::spawn(
        Path::new(env!("CARGO_BIN_EXE_boardsi-worker")),
        fault,
        "process-test/session-1/run-1",
        Snapshot { points, delay_ms },
    )
    .unwrap()
}
fn until(
    c: &mut Client,
    timeout: Duration,
    predicate: impl Fn(&Client) -> bool,
) -> Vec<boardsi_ipc::Message> {
    let end = Instant::now() + timeout;
    let mut events = vec![];
    loop {
        events.extend(c.poll().unwrap());
        if predicate(c) {
            return events;
        }
        assert!(
            Instant::now() < end,
            "timeout state={:?} exit={:?}",
            c.guard.state,
            c.exit
        );
        thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn normal_real_worker_completed_and_clean_exit() {
    let mut c = spawn(None, 10, 2);
    let events = until(&mut c, Duration::from_secs(5), |c| {
        c.exit.is_some() && c.guard.state == State::Completed
    });
    assert!(c.exit.unwrap().success());
    assert_eq!(c.guard.results.len(), 10);
    assert!(events.iter().any(|m| m.body == Body::Heartbeat));
    println!("PROCESS_NORMAL results=10 terminal=Completed exit=success");
}
#[test]
fn cancel_acknowledgement_then_cancelled_then_exit() {
    let mut c = spawn(None, 100, 20);
    until(&mut c, Duration::from_secs(5), |c| {
        c.guard.state == State::Running
    });
    let begin = Instant::now();
    c.cancel().unwrap();
    let events = until(&mut c, Duration::from_secs(5), |c| {
        c.exit.is_some() && c.guard.state == State::Cancelled
    });
    let ack = events
        .iter()
        .position(|m| m.body == Body::CancelAcknowledged)
        .unwrap();
    let end = events
        .iter()
        .position(|m| m.body == Body::Cancelled)
        .unwrap();
    assert!(ack < end);
    assert!(begin.elapsed() < Duration::from_secs(1));
    assert!(c.exit.unwrap().success());
    assert!(c.guard.results.is_empty());
    println!(
        "PROCESS_CANCEL ack_then_cancelled_exit_ms={:.3}",
        begin.elapsed().as_secs_f64() * 1000.0
    );
}
#[test]
fn actual_crash_isolated_client_survives() {
    let mut c = spawn(Some("--fault=crash"), 10, 10);
    until(&mut c, Duration::from_secs(5), |c| c.exit.is_some());
    assert!(!c.exit.unwrap().success());
    assert_eq!(c.guard.state, State::Failed);
    assert!(c.guard.results.is_empty());
    println!("PROCESS_CRASH client_alive=true");
}
#[test]
fn force_kill_isolated_and_reaped() {
    let mut c = spawn(None, 100, 100);
    until(&mut c, Duration::from_secs(5), |c| {
        c.guard.state == State::Running
    });
    c.force_kill().unwrap();
    until(&mut c, Duration::from_secs(5), |c| c.exit.is_some());
    assert_eq!(c.guard.state, State::Failed);
    assert!(!c.exit.unwrap().success());
}
#[test]
fn real_silence_warning_and_ten_second_cancel_option() {
    let mut c = spawn(Some("--fault=unresponsive"), 100, 100);
    until(&mut c, Duration::from_secs(5), |c| {
        c.guard.state == State::Running
    });
    c.cancel().unwrap();
    let begin = Instant::now();
    let mut silence_at = None;
    while begin.elapsed() < Duration::from_millis(10_150) {
        c.poll().unwrap();
        let (silence, hb, force) = c.guard.warnings(Instant::now());
        if silence && hb && silence_at.is_none() {
            silence_at = Some(begin.elapsed());
        }
        if force {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(silence_at.unwrap() < Duration::from_millis(5500));
    assert!(c.guard.warnings(Instant::now()).2);
    assert!(c.exit.is_none());
    assert_eq!(c.guard.state, State::CancelRequested);
    c.force_kill().unwrap();
    until(&mut c, Duration::from_secs(5), |c| c.exit.is_some());
    println!(
        "PROCESS_SILENCE warning_ms={:.3} force_option_ms={:.3}",
        silence_at.unwrap().as_secs_f64() * 1000.0,
        begin.elapsed().as_secs_f64() * 1000.0
    );
}
#[test]
fn missing_heartbeat_not_masked_by_progress() {
    let mut c = spawn(Some("--fault=no-heartbeat"), 30, 250);
    until(&mut c, Duration::from_secs(5), |c| {
        c.guard.state == State::Running
    });
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        c.poll().unwrap();
        let (silence, hb, _) = c.guard.warnings(Instant::now());
        if hb {
            assert!(!silence);
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    c.cancel().unwrap();
    until(&mut c, Duration::from_secs(5), |c| c.exit.is_some());
    assert_eq!(c.guard.state, State::Cancelled);
}
#[test]
fn heartbeat_during_delay_real_interval() {
    let mut c = spawn(None, 5, 600);
    let begin = Instant::now();
    let mut times = vec![];
    while c.exit.is_none() || !c.guard.state.terminal() {
        for m in c.poll().unwrap() {
            if m.body == Body::Heartbeat {
                times.push(begin.elapsed());
            }
        }
        assert!(begin.elapsed() < Duration::from_secs(6));
        thread::sleep(Duration::from_millis(2));
    }
    assert!(times.len() >= 3);
    for pair in times.windows(2) {
        let interval = pair[1] - pair[0];
        assert!(interval >= Duration::from_millis(900) && interval < Duration::from_millis(1300));
        println!(
            "HEARTBEAT_INTERVAL_MS {:.3}",
            interval.as_secs_f64() * 1000.0
        );
    }
}
#[test]
fn dropping_active_client_cleans_up_without_hanging() {
    let begin = Instant::now();
    {
        let mut c = spawn(Some("--fault=unresponsive"), 100, 100);
        until(&mut c, Duration::from_secs(5), |c| {
            c.guard.state == State::Running
        });
    }
    assert!(begin.elapsed() < Duration::from_secs(5));
    {
        let _c = spawn(None, 1000, 0);
        thread::sleep(Duration::from_millis(100));
    }
    assert!(begin.elapsed() < Duration::from_secs(5));
}
#[test]
fn worker_rejects_malformed_and_truncated_frames() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    for bytes in [
        vec![0, 0, 0, 1, b'x'],
        vec![0, 0, 0, 10, b'{'],
        vec![0, 1, 0, 1],
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_boardsi-worker"))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&bytes).unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(!status.success());
                break;
            }
            if Instant::now() > end {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("invalid input did not exit");
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}

// Real Worker request enforcement, not only an in-memory guard test.
#[test]
fn worker_rejects_duplicate_gap_and_foreign_requests() {
    use boardsi_ipc::{encode, read_message, Message};
    use std::{
        io::Write,
        process::{Command, Stdio},
        sync::mpsc,
    };
    let snapshot = Snapshot {
        points: 100,
        delay_ms: 100,
    };
    for case in 0..4 {
        let mut child = Command::new(env!("CARGO_BIN_EXE_boardsi-worker"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        let mut output = child.stdout.take().unwrap();
        input
            .write_all(
                &encode(&Message::new(
                    "run",
                    &snapshot.id(),
                    1,
                    Body::Start {
                        snapshot: snapshot.clone(),
                    },
                ))
                .unwrap(),
            )
            .unwrap();
        let mut bad = Message::new("run", &snapshot.id(), 2, Body::Cancel);
        match case {
            0 => bad.seq = 1,
            1 => bad.seq = 3,
            2 => bad.run_id = "previous-run".into(),
            _ => bad.snapshot_id = "previous-snapshot".into(),
        }
        input.write_all(&encode(&bad).unwrap()).unwrap();
        let (tx, rx) = mpsc::channel();
        let reader = thread::spawn(move || loop {
            let m = read_message(&mut output);
            let stop = !matches!(&m, Ok(Some(_)));
            if tx.send(m).is_err() || stop {
                break;
            }
        });
        let end = Instant::now() + Duration::from_secs(5);
        let mut failed = None;
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(Some(m))) => {
                    if let Body::Failed { code, .. } = m.body {
                        failed = Some(code);
                    }
                }
                Ok(Ok(None)) => break,
                Ok(Err(e)) => panic!("{e}"),
                Err(_) => {}
            }
            if Instant::now() > end {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("request not rejected");
            }
        }
        let expected = match case {
            0 | 1 => "SequenceMismatch",
            2 => "RunMismatch",
            _ => "SnapshotMismatch",
        };
        assert_eq!(failed.as_deref(), Some(expected));
        assert!(!child.wait().unwrap().success());
        reader.join().unwrap();
    }
}
#[test]
fn worker_unsupported_version_has_nonzero_exit_and_separate_diagnostic() {
    use boardsi_ipc::{encode, Message};
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let snapshot = Snapshot {
        points: 1,
        delay_ms: 1,
    };
    let m = Message::new("run", &snapshot.id(), 1, Body::Start { snapshot });
    let bytes = encode(&m).unwrap();
    let json = String::from_utf8(bytes[4..].to_vec())
        .unwrap()
        .replace("\"version\":1", "\"version\":2");
    let mut frame = (json.len() as u32).to_be_bytes().to_vec();
    frame.extend(json.bytes());
    let mut child = Command::new(env!("CARGO_BIN_EXE_boardsi-worker"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&frame).unwrap();
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() > end {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("unsupported version not rejected");
        }
        thread::sleep(Duration::from_millis(5));
    }
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("UnsupportedVersion"));
}
#[test]
fn worker_native_identity_matches_requested_platform() {
    if let Ok(arch) = std::env::var("BOARDSI_EXPECTED_EXECUTABLE_ARCH") {
        assert_eq!(std::env::consts::ARCH, arch);
    }
    if let Ok(os) = std::env::var("BOARDSI_EXPECTED_EXECUTABLE_OS") {
        assert_eq!(std::env::consts::OS, os);
    }
    println!(
        "CLIENT_NATIVE_EXECUTION arch={} os={}",
        std::env::consts::ARCH,
        std::env::consts::OS
    );
}
