//! A_022 P2-M: restricted, read-only PADS V9.4 MILS *LINES* COPPER artwork.
//! This records explicit filled COPCLS/COPCIR only; it is not flood,
//! electrical contact, physical stackup, or a verified fabricated conductor.
use crate::outline_preview::{bounded, fixed, valid_ring};
use std::collections::BTreeSet;

const MAX_COPPER: usize = 64;
const MAX_VERTICES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopperGeometry {
    Polygon(Vec<[i64; 2]>), // Exact 0.01-mil ticks; repeated closure vertex.
    Circle { centre: [i64; 2], radius: i64 },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopperArea {
    pub name: String,
    pub net_label: Option<String>, // Source association, NOT connectivity.
    pub layer: u32,
    pub source_line: usize,
    pub geometry: CopperGeometry,
}
fn token(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max &&
        s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
fn point(row: &str, origin: [i64; 2]) -> Result<[i64; 2], &'static str> {
    if row.len() > 16_384 { return Err("OVERLONG_COPPER_LINE"); }
    let items: Vec<_> = row.split_ascii_whitespace().collect();
    if items.len() != 2 { return Err("UNSUPPORTED_COPPER_ARC"); }
    let x = origin[0].checked_add(fixed(items[0])?).ok_or("COPPER_OVERFLOW")?;
    let y = origin[1].checked_add(fixed(items[1])?).ok_or("COPPER_OVERFLOW")?;
    Ok([bounded(x)?, bounded(y)?])
}
/// Strict source-syntax preview. Explicit COPCUT / COPCCO or multiple pieces
/// cause a hard refusal, rather than inventing a solid area over a void.
pub fn parse(bytes: &[u8], max_layer: u32) -> Result<Vec<CopperArea>, &'static str> {
    if bytes.len() > crate::route_preview::MAX_FILE_BYTES { return Err("INPUT_TOO_LARGE"); }
    if !(1..=64).contains(&max_layer) { return Err("COPPER_LAYER_RANGE"); }
    let first = bytes.split(|&b| b == b'\n').next().ok_or("EMPTY_FILE")?;
    let header = std::str::from_utf8(first).map_err(|_| "INVALID_HEADER_ENCODING")?;
    if !matches!(header.split_ascii_whitespace().next().unwrap_or(""),
        "!PADS-POWERPCB-V9.4-MILS!" |
        "!PADS-POWERPCB-V9.4-MILS-CP1252!" |
        "!PADS-POWERPCB-V9.4-MILS-250L!" |
        "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!") {
        return Err("UNSUPPORTED_REVISION_OR_UNIT");
    }
    let source = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = source.lines().collect();
    let mut inside = false;
    let mut ended = false;
    let mut found = Vec::new();
    let mut names = BTreeSet::new();
    let mut i = 1;
    while i < lines.len() {
        if lines[i].len() > 16_384 { return Err("OVERLONG_COPPER_LINE"); }
        let fields: Vec<_> = lines[i].split_ascii_whitespace().collect();
        if fields.is_empty() || matches!(fields[0], "*REMARK*" | "REMARK") {
            i += 1; continue;
        }
        if fields[0] == "*END*" { ended = true; break; }
        if inside && fields[0] == "*GET*" { return Err("UNSUPPORTED_COPPER_REUSE"); }
        if fields[0].starts_with('*') {
            inside = fields[0] == "*LINES*";
            i += 1; continue;
        }
        if !inside { i += 1; continue; }
        // A standalone void can subtract from visible copper: fail closed.
        // No silent render that hides COPCUT/COPCCO under a filled shape.
        if fields.len() >= 2 && fields[1] == "COPCUT" {
            return Err("UNSUPPORTED_COPPER_VOID");
        }
        if fields.len() < 2 || fields[1] != "COPPER" {
            i += 1; continue;
        }
        // name COPPER x y 1 1 [0 net]. Only declared solid-filled
        // single-piece copper is accepted. Reject external-library reuse.
        if !matches!(fields.len(), 6 | 8) ||
            !token(fields[0], 16) || fields[4] != "1" || fields[5] != "1" ||
            (fields.len() == 8 && (fields[6] != "0" || !token(fields[7], 47))) {
            return Err("UNSUPPORTED_COPPER_HEADER");
        }
        if found.len() >= MAX_COPPER || !names.insert(fields[0].to_owned()) {
            return Err("COPPER_COUNT_OR_DUPLICATE");
        }
        let source_line = i+1;
        let origin = [bounded(fixed(fields[2])?)?, bounded(fixed(fields[3])?)?];
        let name = fields[0].to_owned();
        let net_label = if fields.len()==8 {Some(fields[7].to_owned())}else{None};
        i+=1;
        let piece: Vec<_> = lines.get(i).ok_or("TRUNCATED_COPPER")?
            .split_ascii_whitespace().collect();
        if piece.len()!=5 || !matches!(piece[0], "COPCLS"|"COPCIR") ||
           piece[3]!="0" {
            return Err("UNSUPPORTED_COPPER_PIECE");
        }
        let n = piece[1].parse::<usize>().map_err(|_| "COPPER_CORNER_COUNT")?;
        if (piece[0]=="COPCIR" && n!=2) ||
           (piece[0]=="COPCLS" && !(4..=MAX_VERTICES).contains(&n)) {
            return Err("COPPER_CORNER_COUNT");
        }
        let width = fixed(piece[2])?;
        if !(1..=25_000).contains(&width) { return Err("COPPER_WIDTH_RANGE"); }
        let layer = piece[4].parse::<u32>().map_err(|_| "COPPER_LAYER_RANGE")?;
        if !(1..=max_layer).contains(&layer) { return Err("COPPER_LAYER_RANGE"); }
        i+=1;
        let mut vertices=Vec::with_capacity(n);
        for _ in 0..n {
            let row=lines.get(i).ok_or("TRUNCATED_COPPER")?;
            vertices.push(point(row,origin)?);
            i+=1;
        }
        let geometry=if piece[0]=="COPCIR" {
            let a=vertices[0];let b=vertices[1];
            if (a[0]==b[0])==(a[1]==b[1]) {return Err("UNSUPPORTED_COPPER_CIRCLE_DIAMETER");}
            let diameter=(b[0]-a[0]).abs().max((b[1]-a[1]).abs());
            if diameter<2 || diameter%2!=0 {return Err("UNSUPPORTED_COPPER_CIRCLE_GRID");}
            let centre=[(a[0]+b[0])/2,(a[1]+b[1])/2];
            let radius=diameter/2;
            for coord in [
                [centre[0]-radius,centre[1]],[centre[0]+radius,centre[1]],
                [centre[0],centre[1]-radius],[centre[0],centre[1]+radius]
            ] { bounded(coord[0])?;bounded(coord[1])?; }
            CopperGeometry::Circle{centre,radius}
        }else{
            if !valid_ring(&vertices) {return Err("INVALID_COPPER_RING");}
            CopperGeometry::Polygon(vertices)
        };
        found.push(CopperArea{name,net_label,layer,source_line,geometry});
    }
    if !ended {return Err("MISSING_END_MARKER");}
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE:&str="!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\nGND_ZONE COPPER 100 200 1 1 0 GND\nCOPCLS 5 0.01 0 2\n0 0\n10 0\n10 20\n0 20\n0 0\n*END*\n";
    #[test] fn polygon_grid_and_source_net() {
        let a=parse(SAMPLE.as_bytes(),4).unwrap();
        assert_eq!(a.len(),1);assert_eq!(a[0].source_line,5);
        assert_eq!(a[0].net_label.as_deref(),Some("GND"));
        assert_eq!(a[0].geometry,CopperGeometry::Polygon(vec![
            [10000,20000],[11000,20000],[11000,22000],[10000,22000],[10000,20000]
        ]));
    }
    #[test] fn circle_only_no_net_and_literal_net_are_distinct() {
        let circle=SAMPLE.replace("GND_ZONE COPPER 100 200 1 1 0 GND","GND_ZONE COPPER 100 200 1 1")
            .replace("COPCLS 5 0.01 0 2\n0 0\n10 0\n10 20\n0 20\n0 0",
                     "COPCIR 2 0.01 0 2\n0 0\n20 0");
        let a=parse(circle.as_bytes(),4).unwrap();
        assert_eq!(a[0].net_label,None);
        assert_eq!(a[0].geometry,CopperGeometry::Circle{centre:[11000,20000],radius:1000});
        let literal=parse(SAMPLE.replace(" 0 GND"," 0 NET_NOT_DECLARED").as_bytes(),4).unwrap();
        assert_eq!(literal[0].net_label.as_deref(),Some("NET_NOT_DECLARED"));
    }
    #[test] fn refuses_void_multifill_arc_and_invalid_ring() {
        for sample in [
            SAMPLE.replace("COPCLS","COPCUT"),
            SAMPLE.replace("*END*","CUTZONE COPCUT 0 0 1 0\n*END*"),
            SAMPLE.replace("1 1 0 GND","2 1 0 GND"),
            SAMPLE.replace("COPCLS 5","COPCLS 4"),
            SAMPLE.replace("0 20\n0 0","0 20\n0 1"),
            SAMPLE.replace("10 0\n10 20\n0 20","10 20\n10 0\n0 20"),
            SAMPLE.replace("10 0","10 0 90 0 0 0 0 0")
        ] {assert!(parse(sample.as_bytes(),4).is_err(),"{sample}");}
    }
    #[test] fn rejects_range_truncation_duplicate_and_end() {
        assert!(parse(SAMPLE.as_bytes(),1).is_err());
        assert!(parse(SAMPLE.replace("10 0","10.001 0").as_bytes(),4).is_err());
        assert!(parse(SAMPLE.replace("*END*","").as_bytes(),4).is_err());
        let duplicate=SAMPLE.replace("*END*","GND_ZONE COPPER 0 0 1 1\nCOPCIR 2 0.01 0 1\n0 0\n10 0\n*END*");
        assert!(parse(duplicate.as_bytes(),4).is_err());
    }
}