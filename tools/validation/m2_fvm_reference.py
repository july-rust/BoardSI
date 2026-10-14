"""Independent FVM electrostatic reference for a *synthetic* PEC microstrip.

Original self-contained reference algorithm, not the BoardSI FEM implementation.
Cell-centered finite volumes; face-flux conservation; sparse direct solve.
Domain: all exterior sides and the bottom continuous PEC reference at 0 V.
Rectangular signal PEC at 1 V. Material epsilon_r below z=h, air above.
Only explicitly ideal and lossless, uniform in the longitudinal direction.
No real-board loss or GHz validation is implied.
"""
import argparse
import hashlib
import json
import platform
import sys
import time

import numpy as np
import scipy
import scipy.sparse as sp
from scipy.sparse.linalg import spsolve

EPS0 = 8.8541878188e-12  # F/m, declared numerical constant
MU0 = 1.25663706127e-6  # H/m, declared numerical constant
W = 0.0003  # m; synthetic width
T = 0.00005  # m; synthetic PEC thickness
H = 0.0002  # m; synthetic height
ER = 3.4   # explicit nondispersive relative permittivity


def solve(n=8, span=0.0012, top=0.0012, er=ER):
    """n cell layers through h, with conductor/medium interfaces at faces."""
    if not np.isfinite([n, span, top, er]).all() or min(n, span, top, er) <= 0:
        raise ValueError("invalid geometry or material")
    step = H / n
    nx = round(2 * span / step)
    nz = round(top / step)
    if abs(nx * step / 2 - span) > 1e-12 or abs(nz * step - top) > 1e-12:
        raise ValueError("box must align with the cell grid")
    if any(abs(round(q / step) - q / step) > 1e-10 for q in (W, T, H)):
        raise ValueError("signal boundaries must align with the cell grid")
    if span <= W/2 or top <= H+T:
        raise ValueError("copper intersects outer ground")

    x = -span + (np.arange(nx) + 0.5)*step
    z = (np.arange(nz) + 0.5)*step
    metal = (z[:,None] >= H-1e-13) & (z[:,None] < H+T-1e-13) & (np.abs(x[None,:]) < W/2+1e-13)
    er_cell = np.where(z[:,None] < H, er, 1.) * np.ones((1, nx))
    active = ~metal
    ids = np.full((nz,nx), -1, dtype=np.int32)
    ids[active] = np.arange(np.count_nonzero(active))
    dof = int(np.count_nonzero(active))
    rows, cols, vals = [], [], []
    rhs = np.zeros(dof)
    signal_edges = []

    for iz in range(nz):
        for ix in range(nx):
            if not active[iz,ix]:
                continue
            i = int(ids[iz,ix])
            eps = float(er_cell[iz,ix])
            diag = 0.
            for dz,dx in ((0,1),(0,-1),(1,0),(-1,0)):
                jz,jx = iz+dz, ix+dx
                if jz < 0 or jz >= nz or jx < 0 or jx >= nx:
                    diag += 2*eps  # half cell to 0 V grounded exterior face
                elif metal[jz,jx]:
                    g = 2*eps  # half cell to conductor PEC face at 1 V
                    diag += g
                    rhs[i] += g
                    signal_edges.append((i,g))
                else:
                    eps_next = float(er_cell[jz,jx])
                    g = 2*eps*eps_next/(eps+eps_next)
                    diag += g
                    rows.append(i); cols.append(int(ids[jz,jx])); vals.append(-g)
            rows.append(i); cols.append(i); vals.append(diag)
    A = sp.coo_matrix((vals,(rows,cols)), shape=(dof,dof)).tocsr()
    start = time.perf_counter()
    phi = spsolve(A, rhs)
    elapsed = time.perf_counter()-start
    if not np.isfinite(phi).all():
        raise RuntimeError("nonfinite field solution")
    residual = float(np.linalg.norm(A@phi-rhs)/max(np.linalg.norm(rhs),1e-25))
    c = float(EPS0*sum(g*(1.-phi[i]) for i,g in signal_edges))
    return {"n":n,"span_m":span,"top_m":top,"er":er,
            "nx":nx,"nz":nz,"dof":dof,"nnz":int(A.nnz),
            "C_F_per_m":c,"relative_residual":residual,"linear_solve_wall_s":elapsed}


def derived(material, vacuum):
    c = material["C_F_per_m"]
    c0 = vacuum["C_F_per_m"]
    l = MU0*EPS0/c0
    return {"C_F_per_m":c, "C0_F_per_m":c0, "Lext_H_per_m":l,
            "Z0_ohm":float(np.sqrt(l/c)),
            "delay_s_per_m":float(np.sqrt(l*c)),
            "epsilon_effective":c/c0}


def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--suite", choices=("pilot","holdout"), default="pilot")
    ap.add_argument("--output", default="g0-fvm-results.json")
    args=ap.parse_args()
    # Pilot was executed before the numerical acceptance policy was frozen.
    # Holdout MUST NOT be used to retroactively edit acceptance thresholds.
    cases=([(0.0012,0.0012,[4,8,16,32]),
            (0.0016,0.0016,[8,16]),
            (0.0024,0.0024,[8])] if args.suite=="pilot"
           else [(0.0012,0.0012,[12,24,48]),
                 (0.0016,0.0016,[12,24]),
                 (0.0024,0.0024,[12])])
    began=time.perf_counter()
    records=[]
    for span,top,levels in cases:
        for n in levels:
            field=solve(n,span,top,ER)
            vacuum=solve(n,span,top,1.0)
            records.append({"box_m":span,"n":n,"field":field,
                            "vacuum":vacuum,"derived":derived(field,vacuum)})
            print(f"box={span:.4g} n={n} nodes={field['dof']} "
                  f"C={field['C_F_per_m']:.12g} C0={vacuum['C_F_per_m']:.12g} "
                  f"Z0={derived(field,vacuum)['Z0_ohm']:.8f}",flush=True)
    out={"kind":"independent FVM field reference (not DOLFINx)",
         "suite":args.suite,
         "geometry_m":{"w":W,"t":T,"h":H},
         "relative_permittivity":ER,
         "epsilon0_F_per_m":EPS0,"mu0_H_per_m":MU0,
         "material":"PEC and explicit nonmagnetic zero-loss",
         "box":"all outer faces including bottom are grounded PEC; outer walls are not open boundaries",
         "tool":{"python":sys.version.split()[0],"numpy":np.__version__,
                 "scipy":scipy.__version__,"os":platform.platform()},
         "wall_s":time.perf_counter()-began,
         "records":records}
    data=json.dumps(out,indent=2,sort_keys=True)+"\n"
    with open(args.output,"w",encoding="utf-8") as f:
        f.write(data)
    print("SHA256(output)="+hashlib.sha256(data.encode()).hexdigest())


if __name__=="__main__":
    main()
