//! PH-05: separately validated, symmetric, homogeneous lossless stripline.
//! Geometrical PEC cross section, not a zero-thickness trace or a real-PCB solver.
//! Lateral Neumann is an explicit finite-window approximation.
use super::{
    for_each_triangle, on_grid, solve_field_with_potential, stiffness, xy, FieldEvidence, Grid,
    EPS0, MU0,
};

const MAX_NODES: usize = 240_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StriplineSide {
    HomogeneousNeumann,
    GroundedDirichlet,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StriplineMaterial {
    HomogeneousLossless,
    Layered,
    Lossy,
}
#[derive(Clone, Copy, Debug)]
pub struct StriplineCoupon {
    pub width_m: f64,
    pub thickness_m: f64,
    pub ground_separation_m: f64,
    pub center_z_m: f64,
    pub half_span_m: f64,
    pub epsilon_r: f64,
    pub mu_r: f64,
    pub signal_v: f64,
    pub ground_v: f64,
    pub cells_per_height: usize,
    pub left: StriplineSide,
    pub right: StriplineSide,
    pub material: StriplineMaterial,
    pub signal_ideal_pec: bool,
    pub ground_ideal_pec: bool,
    pub continuous_ground_planes: bool,
    pub longitudinally_uniform: bool,
    pub finite_conductivity_requested: bool,
    pub dielectric_loss_requested: bool,
}
impl StriplineCoupon {
    pub fn ph05(n: usize, half_span_m: f64) -> Self {
        Self {
            width_m: 0.00030,
            thickness_m: 0.00005,
            ground_separation_m: 0.00080,
            center_z_m: 0.00040,
            half_span_m,
            epsilon_r: 3.4,
            mu_r: 1.0,
            signal_v: 1.0,
            ground_v: 0.0,
            cells_per_height: n,
            left: StriplineSide::HomogeneousNeumann,
            right: StriplineSide::HomogeneousNeumann,
            material: StriplineMaterial::HomogeneousLossless,
            signal_ideal_pec: true,
            ground_ideal_pec: true,
            continuous_ground_planes: true,
            longitudinally_uniform: true,
            finite_conductivity_requested: false,
            dielectric_loss_requested: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StriplineField {
    pub evidence: FieldEvidence,
    /// z-major row order, 0 V at both upper/lower PEC planes.
    pub nodal_potentials_v: Vec<f64>,
    /// One electric field [Ex,Ez] per triangle in shared FEM iteration order.
    pub element_fields_v_per_m: Vec<[f64; 2]>,
    pub upper_ground_charge_c_per_m: f64,
    pub lower_ground_charge_c_per_m: f64,
    pub signal_charge_c_per_m: f64,
    pub charge_balance_relative: f64,
    pub ground_asymmetry_relative: f64,
    pub max_z_reflection_error_v: f64,
    pub max_x_reflection_error_v: f64,
    pub max_fringe_ex_v_per_m: f64,
    pub max_pec_interior_field_v_per_m: f64,
}

#[derive(Clone, Debug)]
pub struct StriplineResults {
    pub c_f_per_m: f64,
    pub c0_f_per_m: f64,
    pub lext_h_per_m: f64,
    pub z0_ohm: f64,
    pub delay_s_per_m: f64,
    pub epsilon_effective: f64,
    pub nx: usize,
    pub nz: usize,
    pub dx_m: f64,
    pub dielectric: StriplineField,
    pub vacuum: StriplineField,
    pub physical_status: &'static str,
}

fn stripline_grid(c: &StriplineCoupon) -> Result<Grid, String> {
    let values = [
        c.width_m,
        c.thickness_m,
        c.ground_separation_m,
        c.center_z_m,
        c.half_span_m,
        c.epsilon_r,
        c.mu_r,
        c.signal_v,
        c.ground_v,
    ];
    if values.iter().any(|v| !v.is_finite())
        || c.width_m <= 0.0
        || c.thickness_m <= 0.0
        || c.ground_separation_m <= 0.0
        || c.half_span_m <= 0.0
        || c.epsilon_r < 1.0
    {
        return Err("invalid/nonfinite stripline dimension, permittivity or voltage".into());
    }
    if c.mu_r != 1.0
        || c.signal_v != 1.0
        || c.ground_v != 0.0
        || c.left != StriplineSide::HomogeneousNeumann
        || c.right != StriplineSide::HomogeneousNeumann
        || c.material != StriplineMaterial::HomogeneousLossless
        || !c.signal_ideal_pec
        || !c.ground_ideal_pec
        || !c.continuous_ground_planes
        || !c.longitudinally_uniform
        || c.finite_conductivity_requested
        || c.dielectric_loss_requested
    {
        return Err("unsupported: requires PEC 1-V signal, 0-V planes, Neumann sides, homogeneous nonmagnetic lossless uniform material".into());
    }
    if c.cells_per_height < 8 || c.cells_per_height > 256 || c.cells_per_height % 8 != 0 {
        return Err("unsupported or oversized PH-05 mesh subdivision".into());
    }
    if c.center_z_m != c.ground_separation_m / 2.0 {
        return Err(
            "PH-05 requires an exactly centered symmetric signal (asymmetric PH-06 unsupported)"
                .into(),
        );
    }
    if c.thickness_m >= c.ground_separation_m || c.half_span_m <= c.width_m / 2.0 {
        return Err("signal shorts or exceeds PEC reference/domain".into());
    }
    let dx = c.ground_separation_m / c.cells_per_height as f64;
    if !dx.is_finite() || dx <= 0.0 {
        return Err("nonfinite/underflow FEM mesh size".into());
    }
    let nx_ratio = 2.0 * c.half_span_m / dx;
    if !nx_ratio.is_finite() || nx_ratio <= 0.0 || nx_ratio >= MAX_NODES as f64 {
        return Err("nonfinite or oversized PH-05 mesh axes".into());
    }
    let copper_bottom = c.center_z_m - c.thickness_m / 2.0;
    let copper_top = c.center_z_m + c.thickness_m / 2.0;
    // Explicitly require conductor faces and both lateral boundaries on nodes.
    // Never use implicit nearest-grid rounding of the requested geometry.
    if !on_grid(c.half_span_m, dx)
        || !on_grid(c.width_m / 2.0, dx)
        || !on_grid(c.thickness_m, dx)
        || !on_grid(copper_bottom, dx)
        || !on_grid(copper_top, dx)
        || !on_grid(2.0 * c.half_span_m, dx)
    {
        return Err("nonconforming PEC/box mesh; geometry snapping prohibited".into());
    }
    let nx = nx_ratio.round() as usize;
    let nz = c.cells_per_height;
    let cols = nx.checked_add(1).ok_or("stripline mesh column overflow")?;
    let rows = nz.checked_add(1).ok_or("stripline mesh row overflow")?;
    let count = cols
        .checked_mul(rows)
        .ok_or("stripline node count overflow")?;
    if count > MAX_NODES {
        return Err("PH-05 mesh exceeds explicit memory guard".into());
    }
    let mut dirichlet = vec![None; count];
    let mut signal = vec![false; count];
    let mut mapping = vec![usize::MAX; count];
    let mut dof = 0;
    for j in 0..=nz {
        for i in 0..=nx {
            let id = j * cols + i;
            let x = -c.half_span_m + i as f64 * dx;
            let z = j as f64 * dx;
            let metal = x.abs() <= c.width_m / 2.0 + dx * 1e-8
                && z >= copper_bottom - dx * 1e-8
                && z <= copper_top + dx * 1e-8;
            if metal && (j == 0 || j == nz || i == 0 || i == nx) {
                return Err("PH-05 signal shorted to ground or lateral truncation".into());
            }
            if j == 0 || j == nz {
                dirichlet[id] = Some(0.0);
            } else if metal {
                dirichlet[id] = Some(1.0);
                signal[id] = true;
            } else {
                // Including sidewall nodes: the homogeneous Neumann constraint
                // is NATURAL in the weak form. Never set them to grounded PEC.
                mapping[id] = dof;
                dof += 1;
            }
        }
    }
    if dof == 0 || !signal.iter().any(|v| *v) {
        return Err("missing PH-05 conductor or field degrees of freedom".into());
    }
    Ok(Grid {
        nx,
        nz,
        dx,
        box_m: c.half_span_m,
        dirichlet,
        signal,
        mapping,
        dof,
    })
}

fn solve_stripline_field(
    grid: &Grid,
    c: &StriplineCoupon,
    er: f64,
) -> Result<StriplineField, String> {
    // Full z-domain is dielectric, unlike existing microstrip's air region.
    let (evidence, phi) = solve_field_with_potential(c.ground_separation_m, grid, er)?;
    let cols = grid.nx + 1;
    let mut err_z: f64 = 0.0;
    let mut err_x: f64 = 0.0;
    for j in 0..=grid.nz {
        for i in 0..=grid.nx {
            let p = phi[j * cols + i];
            err_z = err_z.max((p - phi[(grid.nz - j) * cols + i]).abs());
            err_x = err_x.max((p - phi[j * cols + grid.nx - i]).abs());
        }
    }
    let mut fields = Vec::with_capacity(evidence.triangles);
    let mut upper_reaction = 0.0;
    let mut lower_reaction = 0.0;
    let mut max_fringe_ex: f64 = 0.0;
    let mut max_pec_field: f64 = 0.0;
    for_each_triangle(
        |tri, _| {
            let (x0, z0) = xy(tri[0], grid);
            let (x1, z1) = xy(tri[1], grid);
            let (x2, z2) = xy(tri[2], grid);
            let area2 = (x1 - x0) * (z2 - z0) - (x2 - x0) * (z1 - z0);
            let p = [phi[tri[0]], phi[tri[1]], phi[tri[2]]];
            let ex = -(p[0] * (z1 - z2) + p[1] * (z2 - z0) + p[2] * (z0 - z1)) / area2;
            let ez = -(p[0] * (x2 - x1) + p[1] * (x0 - x2) + p[2] * (x1 - x0)) / area2;
            fields.push([ex, ez]);
            let mean_x = (x0 + x1 + x2) / 3.0;
            let mean_z = (z0 + z1 + z2) / 3.0;
            if mean_x.abs() > c.width_m / 2.0
                && (mean_z - c.center_z_m).abs() < c.ground_separation_m / 4.0
            {
                max_fringe_ex = max_fringe_ex.max(ex.abs());
            }
            if tri.iter().all(|&id| grid.signal[id]) {
                max_pec_field = max_pec_field.max(ex.hypot(ez));
            }
            // The conductor-reaction sum on grounded boundary vertices supplies
            // ground charges independently of the stored signal reaction.
            let k = stiffness(tri, grid, er);
            for a in 0..3 {
                let id = tri[a];
                let row = id / cols;
                if row != 0 && row != grid.nz {
                    continue;
                }
                let reaction = (0..3).map(|b| k[a][b] * p[b]).sum::<f64>();
                if row == 0 {
                    lower_reaction += reaction;
                } else {
                    upper_reaction += reaction;
                }
            }
        },
        grid,
        c.ground_separation_m,
        er,
    );
    let lower_charge = EPS0 * lower_reaction;
    let upper_charge = EPS0 * upper_reaction;
    let signal_charge = evidence.c_from_conductor_charge_f_per_m;
    let charge_balance = ((upper_charge + lower_charge + signal_charge) / signal_charge).abs();
    let ground_asymmetry =
        (upper_charge - lower_charge).abs() / ((upper_charge.abs() + lower_charge.abs()) / 2.0);
    if [
        charge_balance,
        ground_asymmetry,
        err_z,
        err_x,
        max_fringe_ex,
        max_pec_field,
    ]
    .iter()
    .any(|v| !v.is_finite())
    {
        return Err("nonfinite PH-05 symmetry, electric field or charge evidence".into());
    }
    Ok(StriplineField {
        evidence,
        nodal_potentials_v: phi,
        element_fields_v_per_m: fields,
        upper_ground_charge_c_per_m: upper_charge,
        lower_ground_charge_c_per_m: lower_charge,
        signal_charge_c_per_m: signal_charge,
        charge_balance_relative: charge_balance,
        ground_asymmetry_relative: ground_asymmetry,
        max_z_reflection_error_v: err_z,
        max_x_reflection_error_v: err_x,
        max_fringe_ex_v_per_m: max_fringe_ex,
        max_pec_interior_field_v_per_m: max_pec_field,
    })
}

pub fn extract_stripline(c: &StriplineCoupon) -> Result<StriplineResults, String> {
    let grid = stripline_grid(c)?;
    let dielectric = solve_stripline_field(&grid, c, c.epsilon_r)?;
    let vacuum = solve_stripline_field(&grid, c, 1.0)?;
    let cap = dielectric.evidence.c_f_per_m;
    let cap0 = vacuum.evidence.c_f_per_m;
    if cap0 <= 0.0 || cap < cap0 * (1.0 - 1e-9) || cap > cap0 * c.epsilon_r * (1.0 + 1e-9) {
        return Err("PH-05 ideal homogeneous dielectric capacitance bounds violated".into());
    }
    let lext = MU0 * EPS0 / cap0;
    let z0 = (lext / cap).sqrt();
    let delay = (lext * cap).sqrt();
    if ![lext, z0, delay].iter().all(|v| v.is_finite()) {
        return Err("PH-05 derived quasi-TEM parameters nonfinite".into());
    }
    Ok(StriplineResults {
        c_f_per_m: cap,
        c0_f_per_m: cap0,
        lext_h_per_m: lext,
        z0_ohm: z0,
        delay_s_per_m: delay,
        epsilon_effective: cap / cap0,
        nx: grid.nx,
        nz: grid.nz,
        dx_m: grid.dx,
        dielectric,
        vacuum,
        physical_status: "PrototypeOnly_IdealSymmetricStripline_NotPCBValidated",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ph05_geometry_boundary_symmetry_energy_and_fringe() {
        let p = StriplineCoupon::ph05(32, 0.0012);
        let grid = stripline_grid(&p).unwrap();
        assert_eq!(grid.nz, 32);
        assert_eq!(grid.nx, 96);
        for i in 0..=grid.nx {
            assert_eq!(grid.dirichlet[i], Some(0.0));
            assert_eq!(grid.dirichlet[grid.nz * (grid.nx + 1) + i], Some(0.0));
        }
        assert_eq!(grid.dirichlet[16 * (grid.nx + 1) + 48], Some(1.0));
        assert_eq!(grid.dirichlet[16 * (grid.nx + 1)], None);
        let o = extract_stripline(&p).unwrap();
        assert_eq!(
            o.dielectric.nodal_potentials_v.len(),
            (o.nx + 1) * (o.nz + 1)
        );
        assert_eq!(
            o.dielectric.element_fields_v_per_m.len(),
            o.dielectric.evidence.triangles
        );
        for f in [&o.dielectric, &o.vacuum] {
            assert!(f.evidence.residual <= 1e-8);
            assert!(f.evidence.charge_energy_relative_difference <= 1e-6);
            assert!(f.max_z_reflection_error_v <= 1e-6);
            assert!(f.max_x_reflection_error_v <= 1e-6);
            assert!(f.ground_asymmetry_relative <= 0.001);
            assert!(f.charge_balance_relative <= 1e-6);
            assert!(f.max_fringe_ex_v_per_m > 0.0);
            assert!(f.max_pec_interior_field_v_per_m <= 1e-6);
        }
        assert!((o.epsilon_effective / 3.4 - 1.0).abs() <= 0.001);
        assert!((o.delay_s_per_m / (MU0 * EPS0 * 3.4).sqrt() - 1.0).abs() <= 0.001);
    }

    #[test]
    fn ph05_monotonic_width_thickness_plane_spacing_and_dielectric() {
        let base = StriplineCoupon::ph05(32, 0.0012);
        let mut narrow = base;
        narrow.width_m = 0.00020;
        let mut wide = base;
        wide.width_m = 0.00040;
        let a = extract_stripline(&narrow).unwrap();
        let b = extract_stripline(&base).unwrap();
        let d = extract_stripline(&wide).unwrap();
        assert!(a.c_f_per_m < b.c_f_per_m && b.c_f_per_m < d.c_f_per_m);
        assert!(a.z0_ohm > b.z0_ohm && b.z0_ohm > d.z0_ohm);
        let mut thick = base;
        thick.thickness_m = 0.00010;
        assert!(extract_stripline(&thick).unwrap().c_f_per_m > b.c_f_per_m);
        let mut tall = base;
        tall.ground_separation_m = 0.001;
        tall.center_z_m = 0.0005;
        tall.cells_per_height = 40; // unchanged dz, all copper edges conform
        assert!(extract_stripline(&tall).unwrap().c_f_per_m < b.c_f_per_m);
        let mut air = base;
        air.epsilon_r = 1.0;
        let a = extract_stripline(&air).unwrap();
        assert!((b.c_f_per_m / a.c_f_per_m - 3.4).abs() < 3.4e-3);
    }

    #[test]
    fn ph05_reject_bad_inputs_without_panic() {
        let base = StriplineCoupon::ph05(32, 0.0012);
        let mut cases = Vec::new();
        let mut c = base;
        c.thickness_m = c.ground_separation_m;
        cases.push(c);
        let mut c = base;
        c.center_z_m = 0.00039;
        cases.push(c);
        let mut c = base;
        c.left = StriplineSide::GroundedDirichlet;
        cases.push(c);
        let mut c = base;
        c.right = StriplineSide::GroundedDirichlet;
        cases.push(c);
        let mut c = base;
        c.material = StriplineMaterial::Layered;
        cases.push(c);
        let mut c = base;
        c.material = StriplineMaterial::Lossy;
        cases.push(c);
        let mut c = base;
        c.finite_conductivity_requested = true;
        cases.push(c);
        let mut c = base;
        c.dielectric_loss_requested = true;
        cases.push(c);
        let mut c = base;
        c.signal_ideal_pec = false;
        cases.push(c);
        let mut c = base;
        c.ground_ideal_pec = false;
        cases.push(c);
        let mut c = base;
        c.continuous_ground_planes = false;
        cases.push(c);
        let mut c = base;
        c.longitudinally_uniform = false;
        cases.push(c);
        let mut c = base;
        c.mu_r = 1.1;
        cases.push(c);
        let mut c = base;
        c.signal_v = 0.9;
        cases.push(c);
        let mut c = base;
        c.ground_v = 1.0;
        cases.push(c);
        let mut c = base;
        c.width_m = 0.0;
        cases.push(c);
        let mut c = base;
        c.width_m = -1.0;
        cases.push(c);
        let mut c = base;
        c.thickness_m = 0.0;
        cases.push(c);
        let mut c = base;
        c.half_span_m = 0.00010;
        cases.push(c);
        let mut c = base;
        c.half_span_m = 0.00121;
        cases.push(c);
        let mut c = base;
        c.half_span_m = 1e300;
        cases.push(c);
        let mut c = base;
        c.epsilon_r = f64::NAN;
        cases.push(c);
        let mut c = base;
        c.epsilon_r = 0.0;
        cases.push(c);
        let mut c = base;
        c.ground_separation_m = f64::INFINITY;
        cases.push(c);
        let mut c = base;
        c.cells_per_height = usize::MAX;
        cases.push(c);
        let mut c = base;
        c.cells_per_height = 0;
        cases.push(c);
        for (index, c) in cases.into_iter().enumerate() {
            let res = std::panic::catch_unwind(|| extract_stripline(&c));
            assert!(res.is_ok(), "negative case {index} panicked");
            assert!(res.unwrap().is_err(), "negative case {index} accepted");
        }
    }
}
