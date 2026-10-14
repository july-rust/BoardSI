use boardsi_model::*;
use std::sync::Arc as Shared;

fn source() -> SourceId {
    Shared::new(SourceIdentity {
        format: SourceFormat::Synthetic,
        format_version: "fixture-v1".into(),
        files: vec![SourceFile {
            logical_path: "synthetic/board".into(),
            sha256: [7; 32],
        }],
        importer_version: "synthetic-generator-v1".into(),
        conditions: vec![
            ("instance".into(), "0".into()),
            ("step".into(), "board".into()),
        ],
    })
}
fn id(kind: ObjectKind, name: &str) -> ObjectId {
    ObjectId::new(&source(), kind, name, "")
}
fn meta(kind: ObjectKind, name: &str) -> ObjectMeta {
    ObjectMeta {
        id: id(kind, name),
        provenance: GeometryProvenance {
            source_object_type: "generated-test-object".into(),
            history: vec![ConversionStep {
                operation: "synthetic identity mapping".into(),
                quality: ConversionQuality::Exact,
                max_error_m: Some(Rational::integer(0)),
                note: "No real CAD data".into(),
            }],
        },
    }
}
fn r(n: i128, d: i128) -> Rational {
    Rational::new(n, d).unwrap()
}
fn evidence() -> PropertySource {
    PropertySource {
        document: "synthetic unit-test assumption".into(),
        revision: "v1".into(),
        method: "generated".into(),
        reference_frequency_hz: Some(1e9),
        temperature_c: Some(25.0),
    }
}
fn confirmed<T>(value: T) -> Property<T> {
    Property::Confirmed {
        value,
        source: evidence(),
    }
}
fn frame(name: &str, tick: Rational, origin: [Rational; 2]) -> CoordinateFrame {
    CoordinateFrame {
        meta: meta(ObjectKind::Frame, name),
        unit_label: "synthetic rational metre grid".into(),
        metres_per_tick: tick,
        origin_m: origin,
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        source_layer_order_reversed: false,
    }
}
fn p(x: i64, y: i64) -> Point {
    Point {
        frame: id(ObjectKind::Frame, "xy"),
        ticks: [x, y],
    }
}
fn layer(i: usize) -> ObjectId {
    id(ObjectKind::Layer, &format!("L{i}"))
}
fn rectangle(x: i64, y: i64, width: i64, height: i64) -> Boundary {
    let v = [
        p(x, y),
        p(x + width, y),
        p(x + width, y + height),
        p(x, y + height),
    ];
    Boundary {
        segments: (0..4)
            .map(|i| BoundarySegment::Line {
                start: v[i].clone(),
                end: v[(i + 1) % 4].clone(),
            })
            .collect(),
    }
}
fn node(
    name: &str,
    owner_kind: ObjectKind,
    owner: &str,
    layer_index: usize,
    site: ContactSite,
) -> ConnectivityNode {
    ConnectivityNode {
        meta: meta(ObjectKind::Node, name),
        conductor: id(owner_kind, owner),
        layer: layer(layer_index),
        site,
    }
}
fn edge(name: &str, a: &str, b: &str, evidence: ConnectionEvidence) -> ConnectivityEdge {
    ConnectivityEdge {
        meta: meta(ObjectKind::Edge, name),
        a: id(ObjectKind::Node, a),
        b: id(ObjectKind::Node, b),
        evidence,
    }
}
fn contact() -> ConnectionEvidence {
    ConnectionEvidence::SameLayerContact {
        evidence: "synthetic exact endpoint contact".into(),
    }
}
fn fixture() -> BoardModel {
    let mut b = BoardModel::empty(BoardKind::Pcb);
    b.sources.push(ImportSource {
        id: source(),
        producer: "test".into(),
        producer_version: "v1".into(),
        capability: ImportCapability::default(),
    });
    b.frames.push(frame("xy", r(1, 1_000_000), [r(0, 1); 2]));
    b.materials.push(Material {
        meta: meta(ObjectKind::Material, "material"),
        name: "synthetic-test-only".into(),
        dk: confirmed(4.0),
        df: confirmed(0.01),
        conductivity_s_per_m: confirmed(5e7),
        valid_band: confirmed(FrequencyRange {
            min_hz: 0.0,
            max_hz: 30e9,
        }),
    });
    let mut z = 0;
    for i in 0..7 {
        let t = if i % 2 == 0 { 10 } else { 90 };
        b.stackup.layers.push(Layer {
            meta: meta(ObjectKind::Layer, &format!("L{i}")),
            name: format!("L{i}"),
            order: i as u32,
            kind: if i % 2 == 0 {
                LayerKind::Copper
            } else {
                LayerKind::Dielectric(DielectricKind::Core)
            },
            z_m: confirmed(r(z, 1_000_000)),
            thickness_m: confirmed(r(t, 1_000_000)),
            material: Some(id(ObjectKind::Material, "material")),
        });
        z += t;
    }
    for (name, role) in [
        ("S", NetRole::Signal),
        ("G", NetRole::Ground),
        ("P", NetRole::Power),
        ("F", NetRole::Floating),
    ] {
        b.nets.push(Net {
            meta: meta(ObjectKind::Net, name),
            name: name.into(),
            role,
        });
    }
    let net = Some(id(ObjectKind::Net, "S"));
    for (name, li, x) in [("top", 0, 0), ("inner", 2, 0), ("separate", 0, 200)] {
        b.traces.push(Trace {
            meta: meta(ObjectKind::Trace, name),
            layer: layer(li),
            net: net.clone(),
            centreline: vec![p(x, 0), p(x + 100, 0)],
            width_m: r(5, 1_000_000),
        });
        b.connectivity.nodes.push(node(
            name,
            ObjectKind::Trace,
            name,
            li,
            ContactSite::TraceVertex(0),
        ));
    }
    for (name, x) in [("island-a", 300), ("island-b", 500)] {
        b.areas.push(CopperArea {
            meta: meta(ObjectKind::Area, name),
            layer: layer(0),
            net: net.clone(),
            outer: rectangle(x, 0, 100, 100),
            cutouts: vec![],
        });
        b.connectivity.nodes.push(node(
            name,
            ObjectKind::Area,
            name,
            0,
            ContactSite::AreaWitness {
                point: p(x + 10, 10),
                evidence: "synthetic area membership assertion; not geometric proof".into(),
            },
        ));
    }
    b.areas[0].cutouts.push(Cutout {
        meta: meta(ObjectKind::Cutout, "hole"),
        boundary: rectangle(340, 40, 20, 20),
    });
    b.padstacks.push(Padstack {
        meta: meta(ObjectKind::Padstack, "via-stack"),
        entries: [0, 2]
            .iter()
            .map(|&i| PadstackLayer {
                layer: layer(i),
                role: PadRole::Conductive,
                shape: PadShape::Circle {
                    diameter_m: r(20, 1_000_000),
                },
            })
            .collect(),
    });
    b.padstacks.push(Padstack {
        meta: meta(ObjectKind::Padstack, "anti-stack"),
        entries: vec![PadstackLayer {
            layer: layer(4),
            role: PadRole::AntiPad,
            shape: PadShape::Circle {
                diameter_m: r(30, 1_000_000),
            },
        }],
    });
    b.pads.push(Pad {
        meta: meta(ObjectKind::Pad, "anti"),
        position: p(0, 0),
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        padstack: id(ObjectKind::Padstack, "anti-stack"),
        net: net.clone(),
    });
    let span = ViaSpan {
        first_layer: layer(0),
        last_layer: layer(2),
    };
    b.drills.push(Drill {
        meta: meta(ObjectKind::Drill, "drill"),
        diameter_m: r(10, 1_000_000),
        plated: true,
        span: span.clone(),
    });
    b.vias.push(Via {
        meta: meta(ObjectKind::Via, "v"),
        position: p(0, 0),
        padstack: id(ObjectKind::Padstack, "via-stack"),
        drill: id(ObjectKind::Drill, "drill"),
        span,
        kind: ViaKind::Blind,
        net,
        connected_layers: vec![layer(0), layer(2)],
    });
    b.connectivity.nodes.push(node(
        "via-top",
        ObjectKind::Via,
        "v",
        0,
        ContactSite::ViaCentre,
    ));
    b.connectivity.nodes.push(node(
        "via-inner",
        ObjectKind::Via,
        "v",
        2,
        ContactSite::ViaCentre,
    ));
    b.outlines.push(Outline {
        meta: meta(ObjectKind::Outline, "board"),
        outer: rectangle(-100, -100, 1000, 1000),
        cutouts: vec![],
    });
    b
}
fn assert_valid(b: &BoardModel) {
    let report = b.validate();
    assert!(!report.has_errors(), "{report:#?}");
    assert!(!report.has_missing(), "{report:#?}");
}
fn reject(b: &BoardModel, error: ModelError) {
    let report = b.validate();
    assert!(report.has_errors() && report.contains(error), "{report:#?}");
    assert!(b.validated_view().is_err());
}
fn connected(b: &BoardModel, a: &str, c: &str) -> bool {
    b.validated_view()
        .unwrap()
        .connected(&id(ObjectKind::Node, a), &id(ObjectKind::Node, c))
        .unwrap()
}

#[test]
fn complete_synthetic_contract_and_shared_view() {
    let b = fixture();
    assert_valid(&b);
    let view = b.validated_view().unwrap();
    assert!(std::ptr::eq(view.model(), &b));
    assert!(!b.validate_materials_at(1e9).has_missing());
    assert_eq!(b.sources[0].capability.model, CapabilityState::Missing);
}
#[test]
fn different_units_and_origins_exact_same_si_position() {
    let a = frame("a", r(1, 1_000_000), [r(0, 1); 2]);
    let b = frame("b", r(1, 1000), [r(1, 1000), r(-1, 1000)]);
    let p1 = Point {
        frame: a.meta.id.clone(),
        ticks: [2000, 3000],
    };
    let p2 = Point {
        frame: b.meta.id.clone(),
        ticks: [1, 4],
    };
    assert_eq!(a.exact_m(&p1).unwrap(), b.exact_m(&p2).unwrap());
    assert_eq!(a.solver_m(&p1).unwrap(), b.solver_m(&p2).unwrap());
    assert_eq!(a.regrid(&p1, r(1, 1000)).unwrap(), [2, 3]);
}
#[test]
fn nanometre_gap_preserved_without_regridding() {
    let f = frame("nano", r(1, 1_000_000_000), [r(0, 1); 2]);
    let a = Point {
        frame: f.meta.id.clone(),
        ticks: [1_000_000, 0],
    };
    let b = Point {
        ticks: [1_000_001, 0],
        ..a.clone()
    };
    assert_ne!(f.exact_m(&a).unwrap(), f.exact_m(&b).unwrap());
    let gap = f.solver_m(&b).unwrap()[0] - f.solver_m(&a).unwrap()[0];
    assert!((gap - 1e-9).abs() < 1e-18);
    assert_eq!(
        f.regrid(&b, r(1, 1_000_000)),
        Err(ModelError::PrecisionLoss)
    );
}
#[test]
fn origin_precision_loss_is_explicit() {
    let f = frame("large", r(1, 1_000_000_000), [r(1_000_000_000, 1); 2]);
    let a = Point {
        frame: f.meta.id.clone(),
        ticks: [1, 0],
    };
    assert!(f.exact_m(&a).is_ok());
    assert_eq!(f.solver_m(&a), Err(ModelError::PrecisionLoss));
}
#[test]
fn coordinate_overflow_rejected() {
    let f = frame("overflow", r(i128::MAX, 1), [r(0, 1); 2]);
    let p = Point {
        frame: f.meta.id.clone(),
        ticks: [2, 0],
    };
    assert_eq!(f.exact_m(&p), Err(ModelError::CoordinateOverflow));
    let f = frame("wide", r(1, 1), [r(0, 1); 2]);
    let p = Point {
        frame: f.meta.id.clone(),
        ticks: [i64::MAX, 0],
    };
    assert_eq!(f.regrid(&p, r(1, 2)), Err(ModelError::CoordinateOverflow));
}
#[test]
fn exact_mirror_and_rotation_negative_extreme() {
    let mut f = frame("rotate", r(1, 1000), [r(0, 1); 2]);
    f.orientation = Orientation::Orthogonal {
        turn: QuarterTurn::One,
        mirror_x: true,
    };
    let p = Point {
        frame: f.meta.id.clone(),
        ticks: [2, 3],
    };
    assert_eq!(f.exact_m(&p).unwrap(), [r(-3, 1000), r(-2, 1000)]);
    let p = Point {
        ticks: [i64::MIN, 0],
        ..p
    };
    assert!(f.exact_m(&p).is_ok());
    f.orientation = Orientation::PendingRotation {
        radians: 0.1,
        mirror_x: false,
    };
    assert_eq!(f.exact_m(&p), Err(ModelError::UnsupportedTransform));
}
#[test]
fn cross_layer_same_xy_never_connects_implicitly() {
    let mut b = fixture();
    assert_valid(&b);
    assert!(!connected(&b, "top", "inner"));
    b.connectivity
        .edges
        .push(edge("wrong", "top", "inner", contact()));
    reject(&b, ModelError::InvalidConnection);
}
#[test]
fn via_joins_only_explicit_span_layers() {
    let mut b = fixture();
    b.connectivity.edges.extend([
        edge("t", "top", "via-top", contact()),
        edge(
            "barrel",
            "via-top",
            "via-inner",
            ConnectionEvidence::ViaBarrel {
                via: id(ObjectKind::Via, "v"),
            },
        ),
        edge("i", "via-inner", "inner", contact()),
    ]);
    assert_valid(&b);
    assert!(connected(&b, "top", "inner"));
    assert!(!connected(&b, "top", "separate"));
    b.vias[0].connected_layers.push(layer(4));
    reject(&b, ModelError::InvalidConnection);
}
#[test]
fn omitted_via_layer_not_connected() {
    let mut b = fixture();
    b.vias[0].connected_layers = vec![layer(0)];
    reject(&b, ModelError::InvalidConnection); // preexisting inner node now has no conductive contact
    b.connectivity
        .nodes
        .retain(|n| n.meta.id != id(ObjectKind::Node, "via-inner"));
    assert_valid(&b);
    assert!(!connected(&b, "top", "inner"));
}
#[test]
fn blind_buried_microvia_contracts() {
    let mut b = fixture();
    b.vias[0].kind = ViaKind::Microvia;
    assert_valid(&b);
    b.vias[0].kind = ViaKind::Through;
    reject(&b, ModelError::InvalidConnection);
    b.vias[0].kind = ViaKind::Buried;
    reject(&b, ModelError::InvalidConnection);
    b.vias[0].span = ViaSpan {
        first_layer: layer(2),
        last_layer: layer(4),
    };
    b.drills[0].span = b.vias[0].span.clone();
    b.vias[0].connected_layers = vec![layer(2), layer(4)];
    for entry in &mut b.padstacks[0].entries {
        if entry.layer == layer(0) {
            entry.layer = layer(4);
        }
    }
    for n in &mut b.connectivity.nodes {
        if n.meta.id == id(ObjectKind::Node, "via-top") {
            n.layer = layer(4);
        }
    }
    assert_valid(&b);
}
#[test]
fn same_name_islands_and_separate_traces_remain_disconnected() {
    let mut b = fixture();
    assert!(!connected(&b, "island-a", "island-b"));
    assert!(!connected(&b, "top", "separate"));
    b.connectivity
        .edges
        .push(edge("invented", "island-a", "island-b", contact()));
    reject(&b, ModelError::InvalidConnection);
}
#[test]
fn antipad_and_cutout_are_not_conductors() {
    let mut b = fixture();
    assert_valid(&b);
    b.connectivity.nodes.push(node(
        "bad-pad",
        ObjectKind::Pad,
        "anti",
        4,
        ContactSite::PadCentre,
    ));
    reject(&b, ModelError::InvalidConnection);
    b.connectivity.nodes.pop();
    b.connectivity.nodes.push(node(
        "bad-cutout",
        ObjectKind::Cutout,
        "hole",
        0,
        ContactSite::AreaWitness {
            point: p(350, 50),
            evidence: "invented".into(),
        },
    ));
    reject(&b, ModelError::InvalidReference);
}
#[test]
fn unplated_drill_and_antipad_via_contacts_rejected() {
    let mut b = fixture();
    b.drills[0].plated = false;
    reject(&b, ModelError::InvalidConnection);
    b.drills[0].plated = true;
    b.padstacks[0].entries[1].role = PadRole::AntiPad;
    reject(&b, ModelError::InvalidConnection);
}
#[test]
fn ground_power_floating_not_shorted() {
    for net in ["G", "P", "F"] {
        let mut b = fixture();
        b.traces[1].net = Some(id(ObjectKind::Net, net));
        b.traces[1].layer = layer(0);
        b.connectivity.nodes[1].layer = layer(0);
        b.connectivity
            .edges
            .push(edge("short", "top", "inner", contact()));
        reject(&b, ModelError::InvalidConnection);
    }
    let mut b = fixture();
    b.traces[0].net = None;
    b.traces[1].net = None;
    b.traces[1].layer = layer(0);
    b.connectivity.nodes[1].layer = layer(0);
    b.connectivity
        .edges
        .push(edge("unknown", "top", "inner", contact()));
    reject(&b, ModelError::InvalidConnection);
}
#[test]
fn missing_properties_reported_without_fr4_defaults() {
    let mut b = fixture();
    b.stackup.layers[1].thickness_m = Property::Missing;
    assert!(b.validate().has_missing());
    assert_eq!(b.stackup.layers[1].thickness_m.value(), None);
    b.materials[0].dk = Property::Missing;
    assert!(b.validate_materials_at(1e9).has_missing());
    assert_eq!(b.materials[0].dk.value(), None);
    b.materials[0].df = Property::Missing;
    b.materials[0].conductivity_s_per_m = Property::Missing;
    let report = b.validate_materials_at(1e9);
    assert!(report
        .issues
        .iter()
        .any(|i| i.context == "Df required (RF, not DC leakage)"));
    assert!(report
        .issues
        .iter()
        .any(|i| i.context == "conductivity required"));
}
#[test]
fn zero_negative_thickness_duplicate_order_bad_z_rejected() {
    for thickness in [r(0, 1), r(-1, 1)] {
        let mut b = fixture();
        b.stackup.layers[0].thickness_m = confirmed(thickness);
        reject(&b, ModelError::InvalidDimension);
    }
    let mut b = fixture();
    b.stackup.layers[1].order = 0;
    reject(&b, ModelError::InvalidStackup);
    let mut b = fixture();
    b.stackup.layers[1].z_m = confirmed(r(11, 1_000_000));
    reject(&b, ModelError::InvalidStackup);
}
#[test]
fn invalid_material_scalars_and_band_rejected() {
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut b = fixture();
        b.materials[0].dk = confirmed(value);
        reject(&b, ModelError::InvalidMaterial);
    }
    for value in [-0.01, f64::NAN, f64::INFINITY] {
        let mut b = fixture();
        b.materials[0].df = confirmed(value);
        reject(&b, ModelError::InvalidMaterial);
    }
    for value in [0.0, -1.0, f64::NAN] {
        let mut b = fixture();
        b.materials[0].conductivity_s_per_m = confirmed(value);
        reject(&b, ModelError::InvalidMaterial);
    }
    let mut b = fixture();
    b.materials[0].df = confirmed(0.0);
    assert_valid(&b);
    b.materials[0].valid_band = confirmed(FrequencyRange {
        min_hz: 1e9,
        max_hz: 0.0,
    });
    reject(&b, ModelError::InvalidMaterial);
}
#[test]
fn assumed_values_and_out_of_band_are_explicit() {
    let mut b = fixture();
    b.materials[0].dk = Property::Assumed {
        value: 4.1,
        source: evidence(),
    };
    assert!(b
        .validate_materials_at(1e9)
        .contains(ModelError::AssumedValue));
    assert!(b
        .validate_materials_at(31e9)
        .contains(ModelError::OutsideMaterialBand));
    b.materials[0].valid_band = Property::Missing;
    assert!(b.validate_materials_at(1e9).has_missing());
}
#[test]
fn stable_ids_and_format_step_condition_separation() {
    let a = fixture();
    let b = fixture();
    assert_eq!(a, b);
    assert_eq!(a.traces[0].meta.id, b.traces[0].meta.id);
    for change in 0..4 {
        let mut other = (*source()).clone();
        match change {
            0 => other.format = SourceFormat::Odb,
            1 => other.conditions[1].1 = "panel".into(),
            2 => other.importer_version = "v2".into(),
            _ => other.files[0].sha256[0] ^= 1,
        }
        assert_ne!(
            ObjectId::new(&Shared::new(other), ObjectKind::Trace, "top", ""),
            a.traces[0].meta.id
        );
    }
    assert_ne!(
        ObjectId::new(&source(), ObjectKind::Trace, "top", "segment-2"),
        a.traces[0].meta.id
    );
}
#[test]
fn duplicate_id_wrong_kind_unknown_source_rejected() {
    let mut b = fixture();
    b.traces.push(b.traces[0].clone());
    reject(&b, ModelError::DuplicateId);
    let mut b = fixture();
    b.traces[0].meta.id.kind = ObjectKind::Via;
    reject(&b, ModelError::InvalidIdentity);
    let mut b = fixture();
    let mut source = (*source()).clone();
    source.importer_version = "unknown".into();
    b.traces[0].meta.id.source = Shared::new(source);
    reject(&b, ModelError::InvalidIdentity);
}
#[test]
fn invalid_layer_frame_padstack_node_references_rejected() {
    let mut b = fixture();
    b.traces[0].layer = layer(99);
    reject(&b, ModelError::InvalidReference);
    let mut b = fixture();
    b.traces[0].layer = layer(1);
    reject(&b, ModelError::InvalidReference);
    let mut b = fixture();
    b.traces[0].centreline[0].frame = id(ObjectKind::Frame, "unknown");
    reject(&b, ModelError::InvalidReference);
    let mut b = fixture();
    b.vias[0].padstack = id(ObjectKind::Padstack, "unknown");
    reject(&b, ModelError::InvalidReference);
    let mut b = fixture();
    b.connectivity
        .edges
        .push(edge("missing", "top", "unknown", contact()));
    reject(&b, ModelError::InvalidReference);
}
#[test]
fn pcb_and_substrate_share_identical_schema() {
    let pcb = fixture();
    let mut substrate = fixture();
    substrate.kind = BoardKind::BgaSubstrate;
    assert_valid(&substrate);
    assert_eq!(pcb.stackup, substrate.stackup);
    assert_eq!(pcb.vias, substrate.vias);
    assert_eq!(pcb.connectivity, substrate.connectivity);
}
#[test]
fn analytic_arcs_and_closed_boundaries() {
    let mut b = fixture();
    b.arcs.push(Arc {
        meta: meta(ObjectKind::Arc, "arc"),
        layer: layer(0),
        net: Some(id(ObjectKind::Net, "S")),
        centre: p(0, 0),
        start: p(10, 0),
        end: p(0, 10),
        clockwise: false,
        width_m: r(1, 1_000_000),
    });
    assert_valid(&b);
    b.arcs[0].end = p(0, 11);
    reject(&b, ModelError::InvalidGeometry);
    let mut b = fixture();
    b.outlines[0].outer.segments.pop();
    reject(&b, ModelError::InvalidGeometry);
}
#[test]
fn component_pin_and_pad_relations() {
    let mut b = fixture();
    b.pads.push(Pad {
        meta: meta(ObjectKind::Pad, "signal-pad"),
        position: p(0, 0),
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        padstack: id(ObjectKind::Padstack, "via-stack"),
        net: Some(id(ObjectKind::Net, "S")),
    });
    b.components.push(Component {
        meta: meta(ObjectKind::Component, "U1"),
        reference: "U1".into(),
        placement: Placement {
            position: p(0, 0),
            orientation: Orientation::Orthogonal {
                turn: QuarterTurn::Two,
                mirror_x: true,
            },
            bottom_side: true,
        },
    });
    b.pins.push(Pin {
        meta: meta(ObjectKind::Pin, "U1.1"),
        component: id(ObjectKind::Component, "U1"),
        number: "1".into(),
        pad: id(ObjectKind::Pad, "signal-pad"),
        net: Some(id(ObjectKind::Net, "S")),
    });
    assert_valid(&b);
    b.pins[0].net = Some(id(ObjectKind::Net, "G"));
    reject(&b, ModelError::InvalidConnection);
}
#[test]
fn provenance_quality_and_sorted_source_contract() {
    let mut b = fixture();
    b.traces[0].meta.provenance.history[0].quality = ConversionQuality::Lossy;
    b.traces[0].meta.provenance.history[0].max_error_m = None;
    reject(&b, ModelError::InvalidProvenance);
    b.traces[0].meta.provenance.history[0].max_error_m = Some(r(1, 1_000_000));
    assert!(b.validate().contains(ModelError::AssumedValue));
    let mut b = fixture();
    let mut s = (*source()).clone();
    s.conditions.reverse();
    b.sources[0].id = Shared::new(s);
    reject(&b, ModelError::InvalidIdentity);
}
#[test]
fn coordinate_failures_surface_in_model_validation() {
    let mut b = fixture();
    b.frames[0].metres_per_tick = r(i128::MAX, 1);
    reject(&b, ModelError::CoordinateOverflow);
    let mut b = fixture();
    b.frames[0].origin_m = [r(1_000_000_000, 1); 2];
    reject(&b, ModelError::PrecisionLoss);
}

#[test]
fn cross_frame_sub_tick_gap_cannot_collapse() {
    let mut b = fixture();
    b.frames
        .push(frame("offset-a", r(1, 1_000_000), [r(1, 1), r(0, 1)]));
    b.frames.push(frame(
        "offset-b",
        r(1, 1_000_000),
        [
            r(1_000_000_000_000_000_001, 1_000_000_000_000_000_000),
            r(0, 1),
        ],
    ));
    let a = Point {
        frame: id(ObjectKind::Frame, "offset-a"),
        ticks: [0, 0],
    };
    let c = Point {
        frame: id(ObjectKind::Frame, "offset-b"),
        ticks: [0, 0],
    };
    assert_ne!(b.exact_point_m(&a).unwrap(), b.exact_point_m(&c).unwrap());
    assert_eq!(b.solver_point_m(&a).unwrap(), b.solver_point_m(&c).unwrap());
    assert_eq!(b.solver_points_m(&[&a, &c]), Err(ModelError::PrecisionLoss));
    b.traces[0].centreline[0] = a;
    b.traces[1].centreline[0] = c;
    reject(&b, ModelError::PrecisionLoss);
}
