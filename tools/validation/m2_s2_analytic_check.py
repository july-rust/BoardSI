"""Independent ideal-lossless ABCD -> complex-S check for PH-01 and PH-02.

The inputs C/C0 come from a separate independent FVM numerical check. This
script validates 2-port mathematical conversion, not actual PCB field physics.
All thresholds were frozen in the G0 protocol before the G1 solver existed.
"""
import cmath
import json
import math
import sys

COMPLEX_TOLERANCE = 1e-12


def near(label, expected, observed, tolerance=COMPLEX_TOLERANCE):
    error = abs(expected - observed)
    if not math.isfinite(error) or error > tolerance:
        raise AssertionError(f"{label}: complex delta {error:.16e} exceeds {tolerance:.1e}")
    return error


def abcd_s(z0, reference, theta):
    """Independent ABCD conversion for exp(+j omega t), equal real ports."""
    a = d = math.cos(theta)
    b = 1j * z0 * math.sin(theta)
    c = 1j * math.sin(theta) / z0
    den = a + b / reference + c * reference + d
    return (
        (a + b / reference - c * reference - d) / den,
        2 * (a * d - b * c) / den,
        2 / den,
        (-a + b / reference - c * reference + d) / den,
    )


def assert_unitary(label, s11, s21):
    power = abs(s11) ** 2 + abs(s21) ** 2
    error = near(label + " power conservation", complex(1.0), complex(power))
    return power, error


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: m2_s2_analytic_check.py captured-fem-result.json")
    with open(sys.argv[1], encoding="utf-8") as f:
        actual = json.load(f)
    if actual.get("status") != "PrototypeOnly_IdealPECBox_NotRealPCBValidated":
        raise AssertionError("unreviewed physical model status")
    if actual.get("n") != 32 or not math.isclose(actual.get("box_m", float("nan")), 0.0012,
                                                 rel_tol=0.0, abs_tol=1e-13):
        raise AssertionError("expected fixed n32 / B=0.0012 m ideal specimen")
    z = float(actual["Z0_ohm"])
    tau = float(actual["delay_s_per_m"])
    if not (math.isfinite(z) and z > 0 and math.isfinite(tau) and tau > 0):
        raise AssertionError("nonphysical line parameters")
    length_m = 0.01
    freq_hz = 1e9
    theta = 2 * math.pi * freq_hz * length_m * tau

    # PH-01: matched-to-extracted-Z0 mathematical reference (NOT the 50-ohm
    # fixture). Compare the actual Rust matched results both to an independent
    # ABCD 2-port and to the simple traveling-wave phase formula.
    m11, m12, m21, m22 = abcd_s(z, z, theta)
    observed_m11 = complex(actual["S11_matched_re"], actual["S11_matched_im"])
    observed_m21 = complex(actual["S21_matched_re"], actual["S21_matched_im"])
    matched_err11 = near("PH-01 matched S11", m11, observed_m11)
    matched_err21 = near("PH-01 matched S21", m21, observed_m21)
    near("PH-01 traveling-wave S11", 0j, observed_m11)
    near("PH-01 traveling-wave S21", cmath.exp(-1j * theta), observed_m21)
    matched_rec_error = max(
        near("PH-01 ABCD S12=S21", m12, observed_m21),
        near("PH-01 ABCD S22=S11", m22, observed_m11),
    )
    matched_power, matched_power_error = assert_unitary("PH-01", observed_m11, observed_m21)

    # PH-02: original fixed 50-ohm mismatch, independently checked in the
    # exact complex plane, not merely |S| or real parts.
    s11, s12, s21, s22 = abcd_s(z, 50.0, theta)
    observed11 = complex(actual["S11_re"], actual["S11_im"])
    observed21 = complex(actual["S21_re"], actual["S21_im"])
    err11 = near("PH-02 S11", s11, observed11)
    err21 = near("PH-02 S21", s21, observed21)
    reciprocity_error = max(
        near("PH-02 S12 reciprocity", s12, observed21),
        near("PH-02 S22 symmetry", s22, observed11),
    )
    power, power_error = assert_unitary("PH-02", observed11, observed21)
    print(json.dumps({
        "scope": "synthetic lossless PEC grounded-box mathematical 2-port only",
        "status": "PASS_NOT_REAL_PCB_VALIDATED",
        "complex_tolerance": COMPLEX_TOLERANCE,
        "PH01_basis": "matched to extracted Z0 (distinct from PH-02)",
        "PH01_complex_error_S11": matched_err11,
        "PH01_complex_error_S21": matched_err21,
        "PH01_complex_reciprocity_error": matched_rec_error,
        "PH01_power_sum": matched_power,
        "PH01_power_error": matched_power_error,
        "PH02_basis": "fixed equal 50 ohm ports",
        "PH02_complex_error_S11": err11,
        "PH02_complex_error_S21": err21,
        "PH02_complex_reciprocity_error": reciprocity_error,
        "PH02_power_sum": power,
        "PH02_power_error": power_error,
        "PH02_S11_ABCD": [s11.real, s11.imag],
        "PH02_S21_ABCD": [s21.real, s21.imag],
    }, indent=2))


def self_test():
    near("on-threshold", 0j, COMPLEX_TOLERANCE + 0j)
    for bad in [1.01e-12, 1e-6, math.inf, math.nan]:
        try:
            near("intentional perturbation", 0j, complex(bad, 0.0))
        except AssertionError:
            continue
        raise AssertionError(f"tolerance regression: accepted bad delta {bad}")
    print("PASS: preregistered 1e-12 mathematical gate rejects perturbations")


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        self_test()
    else:
        main()
