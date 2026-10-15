"""Native CLI numerical gate; independent FVM n128 references from PR #40.

Rounded decimal references from m2-g2-ph07-independent-reference.md, not Rust.
Full independent FVM recomputation remains in the Linux ph07-independent-field job.
"""
import argparse
import json
import math
from pathlib import Path
from m2_ph07_inputs import validate_input

p=argparse.ArgumentParser()
p.add_argument('--root',type=Path,required=True)
a=p.parse_args()
refs=dict(A=86.3034973531e-12,B=93.6663880727e-12,
          C=93.8292836370e-12,D=101.1889103837e-12)
for c,cap in refs.items():
    d=json.loads((a.root/f'native-ph07-{c}.json').read_text())
    validate_input(f'ph07-{c}-n128',d)
    errors={key:abs(d[key]/ref-1) for key,ref in
            [('C_F_per_m',cap),('C0_F_per_m',33.6251585682e-12)]}
    assert all(v<=0.03 for v in errors.values()),errors
    for key in ('field_residual','vacuum_residual'):
        assert math.isfinite(d[key]) and 0<=d[key]<=1e-8
    print(c,'Native FEM vs independently recorded FVM PASS',errors)
for name in ('ph05','ph06'):
    d=json.loads((a.root/f'native-{name}.json').read_text())
    assert d['C_F_per_m']>d['C0_F_per_m']>0
    for key in ('field_residual','vacuum_residual'):
        assert math.isfinite(d[key]) and 0<=d[key]<=1e-8
    assert abs(d['epsilon_effective']/3.4-1)<=0.001
    print(name,'Native homogeneous coupon PASS',d['C_F_per_m'],d['C0_F_per_m'])
