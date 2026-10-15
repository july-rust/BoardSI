"""Independent PH-07 contract inputs, keyed by evidence identity, never FEM values.

SI units. Frozen contract c11440a1; additions in #41 are checks, not new physics.
"""
from copy import deepcopy
import math

INPUT_KEYS = ("case", "n", "B_m", "H_m", "width_m", "copper_thickness_m",
              "air_epsilon_r", "layers", "mask")


def expected_cases():
    cases = {}
    for kind in "ABCD":
        base = dict(case=kind, n=64, B_m=0.0012, H_m=0.0008,
                    width_m=0.00030, copper_thickness_m=0.00005,
                    air_epsilon_r=1.0,
                    layers=[dict(height_m=0.00020, epsilon_r=3.4)], mask=None)
        if kind in "CD":
            base["layers"] = [dict(height_m=0.00010, epsilon_r=3.4),
                              dict(height_m=0.00010, epsilon_r=4.0)]
        if kind in "BD":
            base["mask"] = dict(thickness_m=0.000025, epsilon_r=3.3)
        for tag, changes in [*( (f"n{n}", dict(n=n)) for n in (32,64,128)),
                             ("B12", {}), ("B16", dict(B_m=0.0016)),
                             ("B24", dict(B_m=0.0024)), ("H08", {}),
                             ("H12", dict(n=96,H_m=0.0012)),
                             ("H16", dict(n=128,H_m=0.0016))]:
            inp = deepcopy(base)
            inp.update(changes)
            cases[f"ph07-{kind}-{tag}"] = inp
    for tag, kind, mask_t, mask_er, lo, hi in (
        ("equal","C",0,1,3.4,3.4), ("mask-t12","D",0.0000125,3.3,3.4,4.0),
        ("mask-t50","D",0.00005,3.3,3.4,4.0),
        ("mask-er1","D",0.000025,1.0,3.4,4.0),
        ("mask-er4","D",0.000025,4.0,3.4,4.0),
        ("layer-high","C",0,1,4.0,4.0), ("layer-swapped","C",0,1,4.0,3.4),
        ("vacuum","C",0,1,1.0,1.0)):
        inp = deepcopy(cases[f"ph07-{kind}-n64"])
        inp["layers"][0]["epsilon_r"] = lo
        inp["layers"][1]["epsilon_r"] = hi
        if inp["mask"] is not None:
            inp["mask"] = dict(thickness_m=mask_t,epsilon_r=mask_er)
        cases[f"ph07-{tag}"] = inp
    return cases


def require_equal(actual, expected, path="input"):
    if isinstance(expected, dict):
        if not isinstance(actual, dict) or actual.keys() != expected.keys():
            raise ValueError(f"{path}: missing/extra keys")
        for key, value in expected.items():
            require_equal(actual[key], value, f"{path}.{key}")
    elif isinstance(expected, list):
        if not isinstance(actual,list) or len(actual) != len(expected):
            raise ValueError(f"{path}: wrong layer count")
        for i, value in enumerate(expected):
            require_equal(actual[i],value,f"{path}[{i}]")
    elif isinstance(expected, (int,float)):
        if (type(actual) not in (int,float) or not math.isfinite(actual) or
            not math.isclose(actual,expected,rel_tol=1e-13,abs_tol=0.0)):
            raise ValueError(f"{path}: {actual!r} != frozen {expected!r}")
    elif actual != expected:
        raise ValueError(f"{path}: {actual!r} != frozen {expected!r}")


def validate_input(name, data):
    expected = expected_cases()[name]  # unknown identities are rejected
    require_equal({k:data[k] for k in INPUT_KEYS},expected)
    d = expected["H_m"] / expected["n"]
    nx = round(2*expected["B_m"]/d)
    # Explicit boundary assertions. Positions follow the frozen coordinates:
    # x origin=-B, GND z=0, copper centred x=0 at sum(layer heights).
    require_equal({k:data[k] for k in ("model","bottom_ground_v","signal_v",
        "top_boundary","side_boundary","dx_m","nx","nz","nodes","triangles")},
        dict(model="PH07_IdealMultilayerMicrostrip",bottom_ground_v=0.0,
             signal_v=1.0,top_boundary="Neumann",side_boundary="Neumann",
             dx_m=d,nx=nx,nz=expected["n"],nodes=(nx+1)*(expected["n"]+1),
             triangles=2*(nx*expected["n"]-round(expected["width_m"]/d)*
                          round(expected["copper_thickness_m"]/d))))
    return deepcopy(expected)  # solve from contract, not the emitted JSON
