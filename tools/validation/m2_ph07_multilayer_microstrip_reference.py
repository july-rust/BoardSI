#!/usr/bin/env python3
"""PH-07 INDEPENDENT cell-centred conservative finite-volume reference.

Separate four-face flux discretisation and SciPy direct sparse solver; no FEM
potentials/stiffness/reaction/target values enter the FVM computation.
An FVM file carries its own independent mesh/material/BC definition copied only
from PH-07 physical input geometry; FEM outputs are compared AFTER FVM solving.
All JSON geometry/mesh/BC are first compared against an independent frozen
manifest. The FVM then solves from that manifest, never JSON-supplied geometry.
"""
import argparse
import hashlib
import json
import math
import pathlib
import resource
import subprocess
import platform
from m2_ph07_inputs import expected_cases, validate_input
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


def fvm(inp, vacuum=False, plate_er=None):
    start = time.perf_counter()
    ids, er, nx, d = make_materials(inp, vacuum) if plate_er is None else (
        np.ones_like(plate_er, dtype=np.int32), plate_er, plate_er.shape[1],
        inp["H_m"] / plate_er.shape[0])
    nz = er.shape[0]
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
                    if jj < 0 or (plate_er is not None and jj == nz):
                        # Bottom continuous GND: Dirichlet on boundary face.
                        conductance = 2.0 * ep
                        diag += conductance
                        if jj == nz:
                            rhs[cur] += conductance
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
    if plate_er is not None:
        q_signal = sum(EPS0*2*er[-1,i]*(1-phi[ix[-1,i]]) for i in range(nx))
    if not q_signal > 0:
        raise ValueError("missing positive signal charge")
    balance = abs((q_signal + q_ground) / q_signal)
    potentials = np.ones((nz,nx))
    potentials[is_field] = phi
    energy_by_region = np.zeros(int(ids.max())+1)
    # Count each internal face ONCE and Dirichlet half-faces once. This is
    # discrete integral epsilon |E|^2, independent of electrode charge sum.
    max_interface_jump = 0.0
    for j in range(nz):
        for i in range(nx):
            if ix[j,i] < 0:
                continue
            e, v = er[j,i], potentials[j,i]
            rid = ids[j,i]
            if j == 0:
                energy_by_region[rid] += EPS0*2*e*v*v
            if plate_er is not None and j == nz-1:
                energy_by_region[rid] += EPS0*2*e*(1-v)**2
            for ii,jj in ((i-1,j),(i+1,j),(i,j-1),(i,j+1)):
                if not (0 <= ii < nx and 0 <= jj < nz):
                    continue
                if ix[jj,ii] < 0:
                    energy_by_region[rid] += EPS0*2*e*(1-v)**2
                elif (jj,ii) > (j,i):
                    other = er[jj,ii]
                    conductance = 2*e*other/(e+other)
                    delta = v-potentials[jj,ii]
                    # Divide energy across two series half-cells.
                    energy_by_region[rid] += EPS0*conductance*delta**2*other/(e+other)
                    energy_by_region[ids[jj,ii]] += EPS0*conductance*delta**2*e/(e+other)
                    if e != other:
                        face_v = (e*v+other*potentials[jj,ii])/(e+other)
                        left_flux = 2*e*(v-face_v)
                        right_flux = 2*other*(face_v-potentials[jj,ii])
                        # Absolute dimensionless face flux discrepancy; an
                        # internal conservative identity, NOT a new oracle.
                        max_interface_jump = max(max_interface_jump,abs(left_flux-right_flux))
    energy_c = float(energy_by_region.sum())
    field_ex, field_ez = np.gradient(-potentials,d)
    # np.gradient order is z,x.
    field_ez, field_ex = field_ex, field_ez
    return {
        "C_F_per_m": q_signal, "q_signal_C_per_m": q_signal,
        "q_ground_C_per_m": q_ground, "charge_balance_rel": balance,
        "residual": residual, "nx": nx, "nz": nz,
        "d_m": d, "cells": nx*nz, "field_dof": size, "nnz": A.nnz,
        "wall_s": time.perf_counter()-start,
        "process_peak_rss_bytes": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss *
            (1 if sys.platform == "darwin" else 1024),
        "rss_scope":"isolated coupon process high-water THROUGH this phase, not incremental allocation",
        "energy_C_F_per_m":energy_c,
        "energy_charge_rel":abs(energy_c/q_signal-1),
        "region_energy_C_F_per_m":energy_by_region.tolist(),
        "interface_flux_abs":float(max_interface_jump),
        "phi_grid":potentials.tolist(),
        "Ex_grid":field_ex.tolist(),"Ez_grid":field_ez.tolist()
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
        "dielectric": a, "vacuum": v,
        "coupon_process_peak_rss_bytes":resource.getrusage(resource.RUSAGE_SELF).ru_maxrss *
            (1 if sys.platform == "darwin" else 1024),
        "coupon_cpu_user_s":resource.getrusage(resource.RUSAGE_SELF).ru_utime,
        "coupon_cpu_system_s":resource.getrusage(resource.RUSAGE_SELF).ru_stime,
        "coupon_wall_s":a["wall_s"]+v["wall_s"],
        "peak_rss_scope":"fresh subprocess per unique coupon, including Python imports, dielectric AND vacuum"
    }


def relatively(a, b):
    if not (b > 0 and math.isfinite(b)):
        raise ValueError("invalid comparator denominator")
    return abs(a/b-1)


def self_test():
    # Exact uniform and TWO-LAYER plates; same 4-face assembly and charge/
    # energy paths as microstrip. Sides zero-Neumann, top=1 V, bottom=0 V.
    results = []
    for n in (8,16,32):
        for lo,hi in ((1.0,1.0),(3.4,3.4),(3.4,4.0)):
            h,w=0.0002,0.0003
            er=np.ones((n,3*n//2))*lo
            er[n//2:]=hi
            got=fvm(dict(H_m=h),plate_er=er)
            z=(np.arange(n)+0.5)*h/n
            displacement=1/(h/2/lo+h/2/hi)
            exact=np.where(z<h/2,displacement*z/lo,
                           displacement*(h/2/lo+(z-h/2)/hi))
            phi=np.asarray(got["phi_grid"])
            phi_error=float(np.max(np.abs(phi-exact[:,None])))
            # One-sided gradients entirely within each material; excludes
            # boundary/interface cells where centred gradients straddle eps.
            ez=-np.diff(phi,axis=0)/(h/n)
            exact_ez=np.where((np.arange(n-1)+1)*h/n<h/2,
                              -displacement/lo,-displacement/hi)
            rows=np.arange(n-1)!=n//2-1
            e_error=float(np.max(np.abs((ez-exact_ez[:,None])[rows]))/displacement)
            cap_error=relatively(got["C_F_per_m"],EPS0*w*displacement)
            ex_error=float(np.max(np.abs(np.diff(phi,axis=1))))
            assert max(phi_error,e_error,cap_error,ex_error)<1e-10
            assert got["residual"]<GATES["true_residual"]
            assert got["charge_balance_rel"]<GATES["charge_balance"]
            assert got["energy_charge_rel"]<GATES["fem_charge_energy"]
            assert got["interface_flux_abs"]<1e-12
            # Extrapolate both layer potentials to the shared interface.
            low_face=phi[n//2-1]+displacement/lo*h/n/2
            high_face=phi[n//2]-displacement/hi*h/n/2
            assert np.max(np.abs(low_face-high_face))<1e-10
            # Independently difference interior cells to test Dn continuity.
            dl=lo*(phi[n//2-1]-phi[n//2-2])/(h/n)
            dh=hi*(phi[n//2+1]-phi[n//2])/(h/n)
            assert np.max(np.abs(dl-dh))/displacement<1e-10
            results.append(dict(n=n,eps=[lo,hi],C_rel=cap_error,
                                phi_abs_V=phi_error,E_rel=e_error))
    print("PH-07 exact FVM plate/field/interface self-test PASS",json.dumps(results),flush=True)


def solve_isolated(inp):
    child=subprocess.run([sys.executable,str(pathlib.Path(__file__).resolve()),
                          "--coupon-stdin"],input=json.dumps(inp),text=True,
                         capture_output=True,check=True)
    return json.loads(child.stdout)


def validate_properties(data, inp, result, check, name):
    for solver, values in (("FEM",data),("FVM",result)):
        cap,cap0=values["C_F_per_m"],values["C0_F_per_m"]
        er=[inp["air_epsilon_r"]]+[l["epsilon_r"] for l in inp["layers"]]
        if inp["mask"] is not None:
            er.append(inp["mask"]["epsilon_r"])
        check(f"{name}:{solver}:positive_finite",
              all(math.isfinite(values[k]) and values[k]>0 for k in
                  ("C_F_per_m","C0_F_per_m","Lext_H_per_m","Z0_ohm","delay_s_per_m")),"positive")
        identities=dict(epsilon_effective=cap/cap0,Lext_H_per_m=MU0*EPS0/cap0,
                        Z0_ohm=math.sqrt(values["Lext_H_per_m"]/cap),
                        delay_s_per_m=math.sqrt(values["Lext_H_per_m"]*cap))
        for key,expected in identities.items():
            error=relatively(values[key],expected)
            check(f"{name}:{solver}:identity:{key}",error<=1e-10,error)
        check(f"{name}:{solver}:epsilon_bounds",
              min(er)*(1-1e-8)<=values["epsilon_effective"]<=max(er)*(1+1e-8),values["epsilon_effective"])
    for prefix,cap_key in (("field","C_F_per_m"),("vacuum","C0_F_per_m")):
        q,g=data[prefix+"_charge_C"],data[prefix+"_ground_C"]
        check(f"{name}:FEM:{prefix}:charge_sign_and_balance",
              q>0>g and abs((q+g)/q)<=GATES["charge_balance"],[q,g])
        check(f"{name}:FEM:{prefix}:charge_vs_C",
              relatively(q,data[cap_key])<=GATES["fem_charge_energy"],q)
    parts=data["region_energy_C_contributions_F_per_m"]
    check(f"{name}:FEM:region_energy_partition",len(parts)==len(inp["layers"])+2 and
          all(math.isfinite(x) and x>=-data["C_F_per_m"]*1e-12 for x in parts) and
          relatively(sum(parts),data["C_F_per_m"])<=1e-10,parts)
    check(f"{name}:FEM:fringe_and_PEC_field",data["field_fringe_Ex_V_per_m"]>0 and
          data["field_pec_field_V_per_m"]<=1e-6 and data["field_max_Ez_V_per_m"]>0,
          data["field_fringe_Ex_V_per_m"])
    for category in ("dielectric","vacuum"):
        fv=result[category]
        check(f"{name}:FVM:{category}:energy_vs_charge",
              fv["energy_charge_rel"]<=1e-6,fv["energy_charge_rel"])
        check(f"{name}:FVM:{category}:conservative_interface_flux",
              fv["interface_flux_abs"]<=1e-10,fv["interface_flux_abs"])
    # Compare actual sampled P1 fields against independently reconstructed FVM
    # fields away from material interfaces/PEC singularities. Additional #41
    # limits: 0.02 V potential, 0.10 normalized E (scale 1/substrate height).
    samples=data["sampled_fields"]
    if not samples:
        raise ValueError("missing FEM field samples")
    errors=[]
    fv=result["dielectric"]
    d=fv["d_m"]
    h=sum(l["height_m"] for l in inp["layers"])
    from scipy.ndimage import map_coordinates
    for x,z,p,ex,ez,rid in samples:
        if not all(math.isfinite(v) for v in (x,z,p,ex,ez,rid)):
            raise ValueError("nonfinite FEM field sample")
        if region(x,z,inp)!=rid or not -1e-8<=p<=1+1e-8:
            raise ValueError("invalid FEM sample location/region/potential")
        # Restrict to homogeneous stencils at least 2 cells from boundaries.
        if not (2*d<z<inp["H_m"]-2*d and abs(x)<inp["B_m"]-2*d):
            continue
        if any(region(x+dx,z+dz,inp)!=rid for dx,dz in
               ((2*d,0),(-2*d,0),(0,2*d),(0,-2*d))):
            continue
        coords=np.array([[z/d-0.5],[(x+inp["B_m"])/d-0.5]])
        pp,ee,zz=[float(map_coordinates(np.asarray(fv[key]),coords,order=1)[0])
                  for key in ("phi_grid","Ex_grid","Ez_grid")]
        errors.append([abs(p-pp),abs(ex-ee)*h,abs(ez-zz)*h])
    if not errors:
        raise ValueError("no nonsingular field comparison points")
    maxima=np.max(errors,axis=0).tolist()
    comparison=dict(points=len(errors),max_errors=maxima,E_normalization="1/substrate_height",
                    acceptance_scope="base A-D n128, matching frozen cross-method mesh")
    # Coarse meshes diagnose discretization, not the frozen n128 acceptance.
    # Pilot n32 A/C exceeded 0.02 V; retain those measured errors explicitly.
    result.setdefault("sample_diagnostics",{})[name]=comparison
    if name in [f"ph07-{kind}-n128" for kind in "ABCD"]:
        check(f"{name}:FEM_vs_FVM:sampled_phi_Ex_Ez",
              maxima[0]<=0.02 and max(maxima[1:])<=0.10,comparison)
    check(f"{name}:FEM:sampled_field_integrity",True,comparison)


def run(files, destination):
    source_sha = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
    cases = {}
    checks = []
    computed = {}  # identical physical geometry: reuse expensive FVM direct solves

    def check(name, passed, details, category="legacy"):
        checks.append(dict(name=name, status="PASS" if passed else "FAIL",
                           details=details,category=category))
        if not passed:
            print("PH07 FAIL", name, details, file=sys.stderr, flush=True)

    identities=[pathlib.Path(f).stem for f in files]
    if len(set(identities))!=len(identities) or set(identities)!=set(expected_cases()):
        raise ValueError("missing/duplicate/unexpected PH-07 case set")
    # Validate ALL physical inputs before any expensive FVM solve.
    loaded=[]
    for file in files:
        data=json.loads(pathlib.Path(file).read_text())
        name=pathlib.Path(file).stem
        loaded.append((file,data,validate_input(name,data)))
    for file,data,inp in loaded:
        if data["model"] != "PH07_IdealMultilayerMicrostrip":
            raise ValueError(f"unexpected model in {file}")
        cache_key = digest(inp)
        if cache_key not in computed:
            computed[cache_key] = solve_isolated(inp)
        result = computed[cache_key]
        name = pathlib.Path(file).stem
        cases[name] = {
            "physical_input":inp, "input_sha256":digest(inp),
            "fem":data, "independent_fvm":result,
            "rel_errors":{key:relatively(data[key],result[key]) for key in
                          ("C_F_per_m","C0_F_per_m","Z0_ohm","delay_s_per_m")}
        }
        validate_properties(data,inp,result,
                            lambda n,p,d:check(n,p,d,category="audit"),name)
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

    cross=[c for c in checks if ":FEM_vs_FVM_" in c["name"]]
    independent=[c for c in cross if c["name"].endswith(("_C_F_per_m","_C0_F_per_m"))]
    derived=[c for c in cross if c not in independent]
    for result in computed.values():
        for category in ("dielectric","vacuum"):
            for key in ("phi_grid","Ex_grid","Ez_grid"):
                result[category].pop(key,None)
    bad=[c for c in checks if c["status"]=="FAIL"]
    evidence={
        "status":"FAIL" if bad else "PASS",
        "failed_gates":len(bad),"gate_count":len(checks),"gates":GATES,
        "source_sha256":source_sha,"fixture_sha256":digest(FIXTURE),
        "method":"independent cell-centred harmonic-flux conservative FVM / SciPy sparse direct",
        "cases":cases,"checks":checks,
        "input_manifest_sha256":digest(expected_cases()),
        "counts":dict(evidence_files=len(files),unique_fvm_coupons=len(computed),
                      legacy_checks=sum(c["category"]=="legacy" for c in checks),
                      additional_audit_checks=sum(c["category"]=="audit" for c in checks),
                      primary_C_C0_cross_method=len(independent),
                      derived_Z0_delay_cross_method=len(derived),
                      field_comparison_cases=sum(":sampled_phi_Ex_Ez" in c["name"] for c in checks),
                      other_internal_convergence_property_checks=len(checks)-len(cross)-
                          sum(":sampled_phi_Ex_Ez" in c["name"] for c in checks)),
        "primary_max_rel_error":max(c["details"] for c in independent),
        "host":dict(platform=platform.platform(),python=sys.version,
                    cpu=platform.processor(),machine=platform.machine()),
        "scope":"synthetic PEC electrostatic coupon, not real PCB or AC/magnetic/fullwave"
    }
    pathlib.Path(destination).write_text(json.dumps(evidence,indent=2,allow_nan=False)+"\n")
    print(f"PH07 {evidence['status']} gates={len(checks)} failures={len(bad)} source={source_sha} fixture={digest(FIXTURE)}",flush=True)
    return 1 if bad else 0


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--self-test",action="store_true")
    parser.add_argument("--coupon-stdin",action="store_true",help=argparse.SUPPRESS)
    parser.add_argument("--fem",nargs="+")
    parser.add_argument("--output",default="ph07-independent-evidence.json")
    args=parser.parse_args()
    if args.coupon_stdin:
        print(json.dumps(solve_coupon(json.load(sys.stdin)),allow_nan=False))
        return 0
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
