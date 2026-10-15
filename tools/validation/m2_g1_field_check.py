"""Cross-check the experimental Rust FEM against a fresh, source-locked FVM solve.

This intentionally calls the independent G0 cell-centered finite-volume method,
not the Rust P1 triangular finite-element implementation or its cached output.
Only an explicitly PEC, lossless, nonmagnetic grounded BOX is in scope.
"""
import hashlib
import json
import math
from pathlib import Path
import sys

from m2_fvm_reference import ER, H, T, W, solve

FROZEN_FVM_SHA256 = "c5e05fed106cc78e22150b0c90a828951a439940bb1a8dc8843bd5eb73786929"
FROZEN_N32_C = 8.842_386_680_772_379e-11
FROZEN_N32_C0 = 3.654_801_591_437_518e-11
REQUIRED_REL_C = 0.02  # preregistered G1 numerical tolerance
SOURCE_TOL = 2e-10  # reproducibility of G0 FVM n32 output; not a physical tolerance


def check(condition, message):
    if not condition:
        raise AssertionError(message)


def relative(a, reference):
    return abs(a / reference - 1.0)


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: python m2_g1_field_check.py m2-fem-n32.json")
    script = Path(__file__).with_name("m2_fvm_reference.py")
    manifest = Path(__file__).with_name("m2_g0_source.sha256")
    expected = manifest.read_text(encoding="utf-8").split()[0]
    actual_hash = hashlib.sha256(script.read_bytes()).hexdigest()
    check(expected == FROZEN_FVM_SHA256 == actual_hash, "independent G0 FVM oracle source not locked")
    check((W, T, H, ER) == (0.0003, 0.00005, 0.0002, 3.4),
          "synthetic coupon geometry or material changed")

    specimen = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    check(specimen.get("status") == "PrototypeOnly_IdealPECBox_NotRealPCBValidated",
          "cannot promote physical model to Validated")
    check(specimen.get("n") == 32 and
          math.isclose(specimen.get("box_m", float("nan")), 0.0012, rel_tol=0, abs_tol=1e-13),
          "Rust and FVM domain/mesh mismatch")
    field = solve(n=32, span=0.0012, top=0.0012, er=ER)
    vacuum = solve(n=32, span=0.0012, top=0.0012, er=1.0)
    c, c0 = field["C_F_per_m"], vacuum["C_F_per_m"]
    check(relative(c, FROZEN_N32_C) <= SOURCE_TOL and
          relative(c0, FROZEN_N32_C0) <= SOURCE_TOL,
          "independent FVM n32 reference drift (never tune to Rust output)")
    diff_c = relative(specimen["C_F_per_m"], c)
    diff_c0 = relative(specimen["C0_F_per_m"], c0)
    check(diff_c <= REQUIRED_REL_C and diff_c0 <= REQUIRED_REL_C,
          "Rust FEM against fresh FVM exceeds preregistered 2% numerical gate")
    check(field["relative_residual"] <= 1e-9 and vacuum["relative_residual"] <= 1e-9,
          "independent field linear system residual too high")
    check(specimen.get("field_residual", 1) <= 1e-8 and
          specimen.get("vacuum_residual", 1) <= 1e-8,
          "Rust FEM reported true linear residual too high")

    print(json.dumps({
        "case": "G1 ideal PEC lossless grounded box ONLY",
        "status": "NUMERICAL_COMPARISON_PASS_NOT_REAL_PCB_VALIDATED",
        "oracle_sha256": actual_hash,
        "fvm_C_F_per_m": c,
        "fvm_C0_F_per_m": c0,
        "rust_C_F_per_m": specimen["C_F_per_m"],
        "rust_C0_F_per_m": specimen["C0_F_per_m"],
        "C_relative_error_percent": 100 * diff_c,
        "C0_relative_error_percent": 100 * diff_c0,
        "fvm_dof": field["dof"],
        "rust_dof": specimen["dof"],
        "reference_field_wall_s": field["linear_solve_wall_s"],
        "reference_vacuum_wall_s": vacuum["linear_solve_wall_s"],
        "note": "FVM vs FEM numerical agreement only; no real PCB, frequency, or loss validation"
    }, indent=2))


if __name__ == "__main__":
    main()
