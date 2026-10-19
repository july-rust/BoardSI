//! M0 synthetic-only Canonical Model fixture for the Qt/OS Worker identity proof.
//! Emits TWO distinct immutable versioned frames; NOT an importer or physical SI model.
use boardsi_model::*;
use boardsi_worker::model_snapshot_v2::BoundSyntheticSnapshot;
use std::{error::Error, fs, sync::Arc};

fn source() -> SourceId {
    Arc::new(SourceIdentity {
        format: SourceFormat::Synthetic,
        format_version: "qt-identity-m0-v1".into(),
        files: vec![SourceFile {
            logical_path: "synthetic/qt-model-identity".into(),
            sha256: [37; 32], // synthetic marker only, not a CAD-content proof
        }],
        importer_version: "m0-fixture-v1".into(),
        conditions: vec![],
    })
}
fn metadata(source: &SourceId, kind: ObjectKind, object: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, object, ""),
        provenance: GeometryProvenance {
            source_object_type: "synthetic-qt-fixture".into(),
            history: vec![],
        },
    }
}
fn confirmed<T>(value: T) -> Property<T> {
    Property::Confirmed {
        value,
        source: PropertySource {
            document: "m0-qt-identity-fixture".into(),
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
        producer: "M0 synthetic Qt fixture".into(),
        producer_version: "1".into(),
        capability: ImportCapability::default(),
    });
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
        name: "signal-A".into(),
        role: NetRole::Signal,
    });
    board
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: m0_model_snapshot_fixture FRAME_A FRAME_B".into());
    }
    let mut board = model();
    let a = BoundSyntheticSnapshot::capture(&board, "m0/qt-identity-A", 4, 0)?;
    fs::write(&args[1], a.frame())?;
    board.nets[0].name = "signal-B".into();
    let b = BoundSyntheticSnapshot::capture(&board, "m0/qt-identity-B", 4, 0)?;
    assert_ne!(a.model_sha256(), b.model_sha256());
    assert_ne!(a.snapshot_id(), b.snapshot_id());
    fs::write(&args[2], b.frame())?;
    println!("M0 synthetic-only canonical snapshots A/B generated; no physical SI acceptance");
    Ok(())
}
