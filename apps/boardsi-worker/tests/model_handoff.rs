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
        source: PropertySource {
            document: "unverified synthetic estimate".into(),
            revision: "v1".into(),
            method: "assumption".into(),
            reference_frequency_hz: Some(1e9),
            temperature_c: Some(25.0),
        },
    };
    assert!(board
        .validate()
        .issues
        .iter()
        .any(|issue| { issue.severity == Severity::Assumption }));
    assert!(
        ModelBoundSyntheticClient::spawn(worker(), "assumed/model", &board, snapshot()).is_err()
    );
}

#[test]
fn model_material_readiness_covers_full_synthetic_sweep() {
    let mut board = model();
    board.materials[0].conductivity_s_per_m = Property::Missing;
    assert!(
        ModelBoundSyntheticClient::spawn(worker(), "missing/conductivity", &board, snapshot())
            .is_err()
    );

    let mut board = model();
    board.materials[0].valid_band = confirmed(FrequencyRange {
        min_hz: 1e6,
        max_hz: 30e9,
    });
    assert!(
        ModelBoundSyntheticClient::spawn(worker(), "out-of-band/dc", &board, snapshot()).is_err()
    );

    let mut board = model();
    board.materials[0].valid_band = confirmed(FrequencyRange {
        min_hz: 0.0,
        max_hz: 1e6,
    });
    assert!(
        ModelBoundSyntheticClient::spawn(worker(), "out-of-band/upper", &board, snapshot())
            .is_err()
    );
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


#[test]
fn model_snapshot_v2_is_bound_to_actual_synthetic_contents() {
    use boardsi_worker::model_snapshot_v2::BoundSyntheticSnapshot;
    let mut board = model();
    let before = BoundSyntheticSnapshot::capture(&board, "f4-1/run-a", 5, 0).unwrap();
    let repeat = BoundSyntheticSnapshot::capture(&board, "f4-1/run-b", 5, 0).unwrap();
    assert_eq!(before.model_sha256(), repeat.model_sha256());
    assert_eq!(before.snapshot_id(), repeat.snapshot_id());
    assert_eq!(before.frame().len(), repeat.frame().len());
    board.nets[0].name.push_str("-changed");
    let changed = BoundSyntheticSnapshot::capture(&board, "f4-1/run-a", 5, 0).unwrap();
    assert_ne!(before.model_sha256(), changed.model_sha256());
    assert_ne!(before.snapshot_id(), changed.snapshot_id());
    // Immutable captured payload is unaffected by later edits to the originating fixture.
    assert_eq!(before.model_sha256(), repeat.model_sha256());
    let other_parameters = BoundSyntheticSnapshot::capture(&board, "f4-1/run-a", 6, 0).unwrap();
    assert_ne!(changed.snapshot_id(), other_parameters.snapshot_id());
    assert_eq!(changed.model_sha256(), other_parameters.model_sha256());
}

#[test]
fn model_snapshot_v2_rejects_unrepresented_or_unready_model() {
    use boardsi_worker::model_snapshot_v2::BoundSyntheticSnapshot;
    let board = BoardModel::empty(BoardKind::Pcb);
    assert!(BoundSyntheticSnapshot::capture(&board, "empty", 5, 0).is_err());
    let mut board = model();
    board.stackup.layers[0].material = None;
    assert!(BoundSyntheticSnapshot::capture(&board, "missing", 5, 0).is_err());
    let mut board = model();
    board.materials[0].df = Property::Missing;
    assert!(BoundSyntheticSnapshot::capture(&board, "missing-property", 5, 0).is_err());
    let mut board = model();
    board.sources[0].producer.push_str("-changed");
    let original = BoundSyntheticSnapshot::capture(&model(), "source", 5, 0).unwrap();
    assert_ne!(original.model_sha256(), BoundSyntheticSnapshot::capture(&board, "source", 5, 0).unwrap().model_sha256());
    let mut board = model();
    board.sources[0].capability.object_coverage.push("foreign".into());
    assert_ne!(original.model_sha256(), BoundSyntheticSnapshot::capture(&board, "source", 5, 0).unwrap().model_sha256());
    let board = model();
    assert!(BoundSyntheticSnapshot::capture(&board, "", 5, 0).is_err());
    assert!(BoundSyntheticSnapshot::capture(&board, "invalid", 0, 0).is_err());
}

#[test]
fn model_snapshot_v2_crosses_real_worker_process_and_rejects_tampering() {
    use boardsi_worker::model_snapshot_v2::{read_frame, write_frame, BoundSyntheticSnapshot};
    use std::io::Write;
    use std::process::{Command, Stdio};
    fn execute(frame: &[u8]) -> std::process::Output {
        let mut child=Command::new(worker())
            .arg("--model-snapshot-v2")
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().expect("real OS Rust Worker must spawn");
        {
            let mut stdin=child.stdin.take().unwrap();
            stdin.write_all(frame).unwrap();
        }
        child.wait_with_output().unwrap()
    }
    let snapshot=BoundSyntheticSnapshot::capture(&model(), "f4-1/os-process-1", 5, 0).unwrap();
    let frame=snapshot.frame();
    let good=execute(&frame);
    assert!(good.status.success(), "{}", String::from_utf8_lossy(&good.stderr));
    let receipt=read_frame(&mut &good.stdout[..]).unwrap();
    snapshot.verify_receipt(&receipt).unwrap();

    let mut response:serde_json::Value=serde_json::from_slice(&receipt).unwrap();
    response["run_id"]=serde_json::json!("other-run");
    assert!(snapshot.verify_receipt(&serde_json::to_vec(&response).unwrap()).is_err());
    let mut response:serde_json::Value=serde_json::from_slice(&receipt).unwrap();
    response["synthetic"]=serde_json::json!(false);
    assert!(snapshot.verify_receipt(&serde_json::to_vec(&response).unwrap()).is_err());

    let mut request:serde_json::Value=serde_json::from_slice(&frame[4..]).unwrap();
    request["model_sha256"]=serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
    let broken=serde_json::to_vec(&request).unwrap();
    let mut bad_frame=Vec::new();
    write_frame(&mut bad_frame,&broken).unwrap();
    let rejected=execute(&bad_frame);
    assert_eq!(rejected.status.code(),Some(2),"invalid model digest must fail closed");
    assert!(rejected.stdout.is_empty(),"no forged receipt on failure");
}
