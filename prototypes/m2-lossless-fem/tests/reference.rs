use boardsi_m2_lossless_fem::{extract_ideal, IdealCoupon};

const FVM_C_N32: f64 = 8.842_386_680_772_379e-11;
const FVM_C0_N32: f64 = 3.654_801_591_437_518e-11;

fn rel(a: f64, b: f64) -> f64 {
    (a / b - 1.0).abs()
}

#[test]
fn ph04_17_independent_fvm_and_fem_mesh_gate() {
    // The independently calculated FVM n32 reference is checked again against
    // the source-locked SciPy G0 oracle by tools/validation/m2_g1_field_check.py.
    // It is NOT one of the G0 n12/n24/n48 CSV holdout rows.
    // No Rust FEM/FVM mesh or assembly implementation is shared.
    let medium = extract_ideal(&IdealCoupon::synthetic(16, 0.0012)).unwrap();
    let fine = extract_ideal(&IdealCoupon::synthetic(32, 0.0012)).unwrap();
    assert!(rel(fine.c_f_per_m, FVM_C_N32) <= 0.02,
        "FEM/FVM C difference exceeds preregistered 2%");
    assert!(rel(fine.c0_f_per_m, FVM_C0_N32) <= 0.02,
        "FEM/FVM C0 difference exceeds preregistered 2%");
    assert!(rel(medium.c_f_per_m, fine.c_f_per_m) <= 0.015,
        "FEM mesh C difference exceeds preregistered 1.5%");
    assert!(rel(medium.c0_f_per_m, fine.c0_f_per_m) <= 0.015,
        "FEM mesh C0 difference exceeds preregistered 1.5%");
    assert!(fine.dielectric.residual <= 1e-8);
    assert!(fine.vacuum.residual <= 1e-8);
    println!("PH04 n32 FEM C={:.16e} C0={:.16e}, independent FVM C relative %={:.8} C0 relative %={:.8}",
        fine.c_f_per_m, fine.c0_f_per_m,
        100.0 * rel(fine.c_f_per_m, FVM_C_N32),
        100.0 * rel(fine.c0_f_per_m, FVM_C0_N32));
    println!("PH17 mesh n16->32 C relative %={:.8} C0 relative %={:.8}",
        100.0 * rel(medium.c_f_per_m, fine.c_f_per_m),
        100.0 * rel(medium.c0_f_per_m, fine.c0_f_per_m));
}

#[test]
fn ph18_19_external_pec_box_and_passive_field_gate() {
    // This explicitly models different PEC enclosures; not an open boundary.
    let near = extract_ideal(&IdealCoupon::synthetic(12, 0.0016)).unwrap();
    let far = extract_ideal(&IdealCoupon::synthetic(12, 0.0024)).unwrap();
    for (a,b) in [(near.c_f_per_m,far.c_f_per_m),
                  (near.c0_f_per_m,far.c0_f_per_m),
                  (near.lext_h_per_m,far.lext_h_per_m),
                  (near.z0_ohm,far.z0_ohm),
                  (near.delay_s_per_m,far.delay_s_per_m)] {
        assert!(rel(a,b)<=0.02,"PEC box sensitivity exceeds preregistered 2%");
    }
    println!("PH18 FEM grounded-box 1.6->2.4 mm: C={:.8}% C0={:.8}% Lext={:.8}% Z0={:.8}% delay={:.8}%",
        100.0*rel(near.c_f_per_m,far.c_f_per_m),
        100.0*rel(near.c0_f_per_m,far.c0_f_per_m),
        100.0*rel(near.lext_h_per_m,far.lext_h_per_m),
        100.0*rel(near.z0_ohm,far.z0_ohm),
        100.0*rel(near.delay_s_per_m,far.delay_s_per_m));
    for result in [&near,&far] {
        assert!(result.c_f_per_m >= result.c0_f_per_m);
        assert!(result.c_f_per_m / result.c0_f_per_m <= 3.4);
        assert!(result.dielectric.charge_energy_relative_difference < 1e-6);
        assert!(result.vacuum.charge_energy_relative_difference < 1e-6);
        assert!(result.physical_status.contains("NotRealPCBValidated"));
    }
}
