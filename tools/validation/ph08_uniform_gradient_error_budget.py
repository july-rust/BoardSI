#!/usr/bin/env python3
"""PUBLIC SYNTHETIC uniform HMM recovery diagnostic, never physical certification.

Exercise the independently proved ideal uniform-square quadratic cell samples
through captured binary64 NumPy least-squares, then reuse the exact rational
two-operator decomposition. No private original HMM code or data is accessed.
"""
from __future__ import annotations
from fractions import Fraction as F
from math import inf, nextafter

import numpy as np
from ph08_recovery_operator_perturbation import analyze_two_operator_budget

_ALLOWED_N = frozenset((8, 16, 32))
_POINTS = tuple((i, j) for j in (-1, 0, 1) for i in (-1, 0, 1))
_U, _V = F(1, 4), F(-1, 4)
_CX, _CZ = F(1, 2), F(1, 2)


def _row(u, v):
    return [1, u, v, u*u, u*v, v*v]


def _derivative(h):
    return [
        [F(0), 1/h, F(0), 2*_U/h, _V/h, F(0)],
        [F(0), F(0), 1/h, F(0), _U/h, 2*_V/h],
    ]


def _coefficients(h):
    return [_CX*_CX + _CZ*_CZ - h*h/6,
            2*_CX*h, 2*_CZ*h, h*h, F(0), h*h]


def _physical_gradient(h):
    return [2*(_CX + _U*h), 2*(_CZ + _V*h)]


def analyze_uniform_synthetic(n, *, design_ulp=False, derivative_ulp=False,
                              sample_ulp=False):
    """Compare exact ideal DISCRETE samples to synthetic float64 recovery.

    ULP switches are controlled adversarial perturbations, not stochastic
    estimates of real geometry/assembly error and not proof of worst-case error.
    """
    if type(n) is not int or n not in _ALLOWED_N:
        raise ValueError("allowed synthetic n: 8, 16, 32 (not bool)")
    for flag in (design_ulp, derivative_ulp, sample_ulp):
        if type(flag) is not bool:
            raise ValueError("perturbation selectors must be bool")
    h = F(1, n)
    ar = [[F(v) for v in _row(F(i), F(j))] for i, j in _POINTS]
    dr = _derivative(h)
    cr = _coefficients(h)
    yr = [sum((a*b for a, b in zip(row, cr)), F(0)) for row in ar]
    # Independently verify all reference samples against analytic cell formula.
    assert yr == [(_CX + i*h)**2 + (_CZ + j*h)**2 - h*h/6
                  for i, j in _POINTS]
    assert [sum((a*b for a, b in zip(row, cr)), F(0)) for row in dr] == _physical_gradient(h)

    ac = [list(map(float, row)) for row in ar]
    if design_ulp:
        ac[0][1] = nextafter(ac[0][1], inf)
        ac[0][3] = ac[0][1] * ac[0][1]
        ac[0][4] = ac[0][1] * ac[0][2]
    dc = [list(map(float, row)) for row in dr]
    if derivative_ulp:
        dc[0][1] = nextafter(dc[0][1], inf)

    # The sample expression deliberately uses binary64 arithmetic; it is NOT
    # a run of the original HMM sparse solve or original assembly.
    yn = []
    for i, j in _POINTS:
        x, z = float(_CX) + i*float(h), float(_CZ) + j*float(h)
        y = x*x + z*z - (float(h)*float(h))/6.0
        yn.append(y)
    if sample_ulp:
        yn[0] = nextafter(yn[0], inf)
    a_np = np.asarray(ac, dtype=np.float64)
    y_np = np.asarray(yn, dtype=np.float64)
    coefs, _, rank, _ = np.linalg.lstsq(a_np, y_np, rcond=None)
    if rank != 6:
        raise ValueError("synthetic recovery is rank deficient")
    g_np = np.asarray(dc, dtype=np.float64) @ coefs
    report = analyze_two_operator_budget(
        captured_A=ac, captured_D=dc, reference_A=ar, reference_D=dr,
        numeric_samples=yn, reference_samples=yr,
        reference_gradient=_physical_gradient(h),
        observed_gradient=[float(v) for v in g_np],
        reference_quadratic_coefficients=cr,
    )
    return {
        "scope": "PUBLIC_SYNTHETIC_UNIFORM_QUADRATIC_ONLY",
        "n": n,
        "sample_count": len(yr),
        "synthetic_mutations": {
            "design_ulp": design_ulp,
            "derivative_ulp": derivative_ulp,
            "sample_ulp": sample_ulp,
        },
        "numeric_solver": "numpy.linalg.lstsq(float64), synthetic inputs",
        "ideal_cell_bias": str(-h*h/6),
        "reference_gradient": [str(v) for v in _physical_gradient(h)],
        "exact_decomposition": report,
        "original_HMM_source_executed": False,
        "original_global_sparse_assembly": "NOT_RUN",
        "actual_original_gradient_error": "NOT_MEASURED_IN_THIS_DIAGNOSTIC",
        "full_floating_forward_bound": "BOUND_NOT_CERTIFIED",
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
        print(json.dumps(analyze_uniform_synthetic(n), sort_keys=True))
