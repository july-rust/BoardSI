//! PH-07: synthetic lossless multilayer microstrip, with explicit rectangular solder mask.
//! Only 2D electrostatic PEC quasi-TEM; magnetic Lext is derived from vacuum C0.
use super::stripline::StriplineSide;
use super::{
    on_grid, solve_field_with_cell_material_potential, stiffness, xy, FieldEvidence, Grid, EPS0,
    MU0,
};

const MAX_NODES: usize = 240_000;

#[derive(Clone, Copy, Debug)]
pub struct DielectricLayer {
    pub height_m: f64,
    pub epsilon_r: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct Mask {
    pub thickness_m: f64,
    pub epsilon_r: f64,
}
#[derive(Clone, Debug)]
pub struct MicrostripCoupon {
    pub width_m: f64,
    pub copper_thickness_m: f64,
    pub layers: Vec<DielectricLayer>,
    pub air_epsilon_r: f64,
    pub mask: Option<Mask>,
    pub half_span_m: f64,
    pub top_m: f64,
    pub cells_per_height: usize,
    pub mu_r: f64,
    pub signal_v: f64,
    pub ground_v: f64,
    pub left: StriplineSide,
    pub right: StriplineSide,
    pub top_neumann: bool,
    pub signal_pec: bool,
    pub ground_pec: bool,
    pub uniform_y: bool,
    pub finite_conductivity_requested: bool,
    pub dielectric_loss_requested: bool,
    pub dispersion_requested: bool,
}
impl MicrostripCoupon {
    pub fn ph07(case: char, n: usize, half_span_m: f64, top_m: f64) -> Result<Self, String> {
        if !matches!(case, 'A' | 'B' | 'C' | 'D') {
            return Err("unsupported PH-07 case: must be A/B/C/D".into());
        }
        let mut layers = vec![DielectricLayer {
            height_m: 0.00020,
            epsilon_r: 3.4,
        }];
        if matches!(case, 'C' | 'D') {
            layers = vec![
                DielectricLayer {
                    height_m: 0.00010,
                    epsilon_r: 3.4,
                },
                DielectricLayer {
                    height_m: 0.00010,
                    epsilon_r: 4.0,
                },
            ];
        }
        Ok(Self {
            width_m: 0.00030,
            copper_thickness_m: 0.00005,
            layers,
            air_epsilon_r: 1.0,
            mask: if matches!(case, 'B' | 'D') {
                Some(Mask {
                    thickness_m: 0.000025,
                    epsilon_r: 3.3,
                })
            } else {
                None
            },
            half_span_m,
            top_m,
            cells_per_height: n,
            mu_r: 1.0,
            signal_v: 1.0,
            ground_v: 0.0,
            left: StriplineSide::HomogeneousNeumann,
            right: StriplineSide::HomogeneousNeumann,
            top_neumann: true,
            signal_pec: true,
            ground_pec: true,
            uniform_y: true,
            finite_conductivity_requested: false,
            dielectric_loss_requested: false,
            dispersion_requested: false,
        })
    }
    pub fn substrate_height_m(&self) -> f64 {
        self.layers.iter().map(|l| l.height_m).sum()
    }
    fn copper_at(&self, x: f64, z: f64) -> bool {
        x.abs() < self.width_m / 2.0
            && z >= self.substrate_height_m()
            && z < self.substrate_height_m() + self.copper_thickness_m
    }
    // Cell-centre classification. Index 0=air, 1..=n=substrate, n+1=mask.
    // None denotes a PEC-only cell and is excluded from dielectric assembly.
    fn region(&self, x: f64, z: f64) -> Option<usize> {
        if self.copper_at(x, z) {
            return None;
        }
        let mut bound = 0.0;
        for (k, layer) in self.layers.iter().enumerate() {
            bound += layer.height_m;
            if z < bound {
                return Some(k + 1);
            }
        }
        if let Some(mask) = self.mask {
            let h = self.substrate_height_m();
            let a = self.width_m / 2.0;
            let tm = mask.thickness_m;
            let blanket = z >= h && z < h + tm;
            let sides =
                x.abs() >= a && x.abs() < a + tm && z >= h && z < h + self.copper_thickness_m;
            let top = x.abs() < a + tm
                && z >= h + self.copper_thickness_m
                && z < h + self.copper_thickness_m + tm;
            if blanket || sides || top {
                return Some(self.layers.len() + 1);
            }
        }
        Some(0)
    }
    fn er_for_region(&self, id: usize, vacuum: bool) -> f64 {
        if vacuum {
            return 1.0;
        }
        if id == 0 {
            self.air_epsilon_r
        } else if id <= self.layers.len() {
            self.layers[id - 1].epsilon_r
        } else {
            self.mask.map_or(1.0, |m| m.epsilon_r)
        }
    }
}

#[derive(Clone, Debug)]
pub struct MicrostripField {
    pub evidence: FieldEvidence,
    pub signal_charge_c_per_m: f64,
    pub ground_charge_c_per_m: f64,
    pub charge_balance_relative: f64,
    pub max_x_reflection_error_v: f64,
    pub max_pec_potential_error_v: f64,
    pub max_fringe_ex_v_per_m: f64,
    pub max_abs_ez_v_per_m: f64,
    pub sampled_fields: Vec<[f64; 6]>,
    pub max_pec_interior_field_v_per_m: f64,
    // The summands are contributions to C at a 1 V electrode excitation,
    // NOT independent per-layer line capacitances or charges.
    pub region_energy_capacitance_f_per_m: Vec<f64>,
}
#[derive(Clone, Debug)]
pub struct MicrostripResults {
    pub c_f_per_m: f64,
    pub c0_f_per_m: f64,
    pub lext_h_per_m: f64,
    pub z0_ohm: f64,
    pub delay_s_per_m: f64,
    pub epsilon_effective: f64,
    pub dx_m: f64,
    pub nx: usize,
    pub nz: usize,
    pub dielectric: MicrostripField,
    pub vacuum: MicrostripField,
    pub physical_status: &'static str,
}

fn build_grid(c: &MicrostripCoupon) -> Result<Grid, String> {
    let mut values = vec![
        c.width_m,
        c.copper_thickness_m,
        c.air_epsilon_r,
        c.half_span_m,
        c.top_m,
        c.mu_r,
        c.signal_v,
        c.ground_v,
    ];
    for l in &c.layers {
        values.extend([l.height_m, l.epsilon_r]);
    }
    if let Some(m) = c.mask {
        values.extend([m.thickness_m, m.epsilon_r]);
    }
    if values.iter().any(|x| !x.is_finite())
        || c.width_m <= 0.0
        || c.copper_thickness_m <= 0.0
        || c.half_span_m <= 0.0
        || c.top_m <= 0.0
        || c.air_epsilon_r <= 0.0
        || c.layers.is_empty()
        || c.layers
            .iter()
            .any(|l| l.height_m <= 0.0 || l.epsilon_r <= 0.0)
        || c.mask
            .is_some_and(|m| m.thickness_m <= 0.0 || m.epsilon_r <= 0.0)
    {
        return Err("PH-07 invalid/nonfinite geometry or dielectric".into());
    }
    if c.mu_r != 1.0
        || c.signal_v != 1.0
        || c.ground_v != 0.0
        || c.left != StriplineSide::HomogeneousNeumann
        || c.right != StriplineSide::HomogeneousNeumann
        || !c.top_neumann
        || !c.signal_pec
        || !c.ground_pec
        || !c.uniform_y
        || c.finite_conductivity_requested
        || c.dielectric_loss_requested
        || c.dispersion_requested
    {
        return Err("PH-07 unsupported: requires lossless, nondispersive PEC, mu=1, natural top/sides, fixed 1V/0V".into());
    }
    if c.cells_per_height < 16 || c.cells_per_height > 256 || c.cells_per_height % 16 != 0 {
        return Err("PH-07 unsupported or excessive mesh size".into());
    }
    let h = c.substrate_height_m();
    let upper = h + c.copper_thickness_m + c.mask.map_or(0.0, |m| m.thickness_m);
    if !h.is_finite()
        || h <= 0.0
        || upper >= c.top_m
        || c.width_m / 2.0 + c.mask.map_or(0.0, |m| m.thickness_m) >= c.half_span_m
    {
        return Err("PH-07 missing air region, out-of-domain copper/mask or PEC short".into());
    }
    let dx = c.top_m / c.cells_per_height as f64;
    let nx_f = 2.0 * c.half_span_m / dx;
    if !dx.is_finite() || dx <= 0.0 || !nx_f.is_finite() || nx_f < 2.0 || nx_f >= MAX_NODES as f64 {
        return Err("PH-07 invalid or oversized mesh axes".into());
    }
    let mut boundaries = vec![
        c.width_m / 2.0,
        h,
        h + c.copper_thickness_m,
        c.copper_thickness_m,
        c.half_span_m,
        2.0 * c.half_span_m,
    ];
    let mut layer_z = 0.0;
    for l in &c.layers {
        layer_z += l.height_m;
        boundaries.push(layer_z);
    }
    if let Some(m) = c.mask {
        boundaries.extend([
            m.thickness_m,
            c.width_m / 2.0 + m.thickness_m,
            h + m.thickness_m,
            h + c.copper_thickness_m + m.thickness_m,
        ]);
    }
    if boundaries.iter().any(|&b| !on_grid(b, dx)) {
        return Err(
            "PH-07 nonconforming copper/substrate/mask/box interface; snapping forbidden".into(),
        );
    }
    let nx = nx_f.round() as usize;
    let nz = c.cells_per_height;
    let cols = nx.checked_add(1).ok_or("PH-07 mesh columns overflow")?;
    let rows = nz.checked_add(1).ok_or("PH-07 mesh rows overflow")?;
    let count = cols
        .checked_mul(rows)
        .ok_or("PH-07 mesh node-count overflow")?;
    if count > MAX_NODES {
        return Err("PH-07 mesh exceeds memory guard (240000 nodes)".into());
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
                && z >= h - dx * 1e-8
                && z <= h + c.copper_thickness_m + dx * 1e-8;
            if metal && (j == 0 || j == nz || i == 0 || i == nx) {
                return Err("PH-07 copper shorts to reference/domain edge".into());
            }
            if j == 0 {
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
        return Err("PH-07 missing excitation or field DOF".into());
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

fn field(grid: &Grid, c: &MicrostripCoupon, vacuum: bool) -> Result<MicrostripField, String> {
    let er_at_cell = |i: usize, j: usize| {
        let x = -c.half_span_m + (i as f64 + 0.5) * grid.dx;
        let z = (j as f64 + 0.5) * grid.dx;
        c.region(x, z).map_or(0.0, |r| c.er_for_region(r, vacuum))
    };
    let (evidence, phi) = solve_field_with_cell_material_potential(grid, er_at_cell)?;
    let cols = grid.nx + 1;
    let mut x_error: f64 = 0.0;
    let mut pec_error: f64 = 0.0;
    for j in 0..=grid.nz {
        for i in 0..=grid.nx {
            let idx = j * cols + i;
            x_error = x_error.max((phi[idx] - phi[j * cols + grid.nx - i]).abs());
            if let Some(v) = grid.dirichlet[idx] {
                pec_error = pec_error.max((phi[idx] - v).abs());
            }
        }
    }
    let mut ground_reaction = 0.0;
    let mut fringe: f64 = 0.0;
    let mut max_abs_ez: f64 = 0.0;
    let mut samples = Vec::<[f64; 6]>::new();
    let mut pec_field: f64 = 0.0;
    let mut region_c = vec![0.0; c.layers.len() + 2];
    for j in 0..grid.nz {
        for i in 0..grid.nx {
            let x = -c.half_span_m + (i as f64 + 0.5) * grid.dx;
            let z = (j as f64 + 0.5) * grid.dx;
            let Some(region) = c.region(x, z) else {
                continue;
            };
            let er = c.er_for_region(region, vacuum);
            let bl = j * cols + i;
            let br = bl + 1;
            let tl = bl + cols;
            let tr = tl + 1;
            for (triangle_index, tri) in [[bl, br, tr], [bl, tr, tl]].into_iter().enumerate() {
                let (x0, z0) = xy(tri[0], grid);
                let (x1, z1) = xy(tri[1], grid);
                let (x2, z2) = xy(tri[2], grid);
                let det = (x1 - x0) * (z2 - z0) - (x2 - x0) * (z1 - z0);
                let v = [phi[tri[0]], phi[tri[1]], phi[tri[2]]];
                let ex = -(v[0] * (z1 - z2) + v[1] * (z2 - z0) + v[2] * (z0 - z1)) / det;
                let ez = -(v[0] * (x2 - x1) + v[1] * (x0 - x2) + v[2] * (x1 - x0)) / det;
                max_abs_ez = max_abs_ez.max(ez.abs());
                // Actual element centroids and piecewise-constant P1 fields,
                // sparsely sampled for reproducible 2D field-map diagnostics.
                if triangle_index == 0
                    && i % (grid.nx / 8).max(1) == 0
                    && j % (grid.nz / 8).max(1) == 0
                {
                    samples.push([
                        (x0 + x1 + x2) / 3.0,
                        (z0 + z1 + z2) / 3.0,
                        (v[0] + v[1] + v[2]) / 3.0,
                        ex,
                        ez,
                        region as f64,
                    ]);
                }
                if x.abs() > c.width_m / 2.0
                    && z > c.substrate_height_m() / 2.0
                    && z < c.substrate_height_m() + c.copper_thickness_m * 2.0
                {
                    fringe = fringe.max(ex.abs());
                }
                if tri.iter().all(|&id| grid.signal[id]) {
                    pec_field = pec_field.max(ex.hypot(ez));
                }
                let k = stiffness(tri, grid, er);
                for a in 0..3 {
                    let r: f64 = (0..3).map(|b| k[a][b] * v[b]).sum();
                    region_c[region] += EPS0 * v[a] * r;
                    if tri[a] / cols == 0 {
                        ground_reaction += r;
                    }
                }
            }
        }
    }
    let q_signal = evidence.c_from_conductor_charge_f_per_m;
    let q_ground = EPS0 * ground_reaction;
    let balance = ((q_signal + q_ground) / q_signal).abs();
    if ![
        x_error, pec_error, fringe, max_abs_ez, pec_field, balance, q_ground,
    ]
    .iter()
    .all(|v| v.is_finite())
    {
        return Err("PH-07 nonfinite field or charge".into());
    }
    Ok(MicrostripField {
        evidence,
        signal_charge_c_per_m: q_signal,
        ground_charge_c_per_m: q_ground,
        charge_balance_relative: balance,
        max_x_reflection_error_v: x_error,
        max_pec_potential_error_v: pec_error,
        max_fringe_ex_v_per_m: fringe,
        max_abs_ez_v_per_m: max_abs_ez,
        sampled_fields: samples,
        max_pec_interior_field_v_per_m: pec_field,
        region_energy_capacitance_f_per_m: region_c,
    })
}

pub fn extract_microstrip(c: &MicrostripCoupon) -> Result<MicrostripResults, String> {
    let grid = build_grid(c)?;
    let dielectric = field(&grid, c, false)?;
    let vacuum = field(&grid, c, true)?;
    let cap = dielectric.evidence.c_f_per_m;
    let cap0 = vacuum.evidence.c_f_per_m;
    let max_er = c
        .layers
        .iter()
        .map(|l| l.epsilon_r)
        .fold(c.air_epsilon_r, f64::max)
        .max(c.mask.map_or(1.0, |m| m.epsilon_r));
    let min_er = c
        .layers
        .iter()
        .map(|l| l.epsilon_r)
        .fold(c.air_epsilon_r, f64::min)
        .min(c.mask.map_or(1.0, |m| m.epsilon_r));
    if cap0 <= 0.0 || cap < cap0 * min_er * (1.0 - 1e-8) || cap > cap0 * max_er * (1.0 + 1e-8) {
        return Err("PH-07 effective epsilon violates material bounds".into());
    }
    let lext = MU0 * EPS0 / cap0;
    let z0 = (lext / cap).sqrt();
    let delay = (lext * cap).sqrt();
    if ![lext, z0, delay].iter().all(|v| v.is_finite() && *v > 0.0) {
        return Err("PH-07 invalid derived quasi-TEM quantities".into());
    }
    Ok(MicrostripResults {
        c_f_per_m: cap,
        c0_f_per_m: cap0,
        lext_h_per_m: lext,
        z0_ohm: z0,
        delay_s_per_m: delay,
        epsilon_effective: cap / cap0,
        dx_m: grid.dx,
        nx: grid.nx,
        nz: grid.nz,
        dielectric,
        vacuum,
        physical_status: "PrototypeOnly_IdealMultilayerMaskedMicrostrip_NotPCBValidated",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ph07_pec_exclusion_and_natural_boundary_topology() {
        let c = MicrostripCoupon::ph07('D', 32, 0.0012, 0.0008).unwrap();
        let g = build_grid(&c).unwrap();
        let cols = g.nx + 1;
        assert_eq!(g.dirichlet[0], Some(0.0));
        assert_eq!(g.dirichlet[g.nz * cols], None);
        assert_eq!(g.dirichlet[cols], None);
        let mut included = 0;
        super::super::for_each_cell_material_triangle(|_, _| included += 1, &g, &|i, j| {
            let x = -c.half_span_m + (i as f64 + 0.5) * g.dx;
            let z = (j as f64 + 0.5) * g.dx;
            c.region(x, z).map_or(0.0, |r| c.er_for_region(r, false))
        });
        let copper_cells =
            (c.width_m / g.dx).round() as usize * (c.copper_thickness_m / g.dx).round() as usize;
        assert_eq!(included, 2 * (g.nx * g.nz - copper_cells));
        for id in 0..g.signal.len() {
            if g.signal[id] {
                assert_eq!(g.mapping[id], usize::MAX);
                assert_eq!(g.dirichlet[id], Some(1.0));
            }
        }
    }
    #[test]
    fn ph07_a_to_d_basic_physics() {
        for case in ['A', 'B', 'C', 'D'] {
            let c = MicrostripCoupon::ph07(case, 32, 0.0012, 0.0008).unwrap();
            let r = extract_microstrip(&c).unwrap();
            assert!(r.c_f_per_m > r.c0_f_per_m);
            assert!(r.dielectric.charge_balance_relative < 1e-6);
            assert!(r.vacuum.charge_balance_relative < 1e-6);
            assert!(r.dielectric.max_x_reflection_error_v < 1e-6);
            assert!(r.dielectric.max_pec_potential_error_v < 1e-12);
            assert!(r.dielectric.max_fringe_ex_v_per_m > 0.0);
        }
    }
    #[test]
    fn ph07_mask_and_equal_interface_invariants() {
        let a = MicrostripCoupon::ph07('A', 32, 0.0012, 0.0008).unwrap();
        let b = MicrostripCoupon::ph07('B', 32, 0.0012, 0.0008).unwrap();
        let ra = extract_microstrip(&a).unwrap();
        let rb = extract_microstrip(&b).unwrap();
        assert!(rb.c_f_per_m > ra.c_f_per_m);
        assert!((rb.c0_f_per_m / ra.c0_f_per_m - 1.0).abs() < 1e-8);
        let mut equal = MicrostripCoupon::ph07('C', 32, 0.0012, 0.0008).unwrap();
        equal.layers[1].epsilon_r = 3.4;
        let re = extract_microstrip(&equal).unwrap();
        assert!((re.c_f_per_m / ra.c_f_per_m - 1.0).abs() < 1e-8);
    }
    #[test]
    fn ph07_reject_bad_geometry_and_unsupported_physics() {
        let base = MicrostripCoupon::ph07('D', 32, 0.0012, 0.0008).unwrap();
        let mut cases = Vec::new();
        let mut c = base.clone();
        c.width_m = 0.0;
        cases.push(c);
        let mut c = base.clone();
        c.copper_thickness_m = -1.0;
        cases.push(c);
        let mut c = base.clone();
        c.air_epsilon_r = 0.0;
        cases.push(c);
        let mut c = base.clone();
        c.half_span_m = f64::NAN;
        cases.push(c);
        let mut c = base.clone();
        c.top_m = 0.00025;
        cases.push(c);
        let mut c = base.clone();
        c.mu_r = 1.01;
        cases.push(c);
        let mut c = base.clone();
        c.left = StriplineSide::GroundedDirichlet;
        cases.push(c);
        let mut c = base.clone();
        c.top_neumann = false;
        cases.push(c);
        let mut c = base.clone();
        c.finite_conductivity_requested = true;
        cases.push(c);
        let mut c = base.clone();
        c.dielectric_loss_requested = true;
        cases.push(c);
        let mut c = base.clone();
        c.dispersion_requested = true;
        cases.push(c);
        let mut c = base.clone();
        c.mask = Some(Mask {
            thickness_m: 0.000021,
            epsilon_r: 3.3,
        });
        cases.push(c);
        let mut c = base.clone();
        c.layers[0].height_m = 0.0001001;
        cases.push(c);
        let mut c = base.clone();
        c.layers[0].epsilon_r = f64::INFINITY;
        cases.push(c);
        let mut c = base.clone();
        c.mask = None;
        c.layers.clear();
        cases.push(c);
        let mut c = base.clone();
        c.cells_per_height = usize::MAX;
        cases.push(c);
        let mut c = base;
        c.half_span_m = 1e300;
        cases.push(c);
        for (i, c) in cases.iter().enumerate() {
            let result = std::panic::catch_unwind(|| extract_microstrip(c));
            assert!(result.is_ok(), "case {i} panicked");
            assert!(result.unwrap().is_err(), "case {i} should reject");
        }
    }
}
