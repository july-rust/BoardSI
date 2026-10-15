#!/usr/bin/env python3
"""PH-05 independent *cell-centred FVM* verifier, not Rust P1 FEM.

2D transverse conductor: x in [-w/2,w/2], z in [(b-t)/2,(b+t)/2].
PEC ground at z=0,b; side walls are homogeneous zero-normal-flux Neumann.
Uniform dielectric; rectangular 1-V signal; 0-V top/bottom. Distinct FVM
faces, unknown cell-centres, sparse direct solve, conductor face charge.
Neither FEM potentials nor FEM matrices are inputs to FVM reference.
ONLY a synthetic lossless quasi-TEM ideal-coupon validation is intended.
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
B = 0.00080
ER = 3.4
# Frozen physical coupon and independent mesh fixtures, NOT derived from FEM.
CASES = (
    (32, 0.0012),
    (64, 0.0012),
    (128, 0.0012),
    (64, 0.0016),
    (64, 0.0024),
)
CONTRACT = {
    "fem_vs_fvm_C_C0_Z0_rel_n128": 0.02,
    "fem_mesh_C_C0_rel_64_to_128": 0.015,
    "fvm_mesh_C_C0_rel_64_to_128": 0.01,
    "fem_boundary_C_C0_rel_vs_B2p4": 0.02,
    "math_ratio_rel": 0.001,
    "math_delay_rel": 0.001,
    "true_residual": 1e-8,
    "energy_charge_rel": 1e-6,
    "phi_reflection_v": 1e-6,
    "ground_asymmetry_rel": 0.001,
    "total_charge_imbalance_rel": 1e-6,
}
FIXTURE = {
    "w_m": W, "t_m": T, "b_m": B, "epsilon_r": ER,
    "signal_pec_v": 1, "upper_lower_ground_pec_v": 0,
    "side_boundary": "homogeneous Neumann",
    "material": "homogeneous isotropic lossless mu_r=1",
    "cases": CASES,
}


def aligned(a, step):
    return abs(a / step - round(a / step)) <= 1e-8


def solve_fvm(n, half_span, er=ER):
    """Four-face FV flux stencil; grounded top/bottom, insulated left/right."""
    if not all(np.isfinite(v) for v in (n, half_span, er)):
        raise ValueError("nonfinite FV parameter")
    if int(n) != n or n < 8 or n > 256 or half_span <= W / 2 or er < 1:
        raise ValueError("unsupported FV geometry/material")
    step = B / n
    nx_f = 2 * half_span / step
    if not all(aligned(q, step) for q in
               (half_span, 2 * half_span, W / 2, T, (B - T) / 2, (B + T) / 2)):
        raise ValueError("nonconforming FV electrode/box; no snapping allowed")
    nx = round(nx_f)
    nz = n
    if nx * nz > 240_000:
        raise ValueError("FV mesh exceeds memory cap")
    x = -half_span + (np.arange(nx) + 0.5) * step
    z = (np.arange(nz) + 0.5) * step
    metal = (
        (np.abs(x[None, :]) < W / 2) &
        (z[:, None] > (B - T) / 2) &
        (z[:, None] < (B + T) / 2)
    )
    if metal[0].any() or metal[-1].any() or metal[:, 0].any() or metal[:, -1].any():
        raise ValueError("FV PEC overlaps reference or truncated boundary")
    count = int(np.count_nonzero(~metal))
    if count <= 0 or not np.any(metal):
        raise ValueError("missing FV PEC or empty solution domain")
    index = np.full((nz, nx), -1, dtype=np.int32)
    index[~metal] = np.arange(count, dtype=np.int32)
    rows, columns, entries = [], [], []
    rhs = np.zeros(count)
    contact = []
    upper_ground = []
    lower_ground = []
    t0 = time.perf_counter()
    for j in range(nz):
        for i in range(nx):
            if metal[j, i]:
                continue
            u = int(index[j, i])
            diagonal = 0.0
            for dj, di in ((0, -1), (0, 1), (-1, 0), (1, 0)):
                zz, xx = j + dj, i + di
                if xx < 0 or xx >= nx:
                    # Zero sidewall normal flux: do not include a face term.
                    continue
                if zz < 0 or zz >= nz:
                    # Ground lies half a cell away. g=2*epsilon_r.
                    g = 2.0 * er
                    diagonal += g
                    if zz < 0:
                        lower_ground.append((u, g))
                    else:
                        upper_ground.append((u, g))
                elif metal[zz, xx]:
                    # PEC signal on full rectangular cell face at 1 V.
                    g = 2.0 * er
                    diagonal += g
                    rhs[u] += g
                    contact.append((u, g))
                else:
                    g = er  # homogeneous material: harmonic face epsilon=er
                    diagonal += g
                    rows.append(u)
                    columns.append(int(index[zz, xx]))
                    entries.append(-g)
            rows.append(u)
            columns.append(u)
            entries.append(diagonal)
    a = sp.coo_matrix((entries, (rows, columns)), shape=(count, count)).tocsr()
    xfield = spsolve(a, rhs)
    if not np.isfinite(xfield).all():
        raise ArithmeticError("nonfinite FV solution")
    residual = float(np.linalg.norm(a @ xfield - rhs) /
                     max(np.linalg.norm(rhs), np.finfo(float).tiny))
    charge = EPS0 * sum(g * (1 - xfield[u]) for u, g in contact)
    qtop = -EPS0 * sum(g * xfield[u] for u, g in upper_ground)
    qbottom = -EPS0 * sum(g * xfield[u] for u, g in lower_ground)
    balance = abs((charge + qtop + qbottom) / charge)
    ground_asym = abs(qtop - qbottom) / ((abs(qtop) + abs(qbottom)) / 2)
    full = np.ones((nz, nx))
    full[~metal] = xfield
    xmirror = float(np.max(np.abs(full - full[:, ::-1])))
    zmirror = float(np.max(np.abs(full - full[::-1, :])))
    out = {
        "n": int(n), "B_m": half_span, "C_F_per_m": float(charge),
        "nx": int(nx), "nz": int(nz), "dof": count, "nnz": int(a.nnz),
        "field_residual": residual, "signal_charge_C": float(charge),
        "upper_ground_charge_C": float(qtop), "lower_ground_charge_C": float(qbottom),
        "charge_balance_relative": float(balance),
        "ground_asymmetry_relative": float(ground_asym),
        "x_reflection_max_V": xmirror, "z_reflection_max_V": zmirror,
        "wall_s": time.perf_counter() - t0,
    }
    if (residual > CONTRACT["true_residual"] or balance > 1e-6 or
        ground_asym > 0.001 or xmirror > 1e-6 or zmirror > 1e-6):
        raise ArithmeticError(f"independent FVM failed self-consistency: {out}")
    return out


def derived(mat, vacuum):
    cap = mat["C_F_per_m"]
    cap0 = vacuum["C_F_per_m"]
    inductance = MU0 * EPS0 / cap0
    return {
        "C_F_per_m": cap,
        "C0_F_per_m": cap0,
        "Lext_H_per_m": inductance,
        "Z0_ohm": (inductance / cap) ** 0.5,
        "delay_s_per_m": (inductance * cap) ** 0.5,
        "epsilon_effective": cap / cap0,
    }


def rel(actual, expected):
    return abs(actual / expected - 1)


def examine_fem(fem):
    checks = []
    def gate(name, observed, maximum):
        checks.append({
            "gate": name, "observed": float(observed), "max": float(maximum),
            "status": "PASS" if np.isfinite(observed) and observed <= maximum else "FAIL",
        })

    for n, span in CASES:
        row = fem[(n, span)]
        if (row.get("model") != "PH05_IdealSymmetricStripline" or
            rel(row["w_m"], W) > 1e-10 or rel(row["t_m"], T) > 1e-10 or
            rel(row["b_m"], B) > 1e-10 or rel(row["epsilon_r"], ER) > 1e-10):
            raise ValueError(f"FEM geometry differs from frozen fixture: {(n, span)}")
        gate(f"fem C/C0=er n{n} B{span}", rel(row["epsilon_effective"], ER),
             CONTRACT["math_ratio_rel"])
        gate(f"fem delay identity n{n} B{span}",
             rel(row["delay_s_per_m"], (MU0 * EPS0 * ER) ** 0.5),
             CONTRACT["math_delay_rel"])
        for label in ("field_residual", "vacuum_residual"):
            gate(f"fem {label} n{n} B{span}", row[label], CONTRACT["true_residual"])
        for label in ("field_energy_charge_rel", "vacuum_energy_charge_rel"):
            gate(f"fem {label} n{n} B{span}", row[label], CONTRACT["energy_charge_rel"])
        for label in ("field_x_sym_err_V", "field_z_sym_err_V",
                      "vacuum_x_sym_err_V", "vacuum_z_sym_err_V"):
            gate(f"fem {label} n{n} B{span}", row[label], CONTRACT["phi_reflection_v"])
        for label in ("field_ground_asym_rel", "vacuum_ground_asym_rel"):
            gate(f"fem {label} n{n} B{span}", row[label], CONTRACT["ground_asymmetry_rel"])
        for label in ("field_charge_balance_rel", "vacuum_charge_balance_rel"):
            gate(f"fem {label} n{n} B{span}", row[label],
                 CONTRACT["total_charge_imbalance_rel"])
        if row["field_fringe_Ex_V_per_m"] <= 0 or row["field_pec_field_V_per_m"] > 1e-6:
            gate(f"fem fringe and interior PEC n{n} B{span}", 1, 0)
    return checks, gate


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--fem", nargs="*", default=[], help="Five independent PH-05 FEM JSON CLI outputs")
    ap.add_argument("--output", default="ph05-independent-fvm-evidence.json")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        a = solve_fvm(32, 0.0012, 1.0)
        b = solve_fvm(32, 0.0012, ER)
        assert rel(b["C_F_per_m"] / a["C_F_per_m"], ER) <= 0.001
        print("PH-05 independent FVM self-test PASS; analytic homogeneous-epsilon scaling")
        return

    if len(args.fem) != len(CASES):
        raise SystemExit(f"requires exactly {len(CASES)} separately produced FEM JSON files")
    inputs = {}
    fem_sha256 = {}
    for path in args.fem:
        name = Path(path)
        content = name.read_bytes()
        row = json.loads(content)
        key = (int(row["n"]), float(row["B_m"]))
        if key not in CASES or key in inputs:
            raise SystemExit(f"unexpected or duplicated FEM fixture {key}")
        inputs[key] = row
        fem_sha256[name.name] = hashlib.sha256(content).hexdigest()
    if set(inputs) != set(CASES):
        raise SystemExit("FEM fixture set incomplete")

    references = {}
    start = time.perf_counter()
    for n, span in CASES:
        field = solve_fvm(n, span, ER)
        vacuum = solve_fvm(n, span, 1.0)
        rec = derived(field, vacuum)
        references[(n, span)] = rec
        rec.update({"n": n, "B_m": span, "field": field, "vacuum": vacuum})
        print(f"FVM n={n} B={span:.4g}: C={rec['C_F_per_m']:.12g}, "
              f"C0={rec['C0_F_per_m']:.12g}, Z0={rec['Z0_ohm']:.9g}", flush=True)
    checks, gate = examine_fem(inputs)
    for quantity in ("C_F_per_m", "C0_F_per_m", "Z0_ohm"):
        f = inputs[(128, 0.0012)][quantity]
        v = references[(128, 0.0012)][quantity]
        gate(f"independent FEM/FVM n128 {quantity}",
             rel(f, v), CONTRACT["fem_vs_fvm_C_C0_Z0_rel_n128"])
    for quantity in ("C_F_per_m", "C0_F_per_m"):
        gate(f"FEM mesh64to128 {quantity}",
             rel(inputs[(64, 0.0012)][quantity], inputs[(128, 0.0012)][quantity]),
             CONTRACT["fem_mesh_C_C0_rel_64_to_128"])
        gate(f"FVM mesh64to128 {quantity}",
             rel(references[(64, 0.0012)][quantity], references[(128, 0.0012)][quantity]),
             CONTRACT["fvm_mesh_C_C0_rel_64_to_128"])
        for span in (0.0012, 0.0016):
            gate(f"FEM boundary B{span} vs B0.0024 {quantity}",
                 rel(inputs[(64, span)][quantity], inputs[(64, 0.0024)][quantity]),
                 CONTRACT["fem_boundary_C_C0_rel_vs_B2p4"])
    bad = [x for x in checks if x["status"] == "FAIL"]
    out = {
        "purpose": "PH-05 independent FVM field reference (class F) + preregistered acceptance",
        "not_validated": ["lossy RLGC", "real PCB", "30 GHz band", "asymmetry", "coupling"],
        "source_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "fixture_sha256": hashlib.sha256(json.dumps(FIXTURE, sort_keys=True).encode()).hexdigest(),
        "fem_input_sha256": fem_sha256,
        "toolchain": {
            "python": sys.version.split()[0], "numpy": np.__version__,
            "scipy": scipy.__version__, "platform": platform.platform(),
        },
        "fixture": FIXTURE,
        "contract": CONTRACT,
        "fvm_wall_s": time.perf_counter() - start,
        "records": [references[c] for c in CASES],
        "checks": checks,
        "status": "FAIL" if bad else "PASS",
        "failed_gates": len(bad),
    }
    Path(args.output).write_text(json.dumps(out, indent=2, sort_keys=True)+"\n",
                                 encoding="utf-8")
    for x in checks:
        if x["status"] == "FAIL":
            print("FAIL", x["gate"], f"{x['observed']:.9g}", "limit", x["max"])
    print("PH-05 independent acceptance:", out["status"],
          f"({len(checks)} gates; {len(bad)} FAIL)")
    print("SHA256(source) =", out["source_sha256"])
    print("SHA256(fixture) =", out["fixture_sha256"])
    print("SHA256(evidence) =", hashlib.sha256(Path(args.output).read_bytes()).hexdigest())
    if bad:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
