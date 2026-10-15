//! PH-03 homogeneous parallel-plate mathematical coupon only.
//! Deliberately distinct from the existing finite grounded-box microstrip.
//! Zero-flux lateral sides are natural FE boundaries, not grounded PEC walls.
use super::{for_each_triangle, solve_field_with_potential, xy, FieldEvidence, Grid, EPS0, MU0};

const MAX_NODES: usize = 240_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideBoundary {
    HomogeneousNeumann,
    GroundedDirichlet, // explicitly unsupported for the exact PH-03 oracle
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialModel {
    HomogeneousLossless,
    Layered,
    Lossy,
}

#[derive(Clone, Copy, Debug)]
pub struct PlateCoupon {
    pub width_m: f64,
    pub height_m: f64,
    pub epsilon_r: f64,
    pub mu_r: f64,
    pub bottom_electrode_z_m: f64,
    pub top_electrode_z_m: f64,
    pub bottom_v: f64,
    pub top_v: f64,
    pub left: SideBoundary,
    pub right: SideBoundary,
    pub material: MaterialModel,
    pub ideal_pec: bool,
    pub longitudinally_uniform: bool,
    pub cells_per_height: usize,
}

impl PlateCoupon {
    pub fn ph03(n: usize) -> Self {
        Self {
            width_m: 0.0003,
            height_m: 0.0002,
            epsilon_r: 3.4,
            mu_r: 1.0,
            bottom_electrode_z_m: 0.0,
            top_electrode_z_m: 0.0002,
            bottom_v: 0.0,
            top_v: 1.0,
            left: SideBoundary::HomogeneousNeumann,
            right: SideBoundary::HomogeneousNeumann,
            material: MaterialModel::HomogeneousLossless,
            ideal_pec: true,
            longitudinally_uniform: true,
            cells_per_height: n,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlateField {
    pub evidence: FieldEvidence,
    /// Row-major (z then x), including prescribed horizontal PEC values.
    pub nodal_potentials_v: Vec<f64>,
    /// Electric field vectors [Ex, Ez] for every triangle in mesh iteration order.
    pub element_fields_v_per_m: Vec<[f64; 2]>,
    pub max_potential_error_v: f64,
    pub max_horizontal_potential_span_v: f64,
    pub max_electrode_error_v: f64,
    pub max_ex_relative: f64,
    pub max_ez_relative: f64,
}

#[derive(Debug, Clone)]
pub struct PlateResults {
    pub c_f_per_m: f64,
    pub c0_f_per_m: f64,
    pub lext_h_per_m: f64,
    pub z0_ohm: f64,
    pub delay_s_per_m: f64,
    pub nx: usize,
    pub nz: usize,
    pub dx_m: f64,
    pub dielectric: PlateField,
    pub vacuum: PlateField,
    pub physical_status: &'static str,
}

fn plate_grid(c: &PlateCoupon) -> Result<Grid, String> {
    let lengths = [
        c.width_m, c.height_m, c.epsilon_r, c.mu_r,
        c.bottom_electrode_z_m, c.top_electrode_z_m,
        c.bottom_v, c.top_v,
    ];
    if lengths.iter().any(|v| !v.is_finite()) {
        return Err("nonfinite plate dimension, material or electrode".into());
    }
    if c.width_m <= 0.0 || c.height_m <= 0.0 || c.epsilon_r < 1.0 {
        return Err("invalid plate dimensions or epsilon_r (< 1 unsupported)".into());
    }
    if c.mu_r != 1.0 || c.material != MaterialModel::HomogeneousLossless
        || !c.ideal_pec || !c.longitudinally_uniform
    {
        return Err("unsupported plate material, permeability, PEC or longitudinal topology".into());
    }
    if c.left != SideBoundary::HomogeneousNeumann
        || c.right != SideBoundary::HomogeneousNeumann
    {
        return Err("PH-03 requires homogeneous Neumann on BOTH side boundaries".into());
    }
    if c.bottom_electrode_z_m != 0.0 || c.top_electrode_z_m != c.height_m
        || c.top_electrode_z_m <= c.bottom_electrode_z_m
        || c.bottom_v != 0.0 || c.top_v != 1.0
    {
        return Err("PH-03 requires separate full-width 0-V bottom and 1-V top electrodes".into());
    }
    if c.cells_per_height < 4 || c.cells_per_height > 2048 {
        return Err("unsupported or oversized plate mesh subdivision".into());
    }
    let nz = c.cells_per_height;
    let dx = c.height_m / nz as f64;
    if !dx.is_finite() || dx <= 0.0 {
        return Err("nonfinite or underflow plate mesh spacing".into());
    }
    let ratio = c.width_m / dx;
    if !ratio.is_finite() || ratio < 2.0 || ratio >= MAX_NODES as f64
        || (ratio - ratio.round()).abs() > 1e-8
    {
        return Err("nonconforming or oversized plate width/mesh ratio".into());
    }
    let nx = ratio.round() as usize;
    let cols = nx.checked_add(1).ok_or("plate column overflow")?;
    let rows = nz.checked_add(1).ok_or("plate row overflow")?;
    let count = cols.checked_mul(rows).ok_or("plate node overflow")?;
    if count > MAX_NODES {
        return Err("plate mesh exceeds memory guard".into());
    }
    let mut dirichlet = vec![None; count];
    let signal = (0..count).map(|id| id / cols == nz).collect::<Vec<bool>>();
    let mut mapping = vec![usize::MAX; count];
    let mut dof = 0;
    for j in 0..=nz {
        for i in 0..=nx {
            let id = j * cols + i;
            if j == 0 {
                dirichlet[id] = Some(0.0);
            } else if j == nz {
                dirichlet[id] = Some(1.0);
            } else {
                // Including i==0 and i==nx: zero Neumann is natural in
                // the weak form, with no additional Dirichlet row or RHS.
                mapping[id] = dof;
                dof += 1;
            }
        }
    }
    if dof == 0 {
        return Err("no plate FEM degrees of freedom".into());
    }
    Ok(Grid {
        nx, nz, dx, box_m: c.width_m / 2.0,
        dirichlet, signal, mapping, dof,
    })
}

fn solve_plate_field(c: &PlateCoupon, grid: &Grid, er: f64) -> Result<PlateField, String> {
    let (evidence, potentials) = solve_field_with_potential(c.height_m, grid, er)?;
    let mut max_potential_error_v: f64 = 0.0;
    let mut max_horizontal_potential_span_v: f64 = 0.0;
    let mut max_electrode_error_v: f64 = 0.0;
    for j in 0..=grid.nz {
        let exact = j as f64 * grid.dx / c.height_m;
        let start = j * (grid.nx + 1);
        let row = &potentials[start..start + grid.nx + 1];
        let mut row_min = f64::INFINITY;
        let mut row_max = f64::NEG_INFINITY;
        for &p in row {
            max_potential_error_v = max_potential_error_v.max((p - exact).abs());
            row_min = row_min.min(p);
            row_max = row_max.max(p);
            if j == 0 || j == grid.nz {
                max_electrode_error_v = max_electrode_error_v.max((p - exact).abs());
            }
        }
        max_horizontal_potential_span_v =
            max_horizontal_potential_span_v.max(row_max - row_min);
    }
    let mut element_fields_v_per_m = Vec::with_capacity(evidence.triangles);
    let mut max_ex_relative: f64 = 0.0;
    let mut max_ez_relative: f64 = 0.0;
    let ideal_field = 1.0 / c.height_m;
    for_each_triangle(|tri, _| {
        let (x0, z0) = xy(tri[0], grid);
        let (x1, z1) = xy(tri[1], grid);
        let (x2, z2) = xy(tri[2], grid);
        let twice_area = (x1 - x0) * (z2 - z0) - (x2 - x0) * (z1 - z0);
        let p0 = potentials[tri[0]];
        let p1 = potentials[tri[1]];
        let p2 = potentials[tri[2]];
        let ex = -(p0 * (z1 - z2) + p1 * (z2 - z0) + p2 * (z0 - z1))
            / twice_area;
        let ez = -(p0 * (x2 - x1) + p1 * (x0 - x2) + p2 * (x1 - x0))
            / twice_area;
        max_ex_relative = max_ex_relative.max((ex / ideal_field).abs());
        max_ez_relative = max_ez_relative.max((ez / ideal_field + 1.0).abs());
        element_fields_v_per_m.push([ex, ez]);
    }, grid, c.height_m, er);
    if [
        max_potential_error_v, max_horizontal_potential_span_v,
        max_electrode_error_v, max_ex_relative, max_ez_relative,
    ].iter().any(|v| !v.is_finite()) {
        return Err("nonfinite plate field diagnostic".into());
    }
    Ok(PlateField {
        evidence, nodal_potentials_v: potentials, element_fields_v_per_m,
        max_potential_error_v, max_horizontal_potential_span_v,
        max_electrode_error_v, max_ex_relative, max_ez_relative,
    })
}

pub fn extract_plate(c: &PlateCoupon) -> Result<PlateResults, String> {
    let grid = plate_grid(c)?;
    let dielectric = solve_plate_field(c, &grid, c.epsilon_r)?;
    let vacuum = solve_plate_field(c, &grid, 1.0)?;
    let cap = dielectric.evidence.c_f_per_m;
    let cap0 = vacuum.evidence.c_f_per_m;
    if cap < cap0 * (1.0 - 1e-9)
        || cap > cap0 * c.epsilon_r * (1.0 + 1e-9)
    {
        return Err("plate capacitance violates dielectric bounds".into());
    }
    let lext = MU0 * EPS0 / cap0;
    let z0 = (lext / cap).sqrt();
    let delay = (lext * cap).sqrt();
    if !lext.is_finite() || !z0.is_finite() || !delay.is_finite() {
        return Err("nonfinite derived plate transmission-line quantities".into());
    }
    Ok(PlateResults {
        c_f_per_m: cap, c0_f_per_m: cap0, lext_h_per_m: lext,
        z0_ohm: z0, delay_s_per_m: delay,
        nx: grid.nx, nz: grid.nz, dx_m: grid.dx,
        dielectric, vacuum,
        physical_status: "PrototypeOnly_IdealNeumannPlate_NotRealPCBValidated",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ph03_affine_field_and_distinct_boundary_corner_priority() {
        for n in [8, 16, 32] {
            let result = extract_plate(&PlateCoupon::ph03(n)).unwrap();
            assert_eq!(result.nx, 3 * n / 2);
            assert_eq!(result.nz, n);
            for field in [&result.dielectric, &result.vacuum] {
                assert!(field.max_potential_error_v <= 1e-8);
                assert!(field.max_horizontal_potential_span_v <= 1e-8);
                assert!(field.max_electrode_error_v <= 1e-14);
                assert!(field.max_ex_relative <= 1e-6);
                assert!(field.max_ez_relative <= 1e-6);
                assert!(field.evidence.residual <= 1e-8);
                assert!(field.evidence.charge_energy_relative_difference <= 1e-6);
                assert_eq!(field.element_fields_v_per_m.len(), field.evidence.triangles);
                assert_eq!(field.nodal_potentials_v.len(),
                    (result.nx + 1) * (result.nz + 1));
                assert_eq!(field.nodal_potentials_v[0], 0.0);
                assert_eq!(field.nodal_potentials_v[result.nx], 0.0);
                assert_eq!(field.nodal_potentials_v[result.nz * (result.nx + 1)], 1.0);
                assert_eq!(*field.nodal_potentials_v.last().unwrap(), 1.0);
            }
            let exact_c = EPS0 * 3.4 * 0.0003 / 0.0002;
            let exact_c0 = EPS0 * 0.0003 / 0.0002;
            assert!((result.c_f_per_m / exact_c - 1.0).abs() <= 1e-6);
            assert!((result.c0_f_per_m / exact_c0 - 1.0).abs() <= 1e-6);
        }
    }

    #[test]
    fn ph03_reject_unsupported_or_malformed_without_panic() {
        let mut cases = Vec::new();
        let base = PlateCoupon::ph03(8);
        let mut bad = base; bad.left = SideBoundary::GroundedDirichlet; cases.push(bad);
        let mut bad = base; bad.right = SideBoundary::GroundedDirichlet; cases.push(bad);
        let mut bad = base; bad.material = MaterialModel::Layered; cases.push(bad);
        let mut bad = base; bad.material = MaterialModel::Lossy; cases.push(bad);
        let mut bad = base; bad.ideal_pec = false; cases.push(bad);
        let mut bad = base; bad.longitudinally_uniform = false; cases.push(bad);
        let mut bad = base; bad.mu_r = 1.1; cases.push(bad);
        let mut bad = base; bad.top_electrode_z_m = 0.0001; cases.push(bad);
        let mut bad = base; bad.bottom_electrode_z_m = 0.0001; cases.push(bad);
        let mut bad = base; bad.top_electrode_z_m = 0.0; cases.push(bad);
        let mut bad = base; bad.top_v = 0.0; cases.push(bad);
        let mut bad = base; bad.width_m = -0.0003; cases.push(bad);
        let mut bad = base; bad.width_m = 0.0; cases.push(bad);
        let mut bad = base; bad.height_m = -0.0002; cases.push(bad);
        let mut bad = base; bad.height_m = 0.0; cases.push(bad);
        let mut bad = base; bad.epsilon_r = 0.0; cases.push(bad);
        let mut bad = base; bad.epsilon_r = f64::NAN; cases.push(bad);
        let mut bad = base; bad.mu_r = f64::INFINITY; cases.push(bad);
        let mut bad = base; bad.width_m = f64::INFINITY; cases.push(bad);
        let mut bad = base; bad.height_m = f64::NAN; cases.push(bad);
        let mut bad = base; bad.width_m = 1e300; cases.push(bad);
        let mut bad = base; bad.height_m = 1e-320; cases.push(bad);
        let mut bad = base; bad.width_m = 0.00031; cases.push(bad);
        let mut bad = base; bad.cells_per_height = 0; cases.push(bad);
        let mut bad = base; bad.cells_per_height = usize::MAX; cases.push(bad);
        let mut bad = base; bad.cells_per_height = 2048; cases.push(bad);
        for (i, c) in cases.into_iter().enumerate() {
            let res = std::panic::catch_unwind(|| extract_plate(&c));
            assert!(res.is_ok(), "negative test {i}: panic");
            assert!(res.unwrap().is_err(), "negative test {i}: accepted");
        }
    }
}
