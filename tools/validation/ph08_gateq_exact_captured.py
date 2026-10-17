#!/usr/bin/env python3
"""PH-08 exact arithmetic for a *captured binary64 system*, not physical certification.

Inputs must be literal IEEE-754 binary64 scalars from the original solve. The
exact reference is the exact least-squares solution of THOSE ROUNDED operands.
This does not enclose upstream PDE, coordinates, geometry, or float operations
that assembled the supplied operands.
"""
from __future__ import annotations

from fractions import Fraction as Q
import hashlib
import json
import math
from pathlib import Path


def binary64(value):
    if isinstance(value, bool) or not (
        isinstance(value, float) or
        (getattr(value, "dtype", None) is not None and str(value.dtype) == "float64")
    ):
        raise ValueError("expected already-captured binary64, not integers/strings")
    f = float(value)
    if not math.isfinite(f):
        raise ValueError("nonfinite binary64 operand")
    return Q.from_float(f)


def mul(a, b):
    return [[sum((x * y for x, y in zip(row, column)), Q(0))
             for column in zip(*b)] for row in a]


def inverse(mat):
    n = len(mat)
    rows = [list(row) + [Q(int(i == j)) for j in range(n)]
            for i, row in enumerate(mat)]
    for k in range(n):
        pivot = next((j for j in range(k, n) if rows[j][k]), None)
        if pivot is None:
            raise ValueError("rank-deficient exact normal matrix")
        rows[k], rows[pivot] = rows[pivot], rows[k]
        divisor = rows[k][k]
        rows[k] = [v / divisor for v in rows[k]]
        for i in range(n):
            if i != k:
                factor = rows[i][k]
                rows[i] = [v - factor * w for v, w in zip(rows[i], rows[k])]
    return [row[n:] for row in rows]


def norm_inf(matrix):
    return max(sum(map(abs, row), Q(0)) for row in matrix)


def qstr(v):
    return f"{v.numerator}/{v.denominator}"


MAX_CAPTURE_ROWS = 128  # Conservative bounded *diagnostic* surface, not a physical limit.

def exact_captured_system(A, y, D, observed_c, observed_g):
    """Exact algebraic errors, residuals, and T for rounded binary64 operands.

    observed_g MUST be the measured binary64 output, not a recomputation by
    this routine. The result is diagnostic, NEVER a physical PASS.
    """
    A, y, D = list(A), list(y), list(D)
    if not 6 <= len(A) <= MAX_CAPTURE_ROWS or len(y) != len(A) or len(D) != 2 or len(observed_c) != 6 or len(observed_g) != 2:
        raise ValueError("invalid recovery system shape")
    if any(len(row) != 6 for row in A + D):
        raise ValueError("wrong design or gradient column count")
    aa = [[binary64(v) for v in row] for row in A]
    yy = [[binary64(v)] for v in y]
    dd = [[binary64(v) for v in row] for row in D]
    cc = [[binary64(v)] for v in observed_c]
    gg = [[binary64(v)] for v in observed_g]
    at = list(map(list, zip(*aa)))
    gram = mul(at, aa)
    inv = inverse(gram)
    pinv = mul(inv, at)
    exact_c = mul(pinv, yy)
    exact_g = mul(dd, exact_c)
    trans = mul(dd, pinv)
    constant_response = [sum(row, Q(0)) for row in trans]
    primal = [[row[0] - obs[0]] for row, obs in zip(mul(aa, cc), yy)]
    system_residual = mul(at, primal)
    coeff_error = max(abs(a[0] - b[0]) for a, b in zip(cc, exact_c))
    gradient_error = max(abs(a[0] - b[0]) for a, b in zip(gg, exact_g))
    residual_norm = max(map(lambda row: abs(row[0]), primal))
    normal_residual = max(map(lambda row: abs(row[0]), system_residual))
    rhs = mul(at, yy)
    backward_denom = (norm_inf(gram) * max(abs(row[0]) for row in cc)
                      + max(abs(row[0]) for row in rhs))
    backward = normal_residual / backward_denom if backward_denom else None
    return {
        "scope": "EXACT_CAPTURED_BINARY64_ALGEBRA_ONLY",
        "mathematical_reference": "exact rational LS of already-rounded A,y",
        "rank": 6,
        "coefficient_forward_error_inf_exact": qstr(coeff_error),
        "gradient_forward_error_inf_exact": qstr(gradient_error),
        "primal_residual_inf_exact": qstr(residual_norm),
        "normal_equations_residual_inf_exact": qstr(normal_residual),
        "normal_equations_backward_error_exact": qstr(backward) if backward is not None else None,
        "transfer_norm_inf_exact": qstr(norm_inf(trans)),
        "constant_shift_null_exact": all(v == 0 for v in constant_response),
        "constant_shift_gradient_delta_inf_exact": qstr(max(map(abs, constant_response))),
        "gradient_definition": "D is captured physical-coordinate potential-gradient evaluator; E = -D*c",
        "gradient_evaluation_position": "CAPTURED_IN_D_NOT_INDEPENDENTLY_ATTESTED",
        "exact_coefficients": [qstr(row[0]) for row in exact_c],
        "exact_gradient": [qstr(row[0]) for row in exact_g],
        "least_squares_backward_error_bound": "NOT_CERTIFIED",
        "normal_equations_residual_interpretation": "BACKWARD_RESIDUAL_OF_NORMAL_SYSTEM_ONLY",
        "A_D_input_perturbation_bound": "NOT_CERTIFIED",
        "operator_cond2_bound": "NOT_CERTIFIED",
        "coordinate_assembly_error_bound": "NOT_CERTIFIED",
        "input_potential_error_bound": "NOT_CERTIFIED",
        "pde_discretization_error_bound": "NOT_CERTIFIED",
        "floating_solver_a_priori_bound": "NOT_CERTIFIED",
        "physical_error_bound": "BOUND_NOT_CERTIFIED",
        "physical_gate_q": "FAIL",
        "AC_P1": "PENDING",
        "contract": "NOT FROZEN",
        "rust_fem": "NO-GO",
        "Gate_M": "NOT VERIFIED",
    }


HISTORICAL_RATIOS = (0.953506146487913, 0.45916314091601546)
HISTORICAL_THRESHOLD = 1.8


def prereg_hash(config):
    return hashlib.sha256(json.dumps(config, sort_keys=True, separators=(",", ":"),
                                     allow_nan=False).encode("utf-8")).hexdigest()


def judge_tr(config, pinned_sha256, case, reductions=None, r_inputs=None):
    """Predeclared regime only. No dynamic switch based on observed errors."""
    base = {"historical_gate_q": "FAIL", "physical_gate_q": "FAIL",
            "rate_check_status": "NOT VERIFIED",
            "arithmetic_check_status": "NOT VERIFIED",
            "capture_provenance_status": "NOT VERIFIED",
            "certification_status": "NOT VERIFIED",
            "contract": "NOT FROZEN", "AC_P1": "PENDING",
            "rust_fem": "NO-GO", "Gate_M": "NOT VERIFIED",
            "float_physical_bound": "BOUND_NOT_CERTIFIED"}
    try:
        if prereg_hash(config) != pinned_sha256:
            return dict(base, status="FAIL", reason="preregistration hash mismatch")
        if config.get("version") != 1 or config.get("t_min_reduction") != HISTORICAL_THRESHOLD:
            return dict(base, status="FAIL", reason="unexpected historical criterion")
        entry = config["cases"][case]
        regime = entry["regime"]
        if regime not in ("T", "R"):
            raise ValueError("unregistered regime")
        if regime == "T":
            if reductions is None:
                return dict(base, regime=regime, status="NOT VERIFIED",
                            reason="original T reductions absent")
            if (len(reductions) != 2 or any(isinstance(x, bool) or not isinstance(x, (int, float))
                        or not math.isfinite(x) or x < 0 for x in reductions)):
                raise ValueError("invalid reductions")
            ok = all(x >= HISTORICAL_THRESHOLD for x in reductions)
            return dict(base, regime=regime, status="PASS" if ok else "FAIL",
                        rate_check_status="PASS" if ok else "FAIL",
                        scope="T_RATE_ONLY_NOT_PHYSICAL_APPROVAL")
        if reductions is not None:
            return dict(base, regime=regime, status="FAIL",
                        reason="post-hoc R selection or T data leakage forbidden")
        required = ("original_operator_verified", "exact_reference_verified",
                    "binary64_assembly_bound_certified", "solver_bound_certified",
                    "input_error_bound_certified", "gradient_transfer_bound_certified",
                    "physical_gradient_bound_certified", "negative_controls_verified")
        if not isinstance(r_inputs, dict) or not all(r_inputs.get(k) is True for k in required):
            return dict(base, regime=regime, status="INDETERMINATE",
                        reason="R prerequisites not fully certified")
        # Claimed booleans are not independently audited certificates.
        return dict(base, regime=regime, status="NOT VERIFIED",
                    reason="R certificates still require independent source-bound review")
    except (KeyError, TypeError, ValueError, OverflowError):
        return dict(base, status="FAIL", reason="invalid or unregistered Gate Q rule")


def historical_gate_q():
    return {"status": "FAIL", "ratios": HISTORICAL_RATIOS,
            "min_reduction": HISTORICAL_THRESHOLD}


# This is a *separate* read-only synthetic fixture admission path.
# The policy hash is fixed in reviewable code, not accepted from the evaluator's
# request. A future original-HMM policy MUST be registered and independently
# timestamped in a Verified commit before first evaluation; this fixture does
# not pre-register already-observed Stage 4 or Stage 5B measurements.
REGISTERED_SYNTHETIC_POLICY_SHA256 = "4674bc15a4e796baf44b2ed16ccb473ddeb9a75c684aa678bdbe1eaf73a6630b"
REGISTERED_SYNTHETIC_POLICY_RELATIVE_PATH = "docs/validation/ph08-gateq-tr-synthetic-prereg-v1.json"


def judge_tr_registered(case, reductions=None, r_inputs=None, policy_path=None):
    """Load pinned on-repository *synthetic-only* rules; never certify physics.

    Changing supplied policy and independently recomputing its digest does
    not affect this trusted fixed pin. This is not a real historical
    timestamp/approval attestation, and deliberately cannot return physical PASS.
    """
    base = {"status": "FAIL", "rate_check_status": "NOT VERIFIED",
            "arithmetic_check_status": "NOT VERIFIED",
            "capture_provenance_status": "NOT VERIFIED",
            "certification_status": "NOT VERIFIED",
            "physical_gate_q": "FAIL", "historical_gate_q": "FAIL",
            "AC_P1": "PENDING", "contract": "NOT FROZEN",
            "rust_fem": "NO-GO", "Gate_M": "NOT VERIFIED",
            "float_physical_bound": "BOUND_NOT_CERTIFIED",
            "registration_status": "PROPOSED_ONLY_NOT_ORIGINAL_PREREGISTERED",
            "registered_policy_sha256": REGISTERED_SYNTHETIC_POLICY_SHA256}
    if policy_path is None:
        policy_path = (Path(__file__).resolve().parents[2]
                       / REGISTERED_SYNTHETIC_POLICY_RELATIVE_PATH)
    try:
        data = Path(policy_path).read_bytes()
        if len(data) > 16384:
            raise ValueError("oversize preregistration")
        config = json.loads(data)
        if not isinstance(config, dict) or prereg_hash(config) != REGISTERED_SYNTHETIC_POLICY_SHA256:
            raise ValueError("independently pinned rule hash mismatch")
        if (config.get("schema") != "boardsi.ph08.gateq.synthetic-prereg.v1"
                or config.get("evaluation_scope") != "SYNTHETIC_ONLY_NO_ORIGINAL_HMM_CERTIFICATION"
                or config.get("policy_status") != "PROPOSED_NOT_FROZEN"
                or set(config) != {"schema","version","evaluation_scope","policy_status","t_min_reduction","cases"}):
            raise ValueError("not the reviewed synthetic policy schema")
        entry = config["cases"][case]
        if not isinstance(entry, dict) or set(entry) != {"regime","dataset_id","selection_basis"}:
            raise ValueError("unregistered policy case")
        if not isinstance(entry["dataset_id"], str) or not entry["dataset_id"].startswith("synthetic-only:"):
            raise ValueError("original evidence cannot be evaluated under synthetic policy")
        # P2 hardening: a case name and a caller-provided reduction pair do
        # not attest that those numbers were derived from its dataset. There
        # is deliberately no trusted, independently SHA-pinned *reduction
        # evidence* input in this proposed policy version. Reject any such
        # submission before rate evaluation, including apparent passes.
        if entry["regime"] == "T":
            return dict(base, regime="T", status="NOT VERIFIED",
                        reason="T observations not bound to authenticated dataset and reduction evidence")
        result = judge_tr(config, REGISTERED_SYNTHETIC_POLICY_SHA256, case,
                          reductions=reductions, r_inputs=r_inputs)
        result.update(registration_status=base["registration_status"],
                      registered_policy_sha256=REGISTERED_SYNTHETIC_POLICY_SHA256)
        return result
    except (OSError, UnicodeError, ValueError, TypeError, KeyError, json.JSONDecodeError):
        return dict(base, reason="unregistered, tampered, or unreadable synthetic policy")
