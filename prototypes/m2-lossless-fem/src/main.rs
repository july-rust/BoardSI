use boardsi_m2_lossless_fem::multilayer_microstrip::{extract_microstrip, Mask, MicrostripCoupon};

use boardsi_m2_lossless_fem::asymmetric_stripline::{extract_asymmetric, AsymmetricCoupon};
use boardsi_m2_lossless_fem::plate::{extract_plate, PlateCoupon};
use boardsi_m2_lossless_fem::stripline::{extract_stripline, StriplineCoupon};
use boardsi_m2_lossless_fem::{analytic_ideal_s2, extract_ideal, IdealCoupon};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    // PH-07 opt-in. Legacy PH-03, PH-05, PH-06 and default JSON remain unchanged.
    // microstrip <A|B|C|D> <n> <B_m> <H_m> [mask_thickness_m mask_er eps_bottom eps_top]
    if args.len() > 1 && args[1] == "microstrip" {
        if args.len() != 6 && args.len() != 10 {
            eprintln!("usage: boardsi-m2-lossless-fem microstrip <A|B|C|D> <n> <B_m> <H_m> [mask_t_m mask_er eps_bottom eps_top]");
            return ExitCode::FAILURE;
        }
        let name = match args[2].as_str() {
            "A" => 'A',
            "B" => 'B',
            "C" => 'C',
            "D" => 'D',
            _ => {
                eprintln!("PH-07 invalid case (A/B/C/D)");
                return ExitCode::FAILURE;
            }
        };
        let n: usize = match args[3].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("PH-07 invalid mesh");
                return ExitCode::FAILURE;
            }
        };
        let b: f64 = match args[4].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("PH-07 invalid B");
                return ExitCode::FAILURE;
            }
        };
        let h: f64 = match args[5].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("PH-07 invalid H");
                return ExitCode::FAILURE;
            }
        };
        let mut c = match MicrostripCoupon::ph07(name, n, b, h) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("PH-07 {e}");
                return ExitCode::FAILURE;
            }
        };
        if args.len() == 10 {
            let p: Vec<f64> = match args[6..10]
                .iter()
                .map(|v| v.parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("PH-07 invalid custom material values");
                    return ExitCode::FAILURE;
                }
            };
            if c.mask.is_some() {
                c.mask = Some(Mask {
                    thickness_m: p[0],
                    epsilon_r: p[1],
                });
            } else if p[0] != 0.0 || p[1] != 1.0 {
                eprintln!("PH-07 mask OFF requires zero thickness and epsilon=1 in custom args");
                return ExitCode::FAILURE;
            }
            if c.layers.len() == 1 && p[2] != p[3] {
                eprintln!("PH-07 single layer requires identical custom bottom/top epsilon");
                return ExitCode::FAILURE;
            }
            c.layers[0].epsilon_r = p[2];
            if c.layers.len() > 1 {
                c.layers[1].epsilon_r = p[3];
            }
        }
        return match extract_microstrip(&c) {
            Ok(o) => {
                let d = &o.dielectric;
                let v = &o.vacuum;
                let layers = c
                    .layers
                    .iter()
                    .map(|l| {
                        format!(
                            r#"{{"height_m":{:.16e},"epsilon_r":{:.16e}}}"#,
                            l.height_m, l.epsilon_r
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let mask = c
                    .mask
                    .map(|m| {
                        format!(
                            r#"{{"thickness_m":{:.16e},"epsilon_r":{:.16e}}}"#,
                            m.thickness_m, m.epsilon_r
                        )
                    })
                    .unwrap_or_else(|| "null".to_string());
                let regions = d
                    .region_energy_capacitance_f_per_m
                    .iter()
                    .map(|e| format!("{e:.16e}"))
                    .collect::<Vec<_>>()
                    .join(",");
                let samples = d
                    .sampled_fields
                    .iter()
                    .map(|s| {
                        format!(
                            "[{:.9e},{:.9e},{:.9e},{:.9e},{:.9e},{:.0}]",
                            s[0], s[1], s[2], s[3], s[4], s[5]
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    concat!(
                        r#"{{"model":"PH07_IdealMultilayerMicrostrip","case":"{}","n":{},"B_m":{:.16e},"H_m":{:.16e},"#,
                        r#""width_m":{:.16e},"copper_thickness_m":{:.16e},"air_epsilon_r":{:.16e},"layers":[{}],"mask":{},"#,
                        r#""bottom_ground_v":0.0,"signal_v":1.0,"top_boundary":"Neumann","side_boundary":"Neumann","#,
                        r#""C_F_per_m":{:.16e},"C0_F_per_m":{:.16e},"Lext_H_per_m":{:.16e},"Z0_ohm":{:.16e},"delay_s_per_m":{:.16e},"epsilon_effective":{:.16e},"#,
                        r#""dx_m":{:.16e},"nx":{},"nz":{},"nodes":{},"dof":{},"triangles":{},"nnz":{},"#,
                        r#""field_residual":{:.16e},"vacuum_residual":{:.16e},"field_iterations":{},"vacuum_iterations":{},"#,
                        r#""field_wall_s":{:.9},"vacuum_wall_s":{:.9},"field_charge_C":{:.16e},"vacuum_charge_C":{:.16e},"#,
                        r#""field_ground_C":{:.16e},"vacuum_ground_C":{:.16e},"field_charge_balance_rel":{:.16e},"vacuum_charge_balance_rel":{:.16e},"#,
                        r#""field_energy_charge_rel":{:.16e},"vacuum_energy_charge_rel":{:.16e},"#,
                        r#""field_x_sym_err_V":{:.16e},"vacuum_x_sym_err_V":{:.16e},"field_pec_err_V":{:.16e},"vacuum_pec_err_V":{:.16e},"#,
                        r#""field_fringe_Ex_V_per_m":{:.16e},"vacuum_fringe_Ex_V_per_m":{:.16e},"#,
                        r#""field_pec_field_V_per_m":{:.16e},"field_max_Ez_V_per_m":{:.16e},"region_energy_C_contributions_F_per_m":[{}],"sampled_fields":[{}],"status":"{}"}}"#
                    ),
                    name,
                    n,
                    c.half_span_m,
                    c.top_m,
                    c.width_m,
                    c.copper_thickness_m,
                    c.air_epsilon_r,
                    layers,
                    mask,
                    o.c_f_per_m,
                    o.c0_f_per_m,
                    o.lext_h_per_m,
                    o.z0_ohm,
                    o.delay_s_per_m,
                    o.epsilon_effective,
                    o.dx_m,
                    o.nx,
                    o.nz,
                    (o.nx + 1) * (o.nz + 1),
                    d.evidence.dof,
                    d.evidence.triangles,
                    d.evidence.matrix_nnz,
                    d.evidence.residual,
                    v.evidence.residual,
                    d.evidence.iterations,
                    v.evidence.iterations,
                    d.evidence.elapsed_s,
                    v.evidence.elapsed_s,
                    d.signal_charge_c_per_m,
                    v.signal_charge_c_per_m,
                    d.ground_charge_c_per_m,
                    v.ground_charge_c_per_m,
                    d.charge_balance_relative,
                    v.charge_balance_relative,
                    d.evidence.charge_energy_relative_difference,
                    v.evidence.charge_energy_relative_difference,
                    d.max_x_reflection_error_v,
                    v.max_x_reflection_error_v,
                    d.max_pec_potential_error_v,
                    v.max_pec_potential_error_v,
                    d.max_fringe_ex_v_per_m,
                    v.max_fringe_ex_v_per_m,
                    d.max_pec_interior_field_v_per_m,
                    d.max_abs_ez_v_per_m,
                    regions,
                    samples,
                    o.physical_status
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("unsupported or numerical failure: {e}");
                ExitCode::FAILURE
            }
        };
    }

    // PH-06 is opt-in and never changes earlier CLI output or defaults.
    // asymmetric n B [zc epsilon_bottom epsilon_top interface_z|none]
    if args.len() > 1 && args[1] == "asymmetric" {
        if args.len() != 4 && args.len() != 8 {
            eprintln!("usage: boardsi-m2-lossless-fem asymmetric <n> <B_m> [zc_m epsilon_bottom epsilon_top interface_m|none]");
            return ExitCode::FAILURE;
        }
        let n: usize = match args[2].parse() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("invalid PH-06 n");
                return ExitCode::FAILURE;
            }
        };
        let span: f64 = match args[3].parse() {
            Ok(b) => b,
            Err(_) => {
                eprintln!("invalid PH-06 half-span");
                return ExitCode::FAILURE;
            }
        };
        let mut c = AsymmetricCoupon::ph06(n, span);
        if args.len() == 8 {
            let v: Vec<f64> = match args[4..7]
                .iter()
                .map(|v| v.parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("invalid PH-06 geometry or dielectric");
                    return ExitCode::FAILURE;
                }
            };
            c.center_z_m = v[0];
            c.epsilon_bottom = v[1];
            c.epsilon_top = v[2];
            c.interface_z_m = if args[7] == "none" {
                None
            } else {
                match args[7].parse::<f64>() {
                    Ok(v) => Some(v),
                    Err(_) => {
                        eprintln!("invalid PH-06 interface");
                        return ExitCode::FAILURE;
                    }
                }
            };
        }
        return match extract_asymmetric(&c) {
            Ok(o) => {
                let d = &o.dielectric;
                let v = &o.vacuum;
                let interface = c
                    .interface_z_m
                    .map(|z| format!("{z:.16e}"))
                    .unwrap_or_else(|| "null".to_string());
                println!(
                    concat!(
                        r#"{{"model":"PH06_IdealAsymmetricStripline","n":{},"B_m":{:.16e},"w_m":{:.16e},"t_m":{:.16e},"b_m":{:.16e},"zc_m":{:.16e},"lower_gap_m":{:.16e},"upper_gap_m":{:.16e},"epsilon_bottom":{:.16e},"epsilon_top":{:.16e},"interface_z_m":{},"#,
                        r#""C_F_per_m":{:.16e},"C0_F_per_m":{:.16e},"Lext_H_per_m":{:.16e},"Z0_ohm":{:.16e},"delay_s_per_m":{:.16e},"epsilon_effective":{:.16e},"#,
                        r#""nx":{},"nz":{},"nodes":{},"dof":{},"triangles":{},"nnz":{},"field_residual":{:.16e},"vacuum_residual":{:.16e},"#,
                        r#""field_charge_energy_rel":{:.16e},"vacuum_charge_energy_rel":{:.16e},"field_charge_C":{:.16e},"vacuum_charge_C":{:.16e},"#,
                        r#""field_ground_top_C":{:.16e},"field_ground_bottom_C":{:.16e},"vacuum_ground_top_C":{:.16e},"vacuum_ground_bottom_C":{:.16e},"#,
                        r#""field_charge_balance_rel":{:.16e},"vacuum_charge_balance_rel":{:.16e},"field_x_sym_err_V":{:.16e},"vacuum_x_sym_err_V":{:.16e},"#,
                        r#""field_z_sym_err_V":{:.16e},"field_pec_err_V":{:.16e},"vacuum_pec_err_V":{:.16e},"field_fringe_Ex_V_per_m":{:.16e},"field_pec_field_V_per_m":{:.16e},"#,
                        r#""field_iterations":{},"vacuum_iterations":{},"field_wall_s":{:.9},"vacuum_wall_s":{:.9},"status":"{}"}}"#
                    ),
                    n,
                    c.half_span_m,
                    c.width_m,
                    c.thickness_m,
                    c.ground_separation_m,
                    c.center_z_m,
                    c.lower_gap_m(),
                    c.upper_gap_m(),
                    c.epsilon_bottom,
                    c.epsilon_top,
                    interface,
                    o.c_f_per_m,
                    o.c0_f_per_m,
                    o.lext_h_per_m,
                    o.z0_ohm,
                    o.delay_s_per_m,
                    o.epsilon_effective,
                    o.nx,
                    o.nz,
                    (o.nx + 1) * (o.nz + 1),
                    d.evidence.dof,
                    d.evidence.triangles,
                    d.evidence.matrix_nnz,
                    d.evidence.residual,
                    v.evidence.residual,
                    d.evidence.charge_energy_relative_difference,
                    v.evidence.charge_energy_relative_difference,
                    d.signal_charge_c_per_m,
                    v.signal_charge_c_per_m,
                    d.upper_ground_charge_c_per_m,
                    d.lower_ground_charge_c_per_m,
                    v.upper_ground_charge_c_per_m,
                    v.lower_ground_charge_c_per_m,
                    d.charge_balance_relative,
                    v.charge_balance_relative,
                    d.max_x_reflection_error_v,
                    v.max_x_reflection_error_v,
                    d.max_z_reflection_error_v,
                    d.max_electrode_potential_error_v,
                    v.max_electrode_potential_error_v,
                    d.max_fringe_ex_v_per_m,
                    d.max_pec_interior_field_v_per_m,
                    d.evidence.iterations,
                    v.evidence.iterations,
                    d.evidence.elapsed_s,
                    v.evidence.elapsed_s,
                    o.physical_status
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("unsupported or numerical failure: {e}");
                ExitCode::FAILURE
            }
        };
    }

    // PH-05 is strictly opt-in. Neither the microstrip nor PH-03 CLI is modified.
    // args: stripline <n> <half-span-metres> [width_m thickness_m ground_gap_m epsilon_r]
    if args.len() > 1 && args[1] == "stripline" {
        if args.len() != 4 && args.len() != 8 {
            eprintln!("usage: boardsi-m2-lossless-fem stripline <n> <B_m> [w_m t_m b_m epsilon_r]");
            return ExitCode::FAILURE;
        }
        let n: usize = match args[2].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("invalid PH-05 integer subdivision");
                return ExitCode::FAILURE;
            }
        };
        let span: f64 = match args[3].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("invalid PH-05 lateral half-span");
                return ExitCode::FAILURE;
            }
        };
        let mut coupon = StriplineCoupon::ph05(n, span);
        if args.len() == 8 {
            let custom: Vec<f64> = match args[4..8]
                .iter()
                .map(|v| v.parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("invalid PH-05 geometry/material float");
                    return ExitCode::FAILURE;
                }
            };
            coupon.width_m = custom[0];
            coupon.thickness_m = custom[1];
            coupon.ground_separation_m = custom[2];
            coupon.center_z_m = custom[2] / 2.0;
            coupon.epsilon_r = custom[3];
        }
        return match extract_stripline(&coupon) {
            Ok(o) => {
                let d = &o.dielectric;
                let v = &o.vacuum;
                println!(
                    concat!(
                        "{{\"model\":\"PH05_IdealSymmetricStripline\",",
                        "\"n\":{},\"B_m\":{:.12e},\"w_m\":{:.12e},\"t_m\":{:.12e},",
                        "\"b_m\":{:.12e},\"epsilon_r\":{:.12e},",
                        "\"C_F_per_m\":{:.16e},\"C0_F_per_m\":{:.16e},",
                        "\"Lext_H_per_m\":{:.16e},\"Z0_ohm\":{:.16e},",
                        "\"delay_s_per_m\":{:.16e},\"epsilon_effective\":{:.12e},",
                        "\"nx\":{},\"nz\":{},\"nodes\":{},\"dof\":{},\"triangles\":{},",
                        "\"nnz\":{},\"field_residual\":{:.12e},\"vacuum_residual\":{:.12e},",
                        "\"field_charge_C\":{:.16e},\"vacuum_charge_C\":{:.16e},",
                        "\"field_energy_charge_rel\":{:.12e},\"vacuum_energy_charge_rel\":{:.12e},",
                        "\"field_ground_top_C\":{:.16e},\"field_ground_bottom_C\":{:.16e},",
                        "\"field_charge_balance_rel\":{:.12e},",
                        "\"vacuum_charge_balance_rel\":{:.12e},",
                        "\"field_ground_asym_rel\":{:.12e},\"vacuum_ground_asym_rel\":{:.12e},",
                        "\"field_z_sym_err_V\":{:.12e},\"field_x_sym_err_V\":{:.12e},",
                        "\"vacuum_z_sym_err_V\":{:.12e},\"vacuum_x_sym_err_V\":{:.12e},",
                        "\"field_fringe_Ex_V_per_m\":{:.12e},",
                        "\"field_pec_field_V_per_m\":{:.12e},",
                        "\"field_iterations\":{},\"vacuum_iterations\":{},",
                        "\"field_wall_s\":{:.9},\"vacuum_wall_s\":{:.9},",
                        "\"status\":\"{}\"}}"
                    ),
                    n,
                    coupon.half_span_m,
                    coupon.width_m,
                    coupon.thickness_m,
                    coupon.ground_separation_m,
                    coupon.epsilon_r,
                    o.c_f_per_m,
                    o.c0_f_per_m,
                    o.lext_h_per_m,
                    o.z0_ohm,
                    o.delay_s_per_m,
                    o.epsilon_effective,
                    o.nx,
                    o.nz,
                    (o.nx + 1) * (o.nz + 1),
                    d.evidence.dof,
                    d.evidence.triangles,
                    d.evidence.matrix_nnz,
                    d.evidence.residual,
                    v.evidence.residual,
                    d.signal_charge_c_per_m,
                    v.signal_charge_c_per_m,
                    d.evidence.charge_energy_relative_difference,
                    v.evidence.charge_energy_relative_difference,
                    d.upper_ground_charge_c_per_m,
                    d.lower_ground_charge_c_per_m,
                    d.charge_balance_relative,
                    v.charge_balance_relative,
                    d.ground_asymmetry_relative,
                    v.ground_asymmetry_relative,
                    d.max_z_reflection_error_v,
                    d.max_x_reflection_error_v,
                    v.max_z_reflection_error_v,
                    v.max_x_reflection_error_v,
                    d.max_fringe_ex_v_per_m,
                    d.max_pec_interior_field_v_per_m,
                    d.evidence.iterations,
                    v.evidence.iterations,
                    d.evidence.elapsed_s,
                    v.evidence.elapsed_s,
                    o.physical_status
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("unsupported or numerical failure: {error}");
                ExitCode::FAILURE
            }
        };
    }

    // New PH-03 mode is opt-in. The original <n> <box_m> microstrip CLI
    // remains byte-for-byte the same, including original JSON output.
    if args.len() == 3 && args[1] == "plate" {
        let n: usize = match args[2].parse() {
            Ok(value) => value,
            Err(_) => {
                eprintln!("invalid PH-03 integer mesh subdivision");
                return ExitCode::FAILURE;
            }
        };
        return match extract_plate(&PlateCoupon::ph03(n)) {
            Ok(p) => {
                let d = &p.dielectric;
                let v = &p.vacuum;
                println!(
                    concat!(
                        "{{\"model\":\"PH03_Neumann_plate\",\"n\":{},",
                        "\"width_m\":{:.16e},\"height_m\":{:.16e},\"epsilon_r\":{:.12e},",
                        "\"C_F_per_m\":{:.16e},\"C0_F_per_m\":{:.16e},",
                        "\"Lext_H_per_m\":{:.16e},\"Z0_ohm\":{:.16e},",
                        "\"delay_s_per_m\":{:.16e},",
                        "\"nx\":{},\"nz\":{},\"dof\":{},\"nodes\":{},",
                        "\"triangles\":{},\"nnz\":{},",
                        "\"field_residual\":{:.16e},\"vacuum_residual\":{:.16e},",
                        "\"field_charge_C\":{:.16e},\"vacuum_charge_C\":{:.16e},",
                        "\"field_charge_energy_rel\":{:.16e},",
                        "\"vacuum_charge_energy_rel\":{:.16e},",
                        "\"field_phi_err_V\":{:.16e},\"vacuum_phi_err_V\":{:.16e},",
                        "\"field_x_span_V\":{:.16e},\"vacuum_x_span_V\":{:.16e},",
                        "\"field_pec_err_V\":{:.16e},\"vacuum_pec_err_V\":{:.16e},",
                        "\"field_ex_rel\":{:.16e},\"vacuum_ex_rel\":{:.16e},",
                        "\"field_ez_rel\":{:.16e},\"vacuum_ez_rel\":{:.16e},",
                        "\"field_iterations\":{},\"vacuum_iterations\":{},",
                        "\"field_wall_s\":{:.9},\"vacuum_wall_s\":{:.9},",
                        "\"status\":\"{}\"}}"
                    ),
                    n,
                    0.0003_f64,
                    0.0002_f64,
                    3.4_f64,
                    p.c_f_per_m,
                    p.c0_f_per_m,
                    p.lext_h_per_m,
                    p.z0_ohm,
                    p.delay_s_per_m,
                    p.nx,
                    p.nz,
                    d.evidence.dof,
                    (p.nx + 1) * (p.nz + 1),
                    d.evidence.triangles,
                    d.evidence.matrix_nnz,
                    d.evidence.residual,
                    v.evidence.residual,
                    d.evidence.c_from_conductor_charge_f_per_m,
                    v.evidence.c_from_conductor_charge_f_per_m,
                    d.evidence.charge_energy_relative_difference,
                    v.evidence.charge_energy_relative_difference,
                    d.max_potential_error_v,
                    v.max_potential_error_v,
                    d.max_horizontal_potential_span_v,
                    v.max_horizontal_potential_span_v,
                    d.max_electrode_error_v,
                    v.max_electrode_error_v,
                    d.max_ex_relative,
                    v.max_ex_relative,
                    d.max_ez_relative,
                    v.max_ez_relative,
                    d.evidence.iterations,
                    v.evidence.iterations,
                    d.evidence.elapsed_s,
                    v.evidence.elapsed_s,
                    p.physical_status
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("unsupported or numerical failure: {error}");
                ExitCode::FAILURE
            }
        };
    }

    if args.len() != 3 {
        eprintln!("usage: boardsi-m2-lossless-fem <mesh-cells-per-h> <box-metres>");
        return ExitCode::FAILURE;
    }
    let n: usize = match args[1].parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("invalid integer mesh subdivision");
            return ExitCode::FAILURE;
        }
    };
    let box_m: f64 = match args[2].parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("invalid outer box size in metres");
            return ExitCode::FAILURE;
        }
    };
    match extract_ideal(&IdealCoupon::synthetic(n, box_m)) {
        Ok(o) => {
            let s = match analytic_ideal_s2(&o, 0.01, 1e9, 50.0) {
                Ok(s) => s,
                Err(error) => {
                    eprintln!("scattering math: {error}");
                    return ExitCode::FAILURE;
                }
            };
            // PH-01 uses an explicit matched-to-Z0 basis; the existing 50-ohm
            // PH-02 JSON keys remain unchanged and are never reclassified.
            let matched = match analytic_ideal_s2(&o, 0.01, 1e9, o.z0_ohm) {
                Ok(s) => s,
                Err(error) => {
                    eprintln!("matched scattering math: {error}");
                    return ExitCode::FAILURE;
                }
            };
            println!(
                "{{\"status\":\"{}\",\"n\":{},\"box_m\":{:.12e},                 \"C_F_per_m\":{:.16e},\"C0_F_per_m\":{:.16e},                 \"Lext_H_per_m\":{:.16e},\"Z0_ohm\":{:.12e},                 \"delay_s_per_m\":{:.16e},\"epsilon_effective\":{:.12e},                 \"S11_re\":{:.12e},\"S11_im\":{:.12e},                 \"S21_re\":{:.12e},\"S21_im\":{:.12e},                 \"S11_matched_re\":{:.12e},\"S11_matched_im\":{:.12e},                 \"S21_matched_re\":{:.12e},\"S21_matched_im\":{:.12e},                 \"dof\":{},\"triangles\":{},\"nnz\":{},                 \"field_iterations\":{},\"vacuum_iterations\":{},                 \"field_residual\":{:.8e},\"vacuum_residual\":{:.8e},                 \"charge_energy_rel\":{:.8e},                 \"field_wall_s\":{:.6},\"vacuum_wall_s\":{:.6}}}",
                o.physical_status, n, box_m,
                o.c_f_per_m, o.c0_f_per_m, o.lext_h_per_m,
                o.z0_ohm, o.delay_s_per_m, o.epsilon_effective,
                s.s11.re, s.s11.im, s.s21.re, s.s21.im,
                matched.s11.re, matched.s11.im, matched.s21.re, matched.s21.im,
                o.dielectric.dof, o.dielectric.triangles, o.dielectric.matrix_nnz,
                o.dielectric.iterations, o.vacuum.iterations,
                o.dielectric.residual, o.vacuum.residual,
                o.dielectric.charge_energy_relative_difference,
                o.dielectric.elapsed_s, o.vacuum.elapsed_s
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("unsupported or numerical failure: {error}");
            ExitCode::FAILURE
        }
    }
}
