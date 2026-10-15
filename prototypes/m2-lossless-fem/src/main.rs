use boardsi_m2_lossless_fem::stripline::{extract_stripline, StriplineCoupon};
use boardsi_m2_lossless_fem::{analytic_ideal_s2, extract_ideal, IdealCoupon};
use boardsi_m2_lossless_fem::plate::{extract_plate, PlateCoupon};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    // PH-05 is strictly opt-in. Neither the microstrip nor PH-03 CLI is modified.
    // args: stripline <n> <half-span-metres> [width_m thickness_m ground_gap_m epsilon_r]
    if args.len() > 1 && args[1] == "stripline" {
        if args.len() != 4 && args.len() != 8 {
            eprintln!("usage: boardsi-m2-lossless-fem stripline <n> <B_m> [w_m t_m b_m epsilon_r]");
            return ExitCode::FAILURE;
        }
        let n: usize = match args[2].parse() {
            Ok(v) => v, Err(_) => {
                eprintln!("invalid PH-05 integer subdivision");
                return ExitCode::FAILURE;
            }
        };
        let span: f64 = match args[3].parse() {
            Ok(v) => v, Err(_) => {
                eprintln!("invalid PH-05 lateral half-span");
                return ExitCode::FAILURE;
            }
        };
        let mut coupon = StriplineCoupon::ph05(n, span);
        if args.len() == 8 {
            let custom: Vec<f64> = match args[4..8].iter()
                .map(|v| v.parse::<f64>()).collect::<Result<Vec<_>,_>>() {
                Ok(v) => v, Err(_) => {
                    eprintln!("invalid PH-05 geometry/material float");
                    return ExitCode::FAILURE;
                }
            };
            coupon.width_m=custom[0];
            coupon.thickness_m=custom[1];
            coupon.ground_separation_m=custom[2];
            coupon.center_z_m=custom[2]/2.0;
            coupon.epsilon_r=custom[3];
        }
        return match extract_stripline(&coupon) {
            Ok(o) => {
                let d=&o.dielectric;
                let v=&o.vacuum;
                println!(concat!(
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
                n, coupon.half_span_m, coupon.width_m, coupon.thickness_m,
                coupon.ground_separation_m, coupon.epsilon_r,
                o.c_f_per_m, o.c0_f_per_m, o.lext_h_per_m, o.z0_ohm,
                o.delay_s_per_m, o.epsilon_effective,
                o.nx,o.nz,(o.nx+1)*(o.nz+1),d.evidence.dof,
                d.evidence.triangles,d.evidence.matrix_nnz,
                d.evidence.residual,v.evidence.residual,
                d.signal_charge_c_per_m,v.signal_charge_c_per_m,
                d.evidence.charge_energy_relative_difference,
                v.evidence.charge_energy_relative_difference,
                d.upper_ground_charge_c_per_m,d.lower_ground_charge_c_per_m,
                d.charge_balance_relative,v.charge_balance_relative,
                d.ground_asymmetry_relative,v.ground_asymmetry_relative,
                d.max_z_reflection_error_v,d.max_x_reflection_error_v,
                v.max_z_reflection_error_v,v.max_x_reflection_error_v,
                d.max_fringe_ex_v_per_m,d.max_pec_interior_field_v_per_m,
                d.evidence.iterations,v.evidence.iterations,
                d.evidence.elapsed_s,v.evidence.elapsed_s,o.physical_status);
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
                    n, 0.0003_f64, 0.0002_f64, 3.4_f64,
                    p.c_f_per_m, p.c0_f_per_m, p.lext_h_per_m, p.z0_ohm,
                    p.delay_s_per_m, p.nx, p.nz, d.evidence.dof,
                    (p.nx + 1) * (p.nz + 1), d.evidence.triangles,
                    d.evidence.matrix_nnz, d.evidence.residual, v.evidence.residual,
                    d.evidence.c_from_conductor_charge_f_per_m,
                    v.evidence.c_from_conductor_charge_f_per_m,
                    d.evidence.charge_energy_relative_difference,
                    v.evidence.charge_energy_relative_difference,
                    d.max_potential_error_v, v.max_potential_error_v,
                    d.max_horizontal_potential_span_v, v.max_horizontal_potential_span_v,
                    d.max_electrode_error_v, v.max_electrode_error_v,
                    d.max_ex_relative, v.max_ex_relative,
                    d.max_ez_relative, v.max_ez_relative,
                    d.evidence.iterations, v.evidence.iterations,
                    d.evidence.elapsed_s, v.evidence.elapsed_s,
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
