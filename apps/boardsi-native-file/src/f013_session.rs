//! F_013 read-only Native session, bounded NDJSON and full ObjectId lookup.
use boardsi_model::{BoardModel, ObjectId, ObjectKind};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::{self, BufRead, Read, Write},
    path::Path,
    time::SystemTime,
};

const MAX_REQUEST: usize = 20_000;
const MAX_RESPONSE: usize = 60_000;
const MAX_INDEX: usize = 1_000_000;
const MAX_MEMBERSHIPS: usize = 2_000_000;
type Membership<'a> = (&'static str, usize, &'a ObjectId, &'static str);
const PROTOCOL: &str = "BSI_NATIVE_SESSION_V1";

fn token(value: &str, size: usize) -> bool {
    value.len() == size && value.bytes().all(|b| b.is_ascii_hexdigit())
}

// The model remains owned by LoadedBoard. Index keys borrow canonical IDs;
// no geometry or provenance is copied into the index.
struct ObjectIndex<'a> {
    board: &'a BoardModel,
    entries: BTreeMap<&'a ObjectId, (&'static str, usize)>,
    nets: BTreeMap<&'a ObjectId, Vec<Membership<'a>>>,
    layers: BTreeMap<&'a ObjectId, Vec<Membership<'a>>>,
}
impl<'a> ObjectIndex<'a> {
    fn build(board: &'a BoardModel) -> Result<Self, String> {
        let mut entries = BTreeMap::new();
        macro_rules! add {
            ($category:literal, $items:expr) => {
                for (i, object) in $items.iter().enumerate() {
                    if entries.len() >= MAX_INDEX
                        || entries.insert(&object.meta.id, ($category, i)).is_some()
                    {
                        return Err("BSI_F013_INDEX_LIMIT_OR_DUPLICATE".into());
                    }
                }
            };
        }
        add!("frames", board.frames);
        add!("layers", board.stackup.layers);
        add!("materials", board.materials);
        add!("nets", board.nets);
        add!("components", board.components);
        add!("pins", board.pins);
        add!("traces", board.traces);
        add!("arcs", board.arcs);
        add!("areas", board.areas);
        add!("padstacks", board.padstacks);
        add!("pads", board.pads);
        add!("vias", board.vias);
        add!("drills", board.drills);
        add!("outlines", board.outlines);
        let mut nets = BTreeMap::new();
        let mut layers = BTreeMap::new();
        let mut memberships = 0usize;
        // Each entry records a stored reference, not inferred connectivity.
        let mut add_member = |map: &mut BTreeMap<&'a ObjectId, Vec<Membership<'a>>>,
                              parent: &'a ObjectId,
                              member: Membership<'a>|
         -> Result<(), String> {
            if memberships >= MAX_MEMBERSHIPS {
                return Err("BSI_F013_MEMBERSHIP_LIMIT".into());
            }
            memberships += 1;
            map.entry(parent).or_default().push(member);
            Ok(())
        };
        macro_rules! net_members {
            ($category:literal, $items:expr) => {
                for (i, object) in $items.iter().enumerate() {
                    if let Some(net) = &object.net {
                        add_member(
                            &mut nets,
                            net,
                            ($category, i, &object.meta.id, "stored_net"),
                        )?;
                    }
                }
            };
        }
        macro_rules! layer_members {
            ($category:literal, $items:expr) => {
                for (i, object) in $items.iter().enumerate() {
                    add_member(
                        &mut layers,
                        &object.layer,
                        ($category, i, &object.meta.id, "stored_layer"),
                    )?;
                }
            };
        }
        net_members!("traces", board.traces);
        net_members!("arcs", board.arcs);
        net_members!("areas", board.areas);
        net_members!("pads", board.pads);
        net_members!("vias", board.vias);
        net_members!("pins", board.pins);
        layer_members!("traces", board.traces);
        layer_members!("arcs", board.arcs);
        layer_members!("areas", board.areas);
        for (i, object) in board.vias.iter().enumerate() {
            for layer in &object.connected_layers {
                add_member(
                    &mut layers,
                    layer,
                    ("vias", i, &object.meta.id, "stored_connected_layer"),
                )?;
            }
        }
        // Padstack layer entries are not claims about copper contact.
        for (i, object) in board.padstacks.iter().enumerate() {
            for entry in &object.entries {
                add_member(
                    &mut layers,
                    &entry.layer,
                    ("padstacks", i, &object.meta.id, "stored_padstack_entry"),
                )?;
            }
        }
        for (i, object) in board.pads.iter().enumerate() {
            let (_, stack_index) = entries
                .get(&object.padstack)
                .ok_or("BSI_F013_PADSTACK_REFERENCE")?;
            let stack = board
                .padstacks
                .get(*stack_index)
                .ok_or("BSI_F013_PADSTACK_REFERENCE")?;
            if stack.meta.id != object.padstack {
                return Err("BSI_F013_PADSTACK_REFERENCE".into());
            }
            for entry in &stack.entries {
                add_member(
                    &mut layers,
                    &entry.layer,
                    ("pads", i, &object.meta.id, "stored_padstack_entry"),
                )?;
            }
        }
        Ok(Self {
            board,
            entries,
            nets,
            layers,
        })
    }

    fn member_page(
        &self,
        category: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Option<(usize, Vec<Value>)>, String> {
        let Some((section, raw)) = category.split_once(':') else {
            return Ok(None);
        };
        if !matches!(section, "net-members" | "layer-members") {
            return Ok(None);
        }
        let parent = raw.parse::<usize>().map_err(|_| "BSI_F013_PARENT")?;
        let (map, identity) = if section == "net-members" {
            (
                &self.nets,
                &self
                    .board
                    .nets
                    .get(parent)
                    .ok_or("BSI_F013_PARENT")?
                    .meta
                    .id,
            )
        } else {
            (
                &self.layers,
                &self
                    .board
                    .stackup
                    .layers
                    .get(parent)
                    .ok_or("BSI_F013_PARENT")?
                    .meta
                    .id,
            )
        };
        let members = map.get(identity).map_or(&[][..], Vec::as_slice);
        if offset > members.len() || !(1..=128).contains(&limit) {
            return Err("BSI_F013_MEMBER_PAGE".into());
        }
        let mut used = 0;
        let mut rows = Vec::new();
        for (category, index, id, basis) in members.iter().skip(offset).take(limit) {
            let reference = super::f012_inspection::artwork_ref(self.board, category, *index, id);
            let row =
                json!({"reference":reference,"basis":basis,"physical_contact":"NOT_VERIFIED"});
            let size = row.to_string().len() + 1;
            if size > 32_000 {
                return Err("BSI_F013_ROW_LIMIT".into());
            }
            if used + size > MAX_RESPONSE - 2048 {
                break;
            }
            used += size;
            rows.push(row);
        }
        Ok(Some((members.len(), rows)))
    }

    fn lookup(&self, raw: &Value) -> Result<Value, String> {
        let source = raw["source_index"]
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .and_then(|v| self.board.sources.get(v))
            .ok_or("BSI_F013_ID_SOURCE")?;
        let kind = match raw["kind"].as_str() {
            Some("Frame") => ObjectKind::Frame,
            Some("Layer") => ObjectKind::Layer,
            Some("Material") => ObjectKind::Material,
            Some("Net") => ObjectKind::Net,
            Some("Component") => ObjectKind::Component,
            Some("Pin") => ObjectKind::Pin,
            Some("Trace") => ObjectKind::Trace,
            Some("Arc") => ObjectKind::Arc,
            Some("Area") => ObjectKind::Area,
            Some("Padstack") => ObjectKind::Padstack,
            Some("Pad") => ObjectKind::Pad,
            Some("Via") => ObjectKind::Via,
            Some("Drill") => ObjectKind::Drill,
            Some("Outline") => ObjectKind::Outline,
            _ => return Err("BSI_F013_ID_KIND".into()),
        };
        let id = ObjectId::new(
            &source.id,
            kind,
            raw["source_object"].as_str().ok_or("BSI_F013_ID")?,
            raw["part"].as_str().ok_or("BSI_F013_ID")?,
        );
        let (category, offset) = self.entries.get(&id).ok_or("BSI_F013_ID_MISSING")?;
        Ok(json!({"category":category,"index":offset,"id":raw}))
    }
}

// A session serves the originally digest-validated immutable model, never
// newly read disk data. Detect replacement and changes to that backing file.
#[cfg(windows)]
#[derive(PartialEq, Eq)]
struct WindowsIdentity {
    volume: u64,
    file: [u8; 16],
    change_time: i64,
}

#[cfg(windows)]
fn windows_identity(file: &fs::File) -> Result<WindowsIdentity, String> {
    use std::{ffi::c_void, os::windows::io::AsRawHandle};
    #[repr(C)]
    struct BasicInfo {
        creation: i64,
        access: i64,
        write: i64,
        change: i64,
        attributes: u32,
    }
    #[repr(C)]
    struct IdInfo {
        volume: u64,
        file: [u8; 16],
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetFileInformationByHandleEx(
            handle: *mut c_void,
            class: i32,
            information: *mut c_void,
            size: u32,
        ) -> i32;
    }
    let mut basic = BasicInfo {
        creation: 0,
        access: 0,
        write: 0,
        change: 0,
        attributes: 0,
    };
    let mut id = IdInfo {
        volume: 0,
        file: [0; 16],
    };
    // SAFETY: the handle is owned and valid for the call; both initialized
    // repr(C) buffers have the documented FILE_BASIC_INFO / FILE_ID_INFO
    // layout and exact length. FileBasicInfo=0, FileIdInfo=18.
    let basic_ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            0,
            (&mut basic as *mut BasicInfo).cast(),
            std::mem::size_of::<BasicInfo>() as u32,
        )
    };
    let id_ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            18,
            (&mut id as *mut IdInfo).cast(),
            std::mem::size_of::<IdInfo>() as u32,
        )
    };
    if basic_ok == 0 || id_ok == 0 {
        return Err("BSI_F013_FILE_ID_UNAVAILABLE".into());
    }
    Ok(WindowsIdentity {
        volume: id.volume,
        file: id.file,
        change_time: basic.change,
    })
}

#[derive(PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: SystemTime,
    #[cfg(unix)]
    identity: (u64, u64, i64, i64),
    #[cfg(windows)]
    identity: WindowsIdentity,
}
fn stamp(path: &Path) -> Result<FileStamp, String> {
    // Do not silently accept a newly introduced untracked SQLite sidecar.
    for suffix in ["-wal", "-shm"] {
        let mut sidecar = path.as_os_str().to_os_string();
        sidecar.push(suffix);
        match fs::symlink_metadata(std::path::PathBuf::from(sidecar)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            _ => return Err("BSI_SOURCE_CHANGED".into()),
        }
    }
    let file = fs::File::open(path).map_err(|_| "BSI_SOURCE_MISSING")?;
    let meta = file.metadata().map_err(|_| "BSI_SOURCE_MISSING")?;
    if !meta.is_file() {
        return Err("BSI_SOURCE_CHANGED".into());
    }
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    Ok(FileStamp {
        len: meta.len(),
        modified: meta.modified().map_err(|_| "BSI_SOURCE_CHANGED")?,
        #[cfg(unix)]
        identity: (meta.dev(), meta.ino(), meta.ctime(), meta.ctime_nsec()),
        #[cfg(windows)]
        identity: windows_identity(&file)?,
    })
}

fn send(writer: &mut impl Write, value: &Value) -> Result<(), String> {
    let bytes = value.to_string();
    if bytes.len() + 1 >= MAX_RESPONSE {
        return Err("BSI_F013_RESPONSE_LIMIT".into());
    }
    writeln!(writer, "{bytes}").map_err(|_| "BSI_F013_WRITE")?;
    writer.flush().map_err(|_| "BSI_F013_WRITE".into())
}

fn read_frame(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut line = Vec::new();
    let count = reader
        .take((MAX_REQUEST + 1) as u64)
        .read_until(b'\n', &mut line)
        .map_err(|_| "BSI_F013_READ")?;
    if count == 0 {
        return Ok(None);
    }
    if count > MAX_REQUEST || line.last() != Some(&b'\n') {
        return Err("BSI_F013_FRAME_LIMIT".into());
    }
    serde_json::from_slice(&line)
        .map(Some)
        .map_err(|_| "BSI_F013_FRAME_JSON".into())
}

pub(super) fn run(args: &[OsString]) -> Result<(), String> {
    let open = args.first().and_then(|v| v.to_str()) == Some("open-session");
    if args.len() != if open { 5 } else { 6 } {
        return Err("BSI_F013_USAGE".into());
    }
    let path = Path::new(&args[1]);
    let first = if open { 2 } else { 3 };
    let expected = if open {
        None
    } else {
        Some(args[2].to_str().ok_or("BSI_F013_DIGEST")?)
    };
    let document = args[first].to_str().ok_or("BSI_F013_DOCUMENT")?;
    let session = args[first + 1].to_str().ok_or("BSI_F013_SESSION")?;
    let generation = args[first + 2].to_str().ok_or("BSI_F013_GENERATION")?;
    if expected.is_some_and(|v| !token(v, 64))
        || !token(document, 32)
        || !token(session, 32)
        || !token(generation, 32)
    {
        return Err("BSI_F013_IDENTITY".into());
    }
    #[cfg(feature = "f0031-qualification")]
    {
        let pause_open = open
            && std::env::var("BOARDSI_F0031_QUAL_MODE").ok().as_deref()
                == Some("pause-before-open");
        let pause_inspect = !open
            && std::env::var("BOARDSI_F012_QUAL_MODE").ok().as_deref()
                == Some("pause-before-inspect");
        if pause_open || pause_inspect {
            let marker = std::env::var_os(if pause_open {
                "BOARDSI_F0031_QUAL_MARKER"
            } else {
                "BOARDSI_F012_QUAL_MARKER"
            })
            .ok_or("BSI_QUAL_MARKER_MISSING")?;
            fs::write(
                marker,
                if pause_open {
                    &b"real-native-file-worker-paused"[..]
                } else {
                    &b"real-native-inspection-worker-paused"[..]
                },
            )
            .map_err(|_| "BSI_QUAL_MARKER_WRITE")?;
            loop {
                std::thread::park();
            }
        }
    }
    if open {
        boardsi_native::recover_owned_staging(super::parent(path))
            .map_err(|e| format!("BSI_RECOVERY_IO: {e}"))?;
    }
    // A held read handle denies ordinary Windows writers and replacement.
    #[cfg(windows)]
    let _read_guard = {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .read(true)
            .share_mode(1) // FILE_SHARE_READ only
            .open(path)
            .map_err(|_| "BSI_F013_FILE_GUARD")?
    };
    let initial = stamp(path)?;
    let clock = std::time::Instant::now();
    let digest = super::file_sha256(path)?;
    let first_hash_ms = clock.elapsed().as_millis();
    if expected.is_some_and(|v| v != digest) {
        return Err("BSI_SOURCE_CHANGED".into());
    }
    let load_start = std::time::Instant::now();
    let loaded = boardsi_native::load(path).map_err(super::native_error)?;
    let load_ms = load_start.elapsed().as_millis();
    let index_start = std::time::Instant::now();
    let index = ObjectIndex::build(loaded.model())?;
    let index_ms = index_start.elapsed().as_millis();
    if super::file_sha256(path)? != digest || stamp(path)? != initial {
        return Err("BSI_SOURCE_CHANGED".into());
    }
    let integrity_ms = clock.elapsed().as_millis() - load_ms - index_ms;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let artwork_start = std::time::Instant::now();
    if open {
        let artwork_token = super::f011_artwork_stream::request()?
            .ok_or("BSI_F013_ARTWORK_TOKEN")?;
        let receipt = super::receipt_loaded("open", path, &digest, &loaded, false)?;
        super::f011_artwork_stream::write_receipt(receipt, &artwork_token, &mut output)?;
        if stamp(path)? != initial {
            return Err("BSI_SOURCE_CHANGED".into());
        }
    }
    send(
        &mut output,
        &json!({"protocol":PROTOCOL,"action":"ready","session":session,
            "document":document,"generation":generation,"sha256":digest,
            "indexed_objects":index.entries.len(),"native_loads":1,
            "first_hash_ms":first_hash_ms,"load_ms":load_ms,"index_ms":index_ms,
            "integrity_ms":integrity_ms,"artwork_ms":artwork_start.elapsed().as_millis()}),
    )?;
    while let Some(frame) = read_frame(&mut input)? {
        if frame["protocol"] != PROTOCOL
            || frame["session"] != session
            || frame["document"] != document
            || frame["generation"] != generation
            || frame["sha256"] != digest
        {
            return Err("BSI_F013_IDENTITY".into());
        }
        if stamp(path)? != initial {
            return Err("BSI_SOURCE_CHANGED".into());
        }
        let request = frame["request"].as_str().ok_or("BSI_F013_REQUEST")?;
        if !token(request, 32) {
            return Err("BSI_F013_REQUEST".into());
        }
        let direct = match frame["action"].as_str() {
            Some("query") => None,
            Some("lookup") => Some(index.lookup(&frame["id"])?),
            _ => return Err("BSI_F013_ACTION".into()),
        };
        let category = direct
            .as_ref()
            .map_or(&frame["category"], |v| &v["category"])
            .as_str()
            .ok_or("BSI_F013_CATEGORY")?;
        let offset = direct
            .as_ref()
            .map_or(&frame["offset"], |v| &v["index"])
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or("BSI_F013_OFFSET")?;
        let limit = if direct.is_some() {
            1
        } else {
            frame["limit"]
                .as_u64()
                .and_then(|v| usize::try_from(v).ok())
                .ok_or("BSI_F013_LIMIT")?
        };
        let selection = direct
            .as_ref()
            .or_else(|| frame.get("selection").filter(|v| !v.is_null()))
            .map(Value::to_string);
        let mut result = if let Some((total, rows)) = index.member_page(category, offset, limit)? {
            if selection.is_some() {
                return Err("BSI_F013_SELECTION_INVALID".into());
            }
            let end = offset + rows.len();
            json!({"protocol":"BSI_NATIVE_INSPECT_V1","request":request,
                "sha256":digest,"category":category,"offset":offset,"total":total,
                "next":if end < total { Some(end) } else { None },
                "selection":null,"rows":rows,"physical_si":"NOT_RUN","physical_contact":"NOT_VERIFIED"})
        } else {
            super::f012_inspection::query_loaded(
                &loaded,
                &digest,
                request,
                category,
                offset,
                limit,
                selection.as_deref(),
            )?
        };
        if stamp(path)? != initial {
            return Err("BSI_SOURCE_CHANGED".into());
        }
        result["session"] = json!(session);
        result["document"] = json!(document);
        result["generation"] = json!(generation);
        send(&mut output, &result)?;
    }
    Ok(())
}
