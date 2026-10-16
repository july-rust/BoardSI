#!/usr/bin/env python3
"""Independent exact-rational polynomial recovery diagnostics for PH-08 Issue #51.

This is NOT the original HMM, Gate Q acceptance, Gate M certification or Rust FEM.
The private Stage 4/5B archives are never read by this public synthetic tool.
"""
from __future__ import annotations

import argparse
from fractions import Fraction as F
import hashlib
import json
import math
from pathlib import Path
import platform
import sys

SHAPES = ("uniform", "skew", "graded")
SIZES = (8, 16, 32)
X0, Z0 = F(2, 5), F(1, 3)
STAGE4_SHA256 = "6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9"


def fraction_mpf(v, mp):
    return mp.mpf(v.numerator) / mp.mpf(v.denominator)


def stencil(shape):
    if shape not in SHAPES:
        raise ValueError("unsupported grid shape")
    points = []
    for j in (-1, 0, 1):
        for i in (-1, 0, 1):
            a, b = F(i), F(j)
            if shape == "skew":
                a, b = a + b / 5, b + a / 7
            elif shape == "graded":
                a, b = a + a * a / 10, b + b * b / 12
            points.append((a, b))
    return points


def polynomial(kind, x, z):
    if kind == "quadratic":
        return x*x + z*z
    if kind == "cubic":
        return x*x*x + z*z*z
    raise ValueError("unsupported polynomial")


def exact_gradient(kind):
    if kind == "quadratic":
        return (2*X0, 2*Z0)
    if kind == "cubic":
        return (3*X0*X0, 3*Z0*Z0)
    raise ValueError("unsupported polynomial")


def basis(a, b):
    # Coordinates normalized by h, not raw global powers.
    return (F(1), a, b, a*a, a*b, b*b)


def samples(shape, kind, n, offset=F(0)):
    if not isinstance(n, int) or n < 2:
        raise ValueError("invalid grid size")
    h = F(1, n)
    pts = stencil(shape)
    A = [basis(a, b) for a, b in pts]
    y = [polynomial(kind, X0+h*a, Z0+h*b) + offset for a, b in pts]
    return A, y, h, pts


def exact_solve(square, vector):
    """Independent rational Gaussian elimination, no numpy or float arithmetic."""
    count = len(vector)
    if count != 6 or any(len(row) != count for row in square):
        raise ValueError("expected nonsingular 6x6 system")
    aug = [[F(v) for v in row] + [F(rhs)] for row, rhs in zip(square, vector)]
    for col in range(count):
        pivot = next((r for r in range(col, count) if aug[r][col]), None)
        if pivot is None:
            raise ValueError("singular polynomial basis")
        aug[col], aug[pivot] = aug[pivot], aug[col]
        val = aug[col][col]
        aug[col] = [v / val for v in aug[col]]
        for row in range(count):
            if row == col:
                continue
            fac = aug[row][col]
            if fac:
                aug[row] = [v - fac*w for v, w in zip(aug[row], aug[col])]
    return [row[-1] for row in aug]


def exact_fit(A, y):
    if len(A) != 9 or len(y) != 9 or any(len(row) != 6 for row in A):
        raise ValueError("expected 9 samples / 6 coefficients")
    gram = [[sum(A[k][i] * A[k][j] for k in range(9)) for j in range(6)]
            for i in range(6)]
    rhs = [sum(A[k][i] * y[k] for k in range(9)) for i in range(6)]
    coeff = exact_solve(gram, rhs)
    residual = [sum(A[k][j]*coeff[j] for j in range(6))-y[k] for k in range(9)]
    return coeff, residual


def recovered_gradient(coeff, h):
    if len(coeff) != 6 or h == 0:
        raise ValueError("invalid coefficients or scale")
    return (coeff[1]/h, coeff[2]/h)


def squared_error(got, want):
    if len(got) != 2 or len(want) != 2:
        raise ValueError("gradient must be two-dimensional")
    return sum((a-b)**2 for a, b in zip(got, want)) / 2


def exact_case(shape, kind, n, offset=F(0)):
    A, y, h, pts = samples(shape, kind, n, offset)
    coeff, residual = exact_fit(A, y)
    grad = recovered_gradient(coeff, h)
    theory = exact_gradient(kind)
    return {"A": A, "y": y, "h": h, "pts": pts, "coeff": coeff,
            "residual": residual, "gradient": grad, "theory": theory,
            "squared_error": squared_error(grad, theory)}


def validate_float_vector(values):
    if len(values) != 9 or any(not isinstance(v, (int, float))
                               or not math.isfinite(v) for v in values):
        raise ValueError("invalid/nonfinite float sample values")


def double_case(shape, kind, n):
    import numpy as np
    A, y, h, pts = samples(shape, kind, n)
    arr = np.asarray([[float(v) for v in row] for row in A], dtype=np.float64)
    # Evaluate polynomial using actual float64 arithmetic, unlike exact Fraction.
    hf = float(h)
    yy = np.asarray([polynomial(kind, float(X0)+hf*float(a),
                                float(Z0)+hf*float(b)) for a, b in pts],
                    dtype=np.float64)
    validate_float_vector(yy.tolist())
    coeff, residuals, rank, singular = np.linalg.lstsq(arr, yy, rcond=None)
    if rank != 6 or not np.all(np.isfinite(coeff)):
        raise ValueError("nonfinite or rank-deficient float64 recovery")
    grad = (float(coeff[1]/hf), float(coeff[2]/hf))
    theory = exact_gradient(kind)
    error = math.sqrt(sum((a-float(b))**2 for a, b in zip(grad, theory))/2)
    max_res = float(np.max(np.abs(arr @ coeff - yy)))
    return {"gradient": grad, "error_rms": error,
            "condition_2": float(np.linalg.cond(arr)),
            "residual_max": max_res, "rank": int(rank),
            "input_max": float(np.max(np.abs(yy))),
            "matrix_max": float(np.max(np.abs(arr)))}


def high_precision_case(shape, kind, n, dps=100):
    import mpmath as mp
    if dps < 80:
        raise ValueError("high-precision reference requires >=80 decimal digits")
    A, y, h, _ = samples(shape, kind, n)
    with mp.workdps(dps):
        M = mp.matrix([[fraction_mpf(v, mp) for v in row] for row in A])
        yy = mp.matrix([fraction_mpf(v, mp) for v in y])
        coeff = mp.lu_solve(M.T*M, M.T*yy)
        g = [coeff[i]/fraction_mpf(h, mp) for i in (1, 2)]
        exact = exact_case(shape, kind, n)["gradient"]
        diffs = [abs(g[i]-fraction_mpf(exact[i], mp)) for i in range(2)]
        maxdiff = max(diffs)
        return {"gradient": [mp.nstr(v, 75) for v in g],
                "max_abs_difference_from_fraction_fit": mp.nstr(maxdiff, 8),
                "within_1e_minus_65": bool(maxdiff < mp.mpf("1e-65")),
                "dps": dps}


def sha256_attestation(path, expected):
    """Input provenance negative control; not a claim that a private ZIP exists."""
    if not isinstance(expected, str) or len(expected) != 64 or any(c not in
            "0123456789abcdef" for c in expected):
        raise ValueError("invalid lower-case SHA-256")
    data = Path(path).read_bytes()  # missing file must fail closed
    digest = hashlib.sha256(data).hexdigest()
    if digest != expected:
        raise ValueError("attestation digest mismatch")
    return digest


def report():
    import numpy as np
    import mpmath as mp
    rows, reductions = [], []
    for shape in SHAPES:
        for kind in ("quadratic", "cubic"):
            exact_errors = []
            for n in SIZES:
                ex = exact_case(shape, kind, n)
                dbl = double_case(shape, kind, n)
                high = high_precision_case(shape, kind, n)
                if not high["within_1e_minus_65"]:
                    raise RuntimeError("80+ digit reference did not match exact fractions")
                # Exact residual zero for quadratic. Cubic is not in the fit space.
                if kind == "quadratic" and (ex["squared_error"] or any(ex["residual"])):
                    raise RuntimeError("quadratic exact polynomial reproduction failed")
                e = math.sqrt(float(ex["squared_error"]))
                exact_errors.append(ex["squared_error"])
                rows.append({
                    "grid": shape, "polynomial": kind, "n": n,
                    "exact_gradient_rational": [str(v) for v in ex["gradient"]],
                    "analytical_gradient_rational": [str(v) for v in ex["theory"]],
                    "exact_gradient_error_rms": e,
                    "float64_gradient_error_rms": dbl["error_rms"],
                    "float64_gradient": dbl["gradient"],
                    "mpmath_gradient": high["gradient"],
                    "mpmath_vs_exact_max": high["max_abs_difference_from_fraction_fit"],
                    "condition_2_float64": dbl["condition_2"],
                    "float64_least_squares_residual_max": dbl["residual_max"],
                    "exact_least_squares_residual_max": float(max(map(abs, ex["residual"]))),
                    "float64_rank": dbl["rank"],
                })
            if kind == "cubic":
                if any(x <= 0 for x in exact_errors):
                    raise RuntimeError("cubic not in measurable truncation regime")
                reductions.append({
                    "grid": shape, "polynomial": "cubic",
                    "exact_rational_squared_error_ratios": [
                        str(exact_errors[k]/exact_errors[k+1]) for k in (0, 1)],
                    "exact_gradient_error_reduction": [
                        math.sqrt(float(exact_errors[k]/exact_errors[k+1])) for k in (0, 1)]
                })
    return {
        "scope": "independent nine-site polynomial synthetic oracle; NOT original HMM",
        "qualification": "PROPOSED / NOT FROZEN",
        "historical_stage4_gate_q": "FAIL",
        "stage5b": "NOT RUN",
        "AC_P1": "PENDING",
        "rust_fem": "NO-GO",
        "stage4_known_archive_sha256_not_loaded": STAGE4_SHA256,
        "code_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "versions": {"python": platform.python_version(), "numpy": np.__version__,
                     "mpmath": mp.__version__, "float64_unit_roundoff": sys.float_info.epsilon/2},
        "rows": rows, "cubic_reductions": reductions,
        "caveat": "No certified floating solver error envelope: high-precision comparison is diagnostic, not a Gate Q pass."
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-json", type=Path)
    args = parser.parse_args(argv)
    output = report()
    serialized = json.dumps(output, indent=2, allow_nan=False) + "\n"
    if args.output_json:
        args.output_json.write_text(serialized, encoding="utf-8")
    print(serialized)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
