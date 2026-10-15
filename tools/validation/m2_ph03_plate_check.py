"""Independent G0 mathematical PH-03 plate verifier, NOT a second FEM solver.

Use separately written closed-form exact rectangle equations; never parse Rust
sources or derive reference values from Rust FEM output. Frozen limits are in
docs/validation/m2-ph03-parallel-plate-acceptance-contract.md.
"""
import json
import math
import sys
from decimal import Decimal, getcontext

getcontext().prec = 50
D = Decimal
EPS0 = D("8.8541878188e-12")
MU0 = D("1.25663706127e-6")
W = D("0.0003")
H = D("0.0002")
ER = D("3.4")


def require(ok, why):
    if not ok:
        raise AssertionError(why)


def relative(observed, exact):
    return abs(float(observed) / float(exact) - 1.0)


def references():
    cap = EPS0 * ER * W / H
    cap0 = EPS0 * W / H
    inductance = MU0 * H / W
    return {
        "C_F_per_m": float(cap),
        "C0_F_per_m": float(cap0),
        "Lext_H_per_m": float(inductance),
        "Z0_ohm": float((inductance / cap).sqrt()),
        "delay_s_per_m": float((inductance * cap).sqrt()),
    }


def verify_record(row, expected_n, oracle):
    n = row["n"]
    require(n == expected_n, "wrong requested plate mesh n")
    require(row["model"] == "PH03_Neumann_plate", "wrong geometry/BC identity")
    require(row["status"] == "PrototypeOnly_IdealNeumannPlate_NotRealPCBValidated",
            "physical qualification/status mislabel")
    require(row["width_m"] == float(W) and row["height_m"] == float(H)
            and row["epsilon_r"] == float(ER), "unmatched geometry/dielectric")
    require(row["nx"] == 3 * n // 2 and row["nz"] == n,
            "unexpected rectangle discretization")
    require(row["nodes"] == (row["nx"] + 1) * (n + 1), "node count")
    require(row["dof"] == (n - 1) * (row["nx"] + 1), "Neumann side DOF mismatch")
    require(row["triangles"] == 2 * n * row["nx"], "triangle count")
    require(row["nnz"] > row["dof"] > 0, "invalid sparse matrix")
    errors = {}
    for key, expected in oracle.items():
        actual = row[key]
        require(math.isfinite(actual) and actual > 0.0, "nonfinite "+key)
        err = relative(actual, expected)
        errors[key] = err
        require(err <= 1e-3, "PH-03 0.1% mathematical failure "+key)
    for key in ("C_F_per_m", "C0_F_per_m"):
        require(errors[key] <= 1e-6, "strict affine plate C/C0 diagnostic "+key)
    require(abs(row["C_F_per_m"] / row["C0_F_per_m"] / float(ER) - 1.0) <= 1e-6,
            "plate C/C0 dielectric ratio")
    for stem, cap_key in (("field", "C_F_per_m"), ("vacuum", "C0_F_per_m")):
        require(row[stem+"_residual"] <= 1e-8 and row[stem+"_residual"] >= 0,
                "true residual "+stem)
        require(row[stem+"_charge_energy_rel"] <= 1e-6, "reaction/energy "+stem)
        require(relative(row[stem+"_charge_C"], oracle[cap_key]) <= 1e-3,
                "independent mathematical charge anchor "+stem)
        require(row[stem+"_phi_err_V"] <= 1e-8, "potential "+stem)
        require(row[stem+"_x_span_V"] <= 1e-8, "x potential uniformity "+stem)
        require(row[stem+"_pec_err_V"] <= 1e-14, "electrode corner voltage "+stem)
        require(row[stem+"_ex_rel"] <= 1e-6, "Ex element max "+stem)
        require(row[stem+"_ez_rel"] <= 1e-6, "Ez element max "+stem)
        require(0 <= row[stem+"_wall_s"] < 600, "time measurement "+stem)
        require(0 <= row[stem+"_iterations"] <= 15000, "iteration count "+stem)
    return errors


def verify_all(rows):
    oracle = references()
    # Confirm G0 numeric displays represent rounding of the same constants.
    require(relative(oracle["C_F_per_m"], 4.515635787588e-11) < 1e-12,
            "G0 plate C constant mismatch")
    require(relative(oracle["C0_F_per_m"], 1.32812817282e-11) < 1e-12,
            "G0 plate C0 constant mismatch")
    require(relative(oracle["Lext_H_per_m"], 8.377580408467e-7) < 1e-12,
            "G0 plate Lext constant mismatch")
    require(len(rows) == 3, "expected exactly three mesh witnesses")
    by_n = {}
    for n, row in zip((8, 16, 32), rows):
        by_n[n] = verify_record(row, n, oracle)
    mesh = {}
    for coarse, fine in ((8, 16), (16, 32)):
        mesh[f"{coarse}_to_{fine}"] = {}
        for key in oracle:
            value = relative(rows[(8, 16, 32).index(coarse)][key],
                             rows[(8, 16, 32).index(fine)][key])
            mesh[f"{coarse}_to_{fine}"][key] = value
            require(value <= 1e-6, "PH-03 preregistered mesh change "+key)
    return {"result": "PH03_PASS_MATHEMATICAL_COUPON_ONLY",
            "oracle": oracle, "relative_errors_by_n": by_n,
            "mesh_relative_changes": mesh}


def self_test():
    oracle = references()
    require(oracle["C_F_per_m"] > oracle["C0_F_per_m"], "oracle polarity")
    require(relative(oracle["C_F_per_m"], float(EPS0*ER*W/H)) <= 1e-14,
            "oracle numerical self-check")
    # A falsified mathematical answer must be recognized, not rubber-stamped.
    require(relative(1.002 * oracle["C_F_per_m"], oracle["C_F_per_m"]) > 1e-3,
            "negative check")
    print("PH03 independent closed-form oracle self-test: PASS (no FEM evidence)")


def main():
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        self_test()
        return
    if len(sys.argv) != 4:
        raise SystemExit("usage: m2_ph03_plate_check.py ph03-n8.json ph03-n16.json ph03-n32.json")
    with open(sys.argv[1], encoding="utf-8") as handle:
        n8 = json.load(handle)
    with open(sys.argv[2], encoding="utf-8") as handle:
        n16 = json.load(handle)
    with open(sys.argv[3], encoding="utf-8") as handle:
        n32 = json.load(handle)
    print(json.dumps(verify_all([n8, n16, n32]), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
