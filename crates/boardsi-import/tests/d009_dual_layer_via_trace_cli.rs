#[allow(dead_code)]
#[path = "../src/bin/boardsi-d009-dual-layer-via-trace.rs"]
mod d009;
const SOURCE: &[u8] = include_bytes!("fixtures/d009_dual_layer_via_trace_v94.asc");
fn swap(from: &str, to: &str) -> Vec<u8> {
    let normalized = String::from_utf8(SOURCE.to_vec()).unwrap().replace("\r\n", "\n");
    assert!(normalized.contains(from), "missing fixture anchor");
    normalized.replacen(from, to, 1).into_bytes()
}
#[test]
fn source_only_dual_layer_third_area_chain() {
    let wire = d009::encode(SOURCE, ["RA","RB","RC"],"NET_A",[1,4],0).unwrap();
    assert!(wire.contains("OVERALL|SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE"), "{wire}");
    assert_eq!(wire.matches("CONTACT|").count(), 3);
    assert!(wire.contains("LINK|SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE|0|1|1|2|1"));
    assert!(wire.contains("PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0"));
}
#[test]
fn wrong_net_layer_pad_or_route_fails_closed() {
    assert!(d009::encode(SOURCE,["RA","RB","RC"],"OTHER",[1,4],0).is_err());
    assert!(d009::encode(SOURCE,["RA","RB","RC"],"NET_A",[4,1],0).is_err());
    assert!(d009::encode(SOURCE,["RA","RB","RC"],"NET_A",[1,2],0).is_err());
    let missing = swap("0 35 R","-1 35 R");
    assert!(d009::encode(&missing,["RA","RB","RC"],"NET_A",[1,4],0).is_err());
    let wrong = swap("330 280 4 10 0","330 280 1 10 0");
    assert!(d009::encode(&wrong,["RA","RB","RC"],"NET_A",[1,4],0).is_err());
}
#[test]
fn third_area_endpoint_boundary_rejected() {
    let changed=swap("330 280 4 10 0","280 280 4 10 0");
    let wire=d009::encode(&changed,["RA","RB","RC"],"NET_A",[1,4],0).unwrap();
    assert!(wire.contains("OVERALL|SOURCE_DUAL_LAYER_TRACE_NOT_VERIFIED"), "{wire}");
    assert!(!wire.contains("LINK|SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE"));
}
#[test]
fn third_area_owned_cutout_blocks_endpoint() {
    let raw=String::from_utf8(SOURCE.to_vec()).unwrap().replace("\r\n","\n");
    let mutated=raw.replacen("RC COPPER 0 0 1 1 0 NET_A","RC COPPER 0 0 2 1 0 NET_A",1)
      .replacen("*ROUTE*","COPCUT 5 0.01 0 4\n320 270\n340 270\n340 290\n320 290\n320 270\n*ROUTE*",1);
    let wire=d009::encode(mutated.as_bytes(),["RA","RB","RC"],"NET_A",[1,4],0).unwrap();
    assert!(wire.contains("LINK|SOURCE_COPCUT_RISK"), "{wire}");
    assert!(wire.contains("OVERALL|SOURCE_DUAL_LAYER_TRACE_NOT_VERIFIED"));
}
#[test]
fn via_area_boundary_prevents_candidate() {
    let changed=swap("100 200\n200 200\n200 300\n100 300\n100 200","135 200\n235 200\n235 300\n135 300\n135 200");
    let wire=d009::encode(&changed,["RA","RB","RC"],"NET_A",[1,4],0).unwrap();
    assert!(wire.contains("OVERALL|SOURCE_DUAL_LAYER_TRACE_NOT_VERIFIED"));
}
