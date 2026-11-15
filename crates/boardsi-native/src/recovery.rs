//! F_003_1: app-owned crash staging cleanup; never infer ownership from PID alone.
//! A sidecar remains locked for the whole save. OS releases the lock on hard kill.
//! Recovery only touches structurally verified, unlocked marker/stage pairs.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const MAGIC: &str = "BOARDSI_STAGING_OWNER_V1\n";
const PREFIX: &str = ".bsi-staging-";

#[cfg(unix)]
fn lock(file: &File) -> io::Result<bool> {
    use std::os::unix::io::AsRawFd;
    extern "C" { fn flock(fd: i32, operation: i32) -> i32; }
    // LOCK_EX | LOCK_NB
    let rc = unsafe { flock(file.as_raw_fd(), 2 | 4) };
    if rc == 0 { return Ok(true); }
    let err = io::Error::last_os_error();
    if err.kind() == io::ErrorKind::WouldBlock { Ok(false) } else { Err(err) }
}
#[cfg(windows)]
#[repr(C)]
struct Overlapped {
    internal: usize, internal_high: usize, offset: u32,
    offset_high: u32, event: *mut std::ffi::c_void,
}
#[cfg(windows)]
fn lock(file: &File) -> io::Result<bool> {
    use std::os::windows::io::AsRawHandle;
    #[link(name = "Kernel32")]
    extern "system" {
        fn LockFileEx(file: *mut std::ffi::c_void, flags: u32, reserved: u32,
                      low: u32, high: u32, overlapped: *mut Overlapped) -> i32;
    }
    let mut overlap: Overlapped = unsafe { std::mem::zeroed() };
    // The lock is a single byte and is held until this handle is closed.
    let rc = unsafe { LockFileEx(file.as_raw_handle() as _, 0x3, 0, 1, 0, &mut overlap) };
    if rc != 0 { return Ok(true); }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(33) { Ok(false) } else { Err(error) }
}

fn marker_for(stage: &Path) -> PathBuf {
    let mut name = stage.as_os_str().to_os_string();
    name.push(".owner");
    PathBuf::from(name)
}
fn stage_name(stage: &Path) -> io::Result<String> {
    let name = stage.file_name().and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "staging name"))?;
    if !name.starts_with(PREFIX) || name.len() > 140 ||
        !name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.')) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "staging name"));
    }
    Ok(name.to_owned())
}
fn regular_no_link(path: &Path) -> bool {
    fs::symlink_metadata(path).map(|m| m.file_type().is_file()).unwrap_or(false)
}
pub struct StageOwner {
    stage: PathBuf, marker: PathBuf, file: Option<File>,
}
impl StageOwner {
    pub fn acquire(stage: &Path) -> io::Result<Self> {
        let name = stage_name(stage)?;
        let marker = marker_for(stage);
        let mut file = OpenOptions::new().write(true).read(true)
            .create_new(true).open(&marker)?;
        if !lock(&file)? {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "unexpected owner lock"));
        }
        file.write_all(format!("{MAGIC}{name}\n").as_bytes())?;
        file.sync_all()?;
        Ok(Self {stage:stage.into(), marker, file:Some(file)})
    }
}
impl Drop for StageOwner {
    fn drop(&mut self) {
        // After a successful replace the stage no longer exists. After failure
        // remove it while the process still owns its marker lock.
        let _ = fs::remove_file(&self.stage);
        self.file.take();
        let _ = fs::remove_file(&self.marker);
    }
}
/// Scan only the supplied directory, never recurse, follow links, or delete
/// unknown markers. An active writer holds an OS lock, not an unreliable PID.
pub fn recover_owned_staging(dir: &Path) -> io::Result<usize> {
    if !dir.is_dir() { return Err(io::Error::new(io::ErrorKind::NotFound, "directory")); }
    let mut recovered = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let marker = entry.path();
        let Some(file_name) = marker.file_name().and_then(|x|x.to_str()) else { continue; };
        let Some(base) = file_name.strip_suffix(".owner") else { continue; };
        let stage = dir.join(base);
        if stage_name(&stage).is_err() || !regular_no_link(&marker) { continue; }
        let mut file = match OpenOptions::new().read(true).write(true).open(&marker) {
            Ok(f) => f, Err(_) => continue,
        };
        match lock(&file) {
            Ok(true) => {}, Ok(false) => continue, Err(_) => continue,
        }
        let mut bytes = Vec::new();
        if (&mut file).take(256).read_to_end(&mut bytes).is_err() ||
            bytes != format!("{MAGIC}{base}\n").as_bytes() { continue; }
        // Missing stage means the process already replaced the destination,
        // then crashed before cleaning its marker.
        if stage.exists() && !regular_no_link(&stage) { continue; }
        if stage.exists() && fs::remove_file(&stage).is_err() { continue; }
        drop(file);
        if fs::remove_file(&marker).is_ok() { recovered += 1; }
    }
    Ok(recovered)
}
