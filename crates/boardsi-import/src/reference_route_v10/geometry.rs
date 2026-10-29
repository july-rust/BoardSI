//! Bounded exact Source-Tick rectangle/pad relations for C_014.
//! All Source coordinates are constrained by the importer; widen before subtraction.
pub(super) fn margins(p: [i64;2], r:[i64;4])->[i128;4]{
    [i128::from(p[0])-i128::from(r[0]),i128::from(r[2])-i128::from(p[0]),
     i128::from(p[1])-i128::from(r[1]),i128::from(r[3])-i128::from(p[1])]
}
pub(super) fn pad_inside(p:[i64;2],d:i64,r:[i64;4])->bool{
    d>0 && margins(p,r).iter().all(|m| *m>0 && 2*(*m)>i128::from(d))
}
pub(super) fn hole_inside(h:[i64;4],o:[i64;4])->bool{
    h[0]>o[0] && h[1]>o[1] && h[2]<o[2] && h[3]<o[3]
}
pub(super) fn disjoint(a:[i64;4],b:[i64;4])->bool{
    a[2]<b[0] || b[2]<a[0] || a[3]<b[1] || b[3]<a[1]
}
pub(super) fn classify(p:[i64;2],d:i64,holes:&[[i64;4]])->(&'static str,Option<usize>){
    if holes.is_empty(){return ("SOURCE_CUTOUT_MISSING_RISK",None);}
    for (i,&hole) in holes.iter().enumerate(){
        let m=margins(p,hole);
        if m.iter().all(|v|*v>=0){
            let twice=2*(*m.iter().min().expect("four margins"));
            let status=if twice>i128::from(d){"SOURCE_CUTOUT_GAP_OBSERVED"}
                else if twice==i128::from(d){"SOURCE_CUTOUT_TANGENCY_RISK"}
                else{"SOURCE_REFERENCE_COPPER_OVERLAP_RISK"};
            return (status,Some(i));
        }
    }
    ("SOURCE_REFERENCE_COPPER_OVERLAP_RISK",None)
}
pub(super) fn severity(state:&str)->u8{
    match state{
        "SOURCE_REFERENCE_COPPER_OVERLAP_RISK"=>3,
        "SOURCE_CUTOUT_TANGENCY_RISK"=>2,
        "SOURCE_CUTOUT_MISSING_RISK"=>1,_=>0
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn exact_tangency_and_one_tick(){
        let holes=[[0,0,100,100]];
        assert_eq!(classify([50,50],99,&holes).0,"SOURCE_CUTOUT_GAP_OBSERVED");
        assert_eq!(classify([50,50],100,&holes).0,"SOURCE_CUTOUT_TANGENCY_RISK");
        assert_eq!(classify([50,50],101,&holes).0,"SOURCE_REFERENCE_COPPER_OVERLAP_RISK");
    }
    #[test]
    fn no_touched_or_nested_holes(){
        assert!(disjoint([0,0,10,10],[11,0,20,10]));
        assert!(!disjoint([0,0,10,10],[10,0,20,10]));
        assert!(!disjoint([0,0,10,10],[5,5,20,20]));
    }
}
