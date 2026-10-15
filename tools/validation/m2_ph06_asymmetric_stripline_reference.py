#!/usr/bin/env python3
"""Independent PH-06 cell-centered conservative FVM electrostatic reference.

No Rust FEM output enters the FVM linear system or defines its expected results.
Finite rectangular signal PEC, grounded top/bottom, zero-flux lateral sides.
2-region material interface is mesh-conforming; harmonic face permittivity.
The derived L/Z0/delay are quasi-TEM algebra, NOT a magnetic/full-wave reference.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import sys
import time

import numpy as np
import scipy
import scipy.sparse as sp
from scipy.sparse.linalg import spsolve

EPS0 = 8.8541878188e-12
MU0 = 1.25663706127e-6
W = 0.00030
T = 0.00005
H = 0.00080
ZC = 0.00030
INTERFACE = 0.00035
CASES = ((32, .0012), (64, .0012), (128, .0012),
         (64, .0016), (64, .0024))
# Independent geometry/contract constants, preregistered in PH-06 Markdown.
FIXTURE = {
    "PH06A": {"width_m": W, "thickness_m": T, "height_m": H, "zc_m": ZC,
              "epsilon_bottom": 3.4, "epsilon_top": 3.4,
              "interface_z_m": None},
    "PH06B": {"width_m": W, "thickness_m": T, "height_m": H, "zc_m": ZC,
              "epsilon_bottom": 3.4, "epsilon_top": 4.0,
              "interface_z_m": INTERFACE},
    "cases": CASES, "signal_v": 1.0, "grounds_v": 0.0,
    "side_flux": "homogeneous Neumann", "mu_r": 1.0,
    "status": "Synthetic_2D_PEC_Lossless_NotRealPCB",
}
LIMIT = {
    "fem_fvm": .02,
    "fem_mesh": .015,
    "fvm_mesh": .01,
    "boundary": .02,
    "true_residual": 1e-8,
    "energy_charge": 1e-6,
    "x_reflection_V": 1e-6,
    "electrode_V": 1e-12,
    "charge_balance": 1e-6,
    "homogeneous_identity": .001,
    "recovery": 1e-8,
    "mirror_relative": 1e-6,
    "mirror_charge_relative": 1e-5,
}


def aligned(value, step):
    return np.isfinite(value) and abs(value / step - round(value / step)) <= 1e-8


def validate(n, span, zc, ebot, etop, interface):
    if not (isinstance(n, int) and 8 <= n <= 256 and n % 8 == 0):
        raise ValueError("PH-06 unsupported FV mesh size")
    vals = (span, zc, ebot, etop)
    if not all(np.isfinite(v) for v in vals):
        raise ValueError("nonfinite FV geometry/material")
    if span <= W / 2 or not (T/2 < zc < H-T/2) or min(ebot, etop) < 1:
        raise ValueError("PH-06 invalid FV geometry or epsilon")
    if ebot != etop and interface is None:
        raise ValueError("PH-06 missing layered interface")
    if interface is not None and (
            not np.isfinite(interface) or not 0 < interface < H):
        raise ValueError("PH-06 invalid dielectric interface")
    dz = H/n
    if not all(aligned(v, dz) for v in
               (span, 2*span, W/2, T, zc-T/2, zc+T/2)):
        raise ValueError("nonconforming FV conductor/box")
    if interface is not None and not aligned(interface, dz):
        raise ValueError("nonconforming FV dielectric interface")
    nx = int(round(2*span/dz))
    if nx * n > 240_000:
        raise ValueError("PH-06 FV mesh memory cap")
    return dz, nx


def solve_fvm(n, span, zc=ZC, ebot=3.4, etop=3.4, interface=None):
    """Cell-centred FV: integrate divergence epsilon grad(phi)=0 per dielectric cell.

    On material faces use the harmonic series dielectric conductance
    g = 2/(1/epsilon_left + 1/epsilon_right) for equal half-cell lengths.
    On PEC faces use g=2*epsilon_cell and prescribed metal or ground voltage.
    The resulting SPD system is assembled independently of P1 FEM and solved
    using SciPy sparse DIRECT solve, not the Rust iterative CG implementation.
    """
    dz, nx = validate(n, span, zc, ebot, etop, interface)
    xs = -span+(np.arange(nx)+.5)*dz
    zs = (np.arange(n)+.5)*dz
    metal = ((np.abs(xs[None,:]) < W/2) &
             (zs[:,None] > zc-T/2) & (zs[:,None] < zc+T/2))
    if (metal[0].any() or metal[-1].any() or
            metal[:,0].any() or metal[:,-1].any()):
        raise ValueError("PH-06 FV PEC shorts GND/side")
    errow = np.full(n, ebot, dtype=np.float64)
    if interface is not None:
        errow[zs > interface] = etop
    count = int(np.count_nonzero(~metal))
    if count <= 0 or not np.any(metal):
        raise ValueError("empty PH-06 FV dielectric or signal")
    ids = np.full((n,nx), -1, dtype=np.int32)
    ids[~metal] = np.arange(count, dtype=np.int32)
    rows, cols, values = [], [], []
    rhs = np.zeros(count, dtype=np.float64)
    contact, top, bottom = [], [], []
    start = time.perf_counter()
    for j in range(n):
        for i in range(nx):
            if metal[j,i]:
                continue
            u = int(ids[j,i])
            diagonal = 0.
            for dj,di in ((-1,0),(1,0),(0,-1),(0,1)):
                jj,ii=j+dj,i+di
                if ii<0 or ii>=nx:
                    continue  # exact zero Neumann on lateral wall
                if jj<0 or jj>=n:
                    g = 2.*errow[j]  # half-cell distance to PEC ground
                    diagonal += g
                    (bottom if jj<0 else top).append((u,g))
                elif metal[jj,ii]:
                    g=2.*errow[j]
                    diagonal+=g
                    rhs[u]+=g  # signal potential 1 V
                    contact.append((u,g))
                else:
                    # Opposite cell permittivity, harmonic face flux.
                    eps_other=errow[jj]
                    g=2.*errow[j]*eps_other/(errow[j]+eps_other)
                    diagonal+=g
                    rows.append(u)
                    cols.append(int(ids[jj,ii]))
                    values.append(-g)
            rows.append(u)
            cols.append(u)
            values.append(diagonal)
    a=sp.coo_matrix((values,(rows,cols)),shape=(count,count)).tocsr()
    phi=spsolve(a,rhs)
    residual=float(np.linalg.norm(a@phi-rhs) /
                   max(np.linalg.norm(rhs),np.finfo(float).tiny))
    if not np.isfinite(phi).all() or not np.isfinite(residual):
        raise ArithmeticError("nonfinite PH-06 FV potential/residual")
    qs=float(EPS0*sum(g*(1.-phi[u]) for u,g in contact))
    qt=float(-EPS0*sum(g*phi[u] for u,g in top))
    qb=float(-EPS0*sum(g*phi[u] for u,g in bottom))
    if qs<=0:
        raise ArithmeticError("unphysical PH-06 FV capacitance")
    balance=abs((qs+qt+qb)/qs)
    full=np.ones((n,nx))
    full[~metal]=phi
    xerr=float(np.max(np.abs(full-full[:,::-1])))
    zerr=float(np.max(np.abs(full-full[::-1,:])))
    out={
        "n":n,"B_m":span,"zc_m":zc,"interface_z_m":interface,
        "epsilon_bottom":ebot,"epsilon_top":etop,
        "C_F_per_m":qs,"nx":nx,"nz":n,
        "dof":count,"nnz":int(a.nnz),
        "field_residual":residual,
        "signal_charge_C":qs,
        "upper_ground_charge_C":qt,
        "lower_ground_charge_C":qb,
        "charge_balance_relative":balance,
        "x_reflection_max_V":xerr,
        "z_reflection_max_V":zerr,
        "wall_s":time.perf_counter()-start,
    }
    if residual>LIMIT["true_residual"] or balance>LIMIT["charge_balance"] or \
       xerr>LIMIT["x_reflection_V"]:
        raise ArithmeticError(f"PH-06 FVM own conservation gate failed: {out}")
    return out


def extract(n, span, zc=ZC, ebot=3.4, etop=3.4, interface=None):
    dielectric=solve_fvm(n,span,zc,ebot,etop,interface)
    vacuum=solve_fvm(n,span,zc,1.,1.,interface)
    cap,cap0=dielectric["C_F_per_m"],vacuum["C_F_per_m"]
    lext=MU0*EPS0/cap0
    return {
        "C_F_per_m":cap,"C0_F_per_m":cap0,"Lext_H_per_m":lext,
        "Z0_ohm":(lext/cap)**.5,"delay_s_per_m":(lext*cap)**.5,
        "epsilon_effective":cap/cap0,
        "field":dielectric,"vacuum":vacuum,
    }


def rel(a,b):
    if b==0:
        raise ValueError("relative reference denominator zero")
    return abs(a/b-1.)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--fem",nargs="*",default=[],
                    help="Ten PH-06A/B JSONs: n32/64/128 B.0012 + n64 B.0016/.0024")
    ap.add_argument("--output",default="ph06-independent-fvm-evidence.json")
    ap.add_argument("--self-test",action="store_true")
    args=ap.parse_args()
    if args.self_test:
        a=extract(32,.0012)
        b=extract(32,.0012,ebot=3.4,etop=4.,interface=INTERFACE)
        assert rel(a["epsilon_effective"],3.4)<.001
        assert 3.4-1e-8<=b["epsilon_effective"]<=4.+1e-8
        assert abs(a["field"]["lower_ground_charge_C"]) > \
               1.05*abs(a["field"]["upper_ground_charge_C"])
        print("PH-06 independent FVM analytical self-test PASS",flush=True)
        return

    if len(args.fem)!=2*len(CASES):
        raise SystemExit("PH-06 requires exactly ten independent FEM JSON inputs")
    fem={}
    femhash={}
    for filename in args.fem:
        data=Path(filename).read_bytes()
        row=json.loads(data)
        case="PH06B" if row["epsilon_top"]!=row["epsilon_bottom"] else "PH06A"
        key=(case,int(row["n"]),float(row["B_m"]))
        if key in fem or key[1:] not in CASES:
            raise SystemExit(f"duplicate/invalid PH-06 FEM fixture: {key}")
        frozen=FIXTURE[case]
        for attr, expected in (
                ("w_m",W),("t_m",T),("b_m",H),("zc_m",ZC),
                ("epsilon_bottom",frozen["epsilon_bottom"]),
                ("epsilon_top",frozen["epsilon_top"])):
            if rel(row[attr],expected)>1e-10:
                raise SystemExit(f"FEM physical fixture mismatch {key}, {attr}")
        if row["interface_z_m"]!=frozen["interface_z_m"]:
            raise SystemExit(f"FEM interface differs from frozen fixture: {key}")
        if row["model"]!="PH06_IdealAsymmetricStripline":
            raise SystemExit(f"FEM used wrong model: {key}")
        fem[key]=row
        femhash[Path(filename).name]=hashlib.sha256(data).hexdigest()

    checks=[]
    def gate(label,observed,maximum):
        checks.append({"gate":label,"observed":float(observed),"max":maximum,
                       "status":"PASS" if np.isfinite(observed) and observed<=maximum else "FAIL"})

    refs={}
    start=time.perf_counter()
    for case in ("PH06A","PH06B"):
        cfg=FIXTURE[case]
        for n,span in CASES:
            key=(case,n,span)
            ref=extract(n,span,ZC,cfg["epsilon_bottom"],cfg["epsilon_top"],
                        cfg["interface_z_m"])
            refs[key]=ref
            f=fem[key]
            print(f"{case} n={n} B={span:.4g} FEM C={f['C_F_per_m']:.12g} "
                  f"C0={f['C0_F_per_m']:.12g} Z0={f['Z0_ohm']:.9g} "
                  f"delay={f['delay_s_per_m']:.9g} | FVM "
                  f"C={ref['C_F_per_m']:.12g} C0={ref['C0_F_per_m']:.12g} "
                  f"Z0={ref['Z0_ohm']:.9g} delay={ref['delay_s_per_m']:.9g}",flush=True)
            for field in ("field_residual","vacuum_residual"):
                gate(f"{key} FEM {field}",f[field],LIMIT["true_residual"])
            for field in ("field_charge_energy_rel","vacuum_charge_energy_rel"):
                gate(f"{key} FEM {field}",f[field],LIMIT["energy_charge"])
            for field in ("field_charge_balance_rel","vacuum_charge_balance_rel"):
                gate(f"{key} FEM {field}",f[field],LIMIT["charge_balance"])
            for field in ("field_x_sym_err_V","vacuum_x_sym_err_V"):
                gate(f"{key} FEM {field}",f[field],LIMIT["x_reflection_V"])
            for field in ("field_pec_err_V","vacuum_pec_err_V"):
                gate(f"{key} FEM {field}",f[field],LIMIT["electrode_V"])
            gate(f"{key} FEM fringe must exist",float(f["field_fringe_Ex_V_per_m"]<=0.),0.)
            gate(f"{key} FEM PEC field",f["field_pec_field_V_per_m"],1e-6)
            for quantity in ("C_F_per_m","C0_F_per_m","Z0_ohm","delay_s_per_m"):
                if n==128 and span==.0012:
                    gate(f"{case} independent FEM/FVM n128 {quantity}",
                         rel(f[quantity],ref[quantity]),LIMIT["fem_fvm"])
            if case=="PH06A":
                gate(f"{key} FEM C/C0 homogeneous",
                     rel(f["epsilon_effective"],3.4),LIMIT["homogeneous_identity"])
                gate(f"{key} FEM delay homogeneous",
                     rel(f["delay_s_per_m"],(MU0*EPS0*3.4)**.5),
                     LIMIT["homogeneous_identity"])
            else:
                gate(f"{key} FEM effective epsilon bounded",
                     max(3.4-f["epsilon_effective"],
                         f["epsilon_effective"]-4.,0.),1e-8)
            for quantity in ("Lext_H_per_m","Z0_ohm","delay_s_per_m"):
                c0=f["C0_F_per_m"]
                c=f["C_F_per_m"]
                lex=MU0*EPS0/c0
                expected={"Lext_H_per_m":lex,
                          "Z0_ohm":(lex/c)**.5,
                          "delay_s_per_m":(lex*c)**.5}[quantity]
                gate(f"{key} FEM algebraic {quantity}",rel(f[quantity],expected),1e-10)
        for quantity in ("C_F_per_m","C0_F_per_m"):
            gate(f"{case} FEM mesh64to128 {quantity}",
                 rel(fem[(case,64,.0012)][quantity],
                     fem[(case,128,.0012)][quantity]),LIMIT["fem_mesh"])
            gate(f"{case} FVM mesh64to128 {quantity}",
                 rel(refs[(case,64,.0012)][quantity],
                     refs[(case,128,.0012)][quantity]),LIMIT["fvm_mesh"])
            for span in (.0012,.0016):
                gate(f"{case} FEM boundary B{span} {quantity}",
                     rel(fem[(case,64,span)][quantity],
                         fem[(case,64,.0024)][quantity]),LIMIT["boundary"])
        f=fem[(case,128,.0012)]["field_ground_bottom_C"]
        g=fem[(case,128,.0012)]["field_ground_top_C"]
        rf=refs[(case,128,.0012)]["field"]["lower_ground_charge_C"]
        rg=refs[(case,128,.0012)]["field"]["upper_ground_charge_C"]
        gate(f"{case} FEM/FVM bottom charge sign",float(np.sign(f)!=np.sign(rf)),0.)
        gate(f"{case} FEM/FVM top charge sign",float(np.sign(g)!=np.sign(rg)),0.)
        gate(f"{case} FEM/FVM charge distribution direction",
             float((abs(f)>abs(g))!=(abs(rf)>abs(rg))),0.)
        # Also report per-plane quantitative independently calculated charge.
        for which,key in (("bottom",("field_ground_bottom_C","lower_ground_charge_C")),
                          ("top",("field_ground_top_C","upper_ground_charge_C"))):
            gate(f"{case} FEM/FVM {which} ground charge n128",
                 rel(fem[(case,128,.0012)][key[0]],refs[(case,128,.0012)]["field"][key[1]]),
                 LIMIT["fem_fvm"])
    a=fem[("PH06A",32,.0012)]
    gate("PH06A nearer lower GND dominates",
         float(abs(a["field_ground_bottom_C"])<1.05*abs(a["field_ground_top_C"])),0.)
    gate("PH06A genuine z potential asymmetry",
         float(a["field_z_sym_err_V"]<.01),0.)
    failed=[g for g in checks if g["status"]=="FAIL"]
    document={
        "status":"FAIL" if failed else "PASS",
        "failed_gates":len(failed),
        "not_validated":["real PCB","30 GHz","R/G","skin effect","3D via/slot",
                         "PH-07 multilayer solder mask"],
        "fixture":FIXTURE,"contract":LIMIT,
        "source_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "fixture_sha256":hashlib.sha256(
            json.dumps(FIXTURE,sort_keys=True,separators=(",",":")).encode()).hexdigest(),
        "fem_input_sha256":femhash,
        "toolchain":{"python":sys.version.split()[0],"numpy":np.__version__,
                     "scipy":scipy.__version__,"platform":platform.platform()},
        "references":[{"case":case,"n":n,"B_m":span,**refs[(case,n,span)]}
                      for case in ("PH06A","PH06B") for n,span in CASES],
        "checks":checks,
        "elapsed_fvm_wall_s":time.perf_counter()-start,
    }
    Path(args.output).write_text(json.dumps(document,indent=2,sort_keys=True)+"\n",
                                encoding="utf-8")
    for g in failed:
        print("FAIL",g["gate"],"observed",g["observed"],"max",g["max"])
    print("PH-06 independent FVM acceptance",document["status"],
          f"({len(checks)} gates, {len(failed)} failures)",flush=True)
    print("SOURCE_SHA256",document["source_sha256"])
    print("FIXTURE_SHA256",document["fixture_sha256"])
    print("EVIDENCE_SHA256",hashlib.sha256(Path(args.output).read_bytes()).hexdigest())
    if failed:
        raise SystemExit(1)


if __name__=="__main__":
    main()
