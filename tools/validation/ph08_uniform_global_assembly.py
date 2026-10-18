#!/usr/bin/env python3
"""Bounded PUBLIC synthetic global uniform-square HMM assembly and solve.

No archived/private HMM is imported. Exact Fraction matrix/residual diagnostics
cover the conditional energy from PR #70, not an original physical certificate.
"""
from __future__ import annotations

from fractions import Fraction as Q
from math import inf, isfinite, nextafter, sqrt

import numpy as np

from ph08_gateq_exact_captured import inverse, norm_inf, qstr

_ALLOWED_N = frozenset((2, 3, 4, 8))
_MUTATIONS = frozenset(("none", "wrong_source", "midpoint_boundary", "solution_ulp"))


def _phi(x: Q, z: Q) -> Q:
    return x*x + z*z


def _face_value(key: tuple, h: Q, midpoint: bool = False) -> Q:
    kind, i, j = key
    if kind == "v":
        x, z = i*h, (Q(j) + Q(1, 2))*h
    elif kind == "h":
        x, z = (Q(i) + Q(1, 2))*h, j*h
    else:
        raise ValueError("not a face")
    return _phi(x, z) + (Q(0) if midpoint else h*h/12)


def _candidate(key: tuple, h: Q) -> Q:
    kind, i, j = key
    if kind == "c":
        x, z = (Q(i) + Q(1, 2))*h, (Q(j) + Q(1, 2))*h
        return _phi(x, z) - h*h/6
    return _face_value(key, h)


def _local_stiffness() -> list[list[Q]]:
    # Local order: cell, right, left, up, down.
    # h^2|G|^2 + 2 sum R_f^2
    # = (r-l)^2 + (u-d)^2 + 4*((r+l)/2-c)^2
    #                              + 4*((u+d)/2-c)^2.
    gx = (Q(0), Q(1), Q(-1), Q(0), Q(0))
    gz = (Q(0), Q(0), Q(0), Q(1), Q(-1))
    rx = (Q(-1), Q(1, 2), Q(1, 2), Q(0), Q(0))
    rz = (Q(-1), Q(0), Q(0), Q(1, 2), Q(1, 2))
    return [[gx[a]*gx[b] + gz[a]*gz[b] + 4*rx[a]*rx[b] +
             4*rz[a]*rz[b] for b in range(5)] for a in range(5)]


def assemble(n: int, *, source: Q = Q(-4),
             midpoint_boundary: bool = False):
    """Assemble exact reduced free-DOF K and b; fixed faces enter RHS."""
    if type(n) is not int or n not in _ALLOWED_N:
        raise ValueError("only bounded public synthetic meshes: 2,3,4,8")
    if type(source) is not Q or type(midpoint_boundary) is not bool:
        raise ValueError("invalid exact source or boundary flag")
    h = Q(1, n)
    cells = [("c", i, j) for j in range(n) for i in range(n)]
    verts = [("v", i, j) for j in range(n) for i in range(1, n)]
    horiz = [("h", i, j) for j in range(1, n) for i in range(n)]
    keys = cells + verts + horiz
    ids = {k: p for p, k in enumerate(keys)}
    matrix = [dict() for _ in keys]
    rhs = [source*h*h if k[0] == "c" else Q(0) for k in keys]
    boundary = {}
    local = _local_stiffness()
    for j in range(n):
        for i in range(n):
            nodes = (("c", i, j), ("v", i+1, j), ("v", i, j),
                     ("h", i, j+1), ("h", i, j))
            for a, key_a in enumerate(nodes):
                if key_a not in ids:
                    continue
                p = ids[key_a]
                for b, key_b in enumerate(nodes):
                    coefficient = local[a][b]
                    if key_b in ids:
                        q = ids[key_b]
                        matrix[p][q] = matrix[p].get(q, Q(0)) + coefficient
                    else:
                        val = _face_value(key_b, h, midpoint_boundary)
                        boundary[key_b] = val
                        rhs[p] -= coefficient*val
    return keys, matrix, rhs, boundary


def _matvec(matrix, vector):
    return [sum((a*vector[j] for j, a in row.items()), Q(0))
            for row in matrix]


def _maxabs(items):
    return max((abs(v) for v in items), default=Q(0))


def analyze_global_synthetic(n: int, *, mutation: str = "none") -> dict:
    """Exact assembled-operator checks plus observed NumPy binary64 solve."""
    if type(mutation) is not str or mutation not in _MUTATIONS:
        raise ValueError("unknown synthetic mutation")
    keys, k, b, boundary = assemble(
        n, source=Q(4) if mutation == "wrong_source" else Q(-4),
        midpoint_boundary=(mutation == "midpoint_boundary"))
    h = Q(1, n)
    candidate = [_candidate(key, h) for key in keys]
    ideal_res = [a-bb for a, bb in zip(_matvec(k, candidate), b)]
    if len(k) > 176:
        raise ValueError("synthetic matrix unexpectedly large")
    full = np.zeros((len(keys), len(keys)), dtype=np.float64)
    for i, row in enumerate(k):
        for j, val in row.items():
            full[i, j] = float(val)
    observed = np.linalg.solve(full, np.array([float(v) for v in b]))
    if not np.isfinite(observed).all():
        raise ValueError("nonfinite synthetic solver output")
    if mutation == "solution_ulp":
        observed[0] = nextafter(float(observed[0]), inf)
    captured = [Q.from_float(float(v)) for v in observed]
    exact_numeric_res = [a-bb for a, bb in zip(_matvec(k, captured), b)]
    actual_forward = _maxabs(a-bb for a, bb in zip(captured, candidate))
    residual_norm = _maxabs(exact_numeric_res)
    # This local bound is exact only for the n=2 SYNTHETIC exact K and b.
    # It does not cover upstream input/assembly/rounding, nor original HMM.
    bounded = None
    if n == 2 and not any(ideal_res):
        dense = [[row.get(j, Q(0)) for j in range(len(keys))]
                 for row in k]
        inverse_norm = norm_inf(inverse(dense))
        upper = inverse_norm*residual_norm
        if actual_forward > upper:
            raise AssertionError("synthetic exact residual enclosure failed")
        bounded = {"scope": "CONDITIONAL_EXACT_SYNTHETIC_K_B_ONLY",
                   "inverse_norm_inf_exact": qstr(inverse_norm),
                   "forward_bound_inf_exact": qstr(upper),
                   "observed_forward_inf_exact": qstr(actual_forward),
                   "enclosure_verified": True}
    value = {key: float(v) for key, v in zip(keys, observed)}
    value.update({key: float(v) for key, v in boundary.items()})
    grad_errors = []
    for j in range(n):
        for i in range(n):
            gx = (value[("v", i+1, j)] - value[("v", i, j)])/float(h)
            gz = (value[("h", i, j+1)] - value[("h", i, j)])/float(h)
            grad_errors.extend((gx - float(Q(2*i+1)*h),
                                gz - float(Q(2*j+1)*h)))
    grad_rms = sqrt(sum(v*v for v in grad_errors)/len(grad_errors))
    if not isfinite(grad_rms):
        raise ValueError("nonfinite synthetic gradient")
    return {
        "scope": "PUBLIC_SYNTHETIC_GLOBAL_ASSEMBLY_ONLY",
        "n": n, "mutation": mutation,
        "free_unknown_count": len(keys),
        "cell_count": n*n, "interior_face_count": 2*n*(n-1),
        "exact_ideal_discrete_residual_inf": qstr(_maxabs(ideal_res)),
        "exact_captured_solution_residual_inf": qstr(residual_norm),
        "ideal_candidate_solves_assembled_system": not any(ideal_res),
        "float64_global_solve_gradient_rms_observed": grad_rms,
        "conditional_exact_forward_enclosure": bounded,
        "actual_original_hmm_full_domain": "NOT_RUN",
        "original_recovery_executed": False,
        "original_whole_rms_bound": "BOUND_NOT_CERTIFIED",
        "physical_forward_bound": "BOUND_NOT_CERTIFIED",
        "historical_stage4_gate_q": "FAIL",
        "historical_stage5b_gate_q": "FAIL",
        "AC_P1": "PENDING",
        "TR": "PROPOSED_NOT_FROZEN",
        "contract": "NOT_FROZEN",
        "rust_fem": "NO_GO",
        "Gate_M": "NOT_VERIFIED",
    }


if __name__ == "__main__":
    import json
    for n in sorted(_ALLOWED_N):
        print(json.dumps(analyze_global_synthetic(n), sort_keys=True))
