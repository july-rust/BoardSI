#!/usr/bin/env python3
"""Public synthetic audit of declared PH-08 Recovery layout; never physical approval.

Original source, actual same-material connectivity and runtime method require
independent attestation. Six columns alone prove no physical matrix role.
"""
from __future__ import annotations
import hashlib
import struct
import numpy as np


def arr(x, shape=None):
    a=np.asarray(x,dtype=np.float64)
    if (shape is not None and a.shape!=shape) or not np.isfinite(a).all():
        raise ValueError("nonfinite or wrong-sized recovery input")
    return a


def recovery_matrices(ids, centers, values, materials, target, origin, face_nodes, scale, point):
    """Reconstruct reported cell-only A,y and off-origin physical gradient D.

    The caller must separately prove the original two-ring stencil, face set
    and source code. No face-value observations and no implicit weights.
    """
    if (not isinstance(ids,(list,tuple)) or len(ids)<6 or
        any(isinstance(i,bool) or not isinstance(i,int) or i<0 for i in ids)
        or list(ids)!=sorted(set(ids)) or target not in ids):
        raise ValueError("missing/duplicate/unordered cell IDs")
    n=len(ids)
    c=arr(centers,(n,2)); y=arr(values,(n,))
    o=arr(origin,(2,)); f=arr(face_nodes); s=arr(scale,(2,))
    p=arr(point,(2,))
    if (f.ndim!=2 or f.shape[1]!=2 or f.shape[0]<4 or
        np.any(s<=0) or not np.array_equal(s,np.max(np.abs(f-o),axis=0))):
        raise ValueError("face-node geometry/normalization invalid")
    if not np.array_equal(o,c[ids.index(target)]):
        raise ValueError("origin not target cell centroid")
    if not isinstance(materials,(list,tuple)) or len(materials)!=n or any(
        v!=materials[ids.index(target)] for v in materials):
        raise ValueError("mixed material samples")
    rows=[]
    for center in c:
        u,v=(center-o)/s
        rows.append([1.,u,v,u*u,u*v,v*v])
    u,v=(p-o)/s
    D=np.asarray([[0.,1./s[0],0.,(2*u)/s[0],v/s[0],0.],
                  [0.,0.,1./s[1],0.,u/s[1],(2*v)/s[1]]])
    A=np.asarray(rows)
    if np.linalg.matrix_rank(A)!=6: raise ValueError("rank deficient")
    return A,y,D


def sample_digest(ids,A,y,D):
    """Internal fingerprint only; external pin and original source audit required."""
    a=arr(A); b=arr(y); d=arr(D,(2,6))
    if a.shape!=(len(ids),6) or b.shape!=(len(ids),):
        raise ValueError("bad fingerprint layout")
    h=hashlib.sha256(b"PH08-DECLARED-SYNTHETIC-SAMPLE-V1\0")
    h.update(struct.pack("<Q",len(ids)))
    for i in ids:
        if isinstance(i,bool) or not isinstance(i,int) or i<0:
            raise ValueError("bad cell id")
        h.update(struct.pack("<Q",i))
    for z in (a,b,d): h.update(np.ascontiguousarray(z,dtype="<f8").tobytes())
    return h.hexdigest()


def judge_capture(ids,centers,values,materials,target,origin,face_nodes,scale,
                  point,captured_A,captured_y,captured_D,observed_coefficients,
                  expected_digest=None,source_attested=False,solver_entrypoint=None,
                  weights=None):
    """FAIL on mutations, NOT VERIFIED on incomplete external physical evidence."""
    out={"status":"NOT VERIFIED","historical_gate_q":"FAIL","AC_P1":"PENDING",
         "contract":"NOT FROZEN","rust_fem":"NO-GO",
         "certified_float_bound":"BOUND_NOT_CERTIFIED","scope":"SYNTHETIC_OPERATOR_ONLY"}
    try:
        A,y,D=recovery_matrices(ids,centers,values,materials,target,origin,face_nodes,scale,point)
        ca=arr(captured_A,A.shape);cy=arr(captured_y,y.shape);cd=arr(captured_D,D.shape)
        c=arr(observed_coefficients,(6,))
        if (not np.array_equal(ca,A) or not np.array_equal(cy,y)
            or not np.array_equal(cd,D) or weights is not None):
            out.update(status="FAIL",reason="operator/gradient/weight mismatch")
            return out
        if (not source_attested or expected_digest!=sample_digest(ids,A,y,D)
            or solver_entrypoint!="numpy.linalg.lstsq"):
            out["reason"]="source/provenance/runtime solver not independently attested"
            return out
        from ph08_gateq_recovery_hardening import rational_operator,gradient_transfer
        exact=rational_operator(A,y,role="design")
        T,K=gradient_transfer(A,D)
        out.update(
            observed_coefficients_error_inf=max(abs(float(v)-float(r))
                                            for v,r in zip(c,exact)),
            exact_linear_transfer_norm_inf=float(K),
            constant_shift_null=all(sum(row)==0 for row in T),
            reason="original physical tolerance/certified floating bound unavailable")
        return out
    except (ValueError,TypeError,OverflowError,IndexError,np.linalg.LinAlgError):
        out.update(status="FAIL",reason="invalid recovery capture")
        return out
