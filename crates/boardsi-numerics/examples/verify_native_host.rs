//! CI-only identity check, separate from numerical/physical validation.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: verify_native_host EXPECTED_ARCH EXPECTED_OS"
    );
    assert_eq!(
        std::env::consts::ARCH,
        args[1],
        "compiled executable architecture mismatch"
    );
    assert_eq!(
        std::env::consts::OS,
        args[2],
        "native executable OS mismatch"
    );
    assert_eq!(std::mem::size_of::<f64>(), 8);
    println!(
        "NATIVE_EXECUTION arch={} os={} f64_bits=64",
        std::env::consts::ARCH,
        std::env::consts::OS
    );
}
