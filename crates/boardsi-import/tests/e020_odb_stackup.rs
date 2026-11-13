//! E_020 strictly source-backed stackup tests; synthetic source only.
use boardsi_import::odb_source::OdbFileMap;
use boardsi_import::odb_stackup::audit_unpacked_stackup;
use boardsi_model::Rational;

fn sample(count: usize) -> OdbFileMap {
    let mut files = OdbFileMap::new();
    let mut matrix = String::from("STEP {\nCOL=1\nNAME=pcb\n}\n");
    let mut layers = String::new();
    for i in 1..=count {
        matrix.push_str(&format!(
            "LAYER {{\nROW={i}\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig{i}\nPOLARITY=POSITIVE\n}}\n"
        ));
        files.insert(
            format!("steps/pcb/layers/sig{i}/features"),
            b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n".to_vec(),
        );
        layers.push_str(&format!(
            "<Layer LayerName=\"sig{i}\" LayerType=\"SIGNAL\"><SpecRef MaterialSpecName=\"STD\"><Material MaterialName=\"Cu\"/></SpecRef></Layer>"
        ));
        if i < count {
            layers.push_str(&format!(
                "<Layer LayerName=\"d{i}\" LayerType=\"DIELECTRIC\"><SpecRef MaterialSpecName=\"STD\"><Material MaterialName=\"Dielectric\" PropertyName=\"measured\"/></SpecRef></Layer>"
            ));
        }
    }
    files.insert("matrix/matrix".into(), matrix.into_bytes());
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <StackupFile Version=\"8.1\" DefaultUnits=\"MM\">\
        <EdaData CompanyName=\"Synthetic\">\
        <Specs><Spec SpecName=\"STD\">\
        <Material MaterialName=\"Cu\"><Conductor ConductorType=\"COPPER\"/>\
        <Default_Thickness Thickness=\"0.018\" Finished_Thickness=\"0.020\"/></Material>\
        <Material MaterialName=\"Dielectric\"><Dielectric DielectricType=\"UNDEFINED\">\
        <Properties PropertyName=\"measured\"><Property FrequencyVal=\"1\" Units=\"GHz\" \
        DielectricConstant_Dk=\"4.4\" LossTangent_Df=\"0.015\"/></Properties></Dielectric>\
        <Default_Thickness Thickness=\"0.100\"/></Material>\
        </Spec></Specs>\
        <Stackup StackupName=\"pcb\" StackupThickness=\"1\"><Group GroupName=\"Design\">\
        {layers}</Group></Stackup></EdaData></StackupFile>"
    );
    files.insert("matrix/stackup.xml".into(), xml.into_bytes());
    files
}
fn rewrite(files: &mut OdbFileMap, from: &str, to: &str) {
    let xml = String::from_utf8(files["matrix/stackup.xml"].clone()).unwrap();
    assert!(xml.contains(from));
    files.insert(
        "matrix/stackup.xml".into(),
        xml.replace(from, to).into_bytes(),
    );
}
#[test]
fn e020_exact_material_layer_order_and_source_only() {
    for count in [2, 4, 6, 8, 64] {
        let files = sample(count);
        let result = audit_unpacked_stackup(&files, "pcb").unwrap().unwrap();
        assert_eq!(result.layers.len(), count * 2 - 1);
        assert!(result.complete_z);
        assert_eq!(result.layers[0].layer_name, "sig1");
        assert_eq!(result.layers[0].z_top_m, Some(Rational::integer(0)));
        assert_eq!(
            result.layers[0].thickness_m,
            Some(Rational::new(9, 500_000).unwrap())
        );
        assert_eq!(
            result.layers[1].thickness_m,
            Some(Rational::new(1, 10_000).unwrap())
        );
        assert_eq!(
            result.layers[2].z_top_m,
            Some(Rational::new(59, 500_000).unwrap())
        );
        assert_eq!(result.layers[1].dielectric_properties[0].dk, Some(4.4));
        assert_eq!(result.layers[1].dielectric_properties[0].df, Some(0.015));
        assert_eq!(
            result.layers[1].dielectric_properties[0].frequency_hz,
            Some(1e9)
        );
        assert_ne!(result.matrix_sha256, [0; 32]);
        assert_ne!(result.xml_sha256, [0; 32]);
        let board = boardsi_import::odb_source::import_unpacked(&files, "pcb").unwrap();
        assert_eq!(result.source_identity, board.sources[0].id);
        assert!(board.materials.is_empty());
        assert!(board.drills.is_empty() && board.vias.is_empty());
        assert!(board.connectivity.nodes.is_empty() && board.connectivity.edges.is_empty());
        assert!(board.stackup.layers.iter().all(|l| l.z_m.value().is_none()));
    }
}
#[test]
fn e020_optional_xml_and_supplier_only_do_not_promote() {
    let mut files = sample(4);
    files.remove("matrix/stackup.xml");
    assert!(audit_unpacked_stackup(&files, "pcb").unwrap().is_none());
    files.insert(
        "matrix/stackup.xml".into(),
        b"<StackupFile Version=\"8.1\"><SupplierData CompanyName=\"Supplier\"/></StackupFile>"
            .to_vec(),
    );
    assert!(audit_unpacked_stackup(&files, "pcb").unwrap().is_none());
}
#[test]
fn e020_malformed_untrusted_xml_and_conflicts_fail_closed() {
    let cases = [
        (
            "<StackupFile",
            "<!DOCTYPE StackupFile [<!ENTITY x SYSTEM 'file:///etc/passwd'>]><StackupFile",
            "ODB_STACKUP_XML_EXTERNAL_OR_EXTENSION",
        ),
        (
            "</Group></Stackup>",
            "</Group><Group GroupName=\"Other\"/></Stackup>",
            "ODB_STACKUP_GROUP_AMBIGUOUS",
        ),
        (
            "LayerName=\"sig1\"",
            "LayerName=\"missing\"",
            "ODB_STACKUP_MATRIX_LAYER_MISMATCH",
        ),
        (
            "MaterialName=\"Cu\"/></SpecRef>",
            "MaterialName=\"NonExisting\"/></SpecRef>",
            "ODB_STACKUP_UNKNOWN_MATERIAL",
        ),
        (
            "Thickness=\"0.018\"",
            "Thickness=\"-0.018\"",
            "ODB_STACKUP_THICKNESS",
        ),
        (
            "LayerType=\"DIELECTRIC\"",
            "LayerType=\"SIGNAL\"",
            "ODB_STACKUP_MATERIAL_LAYER_CONFLICT",
        ),
        ("Version=\"8.1\"", "Version=\"7.0\"", "ODB_STACKUP_VERSION"),
        ("<StackupFile", "<NotStackupFile", "ODB_STACKUP_XML_CLOSE"),
    ];
    for (a, b, expected) in cases {
        let mut files = sample(4);
        rewrite(&mut files, a, b);
        let err = audit_unpacked_stackup(&files, "pcb").unwrap_err();
        assert_eq!(err.code, expected, "{a} => {b}");
    }
}
#[test]
fn e020_duplicate_attribute_and_unclosed_xml_are_rejected() {
    let mut files = sample(2);
    rewrite(
        &mut files,
        "DefaultUnits=\"MM\"",
        "DefaultUnits=\"MM\" DefaultUnits=\"MIL\"",
    );
    assert_eq!(
        audit_unpacked_stackup(&files, "pcb").unwrap_err().code,
        "ODB_STACKUP_DUPLICATE_ATTRIBUTE"
    );
    let mut files = sample(2);
    let mut xml = files.remove("matrix/stackup.xml").unwrap();
    xml.truncate(xml.len() - "</StackupFile>".len());
    files.insert("matrix/stackup.xml".into(), xml);
    assert!(audit_unpacked_stackup(&files, "pcb").is_err());
}
#[test]
fn e020_mutated_source_changes_identity_but_never_electrical_graph() {
    let a = sample(4);
    let mut b = a.clone();
    rewrite(&mut b, "Thickness=\"0.100\"", "Thickness=\"0.101\"");
    let first = audit_unpacked_stackup(&a, "pcb").unwrap().unwrap();
    let second = audit_unpacked_stackup(&b, "pcb").unwrap().unwrap();
    assert_ne!(first.source_identity, second.source_identity);
    assert_ne!(first.xml_sha256, second.xml_sha256);
    assert_ne!(first.layers[2].z_top_m, second.layers[2].z_top_m);
    let board = boardsi_import::odb_source::import_unpacked(&b, "pcb").unwrap();
    assert!(board.connectivity.edges.is_empty());
}
#[test]
fn e020_precise_mil_unit_and_precision_failure() {
    let mut files = sample(2);
    rewrite(&mut files, "DefaultUnits=\"MM\"", "DefaultUnits=\"MIL\"");
    rewrite(&mut files, "Thickness=\"0.018\"", "Thickness=\"1\"");
    let result = audit_unpacked_stackup(&files, "pcb").unwrap().unwrap();
    assert_eq!(
        result.layers[0].thickness_m,
        Some(Rational::new(127, 5_000_000).unwrap())
    );
    let mut files = sample(2);
    rewrite(
        &mut files,
        "Thickness=\"0.018\"",
        "Thickness=\"0.00000000001\"",
    );
    assert_eq!(
        audit_unpacked_stackup(&files, "pcb").unwrap_err().code,
        "ODB_STACKUP_PRECISION"
    );
}

#[test]
fn e020_rejects_multi_frequency_thickness_override_and_duplicate_frequency() {
    let mut files = sample(2);
    rewrite(
        &mut files,
        "DielectricConstant_Dk=\"4.4\" LossTangent_Df=\"0.015\"/>",
        "DielectricConstant_Dk=\"4.4\" LossTangent_Df=\"0.015\" Thickness=\"0.100\"/><Property FrequencyVal=\"2\" Units=\"GHz\" DielectricConstant_Dk=\"4.2\"/>",
    );
    assert_eq!(
        audit_unpacked_stackup(&files, "pcb").unwrap_err().code,
        "ODB_STACKUP_AMBIGUOUS_THICKNESS"
    );
    let mut files = sample(2);
    rewrite(
        &mut files,
        "LossTangent_Df=\"0.015\"/>",
        "LossTangent_Df=\"0.015\"/><Property FrequencyVal=\"1\" Units=\"GHz\" DielectricConstant_Dk=\"4.1\"/>",
    );
    assert_eq!(
        audit_unpacked_stackup(&files, "pcb").unwrap_err().code,
        "ODB_STACKUP_DUPLICATE_FREQUENCY"
    );
}
#[test]
fn e020_rejects_invalid_target_and_xml_resource_budget() {
    let mut files = sample(2);
    rewrite(
        &mut files,
        "StackupThickness=\"1\"",
        "StackupThickness=\"-1\"",
    );
    assert_eq!(
        audit_unpacked_stackup(&files, "pcb").unwrap_err().code,
        "ODB_STACKUP_THICKNESS"
    );
    let mut files = sample(2);
    files.insert("matrix/stackup.xml".into(), vec![b' '; 2 * 1024 * 1024 + 1]);
    assert_eq!(
        audit_unpacked_stackup(&files, "pcb").unwrap_err().code,
        "ODB_STACKUP_XML_TOO_LARGE"
    );
}

#[test]
fn e020_nested_group_must_not_hide_dielectric_from_complete_z() {
    let mut files = sample(2);
    rewrite(
        &mut files,
        "<Layer LayerName=\"d1\"",
        "<Group GroupName=\"Hidden\"><Layer LayerName=\"d1\"",
    );
    rewrite(
        &mut files,
        "PropertyName=\"measured\"/></SpecRef></Layer><Layer LayerName=\"sig2\"",
        "PropertyName=\"measured\"/></SpecRef></Layer></Group><Layer LayerName=\"sig2\"",
    );
    assert_eq!(
        audit_unpacked_stackup(&files, "pcb").unwrap_err().code,
        "ODB_STACKUP_NESTED_OR_UNKNOWN_GROUP_CHILD"
    );
}

#[test]
fn e020_xml_matrix_copper_type_conflict_is_not_accepted() {
    let mut files = sample(4);
    rewrite(
        &mut files,
        "LayerName=\"sig1\" LayerType=\"SIGNAL\"",
        "LayerName=\"sig1\" LayerType=\"POWER_GROUND\"",
    );
    assert_eq!(
        audit_unpacked_stackup(&files, "pcb").unwrap_err().code,
        "ODB_STACKUP_MATRIX_LAYER_TYPE_MISMATCH"
    );
}
