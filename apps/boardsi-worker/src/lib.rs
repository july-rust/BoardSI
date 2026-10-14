//! Independent non-GUI test client. Call poll from a scheduler; no computation here.
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
    pub exit: Option<ExitStatus>,
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
        if self.exit.is_none() {
            self.exit = self.child.try_wait()?;
        }
        if let Some(status) = self.exit {
            if !status.success() || (self.eof && !self.guard.state.terminal()) {
                self.guard.transport_failed();
                self.transport_broken = true;
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
