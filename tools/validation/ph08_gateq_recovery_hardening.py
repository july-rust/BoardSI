#!/usr/bin/env python3
"""Fail-closed PH-08 six-coefficient operator diagnostics (PUBLIC SYNTHETIC ONLY).

The intercepted matrix is NOT identified by shape. A role and independent
semantic/provenance attestation are mandatory; this is not HMM qualification.
"""
from __future__ import annotations

from fractions import Fraction
import hashlib
import math
from typing import Optional

import numpy as np

ROLES = ("design", "weighted_design", "normal_equation", "linear_system")
METHODS = ("lstsq", "solve", "qr", "cholesky", "lu", "custom")
SCOPE_SYNTHETIC = "SYNTHETIC_IMPLEMENTATION_ONLY"
SCOPE_OPERATOR = "ORIGINAL_OPERATOR_NUMERICAL_ONLY_NOT_PHYSICAL"


def rational(value):
    v = float(value)
    if not math.isfinite(v):
        raise ValueError("nonfinite binary64 input")
    return Fraction.from_float(v)


def finite_matrix(a, b, *, role):
    if role not in ROLES:
        raise ValueError("operator role not identified")
    A = np.asarray(a, dtype=np.float64)
    y = np.asarray(b, dtype=np.float64)
    if (A.ndim != 2 or A.shape[1] != 6 or A.shape[0] < 6 or
            y.shape != (A.shape[0],) or not np.isfinite(A).all()
            or not np.isfinite(y).all()):
        raise ValueError("unsupported or nonfinite six-coefficient operator")
    if role in ("normal_equation", "linear_system") and A.shape != (6, 6):
        raise ValueError("square linear system requires exactly six equations")
    if role == "normal_equation" and not np.allclose(A, A.T, rtol=1e-12, atol=0.0):
        raise ValueError("normal matrix is not symmetric")
    return A, y


def exact_linear_solve(square, rhs):
    """Fraction Gaussian elimination with nonzero pivoting; singular => error."""
    n = len(rhs)
    if not n or len(square) != n or any(len(row) != n for row in square):
        raise ValueError("non-square rational system")
    aug = [[Fraction(x) for x in row] + [Fraction(rhs[i])]
           for i, row in enumerate(square)]
    for k in range(n):
        pivot = next((i for i in range(k, n) if aug[i][k] != 0), None)
        if pivot is None:
            raise ValueError("rank deficient exact system")
        aug[k], aug[pivot] = aug[pivot], aug[k]
        d = aug[k][k]
        aug[k] = [v / d for v in aug[k]]
        for i in range(n):
            if i == k:
                continue
            factor = aug[i][k]
            if factor:
                aug[i] = [v - factor * x for v, x in zip(aug[i], aug[k])]
    return [row[-1] for row in aug]


def rational_operator(a, b, *, role, weights=None):
    """Reference for exact binary64 operands, not ideal real/PDE operands.

    For design roles this solves an exact weighted least-squares normal
    equation; for system roles it solves the supplied system DIRECTLY.
    An existing 6x6 normal matrix must NOT be squared again.
    """
    A, y = finite_matrix(a, b, role=role)
    M = [[rational(t) for t in row] for row in A]
    v = [rational(t) for t in y]
    if role in ("normal_equation", "linear_system"):
        if weights is not None:
            raise ValueError("weights cannot be reapplied to square system")
        return exact_linear_solve(M, v)
    if weights is None:
        if role == "weighted_design":
            raise ValueError("weight convention unknown")
        w = [Fraction(1)] * len(v)
    else:
        w = [rational(t) for t in weights]
        if len(w) != len(v) or any(t <= 0 for t in w):
            raise ValueError("weights must be finite strictly positive diagonal")
    H = [[sum(w[k] * M[k][i] * M[k][j] for k in range(len(v)))
          for j in range(6)] for i in range(6)]
    rhs = [sum(w[k] * M[k][i] * v[k] for k in range(len(v)))
           for i in range(6)]
    return exact_linear_solve(H, rhs)


def gradient_transfer(a, gradient_map, *, weights=None):
    """EXACT linear map T=D(A^T W A)^-1 A^T W for given binary64 entries.

    Returns rational 2 x m map and exact ||T||_infinity. This is an exact
    perturbation bound for the specified *linear algebra* under ||dy||_infty;
    it does not bound original HMM potential or floating solver errors.
    """
    A = np.asarray(a, dtype=np.float64)
    D = np.asarray(gradient_map, dtype=np.float64)
    if (A.ndim != 2 or A.shape[1] != 6 or A.shape[0] < 6 or
            D.shape != (2, 6) or not np.isfinite(A).all()
            or not np.isfinite(D).all()):
        raise ValueError("explicit finite 2x6 physical-gradient map required")
    n = A.shape[0]
    w = [Fraction(1)] * n if weights is None else [rational(t) for t in weights]
    if len(w) != n or any(t <= 0 for t in w):
        raise ValueError("invalid diagonal weights")
    M = [[rational(t) for t in row] for row in A]
    G = [[rational(t) for t in row] for row in D]
    H = [[sum(w[k] * M[k][i] * M[k][j] for k in range(n))
          for j in range(6)] for i in range(6)]
    cols = []
    for k in range(n):
        rhs = [M[k][i] * w[k] for i in range(6)]
        cols.append(exact_linear_solve(H, rhs))
    T = [[sum(G[q][i] * cols[k][i] for i in range(6))
          for k in range(n)] for q in range(2)]
    envelope = max(sum(abs(v) for v in row) for row in T)
    return T, envelope


def transfer_error(a, gradient_map, potential_error, *, weights=None):
    T, induced_inf = gradient_transfer(a, gradient_map, weights=weights)
    if len(potential_error) != len(T[0]):
        raise ValueError("potential error length does not match samples")
    delta = [rational(t) for t in potential_error]
    dg = [sum(v * d for v, d in zip(row, delta)) for row in T]
    max_err = max(abs(x) for x in delta)
    if any(abs(x) > induced_inf * max_err for x in dg):
        raise ArithmeticError("exact transfer envelope violated")
    return {"gradient_error": [str(x) for x in dg],
            "induced_inf_exact": str(induced_inf),
            "potential_error_inf_exact": str(max_err),
            "gradient_error_inf_bound_exact": str(induced_inf * max_err),
            "scope": "EXACT_OPERATOR_ON_CAPTURED_BINARY64_INPUTS_ONLY"}


def observed_diagnostics(a, b, coefficients, *, role, method, weights=None):
    """Observed residual/condition only; never a certified float bound."""
    if method not in METHODS:
        raise ValueError("unidentified solver algorithm")
    A, y = finite_matrix(a, b, role=role)
    c = np.asarray(coefficients, dtype=np.float64)
    if c.shape != (6,) or not np.isfinite(c).all():
        raise ValueError("invalid observed coefficients")
    if role in ("design", "weighted_design"):
        if method == "solve":
            raise ValueError("direct solve incompatible with rectangular design role")
        if role == "weighted_design" and weights is None:
            raise ValueError("weight convention missing")
        w = np.ones(len(y)) if weights is None else np.asarray(weights, dtype=float)
        if w.shape != y.shape or not np.isfinite(w).all() or np.any(w <= 0):
            raise ValueError("invalid weights")
        H = A.T @ (w[:, None] * A)
        rhs = A.T @ (w * y)
        s = np.linalg.svd(A, compute_uv=False)
        kappa_A = float(s[0] / s[-1]) if s[-1] else math.inf
        primal_residual = float(np.linalg.norm(A @ c - y))
    else:
        if method == "lstsq":
            raise ValueError("least-squares entrypoint does not establish square system role")
        if weights is not None:
            raise ValueError("square role already includes any weights")
        H = A
        rhs = y
        s = np.linalg.svd(H, compute_uv=False)
        kappa_A = None
        primal_residual = None
    if np.linalg.matrix_rank(H) != 6:
        raise ValueError("rank deficient operator")
    normal_residual = float(np.linalg.norm(H @ c - rhs))
    denom = float(np.linalg.norm(H, 2) * np.linalg.norm(c) + np.linalg.norm(rhs))
    return {"matrix_role": role, "observed_method": method, "rank": 6,
            "condition_design_2": kappa_A,
            "condition_supplied_system_2": float(np.linalg.cond(H)),
            "singular_values_supplied": [float(x) for x in s],
            "primal_residual_norm2": primal_residual,
            "system_residual_norm2": normal_residual,
            "observed_relative_system_backward_error": (normal_residual / denom if denom else 0.0),
            "reference": "exact binary64 operands; role must be independently attested",
            "certified_float_bound": "BOUND_NOT_CERTIFIED"}


def verdict(observed, reference, *, matrix_role=None, source_digest=None,
            expected_source_digest=None, same_problem=False,
            gradient_map=None, tolerance=None, coefficient_tolerance=None,
            tolerance_preregistered=False,
            float_bound_certified=False, scope=SCOPE_SYNTHETIC):
    """Do not promote a passing comparison to HMM physical qualification.

    Required independent provenance/semantics must come from evidence, not
    inferred from 6x6 shape or chosen by caller to silence a failing test.
    """
    result = {"status": "NOT VERIFIED", "scope": scope,
              "historical_gate_q": "FAIL", "AC_P1": "PENDING",
              "contract": "NOT FROZEN", "rust_fem": "NO-GO",
              "certified_float_bound": "BOUND_NOT_CERTIFIED"}
    try:
        x = np.asarray(observed, dtype=np.float64)
        y = np.asarray(reference, dtype=np.float64)
        if x.shape != (6,) or y.shape != (6,) or not np.isfinite(x).all() or not np.isfinite(y).all():
            result["status"] = "FAIL"
            result["reason"] = "nonfinite or malformed coefficient vector"
            return result
        if gradient_map is not None:
            D = np.asarray(gradient_map, dtype=np.float64)
            if D.shape != (2, 6) or not np.isfinite(D).all():
                result["reason"] = "gradient mapping invalid"
                return result
        else:
            result["reason"] = "physical gradient mapping not provided"
            return result
        if matrix_role not in ROLES or not same_problem:
            result["reason"] = "operator semantics not independently matched"
            return result
        if (not source_digest or not expected_source_digest or
                source_digest != expected_source_digest or
                len(source_digest) != 64 or
                any(t not in "0123456789abcdef" for t in source_digest)):
            result["reason"] = "source attestation absent/mismatch"
            return result
        if (tolerance is None or coefficient_tolerance is None or
                not tolerance_preregistered or
                not math.isfinite(float(tolerance)) or float(tolerance) < 0 or
                not math.isfinite(float(coefficient_tolerance)) or
                float(coefficient_tolerance) < 0):
            result["reason"] = "pre-registered sensitivity tolerance absent"
            return result
        g_delta = D @ (x - y)
        if not np.isfinite(g_delta).all():
            result["status"] = "FAIL"
            result["reason"] = "nonfinite recovered gradient difference"
            return result
        diff = float(np.max(np.abs(g_delta)))
        coeff_diff = float(np.max(np.abs(x-y)))
        result["gradient_difference_inf_observed"] = diff
        result["coefficient_difference_inf_observed"] = coeff_diff
        if diff > float(tolerance) or coeff_diff > float(coefficient_tolerance):
            result["status"] = "FAIL"
            result["reason"] = "gradient mismatch outside pre-registered tolerance"
            return result
        if scope != SCOPE_SYNTHETIC:
            result["reason"] = "public synthetic judge cannot independently certify original HMM"
            return result
        if scope not in (SCOPE_SYNTHETIC, SCOPE_OPERATOR):
            result["reason"] = "unsupported scope"
            return result
        result["status"] = "VERIFIED"
        result["reason"] = "comparison verified within explicitly limited scope"
        return result
    except (ValueError, TypeError, OverflowError):
        result["reason"] = "invalid verification input"
        return result
