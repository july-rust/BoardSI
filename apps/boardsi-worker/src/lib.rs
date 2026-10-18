//! Independent non-GUI test client. Call poll from a scheduler; no computation here.
pub mod model_handoff;
use boardsi_ipc::{read_message, write_message, Body, Message, RunGuard, Snapshot};
use std::io;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Instant;
pub struct Client {
    child: Child,
    input: Option<ChildStdin>,
    rx: Receiver<io::Result<Option<Message>>>,
    reader: Option<JoinHandle<()>>,
    pub guard: RunGuard,
    run: String,
    snapshot: String,
    next_request: u64,
    /// Final exit status, exposed only after stdout frames and EOF have been drained.
    pub exit: Option<ExitStatus>,
    /// A best-effort Cancel hit a closed pipe; drain the scoped final outcome.
    pub cancel_delivery_error: Option<String>,
    observed_exit: Option<ExitStatus>,
    reported_failure: bool,
    eof: bool,
    transport_broken: bool,
}
impl Client {
    pub fn spawn(
        path: &std::path::Path,
        fault: Option<&str>,
        run: &str,
        snapshot: Snapshot,
    ) -> io::Result<Self> {
        snapshot
            .validate()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let mut command = Command::new(path);
        if let Some(arg) = fault {
            command.arg(arg);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take();
        let mut output = child.stdout.take().expect("piped stdout");
        // At most MAX_FRAME bytes per frame; bounded client queue, bounded worker queue.
        let (tx, rx) = mpsc::sync_channel(8);
        let reader = thread::spawn(move || loop {
            let m = read_message(&mut output);
            let stop = !matches!(&m, Ok(Some(_)));
            if tx.send(m).is_err() || stop {
                break;
            }
        });
        let mut client = Self {
            child,
            input,
            rx,
            reader: Some(reader),
            guard: RunGuard::new(run, &snapshot, Instant::now()),
            run: run.into(),
            snapshot: snapshot.id(),
            next_request: 1,
            exit: None,
            cancel_delivery_error: None,
            observed_exit: None,
            reported_failure: false,
            eof: false,
            transport_broken: false,
        };
        client.send(Body::Start { snapshot })?;
        Ok(client)
    }
    fn send(&mut self, body: Body) -> io::Result<()> {
        let m = Message::new(&self.run, &self.snapshot, self.next_request, body);
        write_message(
            self.input
                .as_mut()
                .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "stdin closed"))?,
            &m,
        )?;
        self.next_request += 1;
        Ok(())
    }
    pub fn cancel(&mut self) -> io::Result<()> {
        self.guard
            .cancel_requested(Instant::now())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        if let Err(e) = self.send(Body::Cancel) {
            if e.kind() == io::ErrorKind::BrokenPipe {
                self.cancel_delivery_error = Some(e.to_string());
                return Ok(());
            }
            self.guard.transport_failed();
            self.transport_broken = true;
            return Err(e);
        }
        Ok(())
    }
    /// Consume a bounded batch, check process exit without waiting. Returns received events.
    pub fn poll(&mut self) -> io::Result<Vec<Message>> {
        let mut events = vec![];
        for _ in 0..32 {
            match self.rx.try_recv() {
                Ok(Ok(Some(m))) => {
                    if self.transport_broken {
                        continue;
                    }
                    if let Err(code) = self.guard.accept(m.clone(), Instant::now()) {
                        self.guard.transport_failed();
                        self.transport_broken = true;
                        return Err(io::Error::new(io::ErrorKind::InvalidData, code));
                    }
                    self.reported_failure |= matches!(m.body, Body::Failed { .. });
                    events.push(m);
                }
                Ok(Ok(None)) | Err(TryRecvError::Disconnected) => {
                    self.eof = true;
                    break;
                }
                Ok(Err(e)) => {
                    self.guard.transport_failed();
                    self.transport_broken = true;
                    return Err(e);
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        if self.observed_exit.is_none() {
            self.observed_exit = self.child.try_wait()?;
        }
        // OS exit may precede the reader enqueueing its final scoped Failed frame.
        // Do not invalidate/discard queued protocol data until the reader reaches EOF.
        if self.eof {
            if let Some(status) = self.observed_exit {
                let expected_failure = self.reported_failure
                    && self.guard.state == boardsi_ipc::State::Failed
                    && matches!(status.code(), Some(1 | 2));
                if (!status.success() && !expected_failure)
                    || (status.success() && self.reported_failure)
                    || !self.guard.state.terminal()
                {
                    self.guard.transport_failed();
                    self.transport_broken = true;
                }
                self.exit = Some(status);
            }
        }
        Ok(events)
    }
    pub fn force_kill(&mut self) -> io::Result<()> {
        self.child.kill()?;
        self.guard.transport_failed();
        self.transport_broken = true;
        Ok(())
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.input.take();
        // Lifecycle cleanup is independent-client code, never Qt's GUI event loop.
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Release a possibly full receiver before joining its producer.
        let (_, disconnected) = mpsc::channel();
        let old = std::mem::replace(&mut self.rx, disconnected);
        drop(old);
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boardsi_ipc::State;
    #[test]
    fn late_cancel_closed_pipe_preserves_delayed_completed_outcome() {
        late_cancel_after_exit(true);
    }
    #[test]
    fn late_cancel_after_os_exit_preserves_delayed_completed_outcome() {
        late_cancel_after_exit(false);
    }
    fn late_cancel_after_exit(close_local_input: bool) {
        let mut command = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/C", "exit 0"]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", "exit 0"]);
            c
        };
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let status = child.wait().unwrap();
        let snapshot = Snapshot {
            points: 1,
            delay_ms: 0,
        };
        let (tx, rx) = mpsc::sync_channel(8);
        let now = Instant::now();
        let mut guard = RunGuard::new("late", &snapshot, now);
        guard
            .accept(Message::new("late", &snapshot.id(), 1, Body::Started), now)
            .unwrap();
        let mut c = Client {
            child,
            input,
            rx,
            reader: None,
            guard,
            run: "late".into(),
            snapshot: snapshot.id(),
            next_request: 2,
            exit: None,
            observed_exit: Some(status),
            reported_failure: false,
            eof: false,
            transport_broken: false,
            cancel_delivery_error: None,
        };
        if close_local_input {
            // Deterministically exercise send's real BrokenPipe path. Merely
            // waiting for ONE process does not prove every pipe reader closed:
            // concurrent spawn can transiently inherit handles before exec.
            drop(c.input.take());
        }
        c.cancel().unwrap();
        if close_local_input {
            assert!(c.cancel_delivery_error.is_some());
        }
        // OS-pipe write may succeed or report BrokenPipe after child exit.
        // Both outcomes must preserve delayed scoped Completed and EOF.
        assert_eq!(c.guard.state, State::CancelRequested);
        assert!(!c.transport_broken);
        tx.send(Ok(Some(Message::new(
            "late",
            &snapshot.id(),
            2,
            Body::Result {
                point: 0,
                frequency_hz: 0.0,
                s11: [0.0, 0.0],
                s21: [1.0, 0.0],
                synthetic: true,
            },
        ))))
        .unwrap();
        tx.send(Ok(Some(Message::new(
            "late",
            &snapshot.id(),
            3,
            Body::Completed,
        ))))
        .unwrap();
        tx.send(Ok(None)).unwrap();
        c.poll().unwrap();
        assert_eq!(c.guard.state, State::Completed);
        assert_eq!(c.guard.results.len(), 1);
        assert!(c.exit.unwrap().success());
        assert!(!c.transport_broken);
    }
    #[test]
    fn observed_nonzero_exit_waits_for_delayed_scoped_failure_and_eof() {
        // A real OS exit is observed before a controlled delayed protocol delivery.
        let mut command = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/C", "exit 2"]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", "exit 2"]);
            c
        };
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let status = child.wait().unwrap();
        assert_eq!(status.code(), Some(2));
        let snapshot = Snapshot {
            points: 1,
            delay_ms: 0,
        };
        let (tx, rx) = mpsc::sync_channel(8);
        let mut c = Client {
            child,
            input: None,
            rx,
            reader: None,
            guard: RunGuard::new("run", &snapshot, Instant::now()),
            run: "run".into(),
            snapshot: snapshot.id(),
            next_request: 1,
            exit: None,
            cancel_delivery_error: None,
            observed_exit: Some(status),
            reported_failure: false,
            eof: false,
            transport_broken: false,
        };
        assert!(c.poll().unwrap().is_empty());
        assert!(c.exit.is_none());
        assert!(!c.transport_broken);
        tx.send(Ok(Some(Message::new(
            "run",
            &snapshot.id(),
            1,
            Body::Started,
        ))))
        .unwrap();
        c.poll().unwrap();
        assert_eq!(c.guard.state, State::Running);
        assert!(c.exit.is_none());
        let failed = Message::new(
            "run",
            &snapshot.id(),
            2,
            Body::Failed {
                code: "DelayedFailure".into(),
                context: "must survive OS exit observation".into(),
            },
        );
        tx.send(Ok(Some(failed.clone()))).unwrap();
        tx.send(Ok(None)).unwrap();
        assert_eq!(c.poll().unwrap(), vec![failed]);
        assert_eq!(c.exit.unwrap().code(), Some(2));
        assert_eq!(c.guard.state, State::Failed);
        assert!(
            !c.transport_broken,
            "scoped expected failure is not transport corruption"
        );
    }
}
