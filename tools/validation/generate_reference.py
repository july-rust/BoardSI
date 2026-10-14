#!/usr/bin/env python3
"""Independent 80-decimal-digit ABCD oracle; never imports production code.
Development-only dependency: mpmath 1.2.1 (BSD), Python 3.12.
Run from repository root: python3 tools/validation/generate_reference.py
Only synthetic circuit parameters, no PCB or Touchstone input.
"""
from pathlib import Path
import mpmath as mp

mp.mp.dps = 80
# Decimal inputs match exactly the values serialized to CSV.
cases = []
for zc in [25, 50, 75, 100]:
    for f in ['1', '1e6', '1e9', '30e9']:
        cases.append(('0', str(zc*zc*1e-10), '0', '1e-10', '0.1', f, '50'))
for r, g in [('1','0'), ('0','0.001'), ('8','0.002'), ('100','0.04')]:
    for f in ['1', '1e6', '1e9', '30e9']:
        for length in ['1e-9', '0.1', '1']:
            cases.append((r,'2.5e-7',g,'1e-10',length,f,'50'))
# Distortionless matched exact attenuation, including exp(-600) and exp(-700).
for length in ['100', '600', '700']:
    cases.append(('50','2.5e-7','0.02','1e-10',length,'1','50'))
# Very low loss, near half-wave, and near numerical-envelope impedance limits.
cases.extend([
    ('1e-14','2.5e-7','1e-18','1e-10','0.1','1e9','50'),
    ('0','5.625e-7','0','1e-10','0.1','666666666.6666666','50'),
    ('0','1e-18','0','1e-10','0.1','1e9','50'),
    ('0','0.25','0','1e-16','1e-6','1e9','50'),
    ('8','2.5e-7','0.002','1e-10','0.1','1e9','75'),
])
rows = ['# R,L,G,C,length_m,frequency_hz,Zref,S11re,S11im,S21re,S21im,Zcre,Zcim,gammare,gammaim']
for case in cases:
    r, l, g, c, length, f, ref = map(mp.mpf, case)
    z = r+mp.j*2*mp.pi*f*l
    y = g+mp.j*2*mp.pi*f*c
    gamma = mp.sqrt(z*y)
    zc = mp.sqrt(z/y)
    a = mp.cosh(gamma*length)
    b = zc*mp.sinh(gamma*length)
    cc = mp.sinh(gamma*length)/zc
    den = 2*a+b/ref+cc*ref
    s11 = (b/ref-cc*ref)/den
    s21 = 2/den
    values = [mp.re(s11),mp.im(s11),mp.re(s21),mp.im(s21),mp.re(zc),mp.im(zc),mp.re(gamma),mp.im(gamma)]
    rows.append(','.join(case)+','+','.join(mp.nstr(x,45) for x in values))
output = Path('crates/boardsi-numerics/tests/fixtures/abcd_reference.csv')
output.write_text('\n'.join(rows)+'\n', encoding='utf-8')
print(f'{len(cases)} synthetic ABCD reference cases written at {mp.mp.dps} decimal digits')

# Supplemental boundary oracle uses the exact binary64 inputs read by Rust.
# This separates f64 input rounding from errors in the numerical algorithm.
boundary_cases = []
def binary_case(values):
    return tuple(repr(float(x)) for x in values)

# Mismatch and phase accumulation, including either side of a half-wave.
for q in ['1e-6', '1e-3', '1', '1e3', '1e6']:
    for phase in [mp.mpf('0.001'), mp.pi/2, mp.pi*(1-mp.mpf('1e-8')),
                  mp.pi, mp.pi*(1+mp.mpf('1e-8')), mp.mpf('1000.3'),
                  mp.mpf('100000.3'), mp.mpf('999999')]:
        length = phase/(2*mp.pi*mp.mpf('1e9')*mp.sqrt(mp.mpf('2.5e-7')*mp.mpf('1e-10')))
        boundary_cases.append(binary_case(['0','2.5e-7','0','1e-10',length,'1e9',50/mp.mpf(q)]))

# Unequal R/L and G/C, large finite attenuation, mismatched references.
for f in ['1', '1e6', '1e9']:
    z = mp.mpf('8')+mp.j*2*mp.pi*mp.mpf(f)*mp.mpf('2.5e-7')
    y = mp.mpf('0.002')+mp.j*2*mp.pi*mp.mpf(f)*mp.mpf('1e-10')
    for attenuation in ['0.001', '1', '100', '600', '699']:
        length = mp.mpf(attenuation)/mp.re(mp.sqrt(z*y))
        for ref in ['25','50','100']:
            boundary_cases.append(binary_case(['8','2.5e-7','0.002','1e-10',length,f,ref]))

# One loss term zero: differing square-root quadrants toward the DC limit.
for r, g in [('1','0'), ('0','0.001'), ('8','0.002')]:
    for f in ['1e-3','1','1e9']:
        for ref in ['25','75']:
            boundary_cases.append(binary_case([r,'2.5e-7',g,'1e-10','0.1',f,ref]))

rows = ['# exact binary64 input ABCD oracle; columns identical to abcd_reference.csv']
for case in boundary_cases:
    r, l, g, c, length, f, ref = [mp.mpf(float(x)) for x in case]
    z = r+mp.j*2*mp.pi*f*l
    y = g+mp.j*2*mp.pi*f*c
    gamma = mp.sqrt(z*y)
    zc = mp.sqrt(z/y)
    a = mp.cosh(gamma*length)
    b = zc*mp.sinh(gamma*length)
    cc = mp.sinh(gamma*length)/zc
    den = 2*a+b/ref+cc*ref
    s11 = (b/ref-cc*ref)/den
    s21 = 2/den
    values = [mp.re(s11),mp.im(s11),mp.re(s21),mp.im(s21),mp.re(zc),mp.im(zc),mp.re(gamma),mp.im(gamma)]
    rows.append(','.join(case)+','+','.join(mp.nstr(x,45) for x in values))
output = Path('crates/boardsi-numerics/tests/fixtures/abcd_boundary_reference.csv')
output.write_text('\n'.join(rows)+'\n', encoding='utf-8')
print(f'{len(boundary_cases)} exact-input boundary ABCD cases written at {mp.mp.dps} decimal digits')

# Review regressions: exact binary64 inputs, including reciprocal scales.
overflow_cases = [
    ('0','1e200','0','1e-200','1e-220','1e210','1e200'),
    ('1e-100','1e-200','1e-100','1e-200','1e-110','1e308','1'),
    ('0','1e-200','0','1e200','1e-220','1e210','1e-200'),
    ('0','1e-200','0','1e-200','1e-110','1e308','1'),
    ('0','5e-324','0','5e-324','1e307','1','1'),
]
rows = ['# exact binary64 input ABCD oracle; columns identical to abcd_reference.csv']
for case in overflow_cases:
    r, l, g, c, length, f, ref = [mp.mpf(float(x)) for x in case]
    z = r+mp.j*2*mp.pi*f*l
    y = g+mp.j*2*mp.pi*f*c
    gamma = mp.sqrt(z*y)
    zc = mp.sqrt(z/y)
    a = mp.cosh(gamma*length)
    b = zc*mp.sinh(gamma*length)
    cc = mp.sinh(gamma*length)/zc
    den = 2*a+b/ref+cc*ref
    s11 = (b/ref-cc*ref)/den
    s21 = 2/den
    values = [mp.re(s11),mp.im(s11),mp.re(s21),mp.im(s21),mp.re(zc),mp.im(zc),mp.re(gamma),mp.im(gamma)]
    rows.append(','.join(case)+','+','.join(mp.nstr(x,45) for x in values))
output = Path('crates/boardsi-numerics/tests/fixtures/abcd_overflow_reference.csv')
output.write_text('\n'.join(rows)+'\n', encoding='utf-8')
print(f'{len(overflow_cases)} exact-input overflow ABCD cases written at {mp.mp.dps} decimal digits')
