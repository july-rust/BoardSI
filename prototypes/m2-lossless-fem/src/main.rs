use boardsi_m2_lossless_fem::{analytic_ideal_s2, extract_ideal, IdealCoupon};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
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
            println!(
                "{{\"status\":\"{}\",\"n\":{},\"box_m\":{:.12e},                 \"C_F_per_m\":{:.16e},\"C0_F_per_m\":{:.16e},                 \"Lext_H_per_m\":{:.16e},\"Z0_ohm\":{:.12e},                 \"delay_s_per_m\":{:.16e},\"epsilon_effective\":{:.12e},                 \"S11_re\":{:.12e},\"S11_im\":{:.12e},                 \"S21_re\":{:.12e},\"S21_im\":{:.12e},                 \"dof\":{},\"triangles\":{},\"nnz\":{},                 \"field_iterations\":{},\"vacuum_iterations\":{},                 \"field_residual\":{:.8e},\"vacuum_residual\":{:.8e},                 \"charge_energy_rel\":{:.8e},                 \"field_wall_s\":{:.6},\"vacuum_wall_s\":{:.6}}}",
                o.physical_status, n, box_m,
                o.c_f_per_m, o.c0_f_per_m, o.lext_h_per_m,
                o.z0_ohm, o.delay_s_per_m, o.epsilon_effective,
                s.s11.re, s.s11.im, s.s21.re, s.s21.im,
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
