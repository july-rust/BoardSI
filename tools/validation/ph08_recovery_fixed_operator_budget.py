#!/usr/bin/env python3
"""PH-08 exact fixed-operator error identity on PUBLIC SYNTHETIC inputs only.

The identity isolates observed-output arithmetic, sampled potential, and
polynomial reconstruction defects at *fixed, already-rounded* binary64 A,D.
It neither reconstructs the hidden original mesh nor certifies PDE/assembly,
physical coordinates, floating point solver, or whole-space error.
"""
from __future__ import annotations

from fractions import Fraction
from itertools import islice
from ph08_gateq_exact_captured import (
    MAX_CAPTURE_ROWS, binary64, inverse, mul, norm_inf, qstr,
)

Q = Fraction


def _reference(value: object) -> Q:
    # Reference values need an independently justified exact source.
    # Never silently convert caller floats to an "exact physical" reference.
    if type(value) is not Q:
        raise ValueError("reference samples/gradient must be exact Fraction")
    if max(value.numerator.bit_length(), value.denominator.bit_length()) > 4096:
        raise ValueError("oversize exact reference")
    return value


def _bounded(values, limit: int):
    """Read no more than limit+1 items even from a one-shot/infinite iterable."""
    try:
        result = list(islice(iter(values), limit + 1))
    except (TypeError, OverflowError) as ex:
        raise ValueError("invalid iterable") from ex
    if len(result) > limit:
        raise ValueError("input exceeds bounded diagnostic size")
    return result


def _vector(values, size: int, convert):
    vals = _bounded(values, size)
    if len(vals) != size:
        raise ValueError("wrong vector length")
    return [[convert(v)] for v in vals]


def _norm_vector(values) -> Q:
    return max((abs(row[0]) for row in values), default=Q(0))


def _difference(left, right):
    return [[a[0] - b[0]] for a, b in zip(left, right)]


def _sum_vectors(*vectors):
    return [[sum((v[i][0] for v in vectors), Q(0))] for i in range(2)]


def analyze_fixed_recovery(A, D, y_numeric, y_reference,
                           physical_gradient_reference, observed_gradient,
                           quadratic_coefficients):
    """Compute exact decomposition; all supplied provenance remains unverified.

    gobs - gphysical = (gobs - T*y_numeric)
                    + T*(y_numeric-y_reference)
                    + (T*y_reference - gphysical).

    T is exact for captured binary64 A,D, but its inputs are NOT guaranteed
    faithful to the original PDE/operator. The triangle inequality is likewise
    fixed-input-only. Accept only rational manufactured reference data.
    """
    raw_rows = _bounded(A, MAX_CAPTURE_ROWS)
    if len(raw_rows) < 6:
        raise ValueError("invalid six-term design matrix")
    raw_grad = _bounded(D, 2)
    if len(raw_grad) != 2:
        raise ValueError("invalid physical gradient evaluation matrix")
    design = []
    for row in raw_rows:
        fields = _bounded(row, 6)
        if len(fields) != 6:
            raise ValueError("invalid six-term design matrix")
        design.append([binary64(v) for v in fields])
    grad_rows = []
    for row in raw_grad:
        fields = _bounded(row, 6)
        if len(fields) != 6:
            raise ValueError("invalid physical gradient evaluation matrix")
        grad_rows.append([binary64(v) for v in fields])
    n = len(design)

    yn = _vector(y_numeric, n, binary64)
    yr = _vector(y_reference, n, _reference)
    gp = _vector(physical_gradient_reference, 2, _reference)
    go = _vector(observed_gradient, 2, binary64)
    cq = _vector(quadratic_coefficients, 6, _reference)

    transpose = [list(col) for col in zip(*design)]
    t = mul(mul(grad_rows, inverse(mul(transpose, design))), transpose)
    gn = mul(t, yn)
    gr = mul(t, yr)
    arithmetic = _difference(go, gn)
    sample = mul(t, _difference(yn, yr))
    polynomial = _difference(gr, gp)
    observed = _difference(go, gp)
    if _sum_vectors(arithmetic, sample, polynomial) != observed:
        raise AssertionError("exact decomposition identity violated")

    sample_delta = _norm_vector(_difference(yn, yr))
    transfer_norm = norm_inf(t)
    sample_bound = transfer_norm * sample_delta
    sample_observed = _norm_vector(sample)
    if sample_observed > sample_bound:
        raise AssertionError("exact fixed-operator bound violated")
    const_null = all(sum(row, Q(0)) == 0 for row in t)
    constant_basis = all(row[0] == Q(1) for row in design)
    exact_quadratic = (mul(design, cq) == yr and mul(grad_rows, cq) == gp)

    components = (arithmetic, sample, polynomial)
    total_bound = sum((_norm_vector(v) for v in components), Q(0))
    if _norm_vector(observed) > total_bound:
        raise AssertionError("exact triangle bound violated")

    return {
        "scope": "CAPTURED_BINARY64_FIXED_OPERATOR_SYNTHETIC_DIAGNOSTIC",
        "identity_verified_exactly": True,
        "rank": 6,
        "sample_count": n,
        "constant_basis_column_exact": constant_basis,
        "constant_shift_null_exact": const_null,
        "quadratic_reproduced_for_supplied_exact_reference": exact_quadratic,
        "transfer_norm_inf_exact": qstr(transfer_norm),
        "sample_input_difference_inf_exact": qstr(sample_delta),
        "sample_error_inf_exact": qstr(sample_observed),
        "fixed_operator_sample_upper_bound_exact": qstr(sample_bound),
        "arithmetic_output_error_inf_exact": qstr(_norm_vector(arithmetic)),
        "polynomial_reconstruction_defect_inf_exact": qstr(_norm_vector(polynomial)),
        "observed_total_error_inf_exact": qstr(_norm_vector(observed)),
        "triangle_upper_bound_exact": qstr(total_bound),
        "arithmetic_error_components_exact": [qstr(v[0]) for v in arithmetic],
        "sample_error_components_exact": [qstr(v[0]) for v in sample],
        "polynomial_defect_components_exact": [qstr(v[0]) for v in polynomial],
        "observed_error_components_exact": [qstr(v[0]) for v in observed],
        "reference_provenance": "CALLER_SUPPLIED_NOT_INDEPENDENTLY_AUTHENTICATED",
        "upstream_coordinate_assembly_error": "NOT_CERTIFIED",
        "physical_sample_input_error": "NOT_CERTIFIED",
        "original_solver_a_priori_error": "NOT_CERTIFIED",
        "pde_discretization_error": "NOT_CERTIFIED",
        "physical_error_bound": "BOUND_NOT_CERTIFIED",
        "historical_gate_q": "FAIL",
        "physical_gate_q": "FAIL",
        "AC_P1": "PENDING",
        "contract": "NOT_FROZEN",
        "rust_fem": "NO_GO",
        "Gate_M": "NOT_VERIFIED",
    }
