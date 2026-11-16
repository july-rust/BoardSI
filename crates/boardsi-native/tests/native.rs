use boardsi_model::*;
use boardsi_native::{load, save};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn path(name: &str) -> std::path::PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "boardsi-f003-{}-{}-{}.bsi",
        std::process::id(),
        now,
        name
    ))
}
fn fixture() -> BoardModel {
    let source = Arc::new(SourceIdentity {
        format: SourceFormat::Synthetic,
        format_version: "1".into(),
        files: vec![SourceFile {
            logical_path: "synthetic/board.asc".into(),
            sha256: [7; 32],
        }],
        importer_version: "F_003-test".into(),
        conditions: vec![("step".into(), "main".into())],
    });
    let mut board = BoardModel::empty(BoardKind::Pcb);
    board.sources.push(ImportSource {
        id: source.clone(),
        producer: "test".into(),
        producer_version: "1".into(),
        capability: ImportCapability::default(),
    });
    let make = |kind, name: &str| ObjectMeta {
        id: ObjectId::new(&source, kind, name, ""),
        provenance: GeometryProvenance {
            source_object_type: "test-source".into(),
            history: vec![],
        },
    };
    let frame = make(ObjectKind::Frame, "f");
    board.frames.push(CoordinateFrame {
        meta: frame.clone(),
        unit_label: "um".into(),
        metres_per_tick: Rational::new(1, 1_000_000).unwrap(),
        origin_m: [Rational::integer(0), Rational::integer(0)],
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        source_layer_order_reversed: false,
    });
    let layer = make(ObjectKind::Layer, "L1");
    board.stackup.layers.push(Layer {
        meta: layer.clone(),
        name: "L1".into(),
        order: 0,
        kind: LayerKind::Copper,
        z_m: Property::Missing,
        thickness_m: Property::Missing,
        material: None,
    });
    board.traces.push(Trace {
        meta: make(ObjectKind::Trace, "T1"),
        layer: layer.id,
        net: None,
        centreline: vec![
            Point {
                frame: frame.id.clone(),
                ticks: [0, 0],
            },
            Point {
                frame: frame.id,
                ticks: [10000, 0],
            },
        ],
        width_m: Rational::new(1, 100_000).unwrap(),
    });
    board
}
#[test]
fn empty_board_round_trip() {
    let p = path("empty");
    let b = BoardModel::empty(BoardKind::Pcb);
    save(&p, &b).unwrap();
    let got = load(&p).unwrap();
    assert!(!got.is_read_only());
    assert_eq!(got.model(), &b);
    std::fs::remove_file(&p).unwrap();
}
#[test]
fn exact_i64_geometry_source_round_trip() {
    let p = path("trace");
    let b = fixture();
    save(&p, &b).unwrap();
    let got = load(&p).unwrap();
    assert_eq!(got.model(), &b);
    std::fs::remove_file(&p).unwrap();
}
#[test]
fn reject_non_native_and_bad_digest_without_modifying_input() {
    let p = path("corrupt");
    let b = fixture();
    save(&p, &b).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute(
        "UPDATE geometry SET envelope=? WHERE geometry_kind=1",
        [vec![0u8; 52]],
    )
    .unwrap();
    drop(conn);
    assert!(load(&p).is_err());
    std::fs::remove_file(&p).unwrap();
    let q = path("other");
    std::fs::write(&q, b"Not native").unwrap();
    assert!(load(&q).is_err());
    std::fs::remove_file(&q).unwrap();
}
#[test]
fn replace_protects_existing_on_invalid_model() {
    let p = path("protect");
    let good = BoardModel::empty(BoardKind::Pcb);
    save(&p, &good).unwrap();
    let before = std::fs::read(&p).unwrap();
    let mut bad = fixture();
    bad.traces[0].width_m = Rational::integer(-1);
    assert!(save(&p, &bad).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), before);
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn all_nine_kinds_in_one_real_sqlite_file() {
    let mut board = fixture();
    let source = board.frames[0].meta.id.source.clone();
    let frame = board.frames[0].meta.id.clone();
    let l1 = board.stackup.layers[0].meta.id.clone();
    let make = |kind, name: &str| ObjectMeta {
        id: ObjectId::new(&source, kind, name, ""),
        provenance: GeometryProvenance {
            source_object_type: "synthetic exact geometry".into(),
            history: vec![],
        },
    };
    let p = |x, y| Point {
        frame: frame.clone(),
        ticks: [x, y],
    };
    let square = |x: i64, y: i64, size: i64| {
        let a = p(x, y);
        let b = p(x + size, y);
        let c = p(x + size, y + size);
        let d = p(x, y + size);
        Boundary {
            segments: vec![
                BoundarySegment::Line {
                    start: a.clone(),
                    end: b.clone(),
                },
                BoundarySegment::Line {
                    start: b,
                    end: c.clone(),
                },
                BoundarySegment::Line {
                    start: c,
                    end: d.clone(),
                },
                BoundarySegment::Line { start: d, end: a },
            ],
        }
    };
    let l2_meta = make(ObjectKind::Layer, "L2");
    let l2 = l2_meta.id.clone();
    board.stackup.layers.push(Layer {
        meta: l2_meta,
        name: "L2".into(),
        order: 1,
        kind: LayerKind::Copper,
        z_m: Property::Missing,
        thickness_m: Property::Missing,
        material: None,
    });
    let width = Rational::new(1, 100_000).unwrap();
    board.arcs.push(boardsi_model::Arc {
        meta: make(ObjectKind::Arc, "ARC1"),
        layer: l1.clone(),
        net: None,
        centre: p(3000, 3000),
        start: p(4000, 3000),
        end: p(3000, 4000),
        clockwise: false,
        width_m: width,
    });
    board.areas.push(CopperArea {
        meta: make(ObjectKind::Area, "AREA1"),
        layer: l1.clone(),
        net: None,
        outer: square(0, 0, 12000),
        cutouts: vec![Cutout {
            meta: make(ObjectKind::Cutout, "AREA-HOLE1"),
            boundary: square(1000, 1000, 1000),
        }],
    });
    board.outlines.push(Outline {
        meta: make(ObjectKind::Outline, "BOARD1"),
        outer: square(-1000, -1000, 16000),
        cutouts: vec![Cutout {
            meta: make(ObjectKind::Cutout, "BOARD-HOLE1"),
            boundary: square(5000, 5000, 1000),
        }],
    });
    let padstack = make(ObjectKind::Padstack, "PS1");
    let ps_id = padstack.id.clone();
    let thermal = PadShape::Thermal {
        round: true,
        centre_size_m: Rational::new(1, 10000).unwrap(),
        outer_size_m: Rational::new(2, 10000).unwrap(),
        spoke_width_m: Rational::new(1, 100000).unwrap(),
        first_spoke_degrees: 45,
        spoke_count: 4,
    };
    board.padstacks.push(Padstack {
        meta: padstack,
        entries: vec![
            PadstackLayer {
                layer: l1.clone(),
                role: PadRole::Conductive,
                shape: PadShape::Circle {
                    diameter_m: Rational::new(1, 10000).unwrap(),
                },
            },
            PadstackLayer {
                layer: l1.clone(),
                role: PadRole::AntiPad,
                shape: PadShape::Rectangle {
                    size_m: [
                        Rational::new(1, 10000).unwrap(),
                        Rational::new(2, 10000).unwrap(),
                    ],
                },
            },
            PadstackLayer {
                layer: l2.clone(),
                role: PadRole::Conductive,
                shape: PadShape::Annulus {
                    outer_diameter_m: Rational::new(2, 10000).unwrap(),
                    inner_diameter_m: Rational::new(1, 10000).unwrap(),
                },
            },
            PadstackLayer {
                layer: l2.clone(),
                role: PadRole::Clearance,
                shape: thermal,
            },
            PadstackLayer {
                layer: l2.clone(),
                role: PadRole::AntiPad,
                shape: PadShape::Custom(square(0, 0, 100)),
            },
        ],
    });
    let pad_meta = make(ObjectKind::Pad, "PAD1");
    let pad_id = pad_meta.id.clone();
    board.pads.push(Pad {
        meta: pad_meta,
        position: p(0, 0),
        orientation: Orientation::PendingRotation {
            radians: 0.37,
            mirror_x: true,
        },
        padstack: ps_id.clone(),
        net: None,
    });
    let span = ViaSpan {
        first_layer: l1.clone(),
        last_layer: l2.clone(),
    };
    let drill_meta = make(ObjectKind::Drill, "DRILL1");
    let drill_id = drill_meta.id.clone();
    board.drills.push(Drill {
        meta: drill_meta,
        diameter_m: Rational::new(1, 20000).unwrap(),
        plated: true,
        span: span.clone(),
    });
    board.vias.push(Via {
        meta: make(ObjectKind::Via, "VIA1"),
        position: p(8000, 8000),
        padstack: ps_id,
        drill: drill_id,
        span,
        kind: ViaKind::Through,
        net: None,
        connected_layers: vec![l1, l2],
    });
    let comp_meta = make(ObjectKind::Component, "U1");
    let comp_id = comp_meta.id.clone();
    board.components.push(Component {
        meta: comp_meta,
        reference: "U1".into(),
        placement: Placement {
            position: p(0, 0),
            orientation: Orientation::Orthogonal {
                turn: QuarterTurn::Two,
                mirror_x: true,
            },
            bottom_side: false,
        },
    });
    board.pins.push(Pin {
        meta: make(ObjectKind::Pin, "PIN1"),
        component: comp_id,
        number: "1".into(),
        pad: pad_id,
        net: None,
    });
    let pth = path("nine-kinds");
    save(&pth, &board).unwrap();
    let read_back = load(&pth).unwrap();
    assert!(!read_back.is_read_only());
    assert_eq!(read_back.model().traces.len(), 1);
    assert_eq!(read_back.model().arcs.len(), 1);
    assert_eq!(read_back.model().areas.len(), 1);
    assert_eq!(read_back.model().areas[0].cutouts.len(), 1);
    assert_eq!(read_back.model().padstacks.len(), 1);
    assert_eq!(read_back.model().pads.len(), 1);
    assert_eq!(read_back.model().vias.len(), 1);
    assert_eq!(read_back.model().drills.len(), 1);
    assert_eq!(read_back.model().outlines.len(), 1);
    assert_eq!(read_back.model().outlines[0].cutouts.len(), 1);
    assert_eq!(read_back.model(), &board);
    // F004 release Golden: freeze the complete nine-kind Canonical sample.
    // Keep the ordinary F003 round-trip test unchanged when no destination
    // was explicitly requested by the independent qualification runner.
    if let Some(output) = std::env::var_os("F004_FULL_NINE_OUTPUT") {
        let conn = rusqlite::Connection::open(&pth).unwrap();
        conn.execute(
            "UPDATE file_meta SET file_uuid='00000000-0000-4000-8000-000000000009', created_utc='2026-11-16T00:00:00Z', producer='F004 frozen nine-kind reference', producer_version='1'",
            [],
        )
        .unwrap();
        drop(conn);
        assert_eq!(load(&pth).unwrap().model(), &board);
        std::fs::copy(&pth, output).unwrap();
    }
    std::fs::remove_file(&pth).unwrap();
}

#[test]
fn cross_os_reference_fixture() {
    let Ok(dir) = std::env::var("F003_CROSS_OS_DIR") else {
        return; // Only run as a multi-platform fixture exchange contract.
    };
    let filename = std::path::Path::new(&dir).join("cross_os_fixture.bsi");
    let expected = fixture();
    if filename.exists() {
        let got = load(&filename).expect("cross-platform SQLite .bsi read");
        assert!(!got.is_read_only());
        assert_eq!(
            got.model(),
            &expected,
            "lossless cross-OS Canonical equality"
        );
    } else {
        assert!(
            std::env::var("F003_CROSS_OS_WRITER").as_deref() == Ok("1"),
            "reader must not silently regenerate missing cross-platform input"
        );
        save(&filename, &expected).expect("native Ubuntu fixture writer");
    }
}

#[test]
fn unknown_feature_and_future_schema_fail_closed() {
    let p = path("version-feature");
    save(&p, &fixture()).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute(
        "INSERT INTO features(name,feature_version,required,opaque_payload) VALUES('vendor-note',1,0,X'0102')",
        [],
    ).unwrap();
    drop(conn);
    assert!(
        load(&p).unwrap().is_read_only(),
        "unknown optional feature must prevent Save As"
    );
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute("UPDATE features SET required=1", []).unwrap();
    drop(conn);
    assert_eq!(load(&p).err().unwrap().code(), "BSI_UNSUPPORTED_REQUIRED");
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute("DELETE FROM features", []).unwrap();
    conn.pragma_update(None, "user_version", 2i64).unwrap();
    drop(conn);
    assert_eq!(load(&p).err().unwrap().code(), "BSI_FUTURE_SCHEMA");
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn verified_sha_provenance_does_not_silently_disappear() {
    let p = path("sha-verified");
    save(&p, &fixture()).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute("UPDATE source_files SET sha_verified=1", [])
        .unwrap();
    drop(conn);
    assert!(load(&p).unwrap().is_read_only());
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn sql_blob_kind_mismatch_fail_closed() {
    let p = path("sql-kind");
    save(&p, &fixture()).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute(
        "UPDATE geometry SET geometry_kind=2 WHERE geometry_kind=1",
        [],
    )
    .unwrap();
    drop(conn);
    assert_eq!(load(&p).err().unwrap().code(), "BSI_INVALID");
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn geometry_bsig_bytes_are_deterministic_across_saves() {
    let a = path("stable-a");
    let b = path("stable-b");
    let board = fixture();
    save(&a, &board).unwrap();
    save(&b, &board).unwrap();
    let read = |file: &std::path::Path| -> Vec<u8> {
        rusqlite::Connection::open(file)
            .unwrap()
            .query_row(
                "SELECT envelope FROM geometry WHERE geometry_kind=1",
                [],
                |r| r.get(0),
            )
            .unwrap()
    };
    assert_eq!(
        read(&a),
        read(&b),
        "BSIG payload bytes must be deterministic"
    );
    std::fs::remove_file(&a).unwrap();
    std::fs::remove_file(&b).unwrap();
}

#[test]
fn material_property_and_layer_evidence_round_trip() {
    let mut board = fixture();
    let src = board.frames[0].meta.id.source.clone();
    let id = ObjectId::new(&src, ObjectKind::Material, "FR4", "");
    let evidence = PropertySource {
        document: "fixture-datasheet".into(),
        revision: "A".into(),
        method: "vendor-declared".into(),
        reference_frequency_hz: Some(1_000_000_000.0),
        temperature_c: Some(23.0),
    };
    board.materials.push(Material {
        meta: ObjectMeta {
            id: id.clone(),
            provenance: GeometryProvenance {
                source_object_type: "synthetic material".into(),
                history: vec![],
            },
        },
        name: "FR-4".into(),
        dk: Property::Confirmed {
            value: 4.4,
            source: evidence.clone(),
        },
        df: Property::Assumed {
            value: 0.015,
            source: evidence.clone(),
        },
        conductivity_s_per_m: Property::Missing,
        valid_band: Property::Confirmed {
            value: FrequencyRange {
                min_hz: 1.0,
                max_hz: 10_000_000_000.0,
            },
            source: evidence.clone(),
        },
    });
    board.stackup.layers[0].material = Some(id);
    board.stackup.layers[0].z_m = Property::Confirmed {
        value: Rational::integer(0),
        source: evidence.clone(),
    };
    board.stackup.layers[0].thickness_m = Property::Assumed {
        value: Rational::new(35, 1_000_000).unwrap(),
        source: evidence,
    };
    let p = path("material-evidence");
    save(&p, &board).unwrap();
    assert_eq!(load(&p).unwrap().model(), &board);
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn unversioned_extra_schema_must_fail_closed() {
    let p = path("schema-drift");
    save(&p, &fixture()).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute_batch("CREATE TABLE hidden_vendor_payload(data BLOB) STRICT")
        .unwrap();
    drop(conn);
    assert_eq!(load(&p).err().unwrap().code(), "BSI_INVALID");
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn save_never_overwrites_read_only_unknown_optional_feature() {
    let p = path("protect-optional-feature");
    let board = fixture();
    save(&p, &board).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute("INSERT INTO features(name,feature_version,required,opaque_payload) VALUES('opaque-evidence',1,0,X'DEADBEEF')",[]).unwrap();
    drop(conn);
    let loaded = load(&p).unwrap();
    assert!(loaded.is_read_only());
    assert!(loaded.model().persistence_read_only);
    let other = path("save-as-opaque-rejection");
    assert_eq!(
        save(&other, loaded.model()).err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    assert_eq!(
        loaded.save_as(&other).err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    assert!(
        !other.exists(),
        "Save As must not create a lossy replacement"
    );
    let original = std::fs::read(&p).unwrap();
    assert_eq!(
        save(&p, &board).err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    assert_eq!(std::fs::read(&p).unwrap(), original);
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn relative_destination_sync_does_not_report_failure_after_replace() {
    let filename = format!(
        "f003-relative-{}-{}.bsi",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let relative = std::path::PathBuf::from(filename);
    let board = BoardModel::empty(BoardKind::Pcb);
    save(&relative, &board).expect("safe relative filename save");
    assert_eq!(load(&relative).unwrap().model(), &board);
    std::fs::remove_file(&relative).unwrap();
}

#[test]
fn sqlite_internal_prefix_is_literal_not_like_wildcard() {
    let p = path("sqlite-name-prefix");
    save(&p, &fixture()).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute_batch(
        "CREATE TABLE sqliteXvendor(data BLOB) STRICT;\
         INSERT INTO sqliteXvendor VALUES(X'DEADBEEF');",
    )
    .unwrap();
    drop(conn);
    assert_eq!(load(&p).err().unwrap().code(), "BSI_INVALID");
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn unrepresentable_geometry_relation_columns_fail_closed() {
    let mut board = fixture();
    let source = board.frames[0].meta.id.source.clone();
    let layer = board.stackup.layers[0].meta.id.clone();
    let frame = board.frames[0].meta.id.clone();
    let stack_id = ObjectId::new(&source, ObjectKind::Padstack, "PS-RELS", "");
    let provenance = GeometryProvenance {
        source_object_type: "synthetic".into(),
        history: vec![],
    };
    board.padstacks.push(Padstack {
        meta: ObjectMeta {
            id: stack_id.clone(),
            provenance: provenance.clone(),
        },
        entries: vec![PadstackLayer {
            layer: layer.clone(),
            role: PadRole::Conductive,
            shape: PadShape::Circle {
                diameter_m: Rational::new(1, 10_000).unwrap(),
            },
        }],
    });
    board.pads.push(Pad {
        meta: ObjectMeta {
            id: ObjectId::new(&source, ObjectKind::Pad, "PAD-RELS", ""),
            provenance,
        },
        position: Point {
            frame,
            ticks: [50, 50],
        },
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        padstack: stack_id,
        net: None,
    });
    let p = path("sql-extra-rel");
    save(&p, &board).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute_batch(
        "UPDATE geometry SET padstack_id=(SELECT object_id FROM geometry WHERE geometry_kind=5) \
         WHERE geometry_kind=1;",
    )
    .unwrap();
    drop(conn);
    assert_eq!(load(&p).err().unwrap().code(), "BSI_INVALID");
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute_batch(
        "UPDATE geometry SET padstack_id=NULL WHERE geometry_kind=1;\
         UPDATE geometry SET layer_id=(SELECT object_id FROM layers LIMIT 1) WHERE geometry_kind=6;"
    ).unwrap();
    drop(conn);
    assert_eq!(load(&p).err().unwrap().code(), "BSI_INVALID");
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn unknown_optional_save_as_cannot_drop_opaque_data() {
    let p = path("opaque-source");
    let other = path("opaque-save-as");
    save(&p, &fixture()).unwrap();
    let conn = rusqlite::Connection::open(&p).unwrap();
    conn.execute("INSERT INTO features(name,feature_version,required,opaque_payload) VALUES('proprietary-optional',1,0,X'CAFE')",[]).unwrap();
    drop(conn);
    let document = load(&p).unwrap();
    assert!(document.is_read_only());
    assert_eq!(
        document.save_as(&other).err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    assert_eq!(
        document.into_editable().err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    assert!(!other.exists());
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn pending_rotation_cannot_hide_invalid_arc_geometry() {
    let mut board = fixture();
    board.frames[0].orientation = Orientation::PendingRotation {
        radians: 0.125,
        mirror_x: false,
    };
    let source = board.frames[0].meta.id.source.clone();
    let frame = board.frames[0].meta.id.clone();
    let centre = Point {
        frame,
        ticks: [100, 100],
    };
    board.arcs.push(boardsi_model::Arc {
        meta: ObjectMeta {
            id: ObjectId::new(&source, ObjectKind::Arc, "PENDING-BAD-ARC", ""),
            provenance: GeometryProvenance {
                source_object_type: "synthetic".into(),
                history: vec![],
            },
        },
        layer: board.stackup.layers[0].meta.id.clone(),
        net: None,
        centre: centre.clone(),
        start: centre.clone(),
        end: centre,
        clockwise: false,
        width_m: Rational::new(1, 100_000).unwrap(),
    });
    let p = path("pending-invalid-arc");
    assert_eq!(save(&p, &board).err().unwrap().code(), "BSI_INVALID");
    assert!(!p.exists());
}

#[test]
fn later_minor_version_is_read_only_and_save_as_cannot_drop_it() {
    let original = path("future-minor");
    save(&original, &fixture()).unwrap();
    let conn = rusqlite::Connection::open(&original).unwrap();
    conn.execute("UPDATE file_meta SET format_minor=1", [])
        .unwrap();
    drop(conn);
    let loaded = load(&original).unwrap();
    assert!(loaded.is_read_only());
    assert!(loaded.model().persistence_read_only);
    let copy = path("future-minor-copy");
    assert_eq!(
        loaded.save_as(&copy).err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    assert_eq!(
        save(&copy, loaded.model()).err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    assert!(!copy.exists());
    std::fs::remove_file(&original).unwrap();
}

#[test]
fn source_file_total_budget_rejects_over_100k_before_writing() {
    let mut board = fixture();
    let mut identity = (*board.sources[0].id).clone();
    identity.files = (0..100_001)
        .map(|n| SourceFile {
            logical_path: format!("synthetic/file-{n:06}.asc"),
            sha256: [7; 32],
        })
        .collect();
    board.sources[0].id = Arc::new(identity);
    let output = path("source-file-budget");
    assert_eq!(save(&output, &board).err().unwrap().code(), "BSI_INVALID");
    assert!(!output.exists());
}

#[test]
fn nonempty_connectivity_graph_round_trip_and_corrupt_witness_fail_closed() {
    let mut board = fixture();
    let source = board.frames[0].meta.id.source.clone();
    let frame = board.frames[0].meta.id.clone();
    let layer = board.stackup.layers[0].meta.id.clone();
    let meta = |kind, key: &str| ObjectMeta {
        id: ObjectId::new(&source, kind, key, ""),
        provenance: GeometryProvenance {
            source_object_type: "synthetic-contact-evidence".into(),
            history: vec![],
        },
    };
    let net = meta(ObjectKind::Net, "NET_A");
    let net_id = net.id.clone();
    board.nets.push(Net {
        meta: net,
        name: "NET_A".into(),
        role: NetRole::Signal,
    });
    board.traces[0].net = Some(net_id.clone());
    let first_trace = board.traces[0].meta.id.clone();
    let second_trace = meta(ObjectKind::Trace, "T2");
    let second_trace_id = second_trace.id.clone();
    board.traces.push(Trace {
        meta: second_trace,
        layer: layer.clone(),
        net: Some(net_id),
        centreline: vec![
            Point {
                frame: frame.clone(),
                ticks: [10000, 0],
            },
            Point {
                frame,
                ticks: [20000, 0],
            },
        ],
        width_m: Rational::new(1, 100_000).unwrap(),
    });
    let node_a = meta(ObjectKind::Node, "TRACE-T1-END");
    let node_b = meta(ObjectKind::Node, "TRACE-T2-START");
    let (a, b) = (node_a.id.clone(), node_b.id.clone());
    board.connectivity.nodes.push(ConnectivityNode {
        meta: node_a,
        conductor: first_trace,
        layer: layer.clone(),
        site: ContactSite::TraceVertex(1),
    });
    board.connectivity.nodes.push(ConnectivityNode {
        meta: node_b,
        conductor: second_trace_id,
        layer,
        site: ContactSite::TraceVertex(0),
    });
    board.connectivity.edges.push(ConnectivityEdge {
        meta: meta(ObjectKind::Edge, "T1-T2-JOINT"),
        a,
        b,
        evidence: ConnectionEvidence::SameLayerContact {
            evidence: "source-declared coincident endpoints".into(),
        },
    });
    assert!(
        !board.validate().has_errors(),
        "synthetic contact must be valid"
    );
    let file = path("graph-contact-evidence");
    save(&file, &board).unwrap();
    let recovered = load(&file).unwrap();
    assert!(!recovered.is_read_only());
    assert_eq!(
        recovered.model(),
        &board,
        "complete nonempty graph must round trip"
    );
    drop(recovered);

    // A valid SQLite enum mutation must still be rejected by native semantic validation.
    let conn = rusqlite::Connection::open(&file).unwrap();
    conn.execute(
        "UPDATE connectivity_nodes SET witness_kind='PadCentre', trace_vertex_index=NULL \
         WHERE object_id=(SELECT MIN(object_id) FROM connectivity_nodes)",
        [],
    )
    .unwrap();
    drop(conn);
    assert_eq!(load(&file).err().unwrap().code(), "BSI_INVALID");
    std::fs::remove_file(&file).unwrap();
}

#[cfg(feature = "f003-qualification")]
#[test]
fn qualification_ten_thousand_geometry_round_trip_with_resource_evidence() {
    let count = std::env::var("F003_STRESS_OBJECTS")
        .ok()
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(10_000);
    assert!((1..=40_000).contains(&count));
    let mut board = fixture();
    let sample = board.traces[0].clone();
    let source = sample.meta.id.source.clone();
    board.traces.clear();
    for i in 0..count {
        let mut trace = sample.clone();
        trace.meta.id = ObjectId::new(&source, ObjectKind::Trace, &format!("bulk-{i:06}"), "");
        let x = i64::try_from(i).unwrap() * 100;
        trace.centreline[0].ticks = [x, 0];
        trace.centreline[1].ticks = [x + 50, 0];
        board.traces.push(trace);
    }
    let p = path("10k-resource");
    let start = std::time::Instant::now();
    save(&p, &board).expect("bounded large native writer");
    let write_elapsed = start.elapsed();
    let size = std::fs::metadata(&p).unwrap().len();
    assert!(size <= 512 * 1024 * 1024);
    let loaded = load(&p).expect("bounded large native reader");
    let read_elapsed = start.elapsed() - write_elapsed;
    assert_eq!(loaded.model().traces.len(), count);
    assert_eq!(loaded.model(), &board);
    println!(
        "F003_RESOURCE_EVIDENCE objects={count} bsi_bytes={size} save_ms={} load_ms={}",
        write_elapsed.as_millis(),
        read_elapsed.as_millis()
    );
    std::fs::remove_file(p).unwrap();
}

// F0031: SQLite WAL can alter the accepted semantic snapshot without touching
// the main .bsi bytes. v1 is single-file DELETE journaling only.
#[test]
fn wal_only_external_change_never_becomes_accepted_native_snapshot() {
    let p = path("wal-only");
    let b = fixture();
    save(&p, &b).unwrap();
    let before = std::fs::read(&p).unwrap();
    let db = rusqlite::Connection::open(&p).unwrap();
    let wal_mode: String = db
        .pragma_query_value(None, "journal_mode", |r| r.get(0))
        .unwrap();
    assert_eq!(wal_mode.to_ascii_lowercase(), "delete");
    let enabled: String = db
        .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
        .unwrap();
    assert_eq!(enabled.to_ascii_lowercase(), "wal");
    db.execute_batch(
        "BEGIN IMMEDIATE; UPDATE file_meta SET board_kind='BgaSubstrate' WHERE singleton=1; COMMIT;",
    )
    .unwrap();
    // Keep the WAL connection open to prevent auto-checkpoint on close.
    assert_eq!(std::fs::read(&p).unwrap()[18], 2);
    assert_eq!(std::fs::read(&p).unwrap()[19], 2);
    assert!(p.with_extension("bsi-wal").exists());
    assert_eq!(load(&p).err().unwrap().code(), "BSI_INVALID");
    // A live WAL DB must not be overwritten or silently canonicalized.
    let unchanged = std::fs::read(&p).unwrap();
    assert!(save(&p, &b).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), unchanged);
    assert_ne!(
        unchanged, before,
        "WAL header must differ from DELETE snapshot"
    );
    drop(db);
    std::fs::remove_file(&p).unwrap();
}
