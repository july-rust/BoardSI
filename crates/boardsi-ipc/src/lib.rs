//! Local-only M0 synthetic protocol. Execution completion is not physical validation.
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};
pub const VERSION: u32 = 1;
pub const MAX_FRAME: usize = 65_536;
pub const HEARTBEAT: Duration = Duration::from_secs(1);
pub const SILENCE_WARNING: Duration = Duration::from_secs(5);
pub const FORCE_OPTION: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub points: u32,
    pub delay_ms: u32,
}
impl Snapshot {
    pub fn id(&self) -> String {
        format!(
            "synthetic-rlgc-v1:points={}:delay_ms={}",
            self.points, self.delay_ms
        )
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(1..=1000).contains(&self.points) || self.delay_ms > 1000 {
            Err("InvalidSnapshot")
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Body {
    Start {
        snapshot: Snapshot,
    },
    Cancel,
    Started,
    Progress {
        done: u32,
        total: u32,
    },
    Result {
        point: u32,
        frequency_hz: f64,
        s11: [f64; 2],
        s21: [f64; 2],
        synthetic: bool,
    },
    Completed,
    Failed {
        code: String,
        context: String,
    },
    CancelAcknowledged,
    Cancelled,
    Heartbeat,
}
impl Body {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Start { .. } | Self::Cancel => "request",
            Self::Started | Self::CancelAcknowledged => "response",
            _ => "event",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub version: u32,
    pub run_id: String,
    pub snapshot_id: String,
    pub seq: u64,
    pub kind: String,
    pub body: Body,
}
impl Message {
    pub fn new(run: &str, snapshot: &str, seq: u64, body: Body) -> Self {
        Self {
            version: VERSION,
            run_id: run.into(),
            snapshot_id: snapshot.into(),
            seq,
            kind: body.kind().into(),
            body,
        }
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != VERSION {
            return Err("UnsupportedVersion");
        }
        if self.run_id.is_empty()
            || self.run_id.len() > 128
            || self.snapshot_id.len() > 256
            || self.seq == 0
            || self.seq > 9_007_199_254_740_991
        {
            return Err("InvalidEnvelope");
        }
        if self.kind != self.body.kind() {
            return Err("InvalidKind");
        }
        if let Body::Start { snapshot } = &self.body {
            snapshot.validate()?;
            if self.snapshot_id != snapshot.id() {
                return Err("SnapshotMismatch");
            }
        }
        if let Body::Result {
            frequency_hz,
            s11,
            s21,
            synthetic,
            ..
        } = &self.body
        {
            if !synthetic
                || *frequency_hz < 0.0
                || !frequency_hz.is_finite()
                || s11.iter().chain(s21).any(|v| !v.is_finite())
            {
                return Err("InvalidResult");
            }
        }
        Ok(())
    }
}
fn invalid(code: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, code)
}
pub fn encode(message: &Message) -> io::Result<Vec<u8>> {
    message.validate().map_err(invalid)?;
    let bytes = serde_json::to_vec(message).map_err(|e| invalid(&e.to_string()))?;
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        return Err(invalid("FrameTooLarge"));
    }
    let mut frame = (bytes.len() as u32).to_be_bytes().to_vec();
    frame.extend(bytes);
    Ok(frame)
}
pub fn write_message(writer: &mut impl Write, message: &Message) -> io::Result<()> {
    writer.write_all(&encode(message)?)?;
    writer.flush()
}
pub fn read_message(reader: &mut impl Read) -> io::Result<Option<Message>> {
    let mut header = [0; 4];
    loop {
        match reader.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    reader.read_exact(&mut header[1..])?;
    let size = u32::from_be_bytes(header) as usize;
    if size == 0 || size > MAX_FRAME {
        return Err(invalid("FrameSize"));
    }
    let mut bytes = vec![0; size];
    reader.read_exact(&mut bytes)?;
    let m: Message = serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
    m.validate().map_err(invalid)?;
    Ok(Some(m))
}
/// Atomic rejection: invalid scope/sequence does not change the accepted cursor.
#[derive(Debug)]
pub struct Scope {
    run: String,
    snapshot: String,
    next: u64,
}
impl Scope {
    pub fn new(run: &str, snapshot: &str) -> Self {
        Self {
            run: run.into(),
            snapshot: snapshot.into(),
            next: 1,
        }
    }
    pub fn accept(&mut self, m: &Message) -> Result<(), &'static str> {
        m.validate()?;
        if m.run_id != self.run {
            return Err("RunMismatch");
        }
        if m.snapshot_id != self.snapshot {
            return Err("SnapshotMismatch");
        }
        if m.seq != self.next {
            return Err("SequenceMismatch");
        }
        self.next += 1;
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Awaiting,
    Running,
    CancelRequested,
    CancelAcknowledged,
    Completed,
    Failed,
    Cancelled,
}
impl State {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}
/// Client-side provisional result collector; no Validated state exists.
pub struct RunGuard {
    scope: Scope,
    pub state: State,
    pub results: Vec<Message>,
    points: u32,
    pub last_rx: Instant,
    pub last_heartbeat: Instant,
    cancel_at: Option<Instant>,
}
impl RunGuard {
    pub fn new(run: &str, snapshot: &Snapshot, now: Instant) -> Self {
        Self {
            scope: Scope::new(run, &snapshot.id()),
            state: State::Awaiting,
            results: vec![],
            points: snapshot.points,
            last_rx: now,
            last_heartbeat: now,
            cancel_at: None,
        }
    }
    pub fn cancel_requested(&mut self, now: Instant) -> Result<(), &'static str> {
        if self.state != State::Running {
            return Err("InvalidState");
        }
        self.state = State::CancelRequested;
        self.cancel_at = Some(now);
        Ok(())
    }
    pub fn accept(&mut self, m: Message, now: Instant) -> Result<(), &'static str> {
        if self.state.terminal() {
            return Err("AlreadyTerminal");
        }
        // Semantic validation before moving sequence/timeout cursors.
        match &m.body {
            Body::Started if self.state == State::Awaiting => {}
            Body::Heartbeat if self.state != State::Awaiting => {}
            Body::CancelAcknowledged if self.state == State::CancelRequested => {}
            Body::Cancelled if self.state == State::CancelAcknowledged => {}
            Body::Failed { .. } => {}
            Body::Progress { done, total }
                if self.state != State::Awaiting
                    && *total == self.points
                    && *done as usize == self.results.len() => {}
            Body::Result {
                point,
                frequency_hz,
                ..
            } if self.state != State::Awaiting
                && *point as usize == self.results.len()
                && *point < self.points
                && *frequency_hz == f64::from(*point) * 1e6 => {}
            Body::Completed
                if self.state == State::Running && self.results.len() == self.points as usize => {}
            _ => return Err("InvalidStateOrPayload"),
        }
        self.scope.accept(&m)?;
        self.last_rx = now;
        match m.body {
            Body::Started => self.state = State::Running,
            Body::Heartbeat => self.last_heartbeat = now,
            Body::CancelAcknowledged => self.state = State::CancelAcknowledged,
            Body::Cancelled => {
                self.state = State::Cancelled;
                self.results.clear();
            }
            Body::Failed { .. } => {
                self.state = State::Failed;
                self.results.clear();
            }
            Body::Completed => self.state = State::Completed,
            Body::Result { .. } => self.results.push(m),
            _ => {}
        }
        Ok(())
    }
    pub fn warnings(&self, now: Instant) -> (bool, bool, bool) {
        if self.state.terminal() {
            return (false, false, false);
        }
        (
            now.duration_since(self.last_rx) >= SILENCE_WARNING,
            now.duration_since(self.last_heartbeat) >= SILENCE_WARNING,
            self.cancel_at
                .is_some_and(|t| now.duration_since(t) >= FORCE_OPTION),
        )
    }
    /// Transport failure/abnormal exit invalidates even a received Completed event.
    pub fn transport_failed(&mut self) {
        self.state = State::Failed;
        self.results.clear();
    }
}
