#!/usr/bin/env python3
"""Stage 5D independent numeric Gate-Q *preflight*. NOT original BoardSI HMM.

Patch tests with three rational geometries, polynomial exact arithmetic,
float64 solver, and a rational enclosure for INPUT CONVERSION ONLY.
All calculations are 2-D reconstruction; no PDE/physical solve is run.
"""
from __future__ import annotations
import hashlib
import json
import math
import platform
from fractions import Fraction as Q
from pathlib import Path
import numpy as np

L = Q(1,1000)    # 1 mm reference length, SI physical coordinates
CX, CZ = Q(7,100)*L, -Q(3,100)*L
N = (8, 16, 32, 64, 128)


def geometries():
    base = [(Q(i),Q(j)) for i in (-1,0,1) for j in (-1,0,1)]
    return {
        'uniform':base,
        'skew':[(xi+eta/Q(4), eta+xi/Q(7)) for xi,eta in base],
        'graded':[(xi+(Q(1,5) if xi>0 else -Q(1,10) if xi<0 else Q(0)),
                   eta+(Q(2,5) if eta>0 else -Q(1,6) if eta<0 else Q(0))) for xi,eta in base],
    }


def basis(x,z): return (Q(1),x,z,x*x,x*z,z*z)


def gauss(a,b):
    n=len(b)
    z=[list(a[i])+[b[i]] for i in range(n)]
    for k in range(n):
        pivot=next((i for i in range(k,n) if z[i][k]!=0),None)
        if pivot is None: raise ArithmeticError('rank-deficient reconstruction')
        z[k],z[pivot]=z[pivot],z[k]
        d=z[k][k]
        z[k]=[v/d for v in z[k]]
        for i in range(n):
            if i!=k:
                m=z[i][k]
                z[i]=[z[i][j]-m*z[k][j] for j in range(n+1)]
    return [z[i][-1] for i in range(n)]


def operators(sites):
    a=[basis(*p) for p in sites]
    gram=[[sum(v[i]*v[j] for v in a) for j in range(6)] for i in range(6)]
    w=[]
    for row in range(6):
        e=[Q(1) if j==row else Q(0) for j in range(6)]
        ginv=gauss(gram,e)
        w.append([sum(ginv[j]*v[j] for j in range(6)) for v in a])
    return a,w


def polynomial(x,z,cubic=False):
    q=Q(3,5)+Q(7,4)*x-Q(9,8)*z+Q(1,3)*x*x+Q(2,5)*x*z-Q(1,6)*z*z
    if cubic: q+=6*x*x*x-3*x*x*z+5*z*z*z
    return q


def gradient(x,z,cubic=False):
    gx=Q(7,4)+Q(2,3)*x+Q(2,5)*z
    gz=-Q(9,8)+Q(2,5)*x-Q(1,3)*z
    if cubic: gx+=18*x*x-6*x*z; gz+=-3*x*x+15*z*z
    return gx/L,gz/L


def weighted_gradient(w,ys,h):
    return tuple(sum(v*y for v,y in zip(row,ys))/h for row in (w[1],w[2]))


def vec_error(a,b):
    return math.hypot(float(a[0]-b[0]),float(a[1]-b[1]))


def evaluate(geo,sites,n,cubic,w,a):
    h=L/n
    xx,zz=CX/L,CZ/L
    samples=[polynomial(xx+u/n,zz+v/n,cubic) for u,v in sites]
    truth=gradient(xx,zz,cubic)
    exact=weighted_gradient(w,samples,h)
    exacterr=vec_error(exact,truth)
    yf=[float(y) for y in samples]
    rounded=[Q(*y.as_integer_ratio()) for y in yf]
    rounded_exact=weighted_gradient(w,rounded,h)
    input_error=vec_error(rounded_exact,truth) if not cubic else vec_error(rounded_exact,exact)
    # This bounds round-to-nearest conversion of exact-valued SAMPLE INPUTS only.
    # No claim about floating assembly / least squares / HMM / geometry errors.
    envelope=[]
    for row in (w[1],w[2]):
        accum=sum(abs(weight)*Q(*math.ulp(float(s)).as_integer_ratio())/(2*h)
                  for weight,s in zip(row,samples))
        envelope.append(accum)
    input_delta=tuple(rounded_exact[k]-exact[k] for k in range(2))
    covered=all(abs(input_delta[k]) <= envelope[k] for k in range(2))
    amat=np.asarray([[float(q) for q in row] for row in a],dtype=np.float64)
    coeff,res,rank,singular=np.linalg.lstsq(amat,np.array(yf),rcond=None)
    bin_grad=np.array([coeff[1]/float(h),coeff[2]/float(h)])
    flt_error=float(np.linalg.norm(bin_grad-np.array([float(t) for t in truth])))
    float_solve_error=float(np.linalg.norm(bin_grad-np.array([float(v) for v in rounded_exact])))
    phys=np.asarray([[1.0, float(CX + h*u),float(CZ+h*v),float(CX + h*u)**2,
                     float(CX + h*u)*float(CZ+h*v),float(CZ+h*v)**2] for u,v in sites])
    rawco,_,rawrank,_=np.linalg.lstsq(phys,np.array(yf),rcond=None)
    rawgrad=np.array([rawco[1]+2*rawco[3]*float(CX)+rawco[4]*float(CZ),
                      rawco[2]+rawco[4]*float(CX)+2*rawco[5]*float(CZ)])
    rawerr=float(np.linalg.norm(rawgrad-np.array([float(t) for t in truth])))
    return {
        'geometry':geo,'n':n,'h_m':float(h),'MMS':'cubic' if cubic else 'quadratic',
        'exact_gradient_error_V_per_m':exacterr,
        'float64_gradient_error_V_per_m':flt_error,
        'input_rounding_effect_V_per_m':input_error,
        'float64_solver_only_difference_V_per_m':float_solve_error,
        'input_conversion_component_bounds_V_per_m':[float(v) for v in envelope],
        'input_rounding_enclosed':covered,
        'normalized_stencil_condition2':float(np.linalg.cond(amat)),
        'normalized_stencil_rank':int(rank),
        'raw_physical_basis_condition2':float(np.linalg.cond(phys)),
        'raw_physical_basis_rank':int(rawrank),
        'raw_basis_gradient_error_V_per_m':rawerr,
    }


def main():
    allrows=[]
    probes=[]
    errors=[]
    for gn,sites in geometries().items():
        a,w=operators(sites)
        assert len(sites)==9 and len(w)==6
        a_float=np.asarray([[float(q) for q in row] for row in a])
        assert np.linalg.matrix_rank(a_float)==6
        for cubic in (False,True):
            cases=[evaluate(gn,sites,n,cubic,w,a) for n in N]
            allrows+=cases
            if not cubic:
                assert all(c['exact_gradient_error_V_per_m']==0 for c in cases),gn
            else:
                ratios=[cases[i-1]['exact_gradient_error_V_per_m']/cases[i]['exact_gradient_error_V_per_m'] for i in range(1,len(cases))]
                assert all(3.99<r<4.01 for r in ratios),(gn,ratios)
                probes.append({'geometry':gn,'cubic_truncation_ratios':ratios})
            assert all(c['input_rounding_enclosed'] for c in cases),gn
            assert all(c['normalized_stencil_rank']==6 for c in cases)
        # Explicit mutations: exact reconstruction against the analytic oracle.
        h=L/N[0]
        sample=[polynomial(CX/L+u/N[0],CZ/L+v/N[0]) for u,v in sites]
        wrong=list(sample); wrong[0]+=Q(1,50000)
        truth=gradient(CX/L,CZ/L)
        mutation_delta=vec_error(weighted_gradient(w,wrong,h),truth)
        assert mutation_delta>Q(1,100), (gn,mutation_delta)
        # Physically dimensioned unit mismatch cannot be accepted as gradient.
        right=weighted_gradient(w,sample,h)
        units_wrong=weighted_gradient(w,sample,1000*h)
        unit_error=vec_error(units_wrong,truth)
        assert unit_error>100, (gn,unit_error)
        sign_error=vec_error(tuple(-x for x in right),truth)
        axes_error=vec_error(tuple(reversed(right)),truth)
        assert sign_error>100 and axes_error>100
        errors.append({'geometry':gn,'single_sample_mutation_gradient_error_V_per_m':mutation_delta,
                       'unit_scale_mutation_gradient_error_V_per_m':unit_error,
                       'gradient_sign_mutation_error_V_per_m':sign_error,
                       'axis_swap_mutation_error_V_per_m':axes_error,
                       'all_detected':True})
    degenerate=[(Q(i),Q(0)) for i in range(-4,5)]
    degeneracy_detected=False
    try: operators(degenerate)
    except ArithmeticError: degeneracy_detected=True
    assert degeneracy_detected
    raw={
        'scope':'Independent quadratic/cubic 2-D polynomial reconstruction. NOT original PH-08 HMM/Gate Q or FEM',
        'platform':{'python':platform.python_version(),'numpy':np.__version__,'platform':platform.platform()},
        'geometries':list(geometries()),'n_values':list(N),'si_physical_unit':'metre; gradient in V/m',
        'tests':{'quadratic_exact': 'PASS 15/15','cubic_exact_O_h2':'PASS 3 geometries x 4 ratios',
                 'input_rounding_enclosure':'PASS 30/30, sample conversion ONLY',
                 'mutation_detection':'PASS 12/12 + degenerate rank',
                 'nondegenerate_rank':'PASS 30/30'},
        'cubic_ratios':probes,'negative_controls':errors,'degenerate_stencil_rejected':degeneracy_detected,
        'cases':allrows,
        'original_stage5b_gateq':'FAIL (unchanged; original evidence archive unavailable)',
        'original_stage5d_gateq':'NOT RUN',
        'AC_P1':'PENDING','contract_frozen':False,'rust_fem_go':False,
        'full_physics_solves':0,'verified_roundoff_enclosure_for_HMM':False,
    }
    output=Path(__file__).with_suffix('.json')
    raw['source_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    output.write_text(json.dumps(raw,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    summary={'tests':raw['tests'],'cubic_ratios':probes,
             'per_geometry_quadratic':[{'geometry':g,'values':[{'n':x['n'],
              'exact':x['exact_gradient_error_V_per_m'], 'float64':x['float64_gradient_error_V_per_m'],
              'input_bound_V_per_m':x['input_conversion_component_bounds_V_per_m'],
              'normalized_condition':x['normalized_stencil_condition2'],
              'raw_condition':x['raw_physical_basis_condition2']} for x in allrows if x['geometry']==g and x['MMS']=='quadratic']} for g in geometries()],
             'negative_controls':errors}
    print(json.dumps(summary,indent=2))

if __name__=='__main__':main()
