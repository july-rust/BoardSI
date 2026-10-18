//! Executed test: validated Canonical Model -> synthetic IPC Snapshot -> Rust Worker.
//! No Qt integration, model-geometry transfer or PCB SI certification implied.
use boardsi_ipc::{Body, Snapshot, State};
use boardsi_model::*;
use boardsi_worker::model_handoff::ModelBoundSyntheticClient;
use std::{
    path::Path,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

fn source() -> SourceId {
    Arc::new(SourceIdentity {
        format: SourceFormat::Synthetic,
        format_version: "handoff-fixture-v1".into(),
        files: vec![SourceFile {
            logical_path: "synthetic/model-handoff".into(),
            sha256: [42; 32], // synthetic fixture marker, NOT a verified raw CAD file hash
        }],
        importer_version: "synthetic-test-v1".into(),
        conditions: vec![],
    })
}
fn metadata(source: &SourceId, kind: ObjectKind, object: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, object, ""),
        provenance: GeometryProvenance {
            source_object_type: "synthetic-fixture".into(),
            history: vec![],
        },
    }
}
fn confirmed<T>(value: T) -> Property<T> {
    Property::Confirmed {
        value,
        source: PropertySource {
            document: "synthetic-model-handoff-test".into(),
            revision: "1".into(),
            method: "generated".into(),
            reference_frequency_hz: Some(1e9),
            temperature_c: Some(25.0),
        },
    }
}
fn model() -> BoardModel {
    let source = source();
    let mut board = BoardModel::empty(BoardKind::Pcb);
    board.sources.push(ImportSource {
        id: source.clone(),
        producer: "M0 test".into(),
        producer_version: "1".into(),
        capability: ImportCapability::default(),
    });
    // A real canonical outline and exact source coordinate frame are required;
    // omitted geometry must not be promoted as model readiness.
    let frame_id = ObjectId::new(&source, ObjectKind::Frame, "xy", "");
    board.frames.push(CoordinateFrame {
        meta: metadata(&source, ObjectKind::Frame, "xy"),
        unit_label: "synthetic metre grid".into(),
        metres_per_tick: Rational::new(1, 1_000_000).unwrap(),
        origin_m: [Rational::integer(0); 2],
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        source_layer_order_reversed: false,
    });
    let vertices = [[0, 0], [1_000, 0], [1_000, 1_000], [0, 1_000]].map(|ticks| Point {
        frame: frame_id.clone(),
        ticks,
    });
    board.outlines.push(Outline {
        meta: metadata(&source, ObjectKind::Outline, "outline"),
        outer: Boundary {
            segments: (0..4)
                .map(|index| BoundarySegment::Line {
                    start: vertices[index].clone(),
                    end: vertices[(index + 1) % 4].clone(),
                })
                .collect(),
        },
        cutouts: vec![],
    });
    let material = metadata(&source, ObjectKind::Material, "copper");
    let material_id = material.id.clone();
    board.materials.push(Material {
        meta: material,
        name: "synthetic copper".into(),
        dk: confirmed(4.0),
        df: confirmed(0.0),
        conductivity_s_per_m: confirmed(5e7),
        valid_band: confirmed(FrequencyRange {
            min_hz: 0.0,
            max_hz: 30e9,
        }),
    });
    board.stackup.layers.push(Layer {
        meta: metadata(&source, ObjectKind::Layer, "L0"),
        name: "synthetic copper layer".into(),
        order: 0,
        kind: LayerKind::Copper,
        z_m: confirmed(Rational::integer(0)),
        thickness_m: confirmed(Rational::new(10, 1_000_000).unwrap()),
        material: Some(material_id),
    });
    board.nets.push(Net {
        meta: metadata(&source, ObjectKind::Net, "signal"),
        name: "signal".into(),
        role: NetRole::Signal,
    });
    board
}
fn worker() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_boardsi-worker"))
}
fn snapshot() -> Snapshot {
    Snapshot {
        points: 5,
        delay_ms: 2,
    }
}
#[test]
fn structural_model_gates_real_synthetic_worker_process() {
    let board = model();
    let report = board.validate();
    assert!(!report.has_errors());
    assert!(!report.has_missing());
    let mut bounded =
        ModelBoundSyntheticClient::spawn(worker(), "model-handoff/run-1", &board, snapshot())
            .unwrap();
    assert!(std::ptr::eq(bounded.model(), &board));
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut synthetic_count = 0;
    loop {
        for msg in bounded.poll().unwrap() {
            if let Body::Result { synthetic, .. } = msg.body {
                assert!(
                    synthetic,
                    "synthetic Worker must never impersonate physical SI"
                );
                synthetic_count += 1;
            }
        }
        if bounded.exit_success().is_some() {
            break;
        }
        assert!(Instant::now() < deadline, "Worker timeout");
        thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(bounded.state(), State::Completed);
    assert!(bounded.exit_success().unwrap());
    assert_eq!(synthetic_count, 5);
    assert_eq!(bounded.result_count(), 5);
}
#[test]
fn malformed_model_is_rejected_before_worker_spawn() {
    let mut board = model();
    board.nets.push(board.nets[0].clone()); // duplicate ObjectId
    let err =
        ModelBoundSyntheticClient::spawn(worker(), "model-handoff/invalid", &board, snapshot())
            .err()
            .expect("must reject duplicate model ID");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}
#[test]
fn missing_properties_and_non_synthetic_sources_fail_closed() {
    let mut board = model();
    board.stackup.layers[0].material = None;
    assert!(
        ModelBoundSyntheticClient::spawn(worker(), "invalid/missing", &board, snapshot()).is_err()
    );
    let mut board = model();
    let mut identity = (*board.sources[0].id).clone();
    identity.format = SourceFormat::PadsAsc;
    board.sources[0].id = Arc::new(identity);
    assert!(
        ModelBoundSyntheticClient::spawn(worker(), "invalid/provenance", &board, snapshot())
            .is_err()
    );
}
#[test]
fn assumed_material_is_not_accepted_as_verified() {
    let mut board = model();
    board.materials[0].conductivity_s_per_m = Property::Assumed {
        value: 5e7,
        rationale: "unverified test guess".into(),
    };
    assert!(board.validate().issues.iter().any(|issue| {
        issue.severity == Severity::Assumption
    }));
    assert!(ModelBoundSyntheticClient::spawn(
        worker(), "assumed/model", &board, snapshot()
    ).is_err());
}

#[test]
fn empty_board_and_invalid_snapshot_fail_closed() {
    let board = BoardModel::empty(BoardKind::Pcb);
    assert!(ModelBoundSyntheticClient::spawn(worker(), "empty", &board, snapshot()).is_err());
    let board = model();
    assert!(ModelBoundSyntheticClient::spawn(
        worker(),
        "invalid-snapshot",
        &board,
        Snapshot {
            points: 0,
            delay_ms: 0
        }
    )
    .is_err());
}
