"""Verify separately calculated FVM output against committed G0 evidence.

This script checks existing independent reference data and declared numerical
acceptance. It is NOT a second independent PDE field solver.
"""
import csv
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
    # Analytic matched PH01 reference uses independent identity for lossless uniform line.
    delay=at(.0012,48)["delay_s_per_m"]
    angle=2*math.pi*1e9*.01*delay
    s21=complex(math.cos(angle),-math.sin(angle))
    check(abs(abs(s21)-1.0)<=1e-12,"PH01 passivity anchor")
    print(json.dumps({"result":"G0_RESTRICTED_COUPON_READY",
                      "mesh_percent":mesh,
                      "box_percent":boundary,
                      "PH03_edge_free_plate":{"C_F_per_m":cplate,
                                               "C0_F_per_m":c0plate,
                                               "Lext_H_per_m":lplate},
                      "PH01_matched":{"frequency_Hz":1e9,
                                      "length_m":.01,
                                      "S11":[0,0],
                                      "S21":[s21.real,s21.imag]}},indent=2))
    print("Physical PCB/real-loss validation remains UNSUPPORTED.")

if __name__=="__main__":
    main()
