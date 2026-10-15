//! Deliberately isolated P1 triangular 2D electrostatic FEM for an ideal PEC coupon.
//! This is NOT a real PCB frequency/loss/3D field solver or product integration.
//! It uses a different discretization and iterative solve from the Python FVM oracle.

use std::time::Instant;

pub mod plate;
pub mod stripline;
pub mod asymmetric_stripline;

pub const EPS0: f64 = 8.854_187_818_8e-12; // F/m
pub const MU0: f64 = 1.256_637_061_27e-6; // H/m

#[derive(Clone, Copy, Debug)]
pub struct IdealCoupon {
    pub width_m: f64,
    pub thickness_m: f64,
    pub dielectric_height_m: f64,
    pub epsilon_r: f64,
    pub box_m: f64,
    pub cells_per_height: usize,
    pub pec_signal: bool,
    pub pec_reference: bool,
    pub explicitly_lossless: bool,
    pub nonmagnetic: bool,
    pub continuous_reference: bool,
    pub longitudinally_uniform: bool,
}

impl IdealCoupon {
    pub fn synthetic(n: usize, box_m: f64) -> Self {
        Self {
            width_m: 0.0003,
            thickness_m: 0.00005,
            dielectric_height_m: 0.0002,
            epsilon_r: 3.4,
            box_m,
            cells_per_height: n,
            pec_signal: true,
            pec_reference: true,
            explicitly_lossless: true,
            nonmagnetic: true,
            continuous_reference: true,
            longitudinally_uniform: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldEvidence {
    pub c_f_per_m: f64,
    pub c_from_conductor_charge_f_per_m: f64,
    pub charge_energy_relative_difference: f64,
    pub residual: f64,
    pub iterations: usize,
    pub dof: usize,
    pub triangles: usize,
    pub matrix_nnz: usize,
    pub elapsed_s: f64,
}

#[derive(Debug, Clone)]
pub struct Extracted {
    pub c_f_per_m: f64,
    pub c0_f_per_m: f64,
    pub lext_h_per_m: f64,
    pub z0_ohm: f64,
    pub delay_s_per_m: f64,
    pub epsilon_effective: f64,
    pub dielectric: FieldEvidence,
    pub vacuum: FieldEvidence,
    // Intentionally not ModelStatus::Validated: FVM references do not qualify real boards.
    pub physical_status: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct ComplexValue {
    pub re: f64,
    pub im: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct LosslessS2 {
    pub s11: ComplexValue,
    pub s12: ComplexValue,
    pub s21: ComplexValue,
    pub s22: ComplexValue,
}

struct Grid {
    nx: usize,
    nz: usize,
    dx: f64,
    box_m: f64,
    dirichlet: Vec<Option<f64>>,
    signal: Vec<bool>,
    mapping: Vec<usize>,
    dof: usize,
}

fn on_grid(value: f64, step: f64) -> bool {
    let count = value / step;
    count.is_finite() && (count - count.round()).abs() <= 1e-8
}

fn make_grid(c: &IdealCoupon) -> Result<Grid, String> {
    let values = [
        c.width_m, c.thickness_m, c.dielectric_height_m, c.epsilon_r, c.box_m,
    ];
    if values.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return Err("nonfinite, absent, zero or negative geometry/dielectric".into());
    }
    if !c.pec_signal || !c.pec_reference || !c.explicitly_lossless ||
       !c.nonmagnetic || !c.continuous_reference || !c.longitudinally_uniform {
        return Err("unsupported: requires explicit PEC, lossless, nonmagnetic, continuous reference and uniform cross section".into());
    }
    if c.epsilon_r < 1.0 {
        return Err("prototype restricts epsilon_r to >= 1".into());
    }
    if c.cells_per_height < 4 || c.cells_per_height > 48 || c.cells_per_height % 4 != 0 {
        return Err("unsupported mesh order/size (n multiple of 4, 4..48)".into());
    }
    if c.box_m <= c.width_m / 2.0 || c.box_m <= c.dielectric_height_m + c.thickness_m {
        return Err("copper overlaps or exceeds grounded outer domain".into());
    }
    let dx = c.dielectric_height_m / c.cells_per_height as f64;
    if !dx.is_finite() || dx <= 0.0 {
        return Err("mesh spacing is nonfinite or underflows to zero".into());
    }
    let wide_box = 2.0 * c.box_m;
    let nx_ratio = wide_box / dx;
    let nz_ratio = c.box_m / dx;
    // Bound BEFORE float-to-integer conversion: Rust saturates out-of-range
    // float casts, and (usize::MAX + 1) would overflow before a memory guard.
    // Any valid grid with <= 240_000 nodes has both axes below 240_000.
    const MAX_NODES: usize = 240_000;
    if !nx_ratio.is_finite() || !nz_ratio.is_finite() ||
       nx_ratio < 1.0 || nz_ratio < 1.0 ||
       nx_ratio >= MAX_NODES as f64 || nz_ratio >= MAX_NODES as f64 {
        return Err("nonfinite or oversized mesh dimensions".into());
    }
    if !on_grid(c.width_m / 2.0, dx) ||
       !on_grid(c.thickness_m, dx) ||
       !on_grid(c.box_m, dx) ||
       !on_grid(wide_box, dx) {
        return Err("nonconforming conductor/material/box grid: reject rather than silently snap".into());
    }
    let nx = nx_ratio.round() as usize;
    let nz = nz_ratio.round() as usize;
    let columns = nx.checked_add(1).ok_or("mesh column count overflow")?;
    let rows = nz.checked_add(1).ok_or("mesh row count overflow")?;
    let count = columns.checked_mul(rows).ok_or("mesh node count overflow")?;
    if count > MAX_NODES {
        return Err("mesh exceeds explicit prototype memory guard".into());
    }
    let mut dirichlet = vec![None; count];
    let mut signal = vec![false; count];
    let mut mapping = vec![usize::MAX; count];
    let mut dof = 0;
    for j in 0..=nz {
        for i in 0..=nx {
            let id = j * (nx + 1) + i;
            let x = -c.box_m + i as f64 * dx;
            let z = j as f64 * dx;
            let metal = x.abs() <= c.width_m / 2.0 + dx * 1e-8 &&
                z >= c.dielectric_height_m - dx * 1e-8 &&
                z <= c.dielectric_height_m + c.thickness_m + dx * 1e-8;
            let wall = i == 0 || i == nx || j == 0 || j == nz;
            if metal && wall {
                return Err("signal shorts reference/exterior".into());
            }
            if wall {
                dirichlet[id] = Some(0.0);
            } else if metal {
                dirichlet[id] = Some(1.0);
                signal[id] = true;
            } else {
                mapping[id] = dof;
                dof += 1;
            }
        }
    }
    if dof == 0 || !signal.iter().any(|v| *v) {
        return Err("missing signal or no independent degrees of freedom".into());
    }
    Ok(Grid { nx, nz, dx, box_m: c.box_m, dirichlet, signal, mapping, dof })
}

fn xy(id: usize, grid: &Grid) -> (f64, f64) {
    let i = id % (grid.nx + 1);
    let j = id / (grid.nx + 1);
    (-grid.box_m + i as f64 * grid.dx, j as f64 * grid.dx)
}

fn stiffness(tri: [usize; 3], grid: &Grid, er: f64) -> [[f64; 3]; 3] {
    let (x0, y0) = xy(tri[0], grid);
    let (x1, y1) = xy(tri[1], grid);
    let (x2, y2) = xy(tri[2], grid);
    let twice_area = (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0);
    let bx = [y1 - y2, y2 - y0, y0 - y1];
    let by = [x2 - x1, x0 - x2, x1 - x0];
    let scale = er / (2.0 * twice_area);
    let mut k = [[0.0; 3]; 3];
    for a in 0..3 {
        for b in 0..3 {
            k[a][b] = scale * (bx[a] * bx[b] + by[a] * by[b]);
        }
    }
    k
}

fn for_each_triangle(mut action: impl FnMut([usize; 3], f64), grid: &Grid, dielectric_height_m: f64, er: f64) {
    for j in 0..grid.nz {
        let relative = if ((j as f64) + 0.5) * grid.dx < dielectric_height_m {
            er
        } else {
            1.0
        };
        for i in 0..grid.nx {
            let bl = j * (grid.nx + 1) + i;
            let br = bl + 1;
            let tl = bl + grid.nx + 1;
            let tr = tl + 1;
            action([bl, br, tr], relative);
            action([bl, tr, tl], relative);
        }
    }
}

struct Csr {
    offsets: Vec<usize>,
    columns: Vec<usize>,
    values: Vec<f64>,
    diag: Vec<f64>,
}

impl Csr {
    fn matvec(&self, x: &[f64], out: &mut [f64]) {
        for (i, y) in out.iter_mut().enumerate() {
            let mut sum = 0.0;
            for p in self.offsets[i]..self.offsets[i + 1] {
                sum += self.values[p] * x[self.columns[p]];
            }
            *y = sum;
        }
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn conjugate_gradient(a: &Csr, b: &[f64]) -> Result<(Vec<f64>, usize, f64), String> {
    let n = b.len();
    let bnorm = dot(b, b).sqrt();
    if !(bnorm.is_finite() && bnorm > 0.0) {
        return Err("singular or missing electrical excitation".into());
    }
    let mut x = vec![0.0; n];
    let mut r = b.to_vec();
    let mut z: Vec<f64> = r.iter().enumerate().map(|(i, v)| *v / a.diag[i]).collect();
    let mut p = z.clone();
    let mut ap = vec![0.0; n];
    let mut rz = dot(&r, &z);
    let mut iterations = 0;
    for iter in 1..=15_000 {
        a.matvec(&p, &mut ap);
        let pap = dot(&p, &ap);
        if !pap.is_finite() || pap <= 0.0 {
            return Err("non-SPD, degenerate or ill-conditioned FEM matrix".into());
        }
        let alpha = rz / pap;
        for i in 0..n {
            x[i] += alpha * p[i];
            r[i] -= alpha * ap[i];
        }
        iterations = iter;
        if dot(&r, &r).sqrt() / bnorm <= 1e-11 {
            break;
        }
        for i in 0..n {
            z[i] = r[i] / a.diag[i];
        }
        let next_rz = dot(&r, &z);
        let beta = next_rz / rz;
        if !beta.is_finite() {
            return Err("nonfinite conjugate-gradient recurrence".into());
        }
        for i in 0..n {
            p[i] = z[i] + beta * p[i];
        }
        rz = next_rz;
    }
    a.matvec(&x, &mut ap);
    for i in 0..n {
        ap[i] -= b[i];
    }
    let relative = dot(&ap, &ap).sqrt() / bnorm;
    if !relative.is_finite() || relative > 1e-8 {
        return Err(format!("FEM failed true-residual tolerance: {relative:.6e}"));
    }
    Ok((x, iterations, relative))
}

fn solve_field(c: &IdealCoupon, grid: &Grid, dielectric_er: f64) -> Result<FieldEvidence, String> {
    solve_field_with_potential(c.dielectric_height_m, grid, dielectric_er).map(|(evidence, _)| evidence)
}

// Shared FE assembly, sparse CSR and CG solve. The separately constructed
// PH-03 grid is passed here without altering the microstrip grid conditions.
fn solve_field_with_potential(
    dielectric_height_m: f64,
    grid: &Grid,
    dielectric_er: f64,
) -> Result<(FieldEvidence, Vec<f64>), String> {
    solve_field_with_material_potential(grid, |z| {
        if z < dielectric_height_m { dielectric_er } else { 1.0 }
    })
}

// PH-06 uses a separate layer-permittivity function while sharing precisely
// the PH-05/G1 P1-CSR-CG assembly and conductor-reaction calculation.
// Material boundaries must be prevalidated as mesh-conforming by each geometry.
fn for_each_material_triangle(
    mut action: impl FnMut([usize; 3], f64),
    grid: &Grid,
    er_at_z: impl Fn(f64) -> f64,
) {
    for j in 0..grid.nz {
        let er = er_at_z((j as f64 + 0.5) * grid.dx);
        for i in 0..grid.nx {
            let bl = j * (grid.nx + 1) + i;
            let br = bl + 1;
            let tl = bl + grid.nx + 1;
            let tr = tl + 1;
            action([bl, br, tr], er);
            action([bl, tr, tl], er);
        }
    }
}

fn solve_field_with_material_potential(
    grid: &Grid,
    er_at_z: impl Fn(f64) -> f64,
) -> Result<(FieldEvidence, Vec<f64>), String> {
    let start = Instant::now();
    let mut rows = vec![Vec::<(usize, f64)>::new(); grid.dof];
    let mut rhs = vec![0.0; grid.dof];
    for_each_material_triangle(|tri, er| {
        let k = stiffness(tri, grid, er);
        for (i, ni) in tri.iter().enumerate() {
            let ri = grid.mapping[*ni];
            if ri == usize::MAX {
                continue;
            }
            for (j, nj) in tri.iter().enumerate() {
                let cj = grid.mapping[*nj];
                if cj == usize::MAX {
                    rhs[ri] -= k[i][j] * grid.dirichlet[*nj].unwrap_or(0.0);
                } else {
                    rows[ri].push((cj, k[i][j]));
                }
            }
        }
    }, grid, &er_at_z);

    let mut offsets = Vec::with_capacity(grid.dof + 1);
    let mut columns = Vec::new();
    let mut values = Vec::new();
    let mut diag = vec![0.0; grid.dof];
    offsets.push(0);
    for (ri, mut row) in rows.into_iter().enumerate() {
        row.sort_unstable_by_key(|item| item.0);
        let mut cursor = 0;
        while cursor < row.len() {
            let col = row[cursor].0;
            let mut val = 0.0;
            while cursor < row.len() && row[cursor].0 == col {
                val += row[cursor].1;
                cursor += 1;
            }
            if val != 0.0 {
                if col == ri {
                    diag[ri] = val;
                }
                columns.push(col);
                values.push(val);
            }
        }
        offsets.push(values.len());
    }
    if diag.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return Err("invalid nonpositive FEM diagonal".into());
    }
    let matrix = Csr { offsets, columns, values, diag };
    let (unknown, iterations, residual) = conjugate_gradient(&matrix, &rhs)?;
    let mut full = vec![0.0; grid.dirichlet.len()];
    for (i, value) in full.iter_mut().enumerate() {
        if let Some(v) = grid.dirichlet[i] {
            *value = v;
        } else {
            *value = unknown[grid.mapping[i]];
        }
    }

    // Energy and conductor-charge reaction are independent diagnostics on the
    // SAME FEM stiffness (not a separate external field oracle).
    let mut energy = 0.0;
    let mut signal_reaction = 0.0;
    for_each_material_triangle(|tri, er| {
        let k = stiffness(tri, grid, er);
        for i in 0..3 {
            let mut row_reaction = 0.0;
            for j in 0..3 {
                row_reaction += k[i][j] * full[tri[j]];
            }
            energy += full[tri[i]] * row_reaction;
            if grid.signal[tri[i]] {
                signal_reaction += row_reaction;
            }
        }
    }, grid, &er_at_z);
    let c_energy = EPS0 * energy;
    let c_charge = EPS0 * signal_reaction;
    if !c_energy.is_finite() || !c_charge.is_finite() || c_energy <= 0.0 || c_charge <= 0.0 {
        return Err("unphysical or nonfinite FEM charge/energy".into());
    }
    let difference = (c_charge / c_energy - 1.0).abs();
    if difference > 1e-6 {
        return Err(format!("field energy and charge disagree: {difference:.3e}"));
    }
    Ok((FieldEvidence {
        c_f_per_m: c_energy,
        c_from_conductor_charge_f_per_m: c_charge,
        charge_energy_relative_difference: difference,
        residual,
        iterations,
        dof: grid.dof,
        triangles: 2 * grid.nx * grid.nz,
        matrix_nnz: matrix.values.len(),
        elapsed_s: start.elapsed().as_secs_f64(),
    }, full))
}

pub fn extract_ideal(c: &IdealCoupon) -> Result<Extracted, String> {
    let grid = make_grid(c)?;
    let dielectric = solve_field(c, &grid, c.epsilon_r)?;
    let vacuum = solve_field(c, &grid, 1.0)?;
    let cap = dielectric.c_f_per_m;
    let cap0 = vacuum.c_f_per_m;
    if cap < cap0 * (1.0 - 1e-9) ||
       cap > cap0 * c.epsilon_r * (1.0 + 1e-9) {
        return Err("capacitance violates passive ideal dielectric bounds".into());
    }
    let lext = MU0 * EPS0 / cap0;
    let z0 = (lext / cap).sqrt();
    let delay = (lext * cap).sqrt();
    if !lext.is_finite() || !z0.is_finite() || !delay.is_finite() {
        return Err("invalid quasi-TEM lossless derived results".into());
    }
    Ok(Extracted {
        c_f_per_m: cap,
        c0_f_per_m: cap0,
        lext_h_per_m: lext,
        z0_ohm: z0,
        delay_s_per_m: delay,
        epsilon_effective: cap / cap0,
        dielectric,
        vacuum,
        physical_status: "PrototypeOnly_IdealPECBox_NotRealPCBValidated",
    })
}

pub fn analytic_ideal_s2(result: &Extracted, length_m: f64, frequency_hz: f64,
                         reference_ohm: f64) -> Result<LosslessS2, String> {
    if !length_m.is_finite() || !frequency_hz.is_finite() ||
       !reference_ohm.is_finite() || length_m < 0.0 || frequency_hz < 0.0 ||
       reference_ohm <= 0.0 {
        return Err("invalid length, frequency or real port impedance".into());
    }
    // Equal real physical 2-port references. This is *ideal* transmission-line
    // mathematics derived from synthetic lossless C,L, NOT a lossy PCB model.
    let theta = 2.0 * std::f64::consts::PI * frequency_hz * length_m * result.delay_s_per_m;
    let (sine, cosine) = theta.sin_cos();
    let ratio = result.z0_ohm / reference_ohm;
    let dr = 2.0 * cosine;
    let di = (ratio + 1.0 / ratio) * sine;
    let norm = dr * dr + di * di;
    if !norm.is_finite() || norm <= 0.0 {
        return Err("invalid scattering normalization".into());
    }
    let s21 = ComplexValue { re: 2.0 * dr / norm, im: -2.0 * di / norm };
    let nd = (ratio - 1.0 / ratio) * sine;
    let s11 = ComplexValue { re: nd * di / norm, im: nd * dr / norm };
    Ok(LosslessS2 { s11, s12: s21, s21, s22: s11 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal_vacuum_c_equals_c0_and_charge_matches_energy() {
        let mut specimen = IdealCoupon::synthetic(8, 0.0012);
        specimen.epsilon_r = 1.0;
        let out = extract_ideal(&specimen).expect("valid ideal vacuum coupon");
        assert!((out.c_f_per_m / out.c0_f_per_m - 1.0).abs() < 1e-11);
        assert!(out.dielectric.charge_energy_relative_difference < 1e-6);
        assert!(out.dielectric.residual < 1e-8);
    }

    #[test]
    fn analytic_lossless_s_is_passive_and_reciprocal() {
        let out = extract_ideal(&IdealCoupon::synthetic(8, 0.0012)).unwrap();
        let matched = analytic_ideal_s2(&out, 0.01, 1e9, out.z0_ohm).unwrap();
        assert!(matched.s11.re.abs() + matched.s11.im.abs() < 1e-12);
        let magnitude = matched.s21.re.powi(2) + matched.s21.im.powi(2);
        assert!((magnitude - 1.0).abs() < 1e-12);
        let mismatch = analytic_ideal_s2(&out, 0.01, 1e9, 50.0).unwrap();
        let unitary = mismatch.s11.re.powi(2) + mismatch.s11.im.powi(2)
            + mismatch.s21.re.powi(2) + mismatch.s21.im.powi(2);
        assert!((unitary - 1.0).abs() < 1e-10);
        assert_eq!(mismatch.s12.re, mismatch.s21.re);
        assert_eq!(mismatch.s22.im, mismatch.s11.im);
    }

    #[test]
    fn ph20_mesh_extremes_reject_without_panicking() {
        let valid = IdealCoupon::synthetic(8, 0.0012);
        let mut cases: Vec<(&str, IdealCoupon)> = Vec::new();
        for (name, value) in [
            ("huge finite box", 1e300),
            ("max finite box", f64::MAX),
            ("infinite box", f64::INFINITY),
            ("nan box", f64::NAN),
            ("negative box", -0.0012),
            ("zero box", 0.0),
            ("excessive finite mesh", 1.2),
            ("nonconforming finite mesh", 0.00121),
        ] {
            let mut specimen = valid;
            specimen.box_m = value;
            cases.push((name, specimen));
        }
        for (name, count) in [
            ("zero subdivisions", 0),
            ("too few subdivisions", 3),
            ("unsupported subdivision count", 5),
            ("excessive subdivisions", usize::MAX),
        ] {
            let mut specimen = valid;
            specimen.cells_per_height = count;
            cases.push((name, specimen));
        }
        for (name, value) in [
            ("negative width", -0.0003),
            ("zero thickness", 0.0),
            ("nan dielectric height", f64::NAN),
            ("infinite dielectric", f64::INFINITY),
            ("underflow mesh spacing", 1e-320),
        ] {
            let mut specimen = valid;
            match name {
                "negative width" => specimen.width_m = value,
                "zero thickness" => specimen.thickness_m = value,
                "nan dielectric height" | "underflow mesh spacing" =>
                    specimen.dielectric_height_m = value,
                _ => specimen.epsilon_r = value,
            }
            cases.push((name, specimen));
        }
        for (name, specimen) in cases {
            let attempted = std::panic::catch_unwind(|| extract_ideal(&specimen));
            assert!(attempted.is_ok(), "{name}: input triggered panic");
            let error = attempted.unwrap();
            assert!(error.is_err(), "{name}: invalid input was accepted");
            assert!(!error.unwrap_err().is_empty(), "{name}: missing diagnostic");
        }
    }

    #[test]
    fn invalid_real_or_missing_material_and_topology_rejected() {
        let good = IdealCoupon::synthetic(8, 0.0012);
        let mut invalid = good;
        invalid.epsilon_r = f64::NAN;
        assert!(extract_ideal(&invalid).is_err());
        invalid = good;
        invalid.epsilon_r = -1.0;
        assert!(extract_ideal(&invalid).is_err());
        invalid = good;
        invalid.explicitly_lossless = false;
        assert!(extract_ideal(&invalid).is_err());
        invalid = good;
        invalid.pec_signal = false;
        assert!(extract_ideal(&invalid).is_err());
        invalid = good;
        invalid.continuous_reference = false;
        assert!(extract_ideal(&invalid).is_err());
        invalid = good;
        invalid.longitudinally_uniform = false;
        assert!(extract_ideal(&invalid).is_err());
        invalid = good;
        invalid.box_m = 0.0002;
        assert!(extract_ideal(&invalid).is_err());
    }
}
