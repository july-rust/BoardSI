use boardsi_numerics::{max_singular_value, Complex, Error, ModelStatus, Rlgc, UniformLine};
fn line() -> UniformLine {
    UniformLine {
        rlgc: Rlgc {
            r_ohm_per_m: 0.0,
            l_h_per_m: 250e-9,
            g_s_per_m: 0.0,
            c_f_per_m: 100e-12,
        },
        length_m: 0.1,
        reference_ohm: 50.0,
    }
}
fn close(a: Complex, b: Complex, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a:?} != {b:?}");
}

#[test]
fn lossless_matched_half_wave_and_delay() {
    let x = line().at(1e9).unwrap();
    close(x.s[0][0], Complex::new(0.0, 0.0), 1e-12);
    close(x.s[1][0], Complex::new(-1.0, 0.0), 1e-12);
    let impedance_error = (x.zc_ohm.unwrap().re / 50.0 - 1.0).abs();
    let delay = x.gamma_per_m.unwrap().im * line().length_m / (std::f64::consts::TAU * 1e9);
    let delay_error = (delay / 0.5e-9 - 1.0).abs();
    println!("lossless impedance_relative_error={impedance_error:.16e}, delay_relative_error={delay_error:.16e}");
    assert!(impedance_error <= 0.001 && delay_error <= 0.001);
    assert_eq!(x.model_status, ModelStatus::Approximate);
}
#[test]
fn dc_policy_and_zero_length() {
    let mut l = line();
    let x = l.at(0.0).unwrap();
    assert_eq!(x.gamma_per_m.unwrap(), Complex::new(0.0, 0.0));
    close(x.zc_ohm.unwrap(), Complex::new(50.0, 0.0), 1e-12);
    assert_eq!(
        x.s,
        [
            [Complex::new(0.0, 0.0), Complex::new(1.0, 0.0)],
            [Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)]
        ]
    );
    l.rlgc.r_ohm_per_m = 1.0;
    assert!(matches!(l.at(0.0), Err(Error::Unsupported(_))));
    l.length_m = 0.0;
    for f in [0.0, 1e9] {
        let x = l.at(f).unwrap();
        assert!(x.zc_ohm.is_none() && x.gamma_per_m.is_none());
        assert_eq!(x.s[1][0], Complex::new(1.0, 0.0));
    }
}
#[test]
fn independent_high_precision_abcd_fixtures() {
    let mut max_error: f64 = 0.0;
    let mut max_passivity: f64 = 0.0;
    let mut max_reciprocity: f64 = 0.0;
    let mut count = 0;
    for row in include_str!("fixtures/abcd_reference.csv")
        .lines()
        .filter(|r| !r.starts_with('#'))
    {
        let v: Vec<f64> = row.split(',').map(|s| s.parse().unwrap()).collect();
        let l = UniformLine {
            rlgc: Rlgc {
                r_ohm_per_m: v[0],
                l_h_per_m: v[1],
                g_s_per_m: v[2],
                c_f_per_m: v[3],
            },
            length_m: v[4],
            reference_ohm: v[6],
        };
        let x = l.at(v[5]).unwrap_or_else(|e| panic!("{row}: {e}"));
        for (a, b) in [
            (x.s[0][0], Complex::new(v[7], v[8])),
            (x.s[1][0], Complex::new(v[9], v[10])),
            (x.s[0][1], Complex::new(v[9], v[10])),
            (x.s[1][1], Complex::new(v[7], v[8])),
        ] {
            max_error = max_error.max((a - b).abs());
            close(a, b, 1e-6);
        }
        let zc = Complex::new(v[11], v[12]);
        let gamma = Complex::new(v[13], v[14]);
        assert!((x.zc_ohm.unwrap() - zc).abs() / zc.abs() < 1e-12);
        assert!((x.gamma_per_m.unwrap() - gamma).abs() / gamma.abs() < 1e-12);
        // Matched high-attenuation fixtures must preserve tiny nonzero S21,
        // not merely pass an absolute tolerance after underflow to zero.
        if v[4] >= 100.0 {
            assert!(
                (x.s[1][0] - Complex::new(v[9], v[10])).abs() / Complex::new(v[9], v[10]).abs()
                    < 1e-10
            );
            assert!(x.s[1][0].abs() > 0.0);
        }
        // This line's symmetric S is normal: analytic singular values |S11 +/- S21|.
        let independent_sigma = (x.s[0][0] + x.s[1][0])
            .abs()
            .max((x.s[0][0] - x.s[1][0]).abs());
        assert!((max_singular_value(x.s) - independent_sigma).abs() < 1e-12);
        assert!(independent_sigma <= 1.0 + 1e-6);
        max_passivity = max_passivity.max(x.passivity_residual);
        max_reciprocity = max_reciprocity.max(x.reciprocity_residual);
        count += 1;
    }
    assert_eq!(count, 72);
    println!("ABCD cases={count}, max_complex_S_absolute_error={max_error:.16e}, max_passivity_residual={max_passivity:.16e}, max_reciprocity_residual={max_reciprocity:.16e}");
}
#[test]
fn lossy_phase_attenuation_and_reciprocity() {
    let mut l = line();
    // Distortionless R/L=G/C=4e8: Zc=50, alpha=2 /m, beta=omega*5e-9 /m.
    l.rlgc.r_ohm_per_m = 100.0;
    l.rlgc.g_s_per_m = 0.04;
    let x = l.at(0.5e9).unwrap();
    close(x.zc_ohm.unwrap(), Complex::new(50.0, 0.0), 1e-12);
    close(x.s[1][0], Complex::new(0.0, -(-0.2_f64).exp()), 1e-12);
    assert!(x.gamma_per_m.unwrap().re >= 0.0 && x.gamma_per_m.unwrap().im > 0.0);
    assert!(x.passivity_residual <= 1e-6 && x.reciprocity_residual == 0.0);
}
#[test]
fn invalid_inputs_rejected_in_every_field() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..7 {
            let mut l = line();
            let mut f = 1e9;
            match index {
                0 => l.rlgc.r_ohm_per_m = value,
                1 => l.rlgc.l_h_per_m = value,
                2 => l.rlgc.g_s_per_m = value,
                3 => l.rlgc.c_f_per_m = value,
                4 => l.length_m = value,
                5 => l.reference_ohm = value,
                _ => f = value,
            }
            assert!(matches!(l.at(f), Err(Error::InvalidInput(_))));
        }
    }
    for index in 0..7 {
        let mut l = line();
        let mut f = 1e9;
        match index {
            0 => l.rlgc.r_ohm_per_m = -1.0,
            1 => l.rlgc.l_h_per_m = -1.0,
            2 => l.rlgc.g_s_per_m = -1.0,
            3 => l.rlgc.c_f_per_m = -1.0,
            4 => l.length_m = -1.0,
            5 => l.reference_ohm = -1.0,
            _ => f = -1.0,
        }
        assert!(matches!(l.at(f), Err(Error::InvalidInput(_))));
    }
    for index in 0..3 {
        let mut l = line();
        match index {
            0 => l.rlgc.l_h_per_m = 0.0,
            1 => l.rlgc.c_f_per_m = 0.0,
            _ => l.reference_ohm = 0.0,
        }
        assert!(matches!(l.at(1e9), Err(Error::InvalidInput(_))));
    }
    let mut l = line();
    l.length_m = 0.0;
    l.rlgc.c_f_per_m = f64::NAN;
    assert!(matches!(l.at(1e9), Err(Error::InvalidInput(_))));
}
#[test]
fn numerical_envelope_and_short_line() {
    let mut l = line();
    l.length_m = 1e-20;
    let x = l.at(1e9).unwrap();
    close(x.s[1][0], Complex::new(1.0, 0.0), 1e-12);
    l.rlgc.l_h_per_m = 562.5e-9;
    let x = l.at(1e9).unwrap();
    assert!(x.s[0][0].im > 0.0); // tiny reflection not rounded to zero
    l = line();
    l.length_m = 1e10;
    assert!(matches!(l.at(1e9), Err(Error::Unsupported(_))));
    l = line();
    l.rlgc.r_ohm_per_m = 100.0;
    l.rlgc.g_s_per_m = 0.04;
    l.length_m = 701.0;
    assert!(matches!(l.at(1.0), Err(Error::Unsupported(_))));
    l = line();
    l.rlgc.r_ohm_per_m = 1e200;
    assert!(matches!(l.at(1e9), Err(Error::Unsupported(_))));
    l = line();
    l.reference_ohm = 1e-200;
    assert!(matches!(l.at(1e9), Err(Error::Unsupported(_))));
    assert!(line().at(f64::MAX).is_err());
    l = line();
    l.rlgc.r_ohm_per_m = 1e-200;
    l.rlgc.l_h_per_m = 1e-200;
    l.rlgc.c_f_per_m = 1e-200;
    assert!(matches!(l.at(1.0), Err(Error::Unsupported(_))));
}
#[test]
fn singular_value_catches_elementwise_false_pass() {
    let z = Complex::new(0.0, 0.0);
    let a = Complex::new(0.8, 0.0);
    assert!((max_singular_value([[a, a], [a, a]]) - 1.6).abs() < 1e-12);
    assert!((max_singular_value([[z, a], [z, z]]) - 0.8).abs() < 1e-12);
    // Known complex unitary matrix.
    let a = Complex::new(0.5_f64.sqrt(), 0.0);
    let b = Complex::new(0.0, 0.5_f64.sqrt());
    assert!((max_singular_value([[a, b], [b, a]]) - 1.0).abs() < 1e-12);
    assert_eq!(max_singular_value([[z, z], [z, z]]), 0.0);
    assert!(max_singular_value([[Complex::new(f64::NAN, 0.0), z], [z, z]]).is_nan());
}
#[test]
fn point_errors_are_isolated() {
    let x = line().at_frequencies(&[0.0, 1e9, -1.0, 30e9]);
    assert_eq!(x.len(), 4);
    assert!(x[0].is_ok() && x[1].is_ok() && x[2].is_err() && x[3].is_ok());
}

#[test]
fn high_mismatch_long_phase_resonance_rejected() {
    let mut l = line();
    l.rlgc.l_h_per_m = 0.25;
    l.rlgc.c_f_per_m = 1e-16;
    l.length_m = 5000.0;
    // Zc/Zref=1e6 and beta*l~50000*pi: finite but numerically sensitive.
    assert!(matches!(l.at(1e9), Err(Error::NumericalInstability(_))));
}

#[test]
fn tiny_distortionless_parameters_preserve_finite_attenuation() {
    // Exact distortionless anchor: Zc=1, gamma=R+j*omega*L.
    // R*omega*C underflows although alpha*length=1 is representable.
    let l = UniformLine {
        rlgc: Rlgc {
            r_ohm_per_m: 1e-120,
            l_h_per_m: 1e-120,
            g_s_per_m: 1e-120,
            c_f_per_m: 1e-120,
        },
        length_m: 1e120,
        reference_ohm: 1.0,
    };
    let x = l.at(1e-101).unwrap();
    close(x.s[1][0], Complex::new((-1.0_f64).exp(), 0.0), 1e-12);
    assert!((x.gamma_per_m.unwrap().re / 1e-120 - 1.0).abs() < 1e-12);
}

#[test]
fn near_resistive_ac_with_unresolvable_phase_is_finite() {
    let l = UniformLine {
        rlgc: Rlgc {
            r_ohm_per_m: 1.0,
            l_h_per_m: 1e-120,
            g_s_per_m: 1.0,
            c_f_per_m: 1e-120,
        },
        length_m: 1.0,
        reference_ohm: 1.0,
    };
    // omega*L and omega*C round to zero at this positive frequency.
    // The exactly matched limiting resistive solution is exp(-1).
    let x = l.at(1e-250).unwrap();
    close(x.s[1][0], Complex::new((-1.0_f64).exp(), 0.0), 1e-12);
    assert_eq!(x.gamma_per_m.unwrap(), Complex::new(1.0, 0.0));
    assert!(matches!(l.at(0.0), Err(Error::Unsupported(_))));
}
