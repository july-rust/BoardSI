//! F_004: fail-closed, versioned migration mechanism. No historical official
//! schema precedes v1; the production step registry is deliberately empty.
use crate::recovery::{recover_owned_staging, StageOwner};
use crate::store::{preflight, replace, uuid};
use crate::{invalid, load, NativeError, Result};
use rusqlite::{Connection, OpenFlags, Transaction};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::Path;

const CURRENT_SCHEMA: i64 = 1;
const MAX_FILE: u64 = 512 * 1024 * 1024;
const APP_ID: i64 = 0x42534931;

type Apply = for<'a> fn(&Transaction<'a>) -> Result<()>;

#[derive(Clone, Copy)]
struct Step {
    from: i64,
    to: i64,
    apply: Apply,
}

// Populate ONLY after an official legacy schema and immutable Golden are
// released. Test-only fake versions cannot leak into this production registry.
const STEPS: &[Step] = &[];

#[derive(Debug, PartialEq, Eq)]
pub struct MigrationReport {
    pub source_schema: i64,
    pub target_schema: i64,
    pub transformed: bool,
}

fn route(from: i64, steps: &[Step]) -> Result<Vec<Step>> {
    if from > CURRENT_SCHEMA {
        return Err(NativeError::FutureSchema);
    }
    if from < 0 {
        return Err(NativeError::UnsupportedRequired);
    }
    let mut result = Vec::new();
    let mut version = from;
    while version != CURRENT_SCHEMA {
        let choices: Vec<_> = steps
            .iter()
            .filter(|s| s.from == version)
            .copied()
            .collect();
        if choices.len() != 1 || choices[0].to <= version || choices[0].to > CURRENT_SCHEMA {
            return Err(NativeError::UnsupportedRequired);
        }
        result.push(choices[0]);
        version = choices[0].to;
    }
    Ok(result)
}
fn digest(path: &Path) -> Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 32768];
    loop {
        let size = file.read(&mut buf)?;
        if size == 0 {
            break;
        }
        hasher.update(&buf[..size]);
    }
    Ok(hasher.finalize().into())
}
fn user_version(path: &Path) -> Result<i64> {
    preflight(path)?;
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    conn.pragma_update(None, "trusted_schema", "OFF")?;
    let app: i64 = conn.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if app != APP_ID {
        return Err(NativeError::NotNative);
    }
    let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version > CURRENT_SCHEMA {
        return Err(NativeError::FutureSchema);
    }
    // Never migrate a corrupted SQLite database, even with a registered step.
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" {
        return Err(invalid("migration input integrity"));
    }
    let fk: i64 = conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
        r.get(0)
    })?;
    if fk != 0 {
        return Err(invalid("migration input foreign key"));
    }
    Ok(version)
}

/// Copy or migrate to a distinct target without reserializing unknown optional
/// extensions. Current v1 -> v1 is byte-preserving, NOT an invented migration.
/// An unsupported historical/future version is refused with no destination write.
pub fn migrate_to_current(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
) -> Result<MigrationReport> {
    execute(source.as_ref(), target.as_ref(), STEPS)
}
fn execute(source: &Path, target: &Path, steps: &[Step]) -> Result<MigrationReport> {
    if source.extension().and_then(|x| x.to_str()) != Some("bsi")
        || target.extension().and_then(|x| x.to_str()) != Some("bsi")
    {
        return Err(invalid(".bsi extension required"));
    }
    if fs::symlink_metadata(source)?.file_type().is_symlink() {
        return Err(invalid("migration source symlink"));
    }
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(invalid("migration target directory"));
    }
    if let Ok(meta) = fs::symlink_metadata(target) {
        if !meta.file_type().is_file() {
            return Err(invalid("migration target type"));
        }
    }
    let source_version = user_version(source)?;
    let chain = route(source_version, steps)?;
    // A current-version source must pass the complete Rust Canonical reader.
    if chain.is_empty() {
        load(source)?;
    }
    let original_sha = digest(source)?;
    // Same-file no-op is explicitly permitted, without touching bytes.
    if source == target && chain.is_empty() {
        return Ok(MigrationReport {
            source_schema: source_version,
            target_schema: CURRENT_SCHEMA,
            transformed: false,
        });
    }
    // Do not erase another file, especially one containing opaque data.
    if source != target && target.exists() {
        let existing = load(target)?;
        if existing.is_read_only() {
            return Err(NativeError::UnsupportedRequired);
        }
    }
    recover_owned_staging(parent)?;
    let mut stage = None;
    for nonce in 0..16u32 {
        let p = parent.join(format!(
            ".bsi-staging-{}-{}-{}",
            std::process::id(),
            uuid(),
            nonce
        ));
        match OpenOptions::new().write(true).create_new(true).open(&p) {
            Ok(_) => {
                stage = Some(p);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    let stage = stage.ok_or_else(|| invalid("migration staging name collision"))?;
    let owner = match StageOwner::acquire(&stage) {
        Ok(v) => v,
        Err(e) => {
            let _ = fs::remove_file(&stage);
            return Err(e.into());
        }
    };
    let result = (|| -> Result<()> {
        fs::copy(source, &stage)?;
        if fs::metadata(&stage)?.len() > MAX_FILE {
            return Err(invalid("migration file budget"));
        }
        if !chain.is_empty() {
            let mut conn = Connection::open(&stage)?;
            conn.pragma_update(None, "trusted_schema", "OFF")?;
            conn.pragma_update(None, "foreign_keys", "ON")?;
            conn.pragma_update(None, "synchronous", "FULL")?;
            // Every version step is checked and committed atomically together.
            let tx = conn.transaction()?;
            for step in &chain {
                let before: i64 = tx.pragma_query_value(None, "user_version", |r| r.get(0))?;
                if before != step.from {
                    return Err(invalid("migration step precondition"));
                }
                (step.apply)(&tx)?;
                let after: i64 = tx.pragma_query_value(None, "user_version", |r| r.get(0))?;
                if after != step.to {
                    return Err(invalid("migration step postcondition"));
                }
            }
            tx.commit()?;
            conn.close().map_err(|(_, e)| NativeError::Sql(e))?;
        } else if digest(&stage)? != original_sha {
            return Err(invalid("v1 copy changed bytes"));
        }
        // Full independent Reader, including DDL, BSIG, feature and Canonical checks.
        load(&stage)?;
        OpenOptions::new().write(true).open(&stage)?.sync_all()?;
        if digest(source)? != original_sha {
            return Err(invalid("migration source changed"));
        }
        replace(&stage, target)
    })();
    drop(owner);
    result?;
    Ok(MigrationReport {
        source_schema: source_version,
        target_schema: CURRENT_SCHEMA,
        transformed: !chain.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use boardsi_model::{BoardKind, BoardModel};

    fn unique(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "boardsi-f004-{}-{}-{label}.bsi",
            std::process::id(),
            uuid()
        ))
    }
    fn test_only_zero_to_v1(tx: &Transaction<'_>) -> Result<()> {
        tx.pragma_update(None, "user_version", 1)?;
        Ok(())
    }
    const FAKE_STEPS: &[Step] = &[Step {
        from: 0,
        to: 1,
        apply: test_only_zero_to_v1,
    }];
    #[test]
    fn strict_registry_and_ambiguous_path_rejected() {
        assert!(route(1, STEPS).unwrap().is_empty());
        assert_eq!(route(2, STEPS).unwrap_err().code(), "BSI_FUTURE_SCHEMA");
        assert_eq!(
            route(0, STEPS).unwrap_err().code(),
            "BSI_UNSUPPORTED_REQUIRED"
        );
        assert_eq!(route(0, FAKE_STEPS).unwrap().len(), 1);
        assert_eq!(
            route(0, &[FAKE_STEPS[0], FAKE_STEPS[0]])
                .unwrap_err()
                .code(),
            "BSI_UNSUPPORTED_REQUIRED"
        );
    }
    #[test]
    fn legacy_simulation_isolated_from_product_registry_and_atomic() {
        let src = unique("fake-legacy");
        let out = unique("migrated");
        crate::save(&src, &BoardModel::empty(BoardKind::Pcb)).unwrap();
        let conn = Connection::open(&src).unwrap();
        conn.pragma_update(None, "user_version", 0).unwrap();
        drop(conn);
        let original = digest(&src).unwrap();
        assert_eq!(
            migrate_to_current(&src, &out).unwrap_err().code(),
            "BSI_UNSUPPORTED_REQUIRED"
        );
        assert!(!out.exists());
        let receipt = execute(&src, &out, FAKE_STEPS).unwrap();
        assert_eq!(receipt.source_schema, 0);
        assert!(receipt.transformed);
        assert_eq!(digest(&src).unwrap(), original);
        assert_eq!(
            load(&out).unwrap().model(),
            &BoardModel::empty(BoardKind::Pcb)
        );
        // The published current version has an idempotent no-op path.
        let copy = unique("idempotent");
        assert!(!migrate_to_current(&out, &copy).unwrap().transformed);
        assert_eq!(digest(&out).unwrap(), digest(&copy).unwrap());
        for p in [&src, &out, &copy] {
            fs::remove_file(p).unwrap();
        }
    }
    #[test]
    fn broken_migration_keeps_original_and_destination_untouched() {
        fn broken(tx: &Transaction<'_>) -> Result<()> {
            tx.execute("DELETE FROM file_meta", [])?;
            tx.pragma_update(None, "user_version", 1)?;
            Ok(())
        }
        let src = unique("source");
        let out = unique("dest");
        crate::save(&src, &BoardModel::empty(BoardKind::Pcb)).unwrap();
        crate::save(&out, &BoardModel::empty(BoardKind::BgaSubstrate)).unwrap();
        let conn = Connection::open(&src).unwrap();
        conn.pragma_update(None, "user_version", 0).unwrap();
        drop(conn);
        let before_src = fs::read(&src).unwrap();
        let before_out = fs::read(&out).unwrap();
        let step = [Step {
            from: 0,
            to: 1,
            apply: broken,
        }];
        assert!(execute(&src, &out, &step).is_err());
        assert_eq!(fs::read(&src).unwrap(), before_src);
        assert_eq!(fs::read(&out).unwrap(), before_out);
        fs::remove_file(src).unwrap();
        fs::remove_file(out).unwrap();
    }
}
