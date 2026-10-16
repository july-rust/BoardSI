#!/usr/bin/env python3
"""Stage 4 original-HMM runtime LS probe; synthetic CI is NOT HMM qualification."""
from __future__ import annotations
import argparse
from fractions import Fraction as F
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile
from ph08_gateq_independent import SHAPES, SIZES, samples
from ph08_evidence_audit import inspect
from ph08_stage4_gateq_replay import (STAGE4_ZIP_SHA256, ZIP_ROOT,
    compare_rows, check_historical_gate)

def rational_fit(a,b):
    """Exactly fit finite 64-bit operands, NOT exact unrounded PDE data."""
    if len(a)<6 or any(len(r)!=6 for r in a) or len(b)!=len(a):
        raise ValueError("unsupported six-column recovery matrix")
    A=[[F.from_float(float(x)) for x in r] for r in a]
    y=[F.from_float(float(v)) for v in b]
    gram=[[sum(row[i]*row[j] for row in A) for j in range(6)] for i in range(6)]
    rhs=[sum(row[i]*v for row,v in zip(A,y)) for i in range(6)]
    aug=[gram[i]+[rhs[i]] for i in range(6)]
    for j in range(6):
        p=next((i for i in range(j,6) if aug[i][j]),None)
        if p is None: raise ValueError("singular polynomial system")
        aug[j],aug[p]=aug[p],aug[j]
        v=aug[j][j]
        aug[j]=[x/v for x in aug[j]]
        for i in range(6):
            if i!=j:
                v=aug[i][j]
                aug[i]=[x-v*t for x,t in zip(aug[i],aug[j])]
    return [r[-1] for r in aug]

def diagnose(a,b,computed=None):
    import numpy as np
    import mpmath as mp
    A=np.asarray(a,dtype=np.float64); y=np.asarray(b,dtype=np.float64)
    if (A.ndim!=2 or A.shape[1]!=6 or A.shape[0]<6 or
        y.shape!=(A.shape[0],) or not np.isfinite(A).all()
        or not np.isfinite(y).all()):
        raise ValueError("invalid/nonfinite recovery inputs")
    s=np.linalg.svd(A,compute_uv=False)
    if int(np.linalg.matrix_rank(A))!=6: raise ValueError("rank deficient")
    exact=rational_fit(A,y)
    got=np.linalg.lstsq(A,y,rcond=None)[0] if computed is None else np.asarray(computed,dtype=float)
    if got.shape!=(6,) or not np.isfinite(got).all():
        raise ValueError("invalid computed coefficients")
    with mp.workdps(100):
        mat=mp.matrix([[mp.mpf(F.from_float(float(v)).numerator)/F.from_float(float(v)).denominator for v in row] for row in A])
        vec=mp.matrix([mp.mpf(F.from_float(float(v)).numerator)/F.from_float(float(v)).denominator for v in y])
        hp=mp.lu_solve(mat.T*mat,mat.T*vec)
        hp_diff=max(abs(hp[i]-mp.mpf(exact[i].numerator)/exact[i].denominator) for i in range(6))
        hp_diff_string=mp.nstr(hp_diff,12)
    return {"rows":int(A.shape[0]),"rank":6,"condition_A_2":float(s[0]/s[-1]),
            "singular_min":float(s[-1]),"singular_max":float(s[0]),
            "residual_norm2":float(np.linalg.norm(A@got-y)),
            "coefficient_diff_vs_exact_binary64_inputs":max(abs(float(got[i])-float(exact[i])) for i in range(6)),
            "mp100_vs_rational_max_coeff":hp_diff_string,
            "input_interpretation":"exactly represented original binary64 operands",
            "actual_gradient_mapping":"NOT VERIFIED",
            "certified_float_bound":"BOUND_NOT_CERTIFIED"}

def synthetic_report():
    import numpy as np
    records=[]
    for shape in SHAPES:
        for kind in ("quadratic","cubic"):
            for n in SIZES:
                A,y,_,_=samples(shape,kind,n)
                r=diagnose(np.asarray(A,dtype=float),np.asarray(y,dtype=float))
                r.update(shape=shape,polynomial=kind,n=n,origin="SYNTHETIC_ONLY")
                records.append(r)
    return {"status":"SYNTHETIC_ONLY","cases":len(records),"rows":records,
            "original_hmm":"NOT RUN","stage5b":"NOT RUN",
            "historical_gate_q":"FAIL","AC_P1":"PENDING",
            "contract":"NOT FROZEN","rust_fem":"NO-GO",
            "certified_float_bound":"BOUND_NOT_CERTIFIED"}

PRIVATE_SCRIPT=r"""
import importlib,inspect,json,sys,hashlib
import numpy as np
mode=sys.argv[3]
if mode not in ("baseline","instrumented"):raise ValueError("unsupported probe mode")
sys.path.insert(0,sys.argv[2]);sys.path.insert(0,sys.argv[1])
hmm=importlib.import_module("m2_ph08_hmm_reference")
module=importlib.import_module("m2_ph08_stage3")
cls=module.Recovery
descriptor=inspect.getattr_static(cls,"polynomial")
if isinstance(descriptor,(staticmethod,classmethod)):
    original=descriptor.__func__
else:
    original=descriptor
active=[False]
stats={"recovery_calls":0,"solver_calls":0,"qualifying_solver_calls":0,
       "nonqualifying_solver_calls":0,"operators":[],"errors":[],
       "operator_role":"UNCLASSIFIED","aliases_and_custom_solver":"NOT VERIFIED",
       "scipy_instrumentation":"NOT INSTALLED"}
if mode=="instrumented":
    def recovery(*args,**kwargs):
        stats["recovery_calls"]+=1
        old=active[0];active[0]=True
        try:return original(*args,**kwargs)
        finally:active[0]=old
    cls.polynomial=(staticmethod(recovery) if isinstance(descriptor,staticmethod)
        else classmethod(recovery) if isinstance(descriptor,classmethod) else recovery)
    def wrap(fn,kind):
        def patched(A,b,*args,**kw):
            out=fn(A,b,*args,**kw)
            if active[0]:
                stats["solver_calls"]+=1
                old=active[0];active[0]=False
                try:
                    a=np.asarray(A);v=np.asarray(b)
                    coeff=out[0] if kind.endswith("lstsq") else out
                    c=np.asarray(coeff)
                    eligible=(a.ndim==2 and a.shape[1]==6 and
                        a.shape[0]>=6 and v.ndim==1 and
                        v.shape==(a.shape[0],) and c.shape==(6,) and
                        np.isfinite(a).all() and np.isfinite(v).all()
                        and np.isfinite(c).all() and
                        (not kind.endswith("solve") or a.shape==(6,6)))
                    if not eligible:
                        stats["nonqualifying_solver_calls"]+=1
                    else:
                        stats["qualifying_solver_calls"]+=1
                        # Cap applies to QUALIFYING operator records, never
                        # incidental 2x2 solves or unrelated function calls.
                        if len(stats["operators"])<24:
                            record={"solver_entrypoint":kind,
                                    "matrix_shape":list(a.shape),
                                    "rhs_shape":list(v.shape),
                                    "solution_shape":list(c.shape),
                                    "operator_role":"UNCLASSIFIED",
                                    "sampling_semantics":"NOT VERIFIED",
                                    "float_bound":"BOUND_NOT_CERTIFIED"}
                            # An observed 'solve' is a direct algebraic solve;
                            # its physical role cannot be deduced from shape.
                            from ph08_gateq_recovery_hardening import observed_diagnostics
                            role="linear_system" if kind.endswith("solve") else "design"
                            try:
                                d=observed_diagnostics(a,v,c,role=role,
                                    method="solve" if kind.endswith("solve") else "lstsq")
                                record["algebraic_diagnostics"]=d
                                record["algebraic_role_hypothesis"]=role
                            except (ValueError,TypeError,np.linalg.LinAlgError):
                                stats["errors"].append("ALGEBRAIC_PROBE_NOT_VERIFIED")
                            stats["operators"].append(record)
                except Exception:
                    stats["errors"].append("PROBE_NOT_VERIFIED")
                finally:active[0]=old
            return out
        return patched
    np.linalg.lstsq=wrap(np.linalg.lstsq,"numpy_lstsq")
    np.linalg.solve=wrap(np.linalg.solve,"numpy_solve")
    try:
        import scipy.linalg as sc
        sc.lstsq=wrap(sc.lstsq,"scipy_lstsq")
        sc.solve=wrap(sc.solve,"scipy_solve")
        stats["scipy_instrumentation"]="INSTALLED"
    except ImportError:
        # SciPy is optional. Its absence is NOT a failure of NumPy-only HMM.
        stats["scipy_instrumentation"]="NOT INSTALLED"
stage4=importlib.import_module("m2_ph08_stage4_qualification")
out=stage4.mms_suite()
functions={}
for label,fn in [("solve",hmm.solve),("Recovery.polynomial",original),
                 ("mms_suite",stage4.mms_suite)]:
    try:
        functions[label]={"module":inspect.getmodule(fn).__name__,
                          "line":inspect.getsourcelines(fn)[1],
                          "signature":str(inspect.signature(fn))}
    except (OSError,TypeError,AttributeError):
        functions[label]={"source_location":"NOT VERIFIED"}
print(json.dumps({"mms":out,"functions":functions,"stats":stats},allow_nan=False))
"""

def original_report(archive):
    archive=Path(archive)
    if not archive.is_file(): raise ValueError("original ZIP unavailable")
    digest=hashlib.sha256(archive.read_bytes()).hexdigest()
    if digest!=STAGE4_ZIP_SHA256: raise ValueError("Stage 4 external digest mismatch")
    audited=inspect(archive,expected_sha256=STAGE4_ZIP_SHA256,
                    expected_entries=338,expected_sources=20)
    if (audited.get("result")!="PASS" or audited.get("checksum_verified")!=338
        or audited.get("source_hash_verified")!=20):
        raise ValueError("Stage 4 ZIP 338/20 integrity not verified")
    with zipfile.ZipFile(archive) as z:
        manifest=json.loads(z.read(ZIP_ROOT+"evidence/manifest.json"))["sources"]
        prior=json.loads(z.read(ZIP_ROOT+"evidence/qualification.json"))["mms"]
        if not isinstance(manifest,dict) or len(manifest)!=20:
            raise ValueError("unexpected snapshot inventory")
        with tempfile.TemporaryDirectory(prefix="ph08-private-") as temp:
            root=Path(temp)
            for name,sha in manifest.items():
                if not isinstance(name,str) or Path(name).name!=name or not name.endswith((".py",".json",".md")):
                    raise ValueError("unsafe snapshot name")
                raw=z.read(ZIP_ROOT+"evidence/source_snapshot/"+name)
                if hashlib.sha256(raw).hexdigest()!=sha.lower():
                    raise ValueError("snapshot hash mismatch")
                (root/name).write_bytes(raw)
            env=os.environ.copy();env.pop("PYTHONPATH",None)
            results={}
            for mode in ("baseline","instrumented"):
                process=subprocess.run([sys.executable,"-c",PRIVATE_SCRIPT,str(root),
                    str(Path(__file__).resolve().parent),mode],cwd=root,env=env,
                    capture_output=True,text=True,timeout=300,check=False)
                if process.returncode:
                    # Never expose original private source/paths in stderr.
                    raise RuntimeError("private HMM execution failed [stderr redacted]")
                results[mode]=json.loads(process.stdout)
            baseline=results["baseline"]
            run=results["instrumented"]
    errs=compare_rows(run["mms"],prior)
    errs.extend("baseline_vs_instrumented: "+str(e) for e in
                compare_rows(run["mms"],baseline["mms"]))
    unchanged=(run["mms"]==baseline["mms"])
    if not unchanged:errs.append("instrumentation_changed_original_mms")
    if run["stats"]["errors"]:errs.append("instrumentation_diagnostic_incomplete")
    if not check_historical_gate(run["mms"]):errs.append("Stage 4 FAIL changed")
    invocations=run["stats"]["recovery_calls"]
    count=len(run["stats"]["operators"])
    qualifying=run["stats"]["qualifying_solver_calls"]
    status=("NOT VERIFIED" if errs else
            "ORIGINAL_RECOVERY_NOT_CAPTURED" if not invocations or not qualifying or not count else
            "ORIGINAL_RECOVERY_EXECUTED_UNCLASSIFIED")
    return {"status":status,"instrumentation_output_identical":unchanged,
            "original_mms_replayed":len(run["mms"].get("rows",[])),
            "historical_replay_mismatches":errs,
            "archive_sha256":digest,"source_function_metadata":run["functions"],
            "recovery_invocations":invocations,
            "operators_captured":count,
            "qualifying_solver_calls":qualifying,
            "nonqualifying_solver_calls":run["stats"]["nonqualifying_solver_calls"],
            "operator_diagnostics":run["stats"]["operators"],
            "instrument_errors":run["stats"]["errors"],
            "reconstruction_gradient":"NOT VERIFIED",
            "input_sampling_semantics":"NOT VERIFIED",
            "certified_float_bound":"BOUND_NOT_CERTIFIED",
            "historical_gate_q":"FAIL","AC_P1":"PENDING",
            "contract":"NOT FROZEN","rust_fem":"NO-GO","stage5b":"NOT RUN"}

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--archive",type=Path)
    p.add_argument("--output-json",type=Path)
    a=p.parse_args()
    try:
        o=original_report(a.archive) if a.archive else synthetic_report()
        rc=0 if o["status"] == "SYNTHETIC_ONLY" else 2
    except (ValueError,OSError,RuntimeError,KeyError,zipfile.BadZipFile,subprocess.TimeoutExpired) as exc:
        o={"status":"NOT VERIFIED","error":str(exc),
           "historical_gate_q":"FAIL","certified_float_bound":"BOUND_NOT_CERTIFIED"}
        rc=2
    o["tool_sha256"]=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    # Private source/probe metadata must never leak into shared console logs.
    console=o if not a.archive else {key:o.get(key) for key in
        ("status","original_mms_replayed","recovery_invocations",
         "operators_captured","historical_gate_q","certified_float_bound")}
    raw=json.dumps(o,allow_nan=False,indent=2)+"\n"
    console_raw=json.dumps(console,allow_nan=False,indent=2)+"\n"
    if a.output_json:a.output_json.write_text(raw,encoding="utf-8")
    print(console_raw)
    return rc
if __name__=="__main__":raise SystemExit(main())
