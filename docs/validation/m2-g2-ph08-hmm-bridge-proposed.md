# PH-08 HMM original-to-oracle operator bridge: PROPOSED / NOT FROZEN

Issue #51. This is a diagnostic component, not physical validation or a new
acceptance contract. Historical Stage 4 Gate Q **FAIL**, AC-P1 **PENDING**,
PH-08 Contract **NOT FROZEN**, Rust FEM **NO-GO**, Stage 5B **NOT RUN**.

## Authorized original evidence and runtime instrumentation
Private Stage 4 ZIP external SHA-256 must equal
`6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`.
Before importing original code, the original archive auditor must PASS 338/338
checksums and 20/20 source inventory records. Verified source snapshots alone
are unpacked privately into a temporary directory. The original
`m2_ph08_stage4_qualification.mms_suite()` executes with an instrumentation
wrapper around `m2_ph08_stage3.Recovery.polynomial`. The wrapper probes
NumPy/SciPy `lstsq` or `solve` entrypoints for six-coefficient systems
actually invoked inside Recovery, without modifying arguments or returns.
It checks the 36 replayed MMS rows against archived evidence and preserves FAIL.
Original function/module/line/signature metadata are emitted for source review.
The hook may MISS aliased or custom solvers; no capture => NOT VERIFIED.

For captured A and b, a separate Fraction Gaussian solver solves normal
equations on EXACT representations of the already-rounded binary64 operands.
mpmath at 100 digits checks the rational solution; float64 rank, singular
values, condition and residual are measured. A QR, LU, or custom original
solver is NOT assumed: method name is recorded when observable.
Normal equations square conditioning and the reference fit does not give
the original solver's floating error bound.

**Not yet verified:** actual HMM cell/face samples, weights applied prior to
the intercepted call, coordinate origin and normalized basis, coefficient
to physical gradient map, solver potential discretization error, the complete
A/B/C original Quadratic/Cubic gradients across uniform/skew/graded n=8,16,32,
or a derivation of observed -h^2/6 potential offset. Exposing original
matrix arithmetic only does not establish identical geometry or physical
meaning. This public tool MUST NOT substitute PR #55's synthetic nine-node
stencil for HMM source sites. No true floating bound is certified:
`BOUND_NOT_CERTIFIED`. No raw private arrays/ZIP are shipped to public CI.

## Gate Q prospective protocol
Regime T: prospective three-or-more-resolution absolute Gradient and
decrease-rate tests against independent SAME-problem physical references.
Regime R: only after source-exact original reconstruction, matching operands
and sampling semantics, reference accuracy, perturbation propagation,
rigorous directed-rounding bounds, preregistered truncation/roundoff crossover,
and mutation sensitivity. If unavailable: NOT VERIFIED or INDETERMINATE,
NEVER post-hoc PASS. Gate M energy bracketing/flux equilibrium/open-domain
C/C0 certification and Stage 5B independent SHA remain unresolved.

## Run (public synthetic)
```sh
python -m pip install numpy==2.3.5 scipy==1.17.0 mpmath==1.3.0
python -m unittest discover -s tools/validation -p 'test_ph08_gateq_hmm_bridge.py' -v
python tools/validation/ph08_gateq_hmm_bridge.py --output-json /tmp/ph08-bridge-synthetic.json
```

## Run (only approved private Stage 4 ZIP)
```sh
python tools/validation/ph08_gateq_hmm_bridge.py --archive /approved/private/boardsi-ph08-stage4-qualification.zip --output-json /private/bridge.json
```
Do NOT publish source ZIP, private inputs or raw instrument traces to GitHub.
Original runtime has NOT been executed in the public CI. The stdout can
contain private source-method metadata and requires redistribution review.
