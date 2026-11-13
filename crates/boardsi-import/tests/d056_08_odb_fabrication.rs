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

#[test]
fn e019_plated_candidate_exact_source_and_no_electrical_promotion() {
    for count in [2, 4, 6, 8, 64] {
        let fixture = product(count);
        let a = import_unpacked_with_holes(&fixture, "pcb", false).unwrap();
        let b = import_unpacked_with_holes(&fixture, "pcb", false).unwrap();
        assert_eq!(a.via_candidates.len(), 1);
        let c = &a.via_candidates[0];
        assert_eq!(c.source_identity, a.board.sources[0].id);
        assert_eq!(c.source_identity, b.via_candidates[0].source_identity);
        assert_eq!(c.candidate_key, b.via_candidates[0].candidate_key);
        assert_eq!(
            c.xy_m,
            [
                Rational::new(1, 800).unwrap(),
                Rational::new(-1, 4000).unwrap()
            ]
        );
        assert_eq!(c.finished_diameter_m, Rational::new(7, 50_000).unwrap());
        assert_eq!(c.tool_diameter_m, Some(Rational::new(1, 6_250).unwrap()));
        assert_eq!(c.fabrication.span.first, 1);
        assert_eq!(c.fabrication.span.last, count as u32);
        assert_eq!(c.source_observation.physical_envelope_layers.len(), count);
        assert_eq!(
            c.source_observation.via_evidence,
            boardsi_model::SourceViaEvidenceState::SourceDeclaredPlated
        );
        assert_eq!(
            c.source_observation.stackup_evidence,
            boardsi_model::SourceStackupEvidence::Missing
        );
        assert_eq!(c.source_observation.source_file, c.hit.source_file);
        assert_eq!(c.source_observation.source_line, c.hit.source_line);
        assert!(c.hit.tool_source_line > 0);
        assert_ne!(c.hit.matrix_source_sha256, [0; 32]);
        assert!(!c.has_verified_barrel() && !c.is_solver_ready());
        assert!(a.board.vias.is_empty() && a.board.drills.is_empty());
        assert!(a.board.connectivity.edges.is_empty());
        assert_eq!(a.source_holes.len(), 3);
        assert_eq!(
            a.source_holes[1].fabrication.plating,
            FabricationPlating::NonPlated
        );
        assert_eq!(
            a.source_holes[2].fabrication.plating,
            FabricationPlating::Unknown
        );
    }
}

#[test]
fn e019_partial_span_and_changed_provenance_are_not_guessed() {
    let mut partial = product(6);
    let matrix = String::from_utf8(partial["matrix/matrix"].clone())
        .unwrap()
        .replace(
            "START_NAME=sig1\nEND_NAME=sig6",
            "START_NAME=sig2\nEND_NAME=sig5",
        );
    partial.insert("matrix/matrix".into(), matrix.into_bytes());
    let a = import_unpacked_with_holes(&partial, "pcb", false).unwrap();
    assert_eq!(a.via_candidates.len(), 1);
    let c = &a.via_candidates[0];
    assert_eq!((c.fabrication.span.first, c.fabrication.span.last), (2, 5));
    assert_eq!(c.source_observation.physical_envelope_layers.len(), 4);
    assert!(a.board.connectivity.edges.is_empty());

    let mut changed = partial.clone();
    changed
        .get_mut("steps/pcb/layers/hole/features")
        .unwrap()
        .extend_from_slice(b"# sha mutation\n");
    let b = import_unpacked_with_holes(&changed, "pcb", false).unwrap();
    assert_ne!(
        a.via_candidates[0].source_identity,
        b.via_candidates[0].source_identity
    );
    assert!(b.board.connectivity.edges.is_empty());
}

#[test]
fn e019_unqualified_process_or_finish_diameter_never_promoted() {
    let mut missing_finish = product(4);
    let path = "steps/pcb/layers/hole/tools";
    let t = String::from_utf8(missing_finish[path].clone())
        .unwrap()
        .replace("FINISH_SIZE=140", "FINISH_SIZE=-1");
    missing_finish.insert(path.into(), t.into_bytes());
    let out = import_unpacked_with_holes(&missing_finish, "pcb", false).unwrap();
    assert_eq!(out.source_holes.len(), 3);
    assert_eq!(out.via_candidates.len(), 0);
    assert!(out.board.vias.is_empty());

    let mut press_fit = product(4);
    let t = String::from_utf8(press_fit[path].clone()).unwrap().replace(
        "TYPE2=STANDARD\nFINISH_SIZE=140",
        "TYPE2=PRESS_FIT\nFINISH_SIZE=140",
    );
    press_fit.insert(path.into(), t.into_bytes());
    let out = import_unpacked_with_holes(&press_fit, "pcb", false).unwrap();
    assert_eq!(out.source_holes.len(), 3);
    assert!(out.via_candidates.is_empty());

    let mut no_plating_proof = product(4);
    let t = String::from_utf8(no_plating_proof[path].clone())
        .unwrap()
        .replace("TYPE=PLATED\nTYPE2=STANDARD", "TYPE=VIA\nTYPE2=STANDARD");
    no_plating_proof.insert(path.into(), t.into_bytes());
    let out = import_unpacked_with_holes(&no_plating_proof, "pcb", false).unwrap();
    assert!(out.via_candidates.is_empty());
    assert!(out.board.connectivity.edges.is_empty());
}

#[test]
fn e019_duplicate_content_different_source_paths_is_unambiguous() {
    let mut files = product(4);
    let real_path = "steps/pcb/layers/hole/features";
    files.insert(
        "steps/unused/layers/hole/features".into(),
        files[real_path].clone(),
    );
    let imported = import_unpacked_with_holes(&files, "pcb", false).unwrap();
    assert_eq!(imported.via_candidates.len(), 1);
    let candidate = &imported.via_candidates[0];
    assert_eq!(candidate.source_observation.source_file, real_path);
    // The old SHA-only, single-hole observer remains fail-closed on ambiguity.
    assert!(imported
        .board
        .observe_source_via_hole(&candidate.source_identity, &candidate.fabrication,)
        .is_err());
    assert!(imported.board.connectivity.edges.is_empty());
}

#[test]
fn e019_batch_multiple_hits_remains_source_only_and_order_stable() {
    let mut files = product(4);
    let path = "steps/pcb/layers/hole/features";
    let mut data = String::from_utf8(files[path].clone())
        .unwrap()
        .replace("F 3", "F 259");
    for n in 0..256 {
        data.push_str(&format!("P {} 0 0 P 7 0;ID=more{}\n", n + 5, n));
    }
    files.insert(path.into(), data.into_bytes());
    let a = import_unpacked_with_holes(&files, "pcb", false).unwrap();
    let b = import_unpacked_with_holes(&files, "pcb", false).unwrap();
    assert_eq!(a.source_holes.len(), 259);
    assert_eq!(a.via_candidates.len(), 257);
    assert_eq!(
        a.via_candidates
            .iter()
            .map(|c| c.candidate_key.as_str())
            .collect::<Vec<_>>(),
        b.via_candidates
            .iter()
            .map(|c| c.candidate_key.as_str())
            .collect::<Vec<_>>(),
    );
    assert!(a.board.vias.is_empty());
    assert!(a.board.drills.is_empty());
    assert!(a.board.connectivity.edges.is_empty());
}
