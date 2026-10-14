"""Independent analytical ABCD -> complex S check for the isolated ideal M2 coupon.

No Rust numerical mapping reused. Geometry to C and C0 is checked separately
against independently solved FVM; this checks only the mathematics of S.
"""
import cmath
import json
import math
import sys

def near(label, computed, observed, tolerance=1e-6):
    error=abs(computed-observed)
    if not math.isfinite(error) or error>tolerance:
        raise AssertionError(f"{label} complex delta {error} exceeds {tolerance}")
    return error

def main():
    if len(sys.argv)!=2:
        raise SystemExit("usage: m2_s2_check.py captured-fem-result.json")
    with open(sys.argv[1],encoding="utf-8") as f:
        actual=json.load(f)
    if actual.get("status")!="PrototypeOnly_IdealPECBox_NotRealPCBValidated":
        raise AssertionError("unreviewed physical model status")
    if actual.get("n")!=32:
        raise AssertionError("expected fixed n32 ideal specimen")
    z=float(actual["Z0_ohm"])
    tau=float(actual["delay_s_per_m"])
    ref=50.0
    length_m=0.01
    freq_hz=1e9
    phase=2*math.pi*freq_hz*length_m*tau
    # Independent textbook ABCD calculation with exp(+jwt), not Rust algebra:
    a=d=math.cos(phase)
    b=1j*z*math.sin(phase)
    c=1j*math.sin(phase)/z
    denominator=a+b/ref+c*ref+d
    s11=(a+b/ref-c*ref-d)/denominator
    s21=2/denominator
    s12=2*(a*d-b*c)/denominator
    s22=(-a+b/ref-c*ref+d)/denominator
    observed11=complex(actual["S11_re"],actual["S11_im"])
    observed21=complex(actual["S21_re"],actual["S21_im"])
    err11=near("S11",s11,observed11)
    err21=near("S21",s21,observed21)
    near("S12 reciprocity",s12,observed21)
    near("S22 reciprocal symmetric",s22,observed11)
    power=abs(s11)**2+abs(s21)**2
    if abs(power-1)>1e-8:
        raise AssertionError("analytic ideal line nonunitary")
    print(json.dumps({"scope":"mathematical lossless ideal 50-ohm 2-port only",
                      "status":"PASS",
                      "complex_error_S11":err11,
                      "complex_error_S21":err21,
                      "S11_analytic":[s11.real,s11.imag],
                      "S21_analytic":[s21.real,s21.imag],
                      "sampled_power_sum":power},indent=2))

if __name__=="__main__":
    main()
