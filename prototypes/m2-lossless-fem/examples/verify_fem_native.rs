use boardsi_m2_lossless_fem::multilayer_microstrip::{extract_microstrip, MicrostripCoupon};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    assert_eq!(std::env::consts::ARCH, args[1]);
    assert_eq!(std::env::consts::OS, args[2]);
    let c = MicrostripCoupon::ph07('D', 32, 0.0012, 0.0008).unwrap();
    let r = extract_microstrip(&c).unwrap();
    println!(
        "Native FEM execution arch={} os={} C={:.16e} C0={:.16e} Z0={:.16e} delay={:.16e}",
        std::env::consts::ARCH,
        std::env::consts::OS,
        r.c_f_per_m,
        r.c0_f_per_m,
        r.z0_ohm,
        r.delay_s_per_m
    );
    assert!(r.dielectric.evidence.residual <= 1e-8);
}
