//! One run per process. stdout is framed protocol only, diagnostics go to stderr.
use boardsi_ipc::{read_message, write_message, Body, Message, Scope, Snapshot, HEARTBEAT};
use boardsi_numerics::{Rlgc, UniformLine};
use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::{Duration, Instant};

fn calculate(snapshot: Snapshot, cancel: Arc<AtomicBool>, tx: mpsc::SyncSender<Body>) {
    let line = UniformLine {
        rlgc: Rlgc {
            r_ohm_per_m: 0.0,
            l_h_per_m: 250e-9,
            g_s_per_m: 0.0,
            c_f_per_m: 100e-12,
        },
        length_m: 0.1,
        reference_ohm: 50.0,
    };
    for point in 0..snapshot.points {
        let end = Instant::now() + Duration::from_millis(u64::from(snapshot.delay_ms));
        while Instant::now() < end {
            if cancel.load(Ordering::Acquire) {
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        if cancel.load(Ordering::Acquire) {
            return;
        }
        let frequency_hz = f64::from(point) * 1e6;
        match line.at(frequency_hz) {
            Ok(r) => {
                if tx
                    .send(Body::Result {
                        point,
                        frequency_hz,
                        s11: [r.s[0][0].re, r.s[0][0].im],
                        s21: [r.s[1][0].re, r.s[1][0].im],
                        synthetic: true,
                    })
                    .is_err()
                {
                    return;
                }
                if tx
                    .send(Body::Progress {
                        done: point + 1,
                        total: snapshot.points,
                    })
                    .is_err()
                {
                    return;
                }
            }
            Err(e) => {
                let _ = tx.send(Body::Failed {
                    code: "NumericalError".into(),
                    context: e.to_string(),
                });
                return;
            }
        }
    }
    let _ = tx.send(Body::Completed);
}
fn run() -> io::Result<i32> {
    let fault = std::env::args().nth(1).unwrap_or_default();
    if ![
        "",
        "--fault=crash",
        "--fault=unresponsive",
        "--fault=no-heartbeat",
    ]
    .contains(&fault.as_str())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unknown test fault",
        ));
    }
    let first = read_message(&mut io::stdin().lock())?
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Start required"))?;
    let snapshot = match &first.body {
        Body::Start { snapshot } => snapshot.clone(),
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Start required")),
    };
    let mut scope = Scope::new(&first.run_id, &first.snapshot_id);
    scope
        .accept(&first)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut seq = 0;
    let mut out = io::stdout().lock();
    let mut emit = |body| {
        seq += 1;
        write_message(
            &mut out,
            &Message::new(&first.run_id, &first.snapshot_id, seq, body),
        )
    };
    emit(Body::Started)?;
    emit(Body::Heartbeat)?;
    if fault == "--fault=crash" {
        std::process::abort();
    }
    // Deliberately ignore commands/heartbeat only under explicit fault injection.
    if fault == "--fault=unresponsive" {
        loop {
            thread::park();
        }
    }
    let (commands_tx, commands_rx) = mpsc::sync_channel(8);
    thread::spawn(move || {
        let mut input = io::stdin().lock();
        loop {
            let m = read_message(&mut input);
            let stop = !matches!(&m, Ok(Some(_)));
            if commands_tx.send(m).is_err() || stop {
                break;
            }
        }
    });
    let cancel = Arc::new(AtomicBool::new(false));
    let (results_tx, results_rx) = mpsc::sync_channel(2);
    let token = cancel.clone();
    let job = thread::spawn(move || calculate(snapshot, token, results_tx));
    let mut next_heartbeat = Instant::now() + HEARTBEAT;
    let mut acknowledged = false;
    let mut failure: Option<(&str, String)> = None;
    loop {
        if failure.is_none() {
            match commands_rx.try_recv() {
                Ok(Ok(Some(m))) => {
                    let check = scope.accept(&m).and_then(|()| {
                        if m.body == Body::Cancel && !acknowledged {
                            Ok(())
                        } else {
                            Err("UnexpectedRequest")
                        }
                    });
                    match check {
                        Ok(()) => {
                            acknowledged = true;
                            cancel.store(true, Ordering::Release);
                            emit(Body::CancelAcknowledged)?;
                        }
                        Err(code) => {
                            failure = Some((code, "request rejected".into()));
                            cancel.store(true, Ordering::Release);
                        }
                    }
                }
                Ok(Ok(None)) | Err(mpsc::TryRecvError::Disconnected) => {
                    failure = Some(("ClientDisconnected", "stdin closed".into()));
                    cancel.store(true, Ordering::Release);
                }
                Ok(Err(e)) => {
                    failure = Some(("InvalidFrame", e.to_string()));
                    cancel.store(true, Ordering::Release);
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if Instant::now() >= next_heartbeat && fault != "--fault=no-heartbeat" {
            emit(Body::Heartbeat)?;
            next_heartbeat = Instant::now() + HEARTBEAT;
        }
        match results_rx.recv_timeout(Duration::from_millis(5)) {
            Ok(body) if !acknowledged && failure.is_none() => {
                let terminal = matches!(body, Body::Completed | Body::Failed { .. });
                let failed = matches!(body, Body::Failed { .. });
                emit(body)?;
                if terminal {
                    job.join()
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "job panic"))?;
                    return Ok(i32::from(failed));
                }
            }
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let joined = job.join();
                if let Some((code, context)) = failure {
                    emit(Body::Failed {
                        code: code.into(),
                        context,
                    })?;
                    return Ok(2);
                }
                if joined.is_err() {
                    emit(Body::Failed {
                        code: "JobPanic".into(),
                        context: "calculation thread panicked".into(),
                    })?;
                    return Ok(2);
                }
                if acknowledged {
                    emit(Body::Cancelled)?;
                    return Ok(0);
                }
                emit(Body::Failed {
                    code: "MissingCompletion".into(),
                    context: "job ended without terminal event".into(),
                })?;
                return Ok(2);
            }
        }
    }
}
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--model-snapshot-v2") {
        match boardsi_worker::model_snapshot_v2::serve_stdio() {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("WORKER_SYNTHETIC_V2_REJECTED {e}");
                std::process::exit(2);
            }
        }
    }
    eprintln!(
        "WORKER_NATIVE_EXECUTION arch={} os={}",
        std::env::consts::ARCH,
        std::env::consts::OS
    );
    match run() {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("WORKER_PROTOCOL_ERROR {e}");
            std::process::exit(2);
        }
    }
}
