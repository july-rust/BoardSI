//! F_004 migration gateway. Only v1 is a published format today.
//!
//! v1 -> v1 is an exact-byte, immutable copy, not a fake v2 conversion.
//! Future sequential DDL steps must be registered explicitly and validated
//! before they can become supported migration paths.
use crate::recovery::{recover_owned_staging, StageOwner};
use crate::store::{load, uuid};
use crate::{invalid, NativeError, Result};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};

pub const CURRENT_SCHEMA: u32 = 1;

/// Bytes and semantics are identical for the only released v1 format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationReceipt {
    pub from_schema: u32,
    pub to_schema: u32,
    pub steps: usize,
    pub copied: bool,
}

#[derive(Clone, Copy)]
struct MigrationStep {
    from: u32,
    to: u32,
    transform: fn(&Connection) -> Result<()>,
}

/// No pre-v1 version was released and no v2 schema has been defined.
/// This empty registry is deliberate: never invent an official upgrade.
const RELEASED_STEPS: &[MigrationStep] = &[];

fn plan(from: u32, to: u32, registry: &[MigrationStep]) -> Result<Vec<MigrationStep>> {
    if from == 0 || from > to || to == 0 {
        return Err(NativeError::UnsupportedRequired);
    }
    let mut current = from;
    let mut path = Vec::new();
    while current < to {
        let mut matches = registry
            .iter()
            .filter(|step| step.from == current && step.to == current + 1);
        let step = matches.next().ok_or(NativeError::UnsupportedRequired)?;
        if matches.next().is_some() {
            return Err(invalid("ambiguous migration step"));
        }
        path.push(*step);
        current += 1;
    }
    Ok(path)
}

/// Each registered transition is applied in one SQLite transaction.
/// No product transition is executed until a new published DDL exists.
fn apply_steps(conn: &mut Connection, steps: &[MigrationStep]) -> Result<()> {
    if steps.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    for step in steps {
        (step.transform)(&tx)?;
        tx.pragma_update(None, "user_version", step.to as i64)?;
    }
    tx.commit()?;
    Ok(())
}

fn digest_file(path: &Path) -> Result<[u8; 32]> {
    let mut reader = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let count = reader.read(&mut buf)?;
        if count == 0 {
            break;
        }
        hasher.update(&buf[..count]);
    }
    Ok(hasher.finalize().into())
}

fn stage_file(parent: &Path) -> Result<(PathBuf, StageOwner)> {
    for nonce in 0..16u32 {
        let stage = parent.join(format!(
            ".bsi-staging-{}-{}-{}",
            std::process::id(),
            uuid(),
            nonce
        ));
        match OpenOptions::new().write(true).create_new(true).open(&stage) {
            Ok(file) => {
                drop(file);
                return match StageOwner::acquire(&stage) {
                    Ok(owner) => Ok((stage, owner)),
                    Err(err) => {
                        let _ = fs::remove_file(stage);
                        Err(err.into())
                    }
                };
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err.into()),
        }
    }
    Err(invalid("migration staging collision"))
}

#[cfg(feature = "f003-qualification")]
fn test_before_publish() -> Result<()> {
    let Ok(mode) = std::env::var("BOARDSI_F004_QUAL_MODE") else {
        return Ok(());
    };
    if mode != "pause" && mode != "fail" {
        return Ok(());
    }
    let marker = std::env::var_os("BOARDSI_F004_QUAL_MARKER")
        .ok_or_else(|| invalid("F004 marker required"))?;
    fs::write(marker, b"f004-validated-staging")?;
    if mode == "fail" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "F004 simulated I/O failure before publishing",
        )
        .into());
    }
    loop {
        std::thread::park_timeout(std::time::Duration::from_millis(100));
    }
}

/// Validate a v1 document, then copy the same bytes to a NEW path through an
/// owned, durable staging file. This is not a lossy Writer round-trip.
///
/// Existing destinations are NEVER overwritten, and the source is NEVER edited.
/// Unknown optional/read-only documents are not implicitly upgraded.
/// When a future schema is released, register and independently test its
/// sequential transform before enabling any nonidentity conversion.
pub fn migrate_to_current(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
) -> Result<MigrationReceipt> {
    let source = source.as_ref();
    let target = target.as_ref();
    if source.extension().and_then(|x| x.to_str()) != Some("bsi")
        || target.extension().and_then(|x| x.to_str()) != Some("bsi")
    {
        return Err(invalid("migration requires .bsi paths"));
    }
    if !fs::symlink_metadata(source)?.file_type().is_file() {
        return Err(invalid("migration source must be regular file"));
    }
    let original = load(source)?;
    if original.is_read_only() {
        return Err(NativeError::UnsupportedRequired);
    }
    let steps = plan(CURRENT_SCHEMA, CURRENT_SCHEMA, RELEASED_STEPS)?;
    // Same-file v1 migration is an idempotent, no-write operation.
    if fs::symlink_metadata(target).is_ok() {
        if fs::canonicalize(source)? == fs::canonicalize(target)? {
            return Ok(MigrationReceipt {
                from_schema: CURRENT_SCHEMA,
                to_schema: CURRENT_SCHEMA,
                steps: 0,
                copied: false,
            });
        }
        return Err(invalid("migration target already exists"));
    }
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(invalid("migration target parent not found"));
    }
    if fs::canonicalize(
        source
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new(".")),
    )? == fs::canonicalize(parent)?
        && source.file_name() == target.file_name()
    {
        return Err(invalid("migration cannot overwrite source"));
    }
    recover_owned_staging(parent)?;
    let prior_sha = digest_file(source)?;
    let (stage, owner) = stage_file(parent)?;
    let result = (|| -> Result<()> {
        fs::copy(source, &stage)?;
        if digest_file(&stage)? != prior_sha || digest_file(source)? != prior_sha {
            return Err(invalid("source changed during migration copy"));
        }
        let mut step_conn = if steps.is_empty() {
            None
        } else {
            Some(Connection::open(&stage)?)
        };
        if let Some(conn) = &mut step_conn {
            conn.pragma_update(None, "foreign_keys", "ON")?;
            conn.pragma_update(None, "trusted_schema", "OFF")?;
            apply_steps(conn, &steps)?;
        }
        drop(step_conn);
        OpenOptions::new().write(true).open(&stage)?.sync_all()?;
        let restored = load(&stage)?;
        if restored.is_read_only() || !crate::same_semantics(restored.model(), original.model()) {
            return Err(invalid("migration lost Canonical semantics"));
        }
        if digest_file(source)? != prior_sha || digest_file(&stage)? != prior_sha {
            return Err(invalid("v1 identity migration byte mismatch"));
        }
        #[cfg(feature = "f003-qualification")]
        test_before_publish()?;
        // Hardlink a completed same-directory staging file to a NEW name.
        // Unlike rename, hard_link fails if another process creates target.
        // The stage owner subsequently unlinks its separate name.
        fs::hard_link(&stage, target)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    drop(owner);
    result?;
    Ok(MigrationReceipt {
        from_schema: CURRENT_SCHEMA,
        to_schema: CURRENT_SCHEMA,
        steps: steps.len(),
        copied: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_one(conn: &Connection) -> Result<()> {
        conn.execute_batch("UPDATE migration_counter SET value=value+1;")?;
        Ok(())
    }
    fn error_step(_: &Connection) -> Result<()> {
        Err(invalid("injected migration rollback"))
    }
    #[test]
    fn registry_is_ordered_strict_and_fails_closed() {
        let registered = [
            MigrationStep {
                from: 1,
                to: 2,
                transform: add_one,
            },
            MigrationStep {
                from: 2,
                to: 3,
                transform: add_one,
            },
        ];
        assert_eq!(plan(1, 3, &registered).unwrap().len(), 2);
        assert!(plan(1, 4, &registered).is_err());
        assert!(plan(0, 1, &registered).is_err());
        assert!(plan(3, 1, &registered).is_err());
        assert_eq!(plan(1, 1, RELEASED_STEPS).unwrap().len(), 0);
        assert!(plan(1, 2, RELEASED_STEPS).is_err());
        assert!(plan(1, 2, &[registered[0], registered[0]]).is_err());
    }
    #[test]
    fn synthetic_steps_commit_once_or_roll_back_entire_transaction() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE migration_counter(value INTEGER NOT NULL); INSERT INTO migration_counter VALUES(0);").unwrap();
        let failing = [
            MigrationStep {
                from: 1,
                to: 2,
                transform: add_one,
            },
            MigrationStep {
                from: 2,
                to: 3,
                transform: error_step,
            },
        ];
        assert!(apply_steps(&mut conn, &failing).is_err());
        let value: i64 = conn
            .query_row("SELECT value FROM migration_counter", [], |r| r.get(0))
            .unwrap();
        assert_eq!(value, 0);
        let valid = [
            MigrationStep {
                from: 1,
                to: 2,
                transform: add_one,
            },
            MigrationStep {
                from: 2,
                to: 3,
                transform: add_one,
            },
        ];
        apply_steps(&mut conn, &valid).unwrap();
        let value: i64 = conn
            .query_row("SELECT value FROM migration_counter", [], |r| r.get(0))
            .unwrap();
        assert_eq!(value, 2);
        let schema: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(schema, 3);
    }
}
