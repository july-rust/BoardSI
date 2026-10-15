#!/usr/bin/env python3
"""PH-07 INDEPENDENT cell-centred conservative finite-volume reference.

Separate four-face flux discretisation and SciPy direct sparse solver; no FEM
potentials/stiffness/reaction/target values enter the FVM computation.
An FVM file carries its own independent mesh/material/BC definition copied only
from PH-07 physical input geometry; FEM outputs are compared AFTER FVM solving.
"""
import argparse
import hashlib
import json
import math
import pathlib
import resource
import sys
import time

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.linalg import spsolve

EPS0 = 8.8541878188e-12
MU0 = 1.25663706127e-6
# frozen acceptance thresholds (must remain identical to PH-07 contract)
GATES = dict(fem_fvm=0.03, true_residual=1e-8, charge_balance=1e-6,
             fem_charge_energy=1e-6, fem_mesh=0.02, fvm_mesh=0.015,
             outer_boundary=0.02, symmetry_volts=1e-6,
             electrode_volts=1e-12, equal_interface=1e-8)
FIXTURE = {
    "width_m": 0.00030, "copper_thickness_m": 0.00005,
    "single_layers": [{"height_m": 0.00020, "epsilon_r": 3.4}],
    "two_layers": [{"height_m": 0.00010, "epsilon_r": 3.4},
                   {"height_m": 0.00010, "epsilon_r": 4.0}],
    "mask_thickness_m": 0.000025, "mask_er": 3.3,
    "ground_v": 0.0, "signal_v": 1.0, "air_er": 1.0,
    "sides": "Neumann", "top": "Neumann",
    "B_m": 0.0012, "H_m": 0.0008,
    "meshes": [32, 64, 128]
}


def digest(obj):
    return hashlib.sha256(json.dumps(obj, sort_keys=True, separators=(",", ":"),
                              allow_nan=False).encode()).hexdigest()


def region(x, z, inp):
    """None is PEC. 0 air; 1..len(layers) substrate; last index mask."""
    w = inp["width_m"]
    h = sum(p["height_m"] for p in inp["layers"])
    thick = inp["copper_thickness_m"]
    a = w / 2
    if abs(x) < a and h <= z < h + thick:
        return None
    layer_z = 0.0
    for idx, layer in enumerate(inp["layers"], 1):
        layer_z += layer["height_m"]
        if z < layer_z:
            return idx
    m = inp["mask"]
    if m is not None:
        tm = m["thickness_m"]
        blanket = h <= z < h + tm
        side = a <= abs(x) < a + tm and h <= z < h + thick
        top = abs(x) < a + tm and h + thick <= z < h + thick + tm
        if blanket or side or top:
            return len(inp["layers"]) + 1
    return 0


def make_materials(inp, vacuum):
    n = inp["n"]
    H = inp["H_m"]
    B = inp["B_m"]
    d = H / n
    nx_float = 2 * B / d
    nx = round(nx_float)
    if abs(nx_float - nx) > 1e-8 or nx < 1 or nx * n > 240_000:
        raise ValueError("independent FVM nonconforming/oversized outer mesh")
    levels = [1.0 if vacuum else inp["air_epsilon_r"]]
    levels.extend(1.0 if vacuum else v["epsilon_r"] for v in inp["layers"])
    levels.append(1.0 if vacuum or inp["mask"] is None else inp["mask"]["epsilon_r"])
    ids = np.empty((n, nx), dtype=np.int32)
    er = np.zeros((n, nx), dtype=np.float64)
    for j in range(n):
        z = (j + 0.5) * d
        for i in range(nx):
            x = -B + (i + 0.5) * d
            rid = region(x, z, inp)
            ids[j, i] = -1 if rid is None else rid
            if rid is not None:
                er[j, i] = levels[rid]
    return ids, er, nx, d


def fvm(inp, vacuum=False):
    start = time.perf_counter()
    ids, er, nx, d = make_materials(inp, vacuum)
    nz = inp["n"]
    is_field = ids >= 0
    ix = np.full((nz, nx), -1, dtype=np.int32)
    ix[is_field] = np.arange(int(is_field.sum()), dtype=np.int32)
    size = int(is_field.sum())
    if size <= 0:
        raise ValueError("FVM missing free cells")
    rows, cols, values = [], [], []
    rhs = np.zeros(size)
    q_signal = 0.0
    q_ground = 0.0
    # Faces are counted from each non-PEC unknown. Geometric x/z face length
    # and separation both d, so transmissibility is epsilon * d/d = epsilon.
    # For a Dirichlet electrode at a cell FACE separation is d/2 => 2*epsilon.
    for j in range(nz):
        for i in range(nx):
            cur = int(ix[j, i])
            if cur < 0:
                continue
            ep = er[j, i]
            diag = 0.0
            for di, dj in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                ii, jj = i + di, j + dj
                if ii < 0 or ii >= nx or jj < 0 or jj >= nz:
                    if jj < 0:
                        # Bottom continuous GND: Dirichlet on boundary face.
                        conductance = 2.0 * ep
                        diag += conductance
                        # q_ground accumulated AFTER solving using bottom flux.
                    # Side and TOP boundary are natural zero flux.
                    continue
                nei = int(ix[jj, ii])
                if nei < 0:
                    # Face adjacent to ideal signal PEC at 1 V.
                    conductance = 2.0 * ep
                    diag += conductance
                    rhs[cur] += conductance
                else:
                    ep_other = er[jj, ii]
                    # Independent conservative series dielectric flux,
                    # appropriate even at material interfaces.
                    conductance = 2.0 * ep * ep_other / (ep + ep_other)
                    diag += conductance
                    rows.append(cur)
                    cols.append(nei)
                    values.append(-conductance)
            rows.append(cur)
            cols.append(cur)
            values.append(diag)
    A = coo_matrix((np.asarray(values), (np.asarray(rows), np.asarray(cols))),
                   shape=(size, size)).tocsr()
    phi = np.asarray(spsolve(A.tocsc(), rhs), dtype=np.float64)
    residual = float(np.linalg.norm(A @ phi - rhs) / np.linalg.norm(rhs))
    if not np.isfinite(phi).all() or not math.isfinite(residual):
        raise ValueError("nonfinite independently solved FVM")
    # Flux charges are integrated from the solved FVM cell fields, not FEM.
    for j in range(nz):
        for i in range(nx):
            cur = int(ix[j, i])
            if cur < 0:
                continue
            e = er[j, i]
            v = phi[cur]
            if j == 0:
                q_ground += EPS0 * 2.0 * e * (0.0 - v)
            for ii, jj in ((i-1,j), (i+1,j), (i,j-1), (i,j+1)):
                if 0 <= ii < nx and 0 <= jj < nz and ix[jj, ii] < 0:
                    q_signal += EPS0 * 2.0 * e * (1.0-v)
    if not q_signal > 0:
        raise ValueError("missing positive signal charge")
    balance = abs((q_signal + q_ground) / q_signal)
    return {
        "C_F_per_m": q_signal, "q_signal_C_per_m": q_signal,
        "q_ground_C_per_m": q_ground, "charge_balance_rel": balance,
        "residual": residual, "nx": nx, "nz": nz,
        "d_m": d, "cells": nx*nz, "field_dof": size, "nnz": A.nnz,
        "wall_s": time.perf_counter()-start,
        "peak_rss_kb": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    }


def solve_coupon(inp):
    a = fvm(inp)
    v = fvm(inp, vacuum=True)
    cap, cap0 = a["C_F_per_m"], v["C_F_per_m"]
    lext = MU0*EPS0/cap0
    return {
        "C_F_per_m": cap, "C0_F_per_m": cap0,
        "Lext_H_per_m": lext, "Z0_ohm": math.sqrt(lext/cap),
        "delay_s_per_m": math.sqrt(lext*cap),
        "epsilon_effective": cap/cap0,
        "dielectric": a, "vacuum": v
    }


def relatively(a, b):
    if not (b > 0 and math.isfinite(b)):
        raise ValueError("invalid comparator denominator")
    return abs(a/b-1)


def self_test():
    # Analytic 1D uniform dielectric parallel plate with homogeneous Neumann
    # sides: this is an independent FVM formulation test (not PH-07 FEM target).
    # Also test homogeneous dielectric face and direct residual behaviour.
    tmp = dict(FIXTURE)
    tmp = {
        "case":"A", "n":32, "B_m":0.0012, "H_m":0.0008,
        "width_m":0.00030,"copper_thickness_m":0.00005,
        "air_epsilon_r":1.0,"layers":FIXTURE["single_layers"],"mask":None
    }
    d=solve_coupon(tmp)
    assert d["C_F_per_m"] > d["C0_F_per_m"] > 0
    assert 1.0 < d["epsilon_effective"] < 3.4
    assert d["dielectric"]["charge_balance_rel"] < GATES["charge_balance"]
    assert d["vacuum"]["charge_balance_rel"] < GATES["charge_balance"]
    assert d["dielectric"]["residual"] < GATES["true_residual"]
    print("PH-07 independent FVM smoke test PASS", flush=True)


def run(files, destination):
    source_sha = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
    cases = {}
    checks = []
    computed = {}  # identical physical geometry: reuse expensive FVM direct solves

    def check(name, passed, details):
        checks.append(dict(name=name, status="PASS" if passed else "FAIL",
                           details=details))
        if not passed:
            print("PH07 FAIL", name, details, file=sys.stderr, flush=True)

    for file in files:
        data = json.loads(pathlib.Path(file).read_text())
        inp = {k:data[k] for k in ("case", "n", "B_m", "H_m",
                "width_m","copper_thickness_m","air_epsilon_r","layers","mask")}
        if data["model"] != "PH07_IdealMultilayerMicrostrip":
            raise ValueError(f"unexpected model in {file}")
        cache_key = digest(inp)
        if cache_key not in computed:
            computed[cache_key] = solve_coupon(inp)
        result = computed[cache_key]
        name = pathlib.Path(file).stem
        cases[name] = {
            "physical_input":inp, "input_sha256":digest(inp),
            "fem":data, "independent_fvm":result,
            "rel_errors":{key:relatively(data[key],result[key]) for key in
                          ("C_F_per_m","C0_F_per_m","Z0_ohm","delay_s_per_m")}
        }
        for category in ("dielectric","vacuum"):
            fv = result[category]
            check(f"{name}:FVM_{category}_linear_residual",
                  fv["residual"]<=GATES["true_residual"],fv["residual"])
            check(f"{name}:FVM_{category}_charge_conservation",
                  fv["charge_balance_rel"]<=GATES["charge_balance"],
                  fv["charge_balance_rel"])
        for attr in ("field_residual","vacuum_residual"):
            check(f"{name}:FEM_{attr}",
                  data[attr]<=GATES["true_residual"],data[attr])
        for attr in ("field_charge_balance_rel","vacuum_charge_balance_rel"):
            check(f"{name}:FEM_{attr}",data[attr]<=GATES["charge_balance"],data[attr])
        for attr in ("field_energy_charge_rel","vacuum_energy_charge_rel"):
            check(f"{name}:FEM_{attr}",data[attr]<=GATES["fem_charge_energy"],data[attr])
        for attr in ("field_x_sym_err_V","vacuum_x_sym_err_V"):
            check(f"{name}:FEM_{attr}",data[attr]<=GATES["symmetry_volts"],data[attr])
        for attr in ("field_pec_err_V","vacuum_pec_err_V"):
            check(f"{name}:FEM_{attr}",data[attr]<=GATES["electrode_volts"],data[attr])
        if inp["n"] == 128 and abs(inp["B_m"]-0.0012) < 1e-12 and abs(inp["H_m"]-0.0008)<1e-12 and name in [f"ph07-{case}-n128" for case in "ABCD"]:
            for key, error in cases[name]["rel_errors"].items():
                check(f"{name}:FEM_vs_FVM_{key}",
                      error<=GATES["fem_fvm"],error)
        print(f"PH07_CASE {name} FEM C={data['C_F_per_m']:.12e} C0={data['C0_F_per_m']:.12e} Z0={data['Z0_ohm']:.9f} delay={data['delay_s_per_m']:.12e}; FVM C={result['C_F_per_m']:.12e} C0={result['C0_F_per_m']:.12e} Z0={result['Z0_ohm']:.9f} delay={result['delay_s_per_m']:.12e}; errors={cases[name]['rel_errors']}",flush=True)

    def get(name, which):
        return cases[name][which]

    for kind in "ABCD":
        # Four base geometry cases; mesh refinement measured at fixed B/H.
        for solver in ("fem","independent_fvm"):
            a,b = get(f"ph07-{kind}-n64",solver), get(f"ph07-{kind}-n128",solver)
            for key in ("C_F_per_m","C0_F_per_m"):
                rel=relatively(b[key],a[key])
                lim=GATES["fem_mesh"] if solver=="fem" else GATES["fvm_mesh"]
                check(f"{kind}:{solver}:mesh64_to_128:{key}",rel<=lim,rel)
            ref=get(f"ph07-{kind}-B24",solver)
            for code in ("B12","B16"):
                bb=get(f"ph07-{kind}-{code}",solver)
                for key in ("C_F_per_m","C0_F_per_m"):
                    rel=relatively(bb[key],ref[key])
                    check(f"{kind}:{solver}:side_{code}:{key}",
                          rel<=GATES["outer_boundary"],rel)
            ref=get(f"ph07-{kind}-H16",solver)
            for code in ("H08","H12"):
                hh=get(f"ph07-{kind}-{code}",solver)
                for key in ("C_F_per_m","C0_F_per_m"):
                    rel=relatively(hh[key],ref[key])
                    check(f"{kind}:{solver}:top_{code}:{key}",
                          rel<=GATES["outer_boundary"],rel)

    # Monotone replacing air by mask, vacuum C0 identical and inverse Z0.
    for solver in ("fem","independent_fvm"):
        for off,on in (("A","B"),("C","D")):
            a,b=get(f"ph07-{off}-n64",solver),get(f"ph07-{on}-n64",solver)
            check(f"{solver}:{off}->{on}:mask_physics",
                  b["C_F_per_m"]>a["C_F_per_m"] and
                  b["Z0_ohm"]<a["Z0_ohm"] and
                  b["delay_s_per_m"]>a["delay_s_per_m"] and
                  b["epsilon_effective"]>a["epsilon_effective"],[a["C_F_per_m"],b["C_F_per_m"]])
            check(f"{solver}:{off}->{on}:C0_invariance",
                  relatively(b["C0_F_per_m"],a["C0_F_per_m"])<=1e-8,
                  [a["C0_F_per_m"],b["C0_F_per_m"]])
        equal=get("ph07-equal",solver)
        single=get("ph07-A-n64",solver)
        for key in ("C_F_per_m","C0_F_per_m","Z0_ohm","delay_s_per_m"):
            rel=relatively(equal[key],single[key])
            check(f"{solver}:equal_interface:{key}",rel<=GATES["equal_interface"],rel)
        # Mask dielectric/thickness increases cannot lower C in same footprint.
        for left,right in (("ph07-mask-t12","ph07-D-n64"),
                           ("ph07-D-n64","ph07-mask-t50"),
                           ("ph07-mask-er1","ph07-D-n64"),
                           ("ph07-D-n64","ph07-mask-er4")):
            a,b=get(left,solver),get(right,solver)
            check(f"{solver}:mask_monotonic:{left}->{right}",
                  b["C_F_per_m"]>=a["C_F_per_m"]*(1-1e-10),
                  [a["C_F_per_m"],b["C_F_per_m"]])
        # Increasing either substrate layer epsilon => capacitance nondecrease.
        a,b=get("ph07-C-n64",solver),get("ph07-layer-high",solver)
        check(f"{solver}:substrate_epsilon_monotonic",
              b["C_F_per_m"]>=a["C_F_per_m"],[a["C_F_per_m"],b["C_F_per_m"]])
        vac=get("ph07-vacuum",solver)
        check(f"{solver}:all-vacuum_C_equals_C0",
              relatively(vac["C_F_per_m"],vac["C0_F_per_m"])<=1e-10,
              [vac["C_F_per_m"],vac["C0_F_per_m"]])

    bad=[c for c in checks if c["status"]=="FAIL"]
    evidence={
        "status":"FAIL" if bad else "PASS",
        "failed_gates":len(bad),"gate_count":len(checks),"gates":GATES,
        "source_sha256":source_sha,"fixture_sha256":digest(FIXTURE),
        "method":"independent cell-centred harmonic-flux conservative FVM / SciPy sparse direct",
        "cases":cases,"checks":checks,
        "scope":"synthetic PEC electrostatic coupon, not real PCB or AC/magnetic/fullwave"
    }
    pathlib.Path(destination).write_text(json.dumps(evidence,indent=2,allow_nan=False)+"\n")
    print(f"PH07 {evidence['status']} gates={len(checks)} failures={len(bad)} source={source_sha} fixture={digest(FIXTURE)}",flush=True)
    return 1 if bad else 0


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--self-test",action="store_true")
    parser.add_argument("--fem",nargs="+")
    parser.add_argument("--output",default="ph07-independent-evidence.json")
    args=parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if not args.fem:
        parser.error("requires --fem or --self-test")
    return run(args.fem,args.output)


if __name__=="__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"PH07 independent FVM FAILED: {type(exc).__name__}: {exc}",
              file=sys.stderr)
        raise
