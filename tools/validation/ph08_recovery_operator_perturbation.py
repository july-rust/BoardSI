#!/usr/bin/env python3
"""Exact two-operator error decomposition for PUBLIC PH-08 synthetic inputs.

An algebraic result conditional on caller-supplied reference A,D,y.
Cannot authenticate original HMM geometry, assembly or PDE solution.
"""
from __future__ import annotations
from fractions import Fraction as Q
from itertools import islice

from ph08_gateq_exact_captured import (
    MAX_CAPTURE_ROWS, binary64, inverse, mul, norm_inf, qstr,
)

# Synthetic-only exact arithmetic must reject hostile aggregate rational growth.
# The 128-row shape ceiling is not a promise that any bit complexity is accepted.
MAX_INPUT_BITS_TOTAL = 4096
MAX_RESULT_BITS = 12000

def _guard_complexity(*matrices):
    total = 0
    for matrix in matrices:
        for row in matrix:
            for value in row:
                bits = max(value.numerator.bit_length(), value.denominator.bit_length())
                total += bits
                if total > MAX_INPUT_BITS_TOTAL:
                    raise ValueError("aggregate exact input complexity budget exceeded")

def _safe_qstr(value):
    if max(value.numerator.bit_length(), value.denominator.bit_length()) > MAX_RESULT_BITS:
        raise ValueError("exact result exceeds bounded serialization complexity")
    return qstr(value)

def _bounded(data, maximum):
    try:
        values = list(islice(iter(data), maximum + 1))
    except (TypeError, OverflowError) as exc:
        raise ValueError("invalid bounded iterable") from exc
    if len(values) > maximum:
        raise ValueError("input exceeds bounded diagnostic size")
    return values

def _rational(value):
    if type(value) is not Q:
        raise ValueError("independent reference must be Fraction, not float")
    if max(value.numerator.bit_length(), value.denominator.bit_length()) > 4096:
        raise ValueError("oversize exact reference")
    return value

def _matrix(data, rows, convert, name):
    top = _bounded(data, rows)
    if len(top) != rows:
        raise ValueError("invalid " + name + " row count")
    result = []
    for row in top:
        values = _bounded(row, 6)
        if len(values) != 6:
            raise ValueError("invalid " + name + " column count")
        result.append([convert(v) for v in values])
    return result

def _vector(data, count, convert):
    values = _bounded(data, count)
    if len(values) != count:
        raise ValueError("wrong vector length")
    return [[convert(v)] for v in values]

def _diff(left, right):
    return [[a[0] - b[0]] for a, b in zip(left, right)]

def _difference_matrix(a, b):
    return [[v - w for v, w in zip(x, y)] for x, y in zip(a, b)]

def _vnorm(v):
    return max((abs(x[0]) for x in v), default=Q(0))

def _transfer(a, d):
    at = [list(column) for column in zip(*a)]
    return mul(mul(d, inverse(mul(at, a))), at)

def analyze_two_operator_budget(
    captured_A, captured_D, reference_A, reference_D, numeric_samples,
    reference_samples, reference_gradient, observed_gradient,
    reference_quadratic_coefficients,
):
    """Exact four-term gradient identity; all source provenance unverified."""
    raw = _bounded(captured_A, MAX_CAPTURE_ROWS)
    n = len(raw)
    if n < 6:
        raise ValueError("six-term basis needs at least six samples")
    a = _matrix(raw, n, binary64, "captured A")
    d = _matrix(captured_D, 2, binary64, "captured D")
    ar = _matrix(reference_A, n, _rational, "reference A")
    dr = _matrix(reference_D, 2, _rational, "reference D")
    yn = _vector(numeric_samples, n, binary64)
    yr = _vector(reference_samples, n, _rational)
    gp = _vector(reference_gradient, 2, _rational)
    go = _vector(observed_gradient, 2, binary64)
    cr = _vector(reference_quadratic_coefficients, 6, _rational)
    _guard_complexity(a, d, ar, dr, yn, yr, gp, go, cr)
    tc = _transfer(a, d)
    tr = _transfer(ar, dr)
    dt = _difference_matrix(tc, tr)
    arithmetic = _diff(go, mul(tc, yn))
    sample = mul(tc, _diff(yn, yr))
    operator = mul(dt, yr)
    reference_defect = _diff(mul(tr, yr), gp)
    observed = _diff(go, gp)
    summed = [[sum((v[i][0] for v in
                   (arithmetic, sample, operator, reference_defect)), Q(0))]
              for i in range(2)]
    if summed != observed:
        raise AssertionError("exact two-operator identity violated")
    sample_bound = norm_inf(tc) * _vnorm(_diff(yn, yr))
    operator_bound = norm_inf(dt) * _vnorm(yr)
    if _vnorm(sample) > sample_bound or _vnorm(operator) > operator_bound:
        raise AssertionError("exact operator bound violated")
    const_null = all(sum(row, Q(0)) == 0 for row in tc) and all(
        sum(row, Q(0)) == 0 for row in tr)
    yvals = [v[0] for v in yr]
    half_range = (max(yvals) - min(yvals)) / 2
    centered_bound = norm_inf(dt) * half_range if const_null else None
    if centered_bound is not None and _vnorm(operator) > centered_bound:
        raise AssertionError("exact centered operator bound violated")
    total_bound = sum(
        (_vnorm(v) for v in (arithmetic, sample, operator, reference_defect)),
        Q(0),
    )
    if _vnorm(observed) > total_bound:
        raise AssertionError("exact triangle bound violated")
    return {
        "scope": "TWO_OPERATOR_EXACT_RATIONAL_PUBLIC_SYNTHETIC_ONLY",
        "identity_verified_exactly": True,
        "sample_count": n,
        "reference_quadratic_reproduced": (
            mul(ar, cr) == yr and mul(dr, cr) == gp
        ),
        "both_constant_shift_null_exact": const_null,
        "captured_transfer_norm_inf_exact": _safe_qstr(norm_inf(tc)),
        "operator_difference_norm_inf_exact": _safe_qstr(norm_inf(dt)),
        "arithmetic_error_inf_exact": _safe_qstr(_vnorm(arithmetic)),
        "sample_error_inf_exact": _safe_qstr(_vnorm(sample)),
        "operator_perturbation_error_inf_exact": _safe_qstr(_vnorm(operator)),
        "reference_polynomial_defect_inf_exact": _safe_qstr(_vnorm(reference_defect)),
        "observed_total_error_inf_exact": _safe_qstr(_vnorm(observed)),
        "sample_upper_bound_exact": _safe_qstr(sample_bound),
        "operator_upper_bound_exact": _safe_qstr(operator_bound),
        "centered_operator_upper_bound_exact": (
            _safe_qstr(centered_bound) if centered_bound is not None else None
        ),
        "triangle_upper_bound_exact": _safe_qstr(total_bound),
        "arithmetic_components_exact": [_safe_qstr(v[0]) for v in arithmetic],
        "sample_components_exact": [_safe_qstr(v[0]) for v in sample],
        "operator_components_exact": [_safe_qstr(v[0]) for v in operator],
        "reference_defect_components_exact": [_safe_qstr(v[0]) for v in reference_defect],
        "reference_provenance": "CALLER_SUPPLIED_NOT_INDEPENDENTLY_AUTHENTICATED",
        "actual_original_A_D_assembly": "NOT_VERIFIED",
        "original_HMM_uniform_h2_over_6_theorem": "NOT_DERIVED",
        "whole_cell_quantified_bounds": "BOUND_NOT_CERTIFIED",
        "physical_error_bound": "BOUND_NOT_CERTIFIED",
        "historical_gate_q": "FAIL",
        "physical_gate_q": "FAIL",
        "AC_P1": "PENDING",
        "contract": "NOT_FROZEN",
        "rust_fem": "NO_GO",
        "Gate_M": "NOT_VERIFIED",
    }
