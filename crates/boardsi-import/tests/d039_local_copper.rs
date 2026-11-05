//! D_039 exact source-owned local copper geometry; never global electrical proof.
use boardsi_import::{d001_pad_endpoint, d039_local_copper};

const SOURCE: &str = include_str!("fixtures/d039_local_copper_v94.asc");

fn qualify(source: &str) -> Result<String, &'static str> {
    d039_local_copper::encode(source.as_bytes(), "NET_A", 1)
}

#[test]
fn two_pad_two_trace_source_bounded_local_copper_graph() {
    let wire = qualify(SOURCE).expect("supported local copper fixture");
    assert!(wire.starts_with("BOARDSI_D039_LOCAL_COPPER_V1|"));
    assert!(wire.contains(
        "|NET_A|1|LOCAL_COPPER_GEOMETRY_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    ));
    assert!(wire.contains("P|0|U1|1|1|20000|30000|4000|4000|R\n"));
    assert!(wire.contains("P|1|U1|2|1|20000|40000|3000|3000|S\n"));
    assert!(wire.contains("T|0|1|20000|30000|20000|35000|1000\n"));
    assert!(wire.contains("T|1|1|20000|40000|20000|35000|1000\n"));
    assert_eq!(wire.matches("\nP|").count(), 2);
    assert_eq!(wire.matches("\nT|").count(), 2);
    assert_eq!(wire.matches("\nE|").count(), 3);
    assert!(
        wire.contains("COUNT|NODES|4|EDGES|3|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED")
    );
    assert!(wire.ends_with("END|LOCAL_COPPER_GEOMETRY_ONLY_NO_GLOBAL_PASS\n"));
    assert_eq!(
        d001_pad_endpoint::audit(SOURCE.as_bytes())
            .unwrap()
            .witnesses
            .len(),
        2
    );
}

#[test]
fn supported_v94_mils_headers_are_source_bound_and_not_electrical_proof() {
    let original = SOURCE.replace("\r\n", "\n");
    for variant in [
        "!PADS-POWERPCB-V9.4-MILS-CP1252!",
        "!PADS-POWERPCB-V9.4-MILS-250L!",
        "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!",
    ] {
        let modified = original.replace("!PADS-POWERPCB-V9.4-MILS!", variant);
        assert_ne!(modified, original);
        let wire = qualify(&modified).expect("supported source-unit header");
        assert!(wire.contains("|LOCAL_COPPER_GEOMETRY_VERIFIED|"));
        assert!(wire.contains("|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"));
    }
    let tabbed = original.replace("!PADS-POWERPCB-V9.4-MILS! DESIGN", "!PADS-POWERPCB-V9.4-MILS!\tDESIGN");
    assert_ne!(tabbed, original);
    assert!(qualify(&tabbed).is_ok());
}

#[test]
fn source_trace_record_reversal_keeps_local_topology_but_changes_sha() {
    let source = SOURCE.replace("\r\n", "\n");
    let reversed = source
        .replace(
            "200 300 1 10 0\n200 350 1 10 0",
            "200 350 1 10 0\n200 300 1 10 0",
        )
        .replace(
            "200 400 1 10 0\n200 350 1 10 0",
            "200 350 1 10 0\n200 400 1 10 0",
        );
    assert_ne!(reversed, source);
    let first = qualify(SOURCE).unwrap();
    let second = qualify(&reversed).unwrap();
    assert_ne!(first.lines().next(), second.lines().next());
    assert_eq!(
        first.split_once('\n').unwrap().1,
        second.split_once('\n').unwrap().1
    );
    assert_eq!(qualify(SOURCE).unwrap(), qualify(SOURCE).unwrap());
    let lexical = source.replace("T0 0 0 0 1\n", "T0 0 0 0 1  \n");
    assert_ne!(lexical, SOURCE);
    assert_ne!(qualify(&lexical).unwrap(), first);
}

#[test]
fn one_tick_gap_and_false_copper_junction_fail_closed() {
    let source = SOURCE.replace("\r\n", "\n");
    let cases = [
        source.replace(
            "200 400 1 10 0\n200 350 1 10 0",
            "200 400 1 10 0\n200 350.01 1 10 0",
        ),
        source.replace(
            "200 400 1 10 0\n200 350 1 10 0",
            "200 400 1 10 0\n200 349.99 1 10 0",
        ),
        source.replace(
            "200 400 1 10 0\n200 350 1 10 0",
            "200 400 1 10 0\n201 350 1 10 0",
        ),
        source.replace("200 350 1 10 0", "200 450 1 10 0"),
        source.replace("200 400 1 10 0", "200 300 1 10 0"),
        source.replace("200 400 1 10 0", "200 400 2 10 0"),
        source.replace(
            "200 400 1 10 0\n200 350 1 10 0",
            "200 400 1 11 0\n200 350 1 11 0",
        ),
    ];
    for case in cases {
        assert_ne!(case, source);
        assert!(qualify(&case).is_err(), "hostile geometry accepted");
    }
}

#[test]
fn wrong_net_pad_ownership_and_source_inventory_fail_closed() {
    let source = SOURCE.replace("\r\n", "\n");
    let cases = [
        source.replace("*SIGNAL* NET_A", "*SIGNAL* NET_B"),
        source.replace("U1 TYPEA 200 300", "U1 TYPEA 200 301"),
        source.replace("-2 40 R 0", "-2 0 R 0"),
        source.replace("-2 30 S 0", "-2 30 RF 0 60 0 0"),
        source.replace(
            "*PART*\nU1 TYPEA 200 300 90 U N 0 -1 0 -1 0",
            "*PART*\nU1 TYPEA 200 300 90 U N 0 -1 0 -1 0\nU1 TYPEA 200 300 90 U N 0 -1 0 -1 0",
        ),
        source.replace("*ROUTE*", "*LINES*\nRA COPPER 0 0 1 1 0 NET_A\n*ROUTE*"),
        source.replace("*ROUTE*", "*VIA*\nSIMPLEVIA 10 2\n*ROUTE*"),
        source.replace("*END*\n", "*END*\n*SIGNAL* NET_A\n"),
        source.replace("*SIGNAL* NET_A", "*SIGNAL* NET_A\n*SIGNAL* NET_A"),
        source.replace(
            "*PARTTYPE*",
            "D2 I 1000 1000 0 2 2 0 0\nT1000 0 0 0 1\nT100 0 0 0 2\nPAD 1 3\n-2 40 R 0\n-1 0 R 0\n0 0 R 0\nPAD 2 3\n-2 30 S 0\n-1 0 R 0\n0 0 R 0\n*PARTTYPE*",
        ),
        source.replace("*PARTTYPE*\nTYPEA D2 UND 0 0 0 0", "*PARTTYPE*\nTYPEA D2 UND 0 0 0 0\nTYPEA D2 UND 0 0 0 0"),
        source.replace("*PARTDECAL*\n", "*PARTDECAL*\nUNKNOWN I 1000 1000 0 2 2 0 0\n"),
    ];
    for case in cases {
        assert_ne!(case, source);
        assert!(qualify(&case).is_err(), "unqualified source accepted");
    }
    assert!(d039_local_copper::encode(SOURCE.as_bytes(), "NET_B", 1).is_err());
    assert!(d039_local_copper::encode(SOURCE.as_bytes(), "NET_A", 4).is_err());
}
