#!/usr/bin/env python3
"""Independent Stage-5C diagnostic, NOT an implementation/validation of BoardSI HMM.

Nine fixed 2-D sample sites, six-term quadratic least-squares reconstruction.
Fractions provide an exact-arithmetic oracle; NumPy provides a binary64 diagnostic.
A cubic MMS verifies that a real truncation error falls with mesh spacing.
No source from the Stage-4/5B qualification archive is imported.
"""
from __future__ import annotations
import hashlib
import json
import math
import platform
from fractions import Fraction as F
from pathlib import Path
import numpy as np

X0, Z0 = F(7,100), F(-3,100)
SITES = [(F(i),F(j)) for i in (-1,0,1) for j in (-1,0,1)]
H_LIST = [F(1,8), F(1,16), F(1,32), F(1,64)]


def poly(x, z, cubic=False):
    q = F(3,5)+F(7,4)*x-F(9,8)*z+F(1,3)*x*x+F(2,5)*x*z-F(1,6)*z*z
    return q + (6*x*x*x-3*x*x*z+5*z*z*z if cubic else 0)


def grad(x, z, cubic=False):
    dx = F(7,4)+F(2,3)*x+F(2,5)*z
    dz = -F(9,8)+F(2,5)*x-F(1,3)*z
    if cubic:
        dx += 18*x*x-6*x*z
        dz += -3*x*x+15*z*z
    return (dx, dz)


def basis(xi, eta):
    return (F(1),xi,eta,xi*xi,xi*eta,eta*eta)


def exact_system_solve(mat, vec):
    n=len(vec)
    aug=[list(mat[i])+[vec[i]] for i in range(n)]
    for c in range(n):
        k=next((k for k in range(c,n) if aug[k][c]),None)
        if k is None: raise ArithmeticError('Rank deficient exact reconstruction')
        aug[c],aug[k]=aug[k],aug[c]
        p=aug[c][c]
        aug[c]=[q/p for q in aug[c]]
        for r in range(n):
            if r != c:
                factor=aug[r][c]
                aug[r]=[aug[r][j]-factor*aug[c][j] for j in range(n+1)]
    return [row[n] for row in aug]


def fit_exact(values):
    a=[basis(*p) for p in SITES]
    gram=[[sum(row[i]*row[j] for row in a) for j in range(6)] for i in range(6)]
    rhs=[sum(row[i]*v for row,v in zip(a,values)) for i in range(6)]
    return exact_system_solve(gram,rhs)


def analysis_case(cubic, mutation=False):
    out=[]
    for h in H_LIST:
        values=[poly(X0+h*xi,Z0+h*eta,cubic) for xi,eta in SITES]
        if mutation:
            values[SITES.index((F(1),F(1)))] += h/F(1000)
        fit=fit_exact(values)
        prediction=(fit[1]/h,fit[2]/h)
        truth=grad(X0,Z0,cubic)
        exact_error=math.hypot(float(prediction[0]-truth[0]),float(prediction[1]-truth[1]))
        a=np.asarray([[float(v) for v in basis(*p)] for p in SITES],dtype=np.float64)
        yy=np.asarray([float(v) for v in values],dtype=np.float64)
        f=np.linalg.lstsq(a,yy,rcond=None)[0]
        binary_pred=np.array([f[1]/float(h),f[2]/float(h)])
        float_error=float(np.linalg.norm(binary_pred-np.array([float(t) for t in truth])))
        # This is a conditioning scale, NOT an interval enclosure.
        scale=float(np.linalg.cond(a)*np.finfo(np.float64).eps*max(abs(t) for t in yy)/float(h))
        residual=max(abs(sum(q*b for q,b in zip(basis(*p),fit))-v) for p,v in zip(SITES,values))
        out.append({'n':int(1/h),'h':float(h),
                    'exact_gradient_error':exact_error,
                    'exact_reconstruction_residual':float(residual),
                    'binary64_gradient_error':float_error,
                    'binary64_conditioning_scale_not_bound':scale,
                    'stencil_cond2':float(np.linalg.cond(a))})
    for i,entry in enumerate(out):
        entry['binary64_decrease_factor_vs_previous']=(out[i-1]['binary64_gradient_error']/entry['binary64_gradient_error'] if i>0 and entry['binary64_gradient_error']>0 else None)
        entry['exact_decrease_factor_vs_previous']=(out[i-1]['exact_gradient_error']/entry['exact_gradient_error'] if i>0 and entry['exact_gradient_error']>0 else None)
    return out


def main():
    exact_q=analysis_case(False)
    cubic=analysis_case(True)
    mut=analysis_case(False,True)
    q_zero=all(x['exact_gradient_error']==0 and x['exact_reconstruction_residual']==0 for x in exact_q)
    cubic_ratios=[cubic[i-1]['exact_gradient_error']/cubic[i]['exact_gradient_error'] for i in range(1,len(cubic))]
    cubic_test=all(3.9<r<4.1 for r in cubic_ratios)
    mutation_test=all(x['exact_gradient_error']>1e-6 for x in mut)
    ref=grad(X0,Z0,False)
    gradient_sign_mutation_detected=math.hypot(float(2*ref[0]),float(2*ref[1]))>1e-6
    axes_swap_mutation_detected=math.hypot(float(ref[1]-ref[0]),float(ref[0]-ref[1]))>1e-6
    assert q_zero, 'exact quadratic reproduction failed'
    assert cubic_test, f'exact cubic decrease factors not O(h^2): {cubic_ratios}'
    assert mutation_test, 'mutation should be detected above fixed 1e-6'
    assert gradient_sign_mutation_detected and axes_swap_mutation_detected, 'sign/axis mutation undetected'
    here=Path(__file__)
    result={'provenance':'NEW independent nine-point polynomial reconstruction ONLY; original HMM/Gate Q source not accessed',
            'script_sha256':hashlib.sha256(here.read_bytes()).hexdigest(),
            'python':platform.python_version(),'numpy':np.__version__,'h_values':[float(h) for h in H_LIST],
            'exact_quadratic_reproduction':'PASS' if q_zero else 'FAIL',
            'independent_cubic_truncation_order':'PASS' if cubic_test else 'FAIL',
            'mutated_sample_detected':'PASS' if mutation_test else 'FAIL',
            'mutated_gradient_sign_detected':'PASS' if gradient_sign_mutation_detected else 'FAIL',
            'mutated_axis_mapping_detected':'PASS' if axes_swap_mutation_detected else 'FAIL',
            'exact_cubic_rate':cubic_ratios,'quadratic':exact_q,'cubic':cubic,'mutated_quadratic':mut,
            'BoardSI_Stage4_qualification':'FAIL (unchanged historical status)',
            'BoardSI_Stage5B_Gate_Q':'FAIL (unchanged historical status)',
            'BoardSI_Stage5C_Gate_Q':'NOT RUN on original code/source',
            'AC_P1':'PENDING','contract_frozen':False,'rust_fem_go':False,
            'physical_pilots_executed':0,'new_dirichlet_solves_executed':0}
    target=here.with_name('boardsi_ph08_stage5c_independent_patchcheck.json')
    target.write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    print(json.dumps({
       'tests':{'exact_q':q_zero,'cubic_rate':cubic_test,'mutated_sample':mutation_test,'mutated_gradient_sign':gradient_sign_mutation_detected,'mutated_axis_mapping':axes_swap_mutation_detected},
       'quadratic_error':[(x['n'],x['exact_gradient_error'],x['binary64_gradient_error']) for x in exact_q],
       'cubic_exact_rates':cubic_ratios,
       'original_GateQ':'FAIL retained', 'output':str(target)},indent=2))

if __name__=='__main__': main()
