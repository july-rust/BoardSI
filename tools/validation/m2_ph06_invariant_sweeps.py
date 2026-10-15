#!/usr/bin/env python3
"""Supplementary PH-06 mathematical invariants, material/geometry sweeps.

Requires separately generated FEM JSON fixtures and PH-05 recovery anchor.
Does not use FEM results as a reference for independent FVM absolute C.
"""
import argparse
import hashlib
import json
from pathlib import Path

from m2_ph06_asymmetric_stripline_reference import extract

FIELDS = ("C_F_per_m", "C0_F_per_m", "Z0_ohm", "delay_s_per_m")
EXPECTED = {
    "symmetric": (.0004,3.4,3.4,None),
    "asymmetric_a": (.0003,3.4,3.4,None),
    "offset_035": (.00035,3.4,3.4,None),
    "mirror_a": (.0005,3.4,3.4,None),
    "two_region_b": (.0003,3.4,4.,.00035),
    "mirror_b": (.0005,4.,3.4,.00045),
    "equal_interface": (.0003,3.4,3.4,.00035),
    "swapped_dielectric": (.0003,4.,3.4,.00035),
    "vacuum": (.0003,1.,1.,None),
}
CASES=tuple(EXPECTED)


def rel(a,b):
    if b==0:
        raise ValueError("zero normalization")
    return abs(a/b-1)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--case",action="append",required=True,
                    help="NAME=filename.json; must include every frozen extra case")
    ap.add_argument("--ph05",required=True)
    ap.add_argument("--output",default="ph06-invariant-sweep-evidence.json")
    args=ap.parse_args()
    paths={}
    for s in args.case:
        name,sep,path=s.partition("=")
        if not sep or name not in EXPECTED or name in paths:
            raise SystemExit(f"invalid/duplicate PH-06 sweep fixture {s}")
        paths[name]=Path(path)
    if set(paths)!=set(EXPECTED):
        raise SystemExit(f"missing extra PH-06 sweep fixtures: {set(EXPECTED)-set(paths)}")
    records={}
    source_hash={}
    checks=[]
    def gate(name,actual,limit):
        checks.append({"gate":name,"observed":float(actual),"max":limit,
                       "status":"PASS" if actual<=limit else "FAIL"})
    for name,path in paths.items():
        payload=path.read_bytes()
        r=json.loads(payload)
        if r["model"]!="PH06_IdealAsymmetricStripline" or r["n"]!=32 or \
           rel(r["B_m"],.0012)>1e-10:
            raise SystemExit(f"wrong FEM model/mesh: {path}")
        zc,eb,et,interface=EXPECTED[name]
        for attr,value in (("zc_m",zc),("epsilon_bottom",eb),("epsilon_top",et)):
            if rel(r[attr],value)>1e-10:
                raise SystemExit(f"unexpected {attr} in {name}")
        if r["interface_z_m"]!=interface:
            raise SystemExit(f"wrong interface for {name}")
        records[name]=r
        source_hash[path.name]=hashlib.sha256(payload).hexdigest()
        gate(name+" positivity C,C0,Z0,delay",float(any(r[f]<=0 for f in FIELDS)),0.)
        gate(name+" charge neutrality",r["field_charge_balance_rel"],1e-6)
    ph05_bytes=Path(args.ph05).read_bytes()
    ph05=json.loads(ph05_bytes)
    if ph05["model"]!="PH05_IdealSymmetricStripline" or ph05["n"]!=32 or \
       rel(ph05["B_m"],.0012)>1e-10:
        raise SystemExit("wrong PH-05 symmetric recovery fixture")
    source_hash[Path(args.ph05).name]=hashlib.sha256(ph05_bytes).hexdigest()

    def compare(name,a,b,maximum):
        for quantity in FIELDS:
            gate(f"{name} {quantity}",rel(a[quantity],b[quantity]),maximum)
    def swapped_charge(name,a,b):
        # Charge reactions should interchange under full physical reflection.
        q=a["field_charge_C"]
        gate(f"{name} upper-to-lower",
             abs(a["field_ground_top_C"]-b["field_ground_bottom_C"])/abs(q),1e-5)
        gate(f"{name} lower-to-upper",
             abs(a["field_ground_bottom_C"]-b["field_ground_top_C"])/abs(q),1e-5)

    compare("symmetric PH-06 -> PH-05",records["symmetric"],ph05,1e-8)
    compare("full mirror PH-06A",records["asymmetric_a"],records["mirror_a"],1e-6)
    compare("full mirror PH-06B",records["two_region_b"],records["mirror_b"],1e-6)
    compare("equal permittivity interface neutral",
            records["asymmetric_a"],records["equal_interface"],1e-8)
    swapped_charge("full mirror PH-06A",records["asymmetric_a"],records["mirror_a"])
    swapped_charge("full mirror PH-06B",records["two_region_b"],records["mirror_b"])
    a=records["asymmetric_a"]
    b=records["two_region_b"]
    d=records["swapped_dielectric"]
    vac=records["vacuum"]
    gate("vacuum C equals C0",rel(vac["C_F_per_m"],vac["C0_F_per_m"]),1e-10)
    gate("vacuum C0 equals baseline C0",rel(vac["C0_F_per_m"],a["C0_F_per_m"]),1e-10)
    gate("top epsilon 3.4 -> 4.0 C must increase",float(b["C_F_per_m"]<=a["C_F_per_m"]),0.)
    gate("bottom epsilon 3.4 -> 4.0 C must increase",float(d["C_F_per_m"]<=a["C_F_per_m"]),0.)
    gate("PH06B epsilon_effective bounded",
         max(3.4-b["epsilon_effective"],b["epsilon_effective"]-4.,0.),1e-8)
    gate("swapped epsilon_effective bounded",
         max(3.4-d["epsilon_effective"],d["epsilon_effective"]-4.,0.),1e-8)
    # Offset sweep C increases as conductor moves closer to the lower ground.
    sym=records["symmetric"]["C_F_per_m"]
    mid=records["offset_035"]["C_F_per_m"]
    off=a["C_F_per_m"]
    gate("zc sweep symmetric < midway < off-centre C",
         float(not (sym < mid < off)),0.)
    # The independent FVM mirror is evaluated using its own direct solver,
    # not fed FEM potentials, stiffness or charge to construct the oracle.
    ref_a=extract(32,.0012,.0003,3.4,3.4,None)
    ref_am=extract(32,.0012,.0005,3.4,3.4,None)
    ref_b=extract(32,.0012,.0003,3.4,4.,.00035)
    ref_bm=extract(32,.0012,.0005,4.,3.4,.00045)
    compare("independent FVM full mirror A",ref_a,ref_am,1e-6)
    compare("independent FVM full mirror B",ref_b,ref_bm,1e-6)
    for label,a1,b1 in (("A",ref_a,ref_am),("B",ref_b,ref_bm)):
        ua=a1["field"]["upper_ground_charge_C"]
        la=a1["field"]["lower_ground_charge_C"]
        ub=b1["field"]["upper_ground_charge_C"]
        lb=b1["field"]["lower_ground_charge_C"]
        norm=abs(a1["field"]["signal_charge_C"])
        gate("independent FVM reflected top-to-bottom "+label,abs(ua-lb)/norm,1e-5)
        gate("independent FVM reflected bottom-to-top "+label,abs(la-ub)/norm,1e-5)

    bad=[c for c in checks if c["status"]=="FAIL"]
    output={
        "status":"FAIL" if bad else "PASS","gates":len(checks),
        "failed_gates":len(bad),"checks":checks,"fem_sha256":source_hash,
        "reference_note":"FVM mesh is separate, P1 FEM is not used as its reference.",
        "cases":{name:{f:records[name][f] for f in FIELDS} for name in CASES},
        "independent_fvm_mirrors":{
            "A":{f:ref_a[f] for f in FIELDS},
            "mirror_A":{f:ref_am[f] for f in FIELDS},
            "B":{f:ref_b[f] for f in FIELDS},
            "mirror_B":{f:ref_bm[f] for f in FIELDS},
        },
    }
    Path(args.output).write_text(json.dumps(output,sort_keys=True,indent=2)+"\n",
                                encoding="utf-8")
    for item in bad:
        print("FAIL",item["gate"],item["observed"],"limit",item["max"])
    print("PH-06 reflected/geometry/dielectric supplementary gates",
          output["status"],len(checks),"failed",len(bad))
    if bad:
        raise SystemExit(1)


if __name__=="__main__":
    main()
