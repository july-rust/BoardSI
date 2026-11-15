//! Only after a complete SQLite and Canonical validation is a board returned.
use crate::{invalid, NativeError, Result};
use crate::{read, write};
use boardsi_model::BoardModel;
use rusqlite::{Connection, OpenFlags};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_FILE: u64 = 512 * 1024 * 1024;
const APP_ID: i64 = 0x42534931;
const DDL: &str = include_str!("../../../schemas/bsi/v1.sql");
pub struct LoadedBoard {
    pub model: BoardModel,
    /// Optional unknown feature or extension: never mutate/re-save without opaque preservation.
    pub read_only: bool,
}
fn uuid() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(now.as_nanos().to_le_bytes());
    hasher.update(std::process::id().to_le_bytes());
    let mut bytes: [u8; 16] = hasher.finalize()[..16].try_into().expect("length");
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes(bytes[..4].try_into().expect("length")),
        u16::from_be_bytes(bytes[4..6].try_into().expect("length")),
        u16::from_be_bytes(bytes[6..8].try_into().expect("length")),
        u16::from_be_bytes(bytes[8..10].try_into().expect("length")),
        (u64::from_be_bytes([
            0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        ]))
    )
}
fn preflight(path: &Path) -> Result<()> {
    let meta = fs::metadata(path)?;
    if !meta.is_file() || meta.len() < 100 || meta.len() > MAX_FILE {
        return Err(invalid("file budget/type"));
    }
    let mut f = File::open(path)?;
    let mut hdr = [0u8; 100];
    f.read_exact(&mut hdr)?;
    if &hdr[..16] != b"SQLite format 3\0" {
        return Err(NativeError::NotNative);
    }
    if u32::from_be_bytes(hdr[68..72].try_into().expect("length")) != APP_ID as u32 {
        return Err(NativeError::NotNative);
    }
    Ok(())
}
fn check_db(conn: &Connection) -> Result<()> {
    let app: i64 = conn.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if app != APP_ID {
        return Err(NativeError::NotNative);
    }
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version > 1 {
        return Err(NativeError::FutureSchema);
    }
    if version != 1 {
        return Err(invalid("schema version"));
    }
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" {
        return Err(invalid("SQLite integrity"));
    }
    let fk_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })?;
    if fk_count != 0 {
        return Err(invalid("SQLite foreign key"));
    }
    Ok(())
}
fn checked_connection(path: &Path) -> Result<Connection> {
    preflight(path)?;
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.pragma_update(None, "trusted_schema", "OFF")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(3))?;
    check_db(&conn)?;
    Ok(conn)
}
pub fn load(path: impl AsRef<Path>) -> Result<LoadedBoard> {
    let conn = checked_connection(path.as_ref())?;
    read::read_board(&conn)
}
fn replace(staging: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            #[link(name = "Kernel32")]
            extern "system" {
                fn ReplaceFileW(
                    replaced: *const u16,
                    replacement: *const u16,
                    backup: *const u16,
                    flags: u32,
                    exclude: *const core::ffi::c_void,
                    reserved: *const core::ffi::c_void,
                ) -> i32;
            }
            let dst: Vec<u16> = dest.as_os_str().encode_wide().chain(Some(0)).collect();
            let src: Vec<u16> = staging.as_os_str().encode_wide().chain(Some(0)).collect();
            let ok = unsafe {
                ReplaceFileW(
                    dst.as_ptr(),
                    src.as_ptr(),
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    std::ptr::null(),
                )
            };
            if ok == 0 {
                return Err(std::io::Error::last_os_error().into());
            }
        }
        #[cfg(not(windows))]
        fs::rename(staging, dest)?;
    } else {
        fs::rename(staging, dest)?;
    }
    #[cfg(unix)]
    if let Some(parent) = dest.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
pub fn save(path: impl AsRef<Path>, board: &BoardModel) -> Result<()> {
    let dest = path.as_ref();
    if dest.extension().and_then(|x| x.to_str()) != Some("bsi") {
        return Err(invalid(".bsi extension required"));
    }
    let parent = dest
        .parent()
        .filter(|x| !x.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(invalid("destination directory"));
    }
    if dest.is_dir() {
        return Err(invalid("destination is directory"));
    }
    // Do not create a truncated destination. Acquire exclusive stage file first.
    let mut stage = None;
    for nonce in 0..16u32 {
        let temp = parent.join(format!(
            ".bsi-staging-{}-{}-{}",
            std::process::id(),
            uuid(),
            nonce
        ));
        match OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(_) => {
                stage = Some(temp);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    let stage = stage.ok_or_else(|| invalid("staging name collision"))?;
    let result = (|| -> Result<()> {
        let mut conn = Connection::open(&stage)?;
        conn.execute_batch(DDL)?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        let tx = conn.transaction()?;
        write::write_board(&tx, board, &uuid())?;
        tx.commit()?;
        check_db(&conn)?;
        conn.close().map_err(|(_, e)| NativeError::Sql(e))?;
        File::open(&stage)?.sync_all()?;
        if fs::metadata(&stage)?.len() > MAX_FILE {
            return Err(invalid("output file exceeds limit"));
        }
        // Fully independent read verifies byte grammar, object graph and semantic invariants.
        let restored = load(&stage)?;
        if restored.read_only || restored.model != *board {
            return Err(invalid("native round-trip semantics mismatch"));
        }
        replace(&stage, dest)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&stage);
    }
    result
}
