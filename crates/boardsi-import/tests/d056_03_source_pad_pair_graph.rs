//! D_056_03 independently enumerated same-layer source pad contact oracles.
//! Positive overlap is a *design Copper candidate*, not a plated-barrel
//! or manufactured-board connection.
use boardsi_import::electrical_graph_qualification::{qualify, SourceCoverage};
use boardsi_import::layered_pad_contacts::augment_pad_contacts;
use boardsi_import::layered_source_graph::{
    ContactKind, CopperFootprint, CopperNode, LayeredSourceGraph,
};
use boardsi_import::via_shapes::ViaLayerShape;

const SHA: [u8;32]=[0x63;32];
fn via(key:&str, layer:u32, x:i64, diameter:i64, drill:i64, shape:ViaLayerShape, net:&str)->CopperNode {
    CopperNode {
        source_key:key.to_owned(), source_line:Some(9),
        physical_copper_layer:layer, declared_net:Some(net.to_owned()),
        footprint:CopperFootprint::ViaPad {
            centre:[x,0], outer_ticks:diameter, drill_ticks:drill, shape,
        },
    }
}
fn smd(key:&str, layer:u32, x:i64, width:i64, height:i64, rectangular:bool, net:&str)->CopperNode {
    CopperNode {
        source_key:key.to_owned(), source_line:None,
        physical_copper_layer:layer, declared_net:Some(net.to_owned()),
        footprint:CopperFootprint::SmdPad {
            centre:[x,0], width_ticks:width, height_ticks:height, rectangular,
        },
    }
}
fn board(nodes:Vec<CopperNode>)->LayeredSourceGraph {
    LayeredSourceGraph {
        source_sha256:SHA, declared_copper_layers:64,nodes,
        geometry_contacts:vec![],unverified_barrels:vec![],
        verified_electrical_edges:vec![],
    }
}
fn count(g:&LayeredSourceGraph,kind:ContactKind)->usize {
    g.geometry_contacts.iter().filter(|e|e.kind==kind).count()
}
#[test]
fn off_center_via_via_round_rings_have_exact_positive_and_tangent_or_hole_negative() {
    let mut yes=board(vec![
        via("a",2,0,40,10,ViaLayerShape::Round,"A"),
        via("b",2,25,40,10,ViaLayerShape::Round,"B"),
    ]);
    augment_pad_contacts(&mut yes).unwrap();
    assert_eq!(count(&yes,ContactKind::ViaPadViaPad),1);
    let r=qualify(&yes,SHA,&SourceCoverage::Partial,&[],&[]).unwrap();
    assert_eq!(r.components,vec![vec![0,1]]);
    assert_eq!(r.net_conflicts.len(),1);
    assert_eq!(r.net_conflicts[0].path_nodes,vec![0,1]);

    // Tangency at the two outer radius boundaries is NOT area.
    let mut tangent=board(vec![
        via("a",2,0,40,10,ViaLayerShape::Round,"A"),
        via("b",2,40,40,10,ViaLayerShape::Round,"B"),
    ]);
    augment_pad_contacts(&mut tangent).unwrap();
    assert_eq!(count(&tangent,ContactKind::ViaPadViaPad),0);
    // A small pad wholly inside the physically empty drilled hole.
    let mut hole=board(vec![
        via("a",2,0,40,30,ViaLayerShape::Annular{inner_diameter_ticks:30},"A"),
        via("b",2,0,10,2,ViaLayerShape::Round,"B"),
    ]);
    augment_pad_contacts(&mut hole).unwrap();
    assert!(hole.geometry_contacts.is_empty());
}
#[test]
fn different_physical_layers_never_bridge_even_with_identical_xy() {
    for layers in [2,4,6,8,64] {
        let mut g=board(vec![
            via("a",1,0,40,10,ViaLayerShape::Round,"A"),
            via("b",layers,0,40,10,ViaLayerShape::Round,"B"),
        ]);
        augment_pad_contacts(&mut g).unwrap();
        assert!(g.geometry_contacts.is_empty());
        assert_eq!(qualify(&g,SHA,&SourceCoverage::Partial,&[],&[]).unwrap().components.len(),2);
    }
}
#[test]
fn square_hole_and_smd_pad_are_not_connected_by_empty_drill_centre() {
    let mut hole=board(vec![
        via("square",3,0,40,20,ViaLayerShape::Square,"NET_A"),
        smd("smd",3,0,4,4,true,"NET_B"),
    ]);
    augment_pad_contacts(&mut hole).unwrap();
    assert!(hole.geometry_contacts.is_empty());

    let mut copper=board(vec![
        via("square",3,0,40,20,ViaLayerShape::Square,"NET_A"),
        smd("smd",3,15,10,8,true,"NET_B"),
    ]);
    augment_pad_contacts(&mut copper).unwrap();
    assert_eq!(count(&copper,ContactKind::ViaPadSmdPad),1);
    assert_eq!(qualify(&copper,SHA,&SourceCoverage::Partial,&[],&[]).unwrap().net_conflicts.len(),1);
}
#[test]
fn rectangle_overlap_is_analytical_and_boundary_is_not_area() {
    let mut contact=board(vec![
        smd("a",4,0,20,12,true,"A"),
        smd("b",4,19,20,12,true,"B"),
    ]);
    augment_pad_contacts(&mut contact).unwrap();
    assert_eq!(count(&contact,ContactKind::SmdPadSmdPad),1);
    let mut tangent=board(vec![
        smd("a",4,0,20,12,true,"A"),
        smd("b",4,20,20,12,true,"B"),
    ]);
    augment_pad_contacts(&mut tangent).unwrap();
    assert!(tangent.geometry_contacts.is_empty());
}
#[test]
fn source_anti_pad_inside_contact_graph_is_rejected_without_mutation() {
    let mut g=board(vec![
        via("bad",1,0,40,10,ViaLayerShape::RoundAntiPad,"A"),
        via("good",1,10,40,10,ViaLayerShape::Round,"B"),
    ]);
    assert_eq!(augment_pad_contacts(&mut g).unwrap_err(),"D05603_ANTIPAD_AS_COPPER");
    assert!(g.geometry_contacts.is_empty());
}
#[test]
fn source_pad_contact_augmentation_is_deterministic_and_idempotent() {
    let mut g=board(vec![
        smd("a",5,0,30,30,true,"N"),
        smd("b",5,10,30,30,true,"N"),
        via("via",5,15,30,5,ViaLayerShape::Round,"N"),
    ]);
    augment_pad_contacts(&mut g).unwrap();
    assert_eq!(g.geometry_contacts.len(),3);
    let first=g.geometry_contacts.clone();
    augment_pad_contacts(&mut g).unwrap();
    assert_eq!(g.geometry_contacts,first);
}
#[test]
fn partly_overlapping_ring_and_circular_smd_positive_without_centre_coincidence() {
    let mut g=board(vec![
        via("annulus",14,0,40,30,ViaLayerShape::Annular{inner_diameter_ticks:30},"N1"),
        smd("round",14,16,16,16,false,"N2"),
    ]);
    augment_pad_contacts(&mut g).unwrap();
    assert_eq!(count(&g,ContactKind::ViaPadSmdPad),1);
    let mut absent=board(vec![
        via("annulus",14,0,40,30,ViaLayerShape::Annular{inner_diameter_ticks:30},"N1"),
        smd("round",14,0,16,16,false,"N2"),
    ]);
    augment_pad_contacts(&mut absent).unwrap();
    assert!(absent.geometry_contacts.is_empty());
}
