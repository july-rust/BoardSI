//! D_056_08 source-only Canonical fabrication contract integration tests.
use boardsi_import::odb_fabrication::import_unpacked_with_holes;
use boardsi_import::odb_source::OdbFileMap;
use boardsi_model::{FabricationHoleShape, FabricationPlating, Rational};

fn product(layers: usize) -> OdbFileMap {
    let mut files = OdbFileMap::new();
    let mut matrix = "STEP {\nCOL=1\nNAME=pcb\n}\n".to_owned();
    for n in 1..=layers {
        matrix.push_str(&format!(
            "LAYER {{\nROW={n}\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig{n}\nPOLARITY=POSITIVE\n}}\n"
        ));
        files.insert(
            format!("steps/pcb/layers/sig{n}/features"),
            b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n".to_vec(),
        );
    }
    matrix.push_str(&format!(
        "LAYER {{\nROW={}\nCONTEXT=BOARD\nTYPE=DRILL\nNAME=hole\nPOLARITY=POSITIVE\nSTART_NAME=sig1\nEND_NAME=sig{layers}\n}}\n",
        layers + 1
    ));
    files.insert("matrix/matrix".into(), matrix.into_bytes());
    files.insert(
        "steps/pcb/layers/hole/tools".into(),
        b"UNITS=MM\nTOOLS {\nNUM=7\nTYPE=PLATED\nTYPE2=STANDARD\nFINISH_SIZE=140\nDRILL_SIZE=160\n}\nTOOLS {\nNUM=8\nTYPE=NON_PLATED\nTYPE2=STANDARD\nFINISH_SIZE=200\n}\nTOOLS {\nNUM=9\nTYPE=VIA\nTYPE2=STANDARD\nFINISH_SIZE=120\n}\n".to_vec(),
    );
    files.insert(
        "steps/pcb/layers/hole/features".into(),
        b"UNITS=MM\nF 3\n$0 r100\nP 1.25 -0.25 0 P 7 0;ID=h1\nP 2.5 1.125 0 P 8 0;ID=h2\nP 3 3 0 P 9 0;ID=h3\n".to_vec(),
    );
    files
}

#[test]
fn arbitrary_layer_count_exact_hole_geometry_and_unknown_plating() {
    for count in [2, 4, 6, 8, 64] {
        let out = import_unpacked_with_holes(&product(count), "pcb", false).unwrap();
        assert_eq!(out.board.stackup.layers.len(), count);
        assert_eq!(out.board.traces.len(), count);
        assert!(out
            .board
            .validate()
            .issues
            .iter()
            .all(|i| i.severity != boardsi_model::Severity::Error));
        assert!(out.board.drills.is_empty());
        assert!(out.board.vias.is_empty());
        assert!(out.board.connectivity.edges.is_empty());
        assert_eq!(out.source_holes.len(), 3);
        let a = &out.source_holes[0];
        assert_eq!(a.xy_m[0], Rational::new(1, 800).unwrap()); // 1.25 mm
        assert_eq!(a.xy_m[1], Rational::new(-1, 4000).unwrap()); // -0.25 mm
        assert_eq!(a.fabrication.span.first, 1);
        assert_eq!(a.fabrication.span.last, count as u32);
        assert_eq!(
            a.fabrication.plating,
            FabricationPlating::SourceDeclaredPlated
        );
        assert_eq!(
            a.fabrication.shape,
            FabricationHoleShape::Round {
                diameter_m: Rational::new(1, 6250).unwrap() // DRILL_SIZE=160 um
            }
        );
        assert_eq!(a.hit.tool_number, 7);
        assert_eq!(a.hit.source_file, "steps/pcb/layers/hole/features");
        assert!(a.hit.source_line > 0 && a.hit.tool_source_line > 0);
        assert_ne!(a.hit.source_sha256, [0; 32]);
        assert_ne!(a.hit.tool_source_sha256, [0; 32]);
        assert_eq!(
            out.source_holes[1].fabrication.plating,
            FabricationPlating::NonPlated
        );
        assert_eq!(
            out.source_holes[2].fabrication.plating,
            FabricationPlating::Unknown
        );
        for h in &out.source_holes {
            assert!(!h.fabrication.has_verified_manufactured_barrel());
            assert!(!h.fabrication.is_electrical_conductor());
        }
    }
}

#[test]
fn missing_span_tool_and_unsupported_process_fail_closed() {
    let mut no_span = product(6);
    let matrix = String::from_utf8(no_span["matrix/matrix"].clone())
        .unwrap()
        .replace("START_NAME=sig1\nEND_NAME=sig6\n", "");
    no_span.insert("matrix/matrix".into(), matrix.into_bytes());
    assert_eq!(
        import_unpacked_with_holes(&no_span, "pcb", false)
            .unwrap_err()
            .code,
        "ODB_HOLE_UNQUALIFIED_SPAN"
    );

    let mut backdrill = product(6);
    let matrix = String::from_utf8(backdrill["matrix/matrix"].clone())
        .unwrap()
        .replace(
            "NAME=hole\nPOLARITY",
            "NAME=hole\nADD_TYPE=BACKDRILL\nPOLARITY",
        );
    backdrill.insert("matrix/matrix".into(), matrix.into_bytes());
    assert_eq!(
        import_unpacked_with_holes(&backdrill, "pcb", false)
            .unwrap_err()
            .code,
        "ODB_HOLE_SPECIAL_OPERATION_UNSUPPORTED"
    );

    let mut bad_tool = product(6);
    let p = "steps/pcb/layers/hole/features";
    let text = String::from_utf8(bad_tool[p].clone())
        .unwrap()
        .replace("P 7 0;ID=h1", "P 77 0;ID=h1");
    bad_tool.insert(p.into(), text.into_bytes());
    assert_eq!(
        import_unpacked_with_holes(&bad_tool, "pcb", false)
            .unwrap_err()
            .code,
        "ODB_DRILL_HIT_UNKNOWN_TOOL"
    );

    let mut missing_diameter = product(6);
    let p = "steps/pcb/layers/hole/tools";
    let text = String::from_utf8(missing_diameter[p].clone())
        .unwrap()
        .replace("FINISH_SIZE=200", "FINISH_SIZE=-1");
    missing_diameter.insert(p.into(), text.into_bytes());
    assert_eq!(
        import_unpacked_with_holes(&missing_diameter, "pcb", false)
            .unwrap_err()
            .code,
        "ODB_HOLE_MISSING_DIAMETER"
    );
}

#[test]
fn optional_hole_import_does_not_change_default_odb_geometry_contract() {
    let sample = product(4);
    let original = boardsi_import::odb_source::import_unpacked(&sample, "pcb").unwrap();
    let enriched = import_unpacked_with_holes(&sample, "pcb", false).unwrap();
    assert_eq!(original, enriched.board);
    assert_eq!(original.traces.len(), 4);
}
