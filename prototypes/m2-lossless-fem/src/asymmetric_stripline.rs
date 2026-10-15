//! PH-06: finite-PEC asymmetric, optionally two-region, lossless stripline.
//! Preserves PH-05 symmetric entrypoints; no lossy or full-wave capability.
use super::{for_each_material_triangle, on_grid, solve_field_with_material_potential,
    stiffness, xy, FieldEvidence, Grid, EPS0, MU0};
use super::stripline::StriplineSide;

const MAX_NODES: usize = 240_000;

#[derive(Clone, Copy, Debug)]
pub struct AsymmetricCoupon {
    pub width_m: f64,
    pub thickness_m: f64,
    pub ground_separation_m: f64,
    pub center_z_m: f64,
    pub half_span_m: f64,
    pub epsilon_bottom: f64,
    pub epsilon_top: f64,
    pub interface_z_m: Option<f64>,
    pub mu_r: f64,
    pub signal_v: f64,
    pub ground_v: f64,
    pub cells_per_height: usize,
    pub left: StriplineSide,
    pub right: StriplineSide,
    pub signal_ideal_pec: bool,
    pub ground_ideal_pec: bool,
    pub continuous_ground_planes: bool,
    pub longitudinally_uniform: bool,
    pub finite_conductivity_requested: bool,
    pub dielectric_loss_requested: bool,
    pub dielectric_dispersion_requested: bool,
}
impl AsymmetricCoupon {
    pub fn ph06(n: usize, half_span_m: f64) -> Self {
        Self {
            width_m: 0.00030, thickness_m: 0.00005,
            ground_separation_m: 0.00080, center_z_m: 0.00030,
            half_span_m, epsilon_bottom: 3.4, epsilon_top: 3.4,
            interface_z_m: None, mu_r: 1.0, signal_v: 1.0, ground_v: 0.0,
            cells_per_height: n,
            left: StriplineSide::HomogeneousNeumann,
            right: StriplineSide::HomogeneousNeumann,
            signal_ideal_pec: true, ground_ideal_pec: true,
            continuous_ground_planes: true, longitudinally_uniform: true,
            finite_conductivity_requested: false, dielectric_loss_requested: false,
            dielectric_dispersion_requested: false,
        }
    }
    pub fn ph06_two_region(n: usize, half_span_m: f64) -> Self {
        let mut c = Self::ph06(n, half_span_m);
        c.epsilon_top = 4.0;
        c.interface_z_m = Some(0.00035);
        c
    }
    pub fn lower_gap_m(&self) -> f64 {
        self.center_z_m - self.thickness_m / 2.0
    }
    pub fn upper_gap_m(&self) -> f64 {
        self.ground_separation_m - self.center_z_m - self.thickness_m / 2.0
    }
    fn er_at(&self, z: f64, vacuum: bool) -> f64 {
        if vacuum { 1.0 } else if self.interface_z_m.is_some_and(|v| z > v) {
            self.epsilon_top
        } else {
            self.epsilon_bottom
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsymmetricField {
    pub evidence: FieldEvidence,
    pub nodal_potentials_v: Vec<f64>,
    pub element_fields_v_per_m: Vec<[f64; 2]>,
    pub signal_charge_c_per_m: f64,
    pub upper_ground_charge_c_per_m: f64,
    pub lower_ground_charge_c_per_m: f64,
    pub charge_balance_relative: f64,
    pub max_x_reflection_error_v: f64,
    pub max_z_reflection_error_v: f64,
    pub max_fringe_ex_v_per_m: f64,
    pub max_pec_interior_field_v_per_m: f64,
    pub max_electrode_potential_error_v: f64,
}

#[derive(Clone, Debug)]
pub struct AsymmetricResults {
    pub c_f_per_m: f64,
    pub c0_f_per_m: f64,
    pub lext_h_per_m: f64,
    pub z0_ohm: f64,
    pub delay_s_per_m: f64,
    pub epsilon_effective: f64,
    pub nx: usize,
    pub nz: usize,
    pub dx_m: f64,
    pub dielectric: AsymmetricField,
    pub vacuum: AsymmetricField,
    pub physical_status: &'static str,
}

fn asymmetric_grid(c: &AsymmetricCoupon) -> Result<Grid, String> {
    let values = [c.width_m, c.thickness_m, c.ground_separation_m, c.center_z_m,
        c.half_span_m, c.epsilon_bottom, c.epsilon_top, c.mu_r,
        c.signal_v, c.ground_v];
    if values.iter().any(|v| !v.is_finite()) ||
        c.width_m <= 0.0 || c.thickness_m <= 0.0 ||
        c.ground_separation_m <= 0.0 || c.half_span_m <= 0.0 ||
        c.epsilon_bottom < 1.0 || c.epsilon_top < 1.0 {
        return Err("invalid/nonfinite PH-06 geometry, relative dielectric or voltage".into());
    }
    if c.mu_r != 1.0 || c.signal_v != 1.0 || c.ground_v != 0.0 ||
        c.left != StriplineSide::HomogeneousNeumann ||
        c.right != StriplineSide::HomogeneousNeumann ||
        !c.signal_ideal_pec || !c.ground_ideal_pec ||
        !c.continuous_ground_planes || !c.longitudinally_uniform ||
        c.finite_conductivity_requested || c.dielectric_loss_requested ||
        c.dielectric_dispersion_requested {
        return Err("unsupported PH-06: requires PEC 1-V signal, 0-V planes, Neumann sides, mu=1, lossless nondispersive uniform longitudinal topology".into());
    }
    if c.cells_per_height < 8 || c.cells_per_height > 256 ||
        c.cells_per_height % 8 != 0 {
        return Err("unsupported or oversized PH-06 mesh subdivision".into());
    }
    if c.lower_gap_m() <= 0.0 || c.upper_gap_m() <= 0.0 ||
        c.half_span_m <= c.width_m / 2.0 {
        return Err("PH-06 signal shorts GND or lateral domain".into());
    }
    if c.epsilon_top != c.epsilon_bottom && c.interface_z_m.is_none() {
        return Err("PH-06 layered dielectric requires explicit material interface".into());
    }
    if let Some(z) = c.interface_z_m {
        if !z.is_finite() || z <= 0.0 || z >= c.ground_separation_m {
            return Err("PH-06 invalid material interface location".into());
        }
    }
    let dz = c.ground_separation_m / c.cells_per_height as f64;
    if !dz.is_finite() || dz <= 0.0 {
        return Err("PH-06 underflow or nonfinite grid spacing".into());
    }
    let nx_float = 2.0 * c.half_span_m / dz;
    if !nx_float.is_finite() || nx_float <= 0.0 ||
        nx_float >= MAX_NODES as f64 {
        return Err("PH-06 nonfinite or oversized grid axis".into());
    }
    if !on_grid(c.half_span_m, dz) || !on_grid(2.0 * c.half_span_m, dz) ||
        !on_grid(c.width_m / 2.0, dz) || !on_grid(c.lower_gap_m(), dz) ||
        !on_grid(c.lower_gap_m() + c.thickness_m, dz) ||
        !on_grid(c.thickness_m, dz) ||
        c.interface_z_m.is_some_and(|z| !on_grid(z, dz)) {
        return Err("PH-06 nonconforming conductor/interface/box mesh; geometry snapping forbidden".into());
    }
    let nx = nx_float.round() as usize;
    let nz = c.cells_per_height;
    let cols = nx.checked_add(1).ok_or("PH-06 grid column overflow")?;
    let rows = nz.checked_add(1).ok_or("PH-06 grid row overflow")?;
    let total = cols.checked_mul(rows).ok_or("PH-06 grid size overflow")?;
    if total > MAX_NODES {
        return Err("PH-06 mesh exceeds explicit memory guard".into());
    }
    let mut dirichlet = vec![None; total];
    let mut signal = vec![false; total];
    let mut mapping = vec![usize::MAX; total];
    let mut dof = 0;
    let low = c.lower_gap_m();
    let high = c.ground_separation_m - c.upper_gap_m();
    for j in 0..=nz {
        for i in 0..=nx {
            let id = j * cols + i;
            let x = -c.half_span_m + i as f64 * dz;
            let z = j as f64 * dz;
            let metal = x.abs() <= c.width_m/2.0 + dz*1e-8 &&
                z >= low - dz*1e-8 && z <= high + dz*1e-8;
            if metal && (j == 0 || j == nz || i == 0 || i == nx) {
                return Err("PH-06 signal shorted to GND/outer edge".into());
            }
            if j == 0 || j == nz {
                dirichlet[id] = Some(0.0);
            } else if metal {
                dirichlet[id] = Some(1.0);
                signal[id] = true;
            } else {
                // Side nodes are free: natural zero normal flux.
                mapping[id] = dof;
                dof += 1;
            }
        }
    }
    if dof == 0 || !signal.iter().any(|v| *v) {
        return Err("PH-06 missing conductor or free field degrees".into());
    }
    Ok(Grid { nx, nz, dx: dz, box_m: c.half_span_m,
        dirichlet, signal, mapping, dof })
}

fn field(grid: &Grid, c: &AsymmetricCoupon, vacuum: bool)
    -> Result<AsymmetricField, String>
{
    let er = |z| c.er_at(z, vacuum);
    let (evidence, phi) = solve_field_with_material_potential(grid, er)?;
    let cols = grid.nx + 1;
    let mut err_x: f64 = 0.0;
    let mut err_z: f64 = 0.0;
    let mut err_electrode: f64 = 0.0;
    for j in 0..=grid.nz {
        for i in 0..=grid.nx {
            let idx = j*cols+i;
            err_x = err_x.max((phi[idx]-phi[j*cols+grid.nx-i]).abs());
            err_z = err_z.max((phi[idx]-phi[(grid.nz-j)*cols+i]).abs());
            if let Some(v) = grid.dirichlet[idx] {
                err_electrode = err_electrode.max((phi[idx]-v).abs());
            }
        }
    }
    let mut fields = Vec::with_capacity(evidence.triangles);
    let mut upper_reaction = 0.0;
    let mut lower_reaction = 0.0;
    let mut fringe: f64 = 0.0;
    let mut pec_field: f64 = 0.0;
    for_each_material_triangle(|tri, element_er| {
        let (x0,z0)=xy(tri[0],grid);
        let (x1,z1)=xy(tri[1],grid);
        let (x2,z2)=xy(tri[2],grid);
        let det=(x1-x0)*(z2-z0)-(x2-x0)*(z1-z0);
        let v=[phi[tri[0]],phi[tri[1]],phi[tri[2]]];
        let ex=-(v[0]*(z1-z2)+v[1]*(z2-z0)+v[2]*(z0-z1))/det;
        let ez=-(v[0]*(x2-x1)+v[1]*(x0-x2)+v[2]*(x1-x0))/det;
        fields.push([ex,ez]);
        let mean_x=(x0+x1+x2)/3.0;
        let mean_z=(z0+z1+z2)/3.0;
        if mean_x.abs()>c.width_m/2.0 &&
            (mean_z-c.center_z_m).abs()<c.ground_separation_m/4.0 {
            fringe=fringe.max(ex.abs());
        }
        if tri.iter().all(|&id| grid.signal[id]) {
            pec_field=pec_field.max(ex.hypot(ez));
        }
        // Ground charges are separate Dirichlet reactions; no forced symmetry.
        let k=stiffness(tri,grid,element_er);
        for a in 0..3 {
            let row=tri[a]/cols;
            if row != 0 && row != grid.nz { continue; }
            let rx=(0..3).map(|b|k[a][b]*v[b]).sum::<f64>();
            if row == 0 { lower_reaction+=rx; } else { upper_reaction+=rx; }
        }
    },grid,er);
    let q_signal=evidence.c_from_conductor_charge_f_per_m;
    let q_lower=EPS0*lower_reaction;
    let q_upper=EPS0*upper_reaction;
    let balance=((q_signal+q_lower+q_upper)/q_signal).abs();
    if [balance,err_x,err_z,err_electrode,fringe,pec_field]
        .iter().any(|x|!x.is_finite()) ||
        !q_lower.is_finite() || !q_upper.is_finite() {
        return Err("PH-06 nonfinite field or reaction".into());
    }
    Ok(AsymmetricField {
        evidence, nodal_potentials_v:phi, element_fields_v_per_m:fields,
        signal_charge_c_per_m:q_signal,
        upper_ground_charge_c_per_m:q_upper, lower_ground_charge_c_per_m:q_lower,
        charge_balance_relative:balance, max_x_reflection_error_v:err_x,
        max_z_reflection_error_v:err_z, max_fringe_ex_v_per_m:fringe,
        max_pec_interior_field_v_per_m:pec_field,
        max_electrode_potential_error_v:err_electrode,
    })
}

pub fn extract_asymmetric(c: &AsymmetricCoupon)->Result<AsymmetricResults,String>{
    let grid=asymmetric_grid(c)?;
    let dielectric=field(&grid,c,false)?;
    let vacuum=field(&grid,c,true)?;
    let cap=dielectric.evidence.c_f_per_m;
    let cap0=vacuum.evidence.c_f_per_m;
    let min_er=c.epsilon_bottom.min(c.epsilon_top);
    let max_er=c.epsilon_bottom.max(c.epsilon_top);
    if cap0<=0.0 || cap<cap0*min_er*(1.0-1e-8) ||
        cap>cap0*max_er*(1.0+1e-8) {
        return Err("PH-06 effective epsilon violates passive material bounds".into());
    }
    let lext=MU0*EPS0/cap0;
    let z0=(lext/cap).sqrt();
    let delay=(lext*cap).sqrt();
    if ![lext,z0,delay].iter().all(|v|v.is_finite() && *v>0.0) {
        return Err("PH-06 invalid quasi-TEM derived quantity".into());
    }
    Ok(AsymmetricResults {
        c_f_per_m:cap,c0_f_per_m:cap0,lext_h_per_m:lext,z0_ohm:z0,
        delay_s_per_m:delay,epsilon_effective:cap/cap0,nx:grid.nx,
        nz:grid.nz,dx_m:grid.dx,dielectric,vacuum,
        physical_status:"PrototypeOnly_IdealAsymmetricStripline_NotPCBValidated",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ph06_asymmetry_field_and_charge() {
        let o=extract_asymmetric(&AsymmetricCoupon::ph06(32,0.0012)).unwrap();
        assert!(o.dielectric.lower_ground_charge_c_per_m.abs() >
            1.05*o.dielectric.upper_ground_charge_c_per_m.abs());
        assert!(o.dielectric.max_z_reflection_error_v>0.01);
        for f in [&o.dielectric,&o.vacuum] {
            assert!(f.evidence.residual<=1e-8);
            assert!(f.evidence.charge_energy_relative_difference<=1e-6);
            assert!(f.charge_balance_relative<=1e-6);
            assert!(f.max_x_reflection_error_v<=1e-6);
            assert!(f.max_electrode_potential_error_v<=1e-12);
            assert!(f.max_fringe_ex_v_per_m>0.0);
            assert!(f.max_pec_interior_field_v_per_m<1e-6);
        }
        assert!((o.epsilon_effective/3.4-1.0).abs()<=1e-3);
    }
    #[test]
    fn ph06_two_region_and_mirror() {
        let c=AsymmetricCoupon::ph06_two_region(32,0.0012);
        let a=extract_asymmetric(&c).unwrap();
        assert!(a.epsilon_effective>=3.4-1e-8);
        assert!(a.epsilon_effective<=4.0+1e-8);
        let mut refl=c;
        refl.center_z_m=c.ground_separation_m-c.center_z_m;
        refl.interface_z_m=c.interface_z_m.map(|z|c.ground_separation_m-z);
        refl.epsilon_bottom=c.epsilon_top;
        refl.epsilon_top=c.epsilon_bottom;
        let b=extract_asymmetric(&refl).unwrap();
        assert!((a.c_f_per_m/b.c_f_per_m-1.0).abs()<1e-6);
        assert!((a.c0_f_per_m/b.c0_f_per_m-1.0).abs()<1e-6);
        assert!((a.dielectric.lower_ground_charge_c_per_m -
                 b.dielectric.upper_ground_charge_c_per_m).abs()/
                 a.dielectric.signal_charge_c_per_m<1e-5);
    }
    #[test]
    fn ph06_symmetric_recovery() {
        let mut c=AsymmetricCoupon::ph06(32,0.0012);
        c.center_z_m=0.0004;
        let ph06=extract_asymmetric(&c).unwrap();
        let ph05=super::super::stripline::extract_stripline(
            &super::super::stripline::StriplineCoupon::ph05(32,0.0012)).unwrap();
        assert!((ph06.c_f_per_m/ph05.c_f_per_m-1.0).abs()<1e-8);
        assert!((ph06.c0_f_per_m/ph05.c0_f_per_m-1.0).abs()<1e-8);
    }
    #[test]
    fn ph06_rejects_invalid_and_unsupported() {
        let base=AsymmetricCoupon::ph06(32,0.0012);
        let mut cases=Vec::new();
        let mut c=base;c.center_z_m=0.0;cases.push(c);
        let mut c=base;c.center_z_m=0.0008;cases.push(c);
        let mut c=base;c.thickness_m=0.0008;cases.push(c);
        let mut c=base;c.width_m=0.0;cases.push(c);
        let mut c=base;c.epsilon_top=4.0;cases.push(c);
        let mut c=base;c.interface_z_m=Some(0.000351);cases.push(c);
        let mut c=base;c.interface_z_m=Some(0.001);cases.push(c);
        let mut c=base;c.interface_z_m=Some(f64::NAN);cases.push(c);
        let mut c=base;c.epsilon_bottom=f64::INFINITY;cases.push(c);
        let mut c=base;c.epsilon_top=0.0;cases.push(c);
        let mut c=base;c.center_z_m=f64::NAN;cases.push(c);
        let mut c=base;c.mu_r=1.1;cases.push(c);
        let mut c=base;c.finite_conductivity_requested=true;cases.push(c);
        let mut c=base;c.dielectric_loss_requested=true;cases.push(c);
        let mut c=base;c.dielectric_dispersion_requested=true;cases.push(c);
        let mut c=base;c.signal_ideal_pec=false;cases.push(c);
        let mut c=base;c.left=StriplineSide::GroundedDirichlet;cases.push(c);
        let mut c=base;c.half_span_m=1e300;cases.push(c);
        let mut c=base;c.cells_per_height=usize::MAX;cases.push(c);
        for (i,c) in cases.into_iter().enumerate() {
            let r=std::panic::catch_unwind(||extract_asymmetric(&c));
            assert!(r.is_ok(),"negative case {i} panicked");
            assert!(r.unwrap().is_err(),"negative case {i} accepted");
        }
    }
}
