#[allow(dead_code)]
#[path = "../src/bin/boardsi-d011-source-contact-graph.rs"]
mod d011;
const SOURCE: &[u8] = include_bytes!("fixtures/d010_two_trace_four_area_v94.asc");
fn graph(source: &[u8]) -> Result<String, &'static str> {
    d011::encode(source, ["RA", "RB", "RC", "RD"], "NET_A", [1, 4], 0)
}
#[test]
fn eight_nodes_six_edges_two_components_no_barrel() {
    let w = graph(SOURCE).unwrap();
    assert_eq!(w.lines().filter(|s| s.starts_with("NODE|")).count(), 8);
    assert_eq!(w.lines().filter(|s| s.starts_with("EDGE|")).count(), 6);
    assert!(w.contains("|SI_NOT_RUN|8|6|2|0\n"));
    assert!(w.contains("EDGE|2|2|3|SOURCE_CONTACT_CANDIDATE|1"));
    assert!(w.contains("EDGE|5|6|7|SOURCE_CONTACT_CANDIDATE|4"));
    assert!(w.ends_with("END|SOURCE_ONLY_NO_BARREL_EDGE\n"));
    assert_eq!(w, graph(SOURCE).unwrap());
}
#[test]
fn source_sha_changes_without_changing_topology() {
    let data=String::from_utf8(SOURCE.to_vec()).unwrap()
        .replace("U1.1 U2.2", "U1.1  U2.2");
    let fresh=graph(data.as_bytes()).unwrap();
    let original=graph(SOURCE).unwrap();
    assert_ne!(fresh, original);
    assert_eq!(fresh.lines().filter(|s|s.starts_with("EDGE|")).collect::<Vec<_>>(),
               original.lines().filter(|s|s.starts_with("EDGE|")).collect::<Vec<_>>());
}
#[test]
fn reject_boundary_wrong_layer_and_net() {
    let text=String::from_utf8(SOURCE.to_vec()).unwrap();
    let boundary=text.replacen("330 280 4 10 0","280 280 4 10 0",1);
    assert!(graph(boundary.as_bytes()).is_err());
    assert!(d011::encode(SOURCE,["RA","RB","RC","RD"],"OTHER",[1,4],0).is_err());
    assert!(d011::encode(SOURCE,["RA","RB","RC","RD"],"NET_A",[1,2],0).is_err());
}
