//! Public synthetic-only C_003 identity/freshness regression.
use boardsi_import::scenario_contract::{
    classify_result, ReceiptError, ResultFreshness, ResultScope, ScenarioDraft, ScenarioError, SCHEMA,
};

fn draft() -> ScenarioDraft {
    ScenarioDraft {
        source_sha256: "a".repeat(64),
        canonical_model_sha256: "b".repeat(64),
        geometry_revision: "public-source-geometry-v1".into(),
        stackup_revision: "layer-stack-1".into(),
        material_provenance_revision: "assumed-dk-df-source-1".into(),
        signal_net: "SIG".into(),
        reference_net: "GND".into(),
        port_load_revision: "ports-and-load-v1".into(),
        group_coupling_revision: "group-v1".into(),
        solver_model_revision: "unsupported-model-v1".into(),
        engine_version: "engine-v0".into(),
        backend_precision: "cpu-f64".into(),
        frequencies_hz: vec![0, 1_000_000_000, 2_000_000_000],
    }
}

#[test]
fn deterministic_snapshot_and_separate_run_identity() {
    let a = draft().freeze().unwrap();
    let b = draft().freeze().unwrap();
    assert_eq!(a, b);
    assert_eq!(a.snapshot_sha256().len(), 64);
    assert_ne!(a.snapshot_sha256(), draft().source_sha256);
    assert!(!a.solver_ready());
    assert!(!a.cache_reuse_eligible());
    let one = ResultScope::new("run-one", a.clone()).unwrap();
    let two = ResultScope::new("run-two", a).unwrap();
    assert_eq!(
        one.snapshot().snapshot_sha256(),
        two.snapshot().snapshot_sha256()
    );
    assert_ne!(
        one.synthetic_receipt().unwrap().run_id,
        two.synthetic_receipt().unwrap().run_id
    );
}

#[test]
fn all_physical_provenance_changes_invalidate_content() {
    let before = draft().freeze().unwrap();
    let mut cases = Vec::new();
    let mut value = draft();
    value.source_sha256 = "c".repeat(64);
    cases.push(value);
    let mut value = draft();
    value.canonical_model_sha256 = "d".repeat(64);
    cases.push(value);
    let mut value = draft();
    value.geometry_revision = "changed-copper-cutout-island-or-overlay".into();
    cases.push(value);
    let mut value = draft();
    value.stackup_revision = "new-thickness".into();
    cases.push(value);
    let mut value = draft();
    value.material_provenance_revision = "new-dk-df-provenance".into();
    cases.push(value);
    let mut value = draft();
    value.signal_net = "NEW_SIG".into();
    cases.push(value);
    let mut value = draft();
    value.reference_net = "NEW_GND".into();
    cases.push(value);
    let mut value = draft();
    value.port_load_revision = "changed-port-load".into();
    cases.push(value);
    let mut value = draft();
    value.group_coupling_revision = "changed-group".into();
    cases.push(value);
    for changed in cases {
        let after = changed.freeze().unwrap();
        assert_ne!(before.conditions_sha256(), after.conditions_sha256());
        assert_ne!(before.snapshot_sha256(), after.snapshot_sha256());
    }
}

#[test]
fn sweep_rejects_previous_completion_but_preserves_condition_identity() {
    let before = draft().freeze().unwrap();
    let mut changed = draft();
    changed.frequencies_hz = vec![0, 500_000_000, 1_000_000_000];
    let after = changed.freeze().unwrap();
    assert_eq!(before.conditions_sha256(), after.conditions_sha256());
    assert_eq!(before.compatibility_sha256(), after.compatibility_sha256());
    assert_ne!(before.snapshot_sha256(), after.snapshot_sha256());
    let receipt = ResultScope::new("run", before)
        .unwrap()
        .synthetic_receipt()
        .unwrap();
    assert_eq!(receipt.freshness(&after), ResultFreshness::OutOfDate);
    assert!(!after.cache_reuse_eligible());
}

#[test]
fn ui_only_theme_zoom_changes_have_no_physics_input() {
    // UI-only values deliberately do not exist in the contract.
    let before = draft().freeze().unwrap();
    let theme = "dark";
    let zoom = 1.5;
    assert_eq!(theme, "dark");
    assert_eq!(zoom, 1.5);
    assert_eq!(before, draft().freeze().unwrap());
}

#[test]
fn engine_model_and_precision_are_incompatible() {
    let a = draft().freeze().unwrap();
    let receipt = ResultScope::new("run", a)
        .unwrap()
        .synthetic_receipt()
        .unwrap();
    let mut changes = Vec::new();
    let mut item = draft();
    item.engine_version = "engine-new".into();
    changes.push(item);
    let mut item = draft();
    item.solver_model_revision = "model-new".into();
    changes.push(item);
    let mut item = draft();
    item.backend_precision = "other-precision".into();
    changes.push(item);
    for changed in changes {
        let new = changed.freeze().unwrap();
        assert_eq!(receipt.freshness(&new), ResultFreshness::Incompatible);
    }
}

#[test]
fn freeze_clones_values_and_edit_revert_restores_identity() {
    let mut editable = draft();
    let frozen = editable.freeze().unwrap();
    assert_eq!(frozen.input().material_provenance_revision, "assumed-dk-df-source-1");
    editable.material_provenance_revision = "changed".into();
    assert_eq!(frozen.input().material_provenance_revision, "assumed-dk-df-source-1");
    assert_ne!(frozen, editable.freeze().unwrap());
    editable.material_provenance_revision = "assumed-dk-df-source-1".into();
    assert_eq!(frozen, editable.freeze().unwrap());
}

#[test]
fn reject_missing_fields_bad_digests_and_reference_alias() {
    let mut item = draft();
    item.source_sha256 = "too-short".into();
    assert_eq!(item.freeze().unwrap_err(), ScenarioError::InvalidDigest);
    let mut item = draft();
    item.canonical_model_sha256 = "B".repeat(64);
    assert_eq!(item.freeze().unwrap_err(), ScenarioError::InvalidDigest);
    let mut item = draft();
    item.port_load_revision = " ".into();
    assert_eq!(item.freeze().unwrap_err(), ScenarioError::MissingInput);
    let mut item = draft();
    item.geometry_revision = "a\0b".into();
    assert_eq!(item.freeze().unwrap_err(), ScenarioError::MissingInput);
    let mut item = draft();
    item.material_provenance_revision = "X".repeat(513);
    assert_eq!(item.freeze().unwrap_err(), ScenarioError::OversizedInput);
    let mut item = draft();
    item.reference_net = "SIG".into();
    assert_eq!(
        item.freeze().unwrap_err(),
        ScenarioError::SameSignalReference
    );
}

#[test]
fn invalid_frequency_grids_fail_closed() {
    for frequencies in [
        vec![],
        vec![1, 1],
        vec![2, 1],
        vec![30_000_000_001],
        vec![0; 4097],
    ] {
        let mut item = draft();
        item.frequencies_hz = frequencies;
        assert_eq!(
            item.freeze().unwrap_err(),
            ScenarioError::InvalidFrequencyGrid
        );
    }
}

#[test]
fn never_advance_on_wrong_run_hash_sequence_or_frequency() {
    let frozen = draft().freeze().unwrap();
    let mut scope = ResultScope::new("run-a", frozen.clone()).unwrap();
    let good = scope.synthetic_receipt().unwrap();
    let mut bad = good.clone();
    bad.run_id = "old-run".into();
    assert_eq!(scope.accept(&bad), Err(ReceiptError::WrongRun));
    let mut bad = good.clone();
    bad.snapshot_sha256 = "f".repeat(64);
    assert_eq!(scope.accept(&bad), Err(ReceiptError::WrongSnapshot));
    let mut bad = good.clone();
    bad.schema = "wrong-version".into();
    assert_eq!(scope.accept(&bad), Err(ReceiptError::WrongSchema));
    let mut bad = good.clone();
    bad.synthetic = false;
    assert_eq!(scope.accept(&bad), Err(ReceiptError::NonSynthetic));
    let mut bad = good.clone();
    bad.sequence = 2;
    assert_eq!(scope.accept(&bad), Err(ReceiptError::InvalidSequence));
    let mut bad = good.clone();
    bad.frequency_hz = 100;
    assert_eq!(scope.accept(&bad), Err(ReceiptError::InvalidFrequency));
    assert_eq!(scope.accepted(), 0);
    assert_eq!(scope.accept(&good), Ok(()));
    assert_eq!(scope.accepted(), 1);
    assert_eq!(scope.accept(&good), Err(ReceiptError::InvalidSequence));
    let second = scope.synthetic_receipt().unwrap();
    assert_eq!(scope.accept(&second), Ok(()));
    let third = scope.synthetic_receipt().unwrap();
    assert_eq!(scope.accept(&third), Ok(()));
    assert!(scope.synthetic_receipt().is_none());
    assert_eq!(scope.accept(&third), Err(ReceiptError::InvalidSequence));
}

#[test]
fn current_and_rejected_states_do_not_claim_si_accuracy() {
    let current = draft().freeze().unwrap();
    let mut receipt = ResultScope::new("run", current.clone())
        .unwrap()
        .synthetic_receipt()
        .unwrap();
    assert_eq!(receipt.schema, SCHEMA);
    assert_eq!(receipt.freshness(&current), ResultFreshness::Current);
    receipt.snapshot_sha256 = "bad".into();
    assert_eq!(receipt.freshness(&current), ResultFreshness::Rejected);
    receipt.snapshot_sha256 = current.snapshot_sha256().into();
    receipt.frequency_hz = 123;
    assert_eq!(receipt.freshness(&current), ResultFreshness::Rejected);
}

#[test]
fn invalid_run_ids_fail() {
    let scenario = draft().freeze().unwrap();
    assert!(ResultScope::new("", scenario.clone()).is_err());
    assert!(ResultScope::new("evil/run", scenario.clone()).is_err());
    assert!(ResultScope::new(&"x".repeat(129), scenario).is_err());
}

#[test]
fn missing_result_has_distinct_not_run_state() {
    let current = draft().freeze().unwrap();
    assert_eq!(classify_result(None, &current), ResultFreshness::NotRun);
    let receipt = ResultScope::new("run", current.clone())
        .unwrap()
        .synthetic_receipt()
        .unwrap();
    assert_eq!(
        classify_result(Some(&receipt), &current),
        ResultFreshness::Current
    );
}
