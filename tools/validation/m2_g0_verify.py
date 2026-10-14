"""Verify separately calculated FVM output against committed G0 evidence.

This script checks existing independent reference data and declared numerical
acceptance. It is NOT a second independent PDE field solver.
"""
import csv
import hashlib
import json
import math
import sys
from pathlib import Path

EPS0=8.8541878188e-12
MU0=1.25663706127e-6
W=0.0003
H=0.0002
ER=3.4

def check(expr, message):
    if not expr:
        raise AssertionError(message)

def main():
    if len(sys.argv)!=3:
        raise SystemExit("usage: m2_g0_verify.py committed.csv reproduced.json")
    with open(sys.argv[1], encoding="utf-8", newline="") as file:
        fixed=list(csv.DictReader(file))
    with open(sys.argv[2], encoding="utf-8") as file:
        generated=json.load(file)
    measured=generated["records"]
    check(len(fixed)==len(measured)==6,"expected six heldout geometry/mesh cases")
    index={(float(t["box_m"]), int(t["n"])):t for t in measured}
    for row in fixed:
        case=(float(row["box_m"]),int(row["n"]))
        check(case in index,"missing expected reference case")
        t=index[case]
        check(t["field"]["dof"]==int(row["field_dof"]),"DOF mismatch")
        mapping={"C_F_per_m":t["field"]["C_F_per_m"],
                 "C0_F_per_m":t["vacuum"]["C_F_per_m"],
                 "Lext_H_per_m":t["derived"]["Lext_H_per_m"],
                 "Z0_ohm":t["derived"]["Z0_ohm"],
                 "delay_s_per_m":t["derived"]["delay_s_per_m"]}
        # Cross-host factorization residuals are bounded, not bitwise references.
        # Discrete residuals are independently required <= 1e-9 below.
        for key,value in mapping.items():
            reference=float(row[key])
            check(math.isfinite(value) and math.isclose(value,reference,rel_tol=2e-10,abs_tol=1e-19),
                  "reference fixture drift "+str(case)+" "+key)
        check(t["field"]["relative_residual"]<=1e-9 and
              t["vacuum"]["relative_residual"]<=1e-9,"discrete residual threshold")
        c,c0=t["field"]["C_F_per_m"],t["vacuum"]["C_F_per_m"]
        check(0<c0<=c and 1<=c/c0<=ER+1e-9,"passive C/C0 bounds")
        check(t["derived"]["Lext_H_per_m"]>0 and
              t["derived"]["Z0_ohm"]>0 and
              t["derived"]["delay_s_per_m"]>0,"positive idealized outputs")
    def at(box,n):
        return index[(box,n)]["derived"]
    keys=("C_F_per_m","C0_F_per_m","Lext_H_per_m","Z0_ohm","delay_s_per_m")
    mesh={k:100*abs(at(.0012,24)[k]/at(.0012,48)[k]-1) for k in keys}
    boundary={k:100*abs(at(.0016,12)[k]/at(.0024,12)[k]-1) for k in keys}
    check(all(v<=1.0 for v in mesh.values()),"mesh holdout failed")
    check(all(v<=2.0 for v in boundary.values()),"exterior holdout failed")
    # Edge-free mathematical plate anchor: explicitly NOT microstrip physical solution.
    cplate=EPS0*ER*W/H
    c0plate=EPS0*W/H
    lplate=MU0*H/W
    check(math.isclose(lplate,MU0*EPS0/c0plate,rel_tol=1e-14),"PH03 dimensions")
    check(math.isclose(cplate/c0plate,ER,rel_tol=1e-14),"PH03 dielectric ratio")
    # Pre-registered source digest: verify exact oracle bytes before declaring G0.
    reference_py=Path(__file__).with_name("m2_fvm_reference.py")
    digest_file=Path(__file__).with_name("m2_g0_source.sha256")
    expected_digest=digest_file.read_text(encoding="utf-8").split()[0]
    actual_digest=hashlib.sha256(reference_py.read_bytes()).hexdigest()
    check(expected_digest=="c5e05fed106cc78e22150b0c90a828951a439940bb1a8dc8843bd5eb73786929","source hash manifest drift")
    check(actual_digest==expected_digest,"independent field source SHA-256 mismatch")

    # PH-01: FIXED 50-ohm reference, so this coupon is mismatched (nonzero S11).
    # Cross-check independently derived ABCD and multiple-reflection formulas.
    model=at(.0012,48)
    z0=model["Z0_ohm"]
    delay=model["delay_s_per_m"]
    theta=2*math.pi*1e9*.01*delay
    a=math.cos(theta)
    b=1j*z0*math.sin(theta)
    c=1j*math.sin(theta)/z0
    denom=2*a+b/50.0+50.0*c
    s11=(b/50.0-50.0*c)/denom
    s21=2.0/denom
    rho=(z0-50.0)/(z0+50.0)
    e=complex(math.cos(theta),-math.sin(theta))
    reflection=rho*(1-e*e)/(1-rho*rho*e*e)
    transmission=(1-rho*rho)*e/(1-rho*rho*e*e)
    check(abs(s11-reflection)<=1e-12 and abs(s21-transmission)<=1e-12,
          "independent PH01 ABCD/multiple-reflection disagreement")
    check(abs(s11-complex(0.01651839692653288,0.048232430752762685))<=1e-11 and
          abs(s21-complex(0.9448265269029662,-0.3235793708615182))<=1e-11,
          "frozen PH01 50-ohm mismatch reference drift")
    check(abs(abs(s11)**2+abs(s21)**2-1.0)<=1e-12,
          "PH01 passive lossless scattering")
    print(json.dumps({"result":"G0_RESTRICTED_COUPON_READY",
                      "source_sha256":actual_digest,
                      "mesh_percent":mesh,
                      "box_percent":boundary,
                      "PH03_edge_free_plate":{"C_F_per_m":cplate,
                                               "C0_F_per_m":c0plate,
                                               "Lext_H_per_m":lplate},
                      "PH01_mismatched_50ohm":{"frequency_Hz":1e9,
                                               "length_m":.01,
                                               "reference_ohm":50.0,
                                               "S11":[s11.real,s11.imag],
                                               "S21":[s21.real,s21.imag]}},indent=2))
    print("Physical PCB/real-loss validation remains UNSUPPORTED.")
UNSUPPORTED.")

if __name__=="__main__":
    main()
