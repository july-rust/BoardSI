#!/usr/bin/env python3
"""Exact PUBLIC SYNTHETIC uniform-square HMM quadratic identity (no private source).

For q(x,z)=x^2+z^2, source=-4 and constant eps=1, the ideal local
square energy is h^2|G|^2 + 2 sum_f R_f^2. All arithmetic uses Fraction.
This proves a conditional exact-arithmetic uniform-mesh identity, not an
a-priori floating error bound, not a full original HMM reexecution.
"""
from fractions import Fraction as Q

DIRS = {"R": (1, 0), "L": (-1, 0), "U": (0, 1), "D": (0, -1)}
MAX_N = 32


def potential(x, z):
    return x*x + z*z


def _maxabs(values):
    return max((abs(v) for v in values), default=Q(0))


def verify_square(n, *, cell_bias=None, face_bias=None, source=Q(-4),
                  midpoint_boundary=False):
    """Return exact residuals of the independent square-cell energy equations.

    Equation: K_cell = -2 sum(R_f), K_face_f = h n_f.G
      + 2 R_f - sum_g R_g (n_g.n_f).
    For ideal full quadratic face means, the boundary Simpson rule is exact.
    """
    if type(n) is not int or not 2 <= n <= MAX_N:
        raise ValueError("grid n outside bounded public synthetic range")
    if type(source) is not Q:
        raise ValueError("source must be exact Fraction")
    h = Q(1, n)
    expected_cell_bias = -h*h/6
    expected_face_bias = h*h/12
    cb = expected_cell_bias if cell_bias is None else cell_bias
    fb = expected_face_bias if face_bias is None else face_bias
    if type(cb) is not Q or type(fb) is not Q:
        raise ValueError("biases must be exact Fractions")
    flux = {}
    cell_residuals, gradient_errors, boundary_errors = [], [], []
    cell_shift_errors, face_shift_errors = [], []
    for j in range(n):
        for i in range(n):
            x, z = Q(2*i+1, 2*n), Q(2*j+1, 2*n)
            uc = potential(x, z) + cb
            faces = {}
            for side, (nx, nz) in DIRS.items():
                mx, mz = x + h*nx/2, z + h*nz/2
                faces[side] = potential(mx, mz) + fb
                # The independent boundary oracle is the Simpson line mean.
                tangent = (-nz, nx)
                xa, za = mx - h*tangent[0]/2, mz - h*tangent[1]/2
                xb, zb = mx + h*tangent[0]/2, mz + h*tangent[1]/2
                simpson = (potential(xa, za) + 4*potential(mx, mz)
                           + potential(xb, zb))/6
                is_boundary = ((side == "L" and i == 0)
                               or (side == "R" and i == n-1)
                               or (side == "D" and j == 0)
                               or (side == "U" and j == n-1))
                if is_boundary:
                    target = potential(mx, mz) if midpoint_boundary else simpson
                    boundary_errors.append(faces[side] - target)
                face_shift_errors.append(faces[side] - potential(mx, mz)
                                         - expected_face_bias)
            gx = (faces["R"] - faces["L"])/h
            gz = (faces["U"] - faces["D"])/h
            gradient_errors += [gx - 2*x, gz - 2*z]
            cell_shift_errors.append(uc - potential(x, z) - expected_cell_bias)
            remainders = {side: faces[side] - uc
                          - h*(nx*gx+nz*gz)/2
                          for side, (nx, nz) in DIRS.items()}
            # Derivatives of the independent nonnegative square HMM energy.
            cell_residuals.append(-2*sum(remainders.values(), Q(0))
                                  - source*h*h)
            out = {}
            for side, (nx, nz) in DIRS.items():
                cross = sum((r*(nx*tx+nz*tz)
                            for s, r in remainders.items()
                            for tx, tz in [DIRS[s]]), Q(0))
                out[side] = h*(nx*gx+nz*gz) + 2*remainders[side] - cross
            flux[i, j] = out
    continuity = []
    for j in range(n):
        for i in range(n):
            if i+1 < n:
                continuity.append(flux[i, j]["R"] + flux[i+1, j]["L"])
            if j+1 < n:
                continuity.append(flux[i, j]["U"] + flux[i, j+1]["D"])
    return {
        "scope": "IDEAL_EXACT_UNIFORM_SQUARE_SYNTHETIC_ONLY",
        "n": n, "cell_count": n*n, "interior_face_count": 2*n*(n-1),
        "cell_equation_residual_inf": _maxabs(cell_residuals),
        "interior_face_residual_inf": _maxabs(continuity),
        "boundary_mean_residual_inf": _maxabs(boundary_errors),
        "quadratic_gradient_error_inf": _maxabs(gradient_errors),
        "cell_shift_error_inf": _maxabs(cell_shift_errors),
        "face_shift_error_inf": _maxabs(face_shift_errors),
        "expected_cell_bias": expected_cell_bias,
        "expected_face_bias": expected_face_bias,
        "original_float_replay": "NOT_RUN",
        "original_HMM_full_domain_error_bound": "BOUND_NOT_CERTIFIED",
        "historical_gate_q": "FAIL",
        "contract": "NOT_FROZEN", "rust_fem": "NO_GO",
    }
