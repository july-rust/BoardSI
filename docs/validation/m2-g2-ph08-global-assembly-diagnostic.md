# PH-08 — bounded public synthetic GLOBAL assembly diagnostic (NOT original HMM)

**Issue:** #51 · **Baseline:** main d60fd4d798852aad06c3e1d7c9e48987d88e6751
**Scientific scope:** independent PUBLIC-only uniform square HMM assembly, **not** Stage 4/5B original archive execution or physical certification.

## Why this is different from PR #70 and #72

PR #70 proves cell/face equations pointwise for a conditional uniform-square HMM local energy. PR #72 solves a separate nine-point quadratic recovery problem. Neither builds the whole coupled global linear system. This diagnostic explicitly enumerates **all** cell and shared interior-face unknowns for synthetic n=2,3,4,8 meshes; assembles and eliminates prescribed boundary faces, then performs three distinct checks:

1. **CONDITIONAL_EXACT:** exact rational stiffness/right-hand side constructed from the independently stated local bilinear energy, and the PR #70 candidate state substituted against the **whole** reduced coupled operator. Every residual component is computed with Fraction, without tolerance.
2. **OBSERVED:** independently solve the synthetic reduced float64 system with NumPy. Interpret the resulting rounded float64 vector exactly as fractions and recompute its residual against the exact assembled rational K,b. Evaluate a synthetic full-mesh center-gradient RMS, without confusing it with original Gate Q's actual Recovery/RMS definition.
3. **CONDITIONAL_EXACT SYNTHETIC ENCLOSURE:** **only n=2** and only if the exact candidate satisfies Kx=b, compute the exact rational induced infinity norm of K inverse and verify
   `||x_float - x_candidate||_inf <= ||K^{-1}||_inf ||b-Kx_float||_inf`.
   This is a guaranteed mathematical inequality for this *specified synthetic rational K,b* and captured float output. It **does not** certify the original float global assembly, original solver, physical-source/boundary inputs or original RMS. For n=3/4/8 the inverse-norm enclosure is explicitly absent.

The local energy is `h^2 |G|^2 + 2 sum R_f^2`, with cell/face order (cell,R,L,U,D), and uniform square:
`Gx=(R-L)/h`, `Gz=(U-D)/h`, `R_R=R_L=(R+L)/2-cell`, `R_U=R_D=(U+D)/2-cell`.
This yields exact `(R-L)^2+(U-D)^2+4((R+L)/2-cell)^2+4((U+D)/2-cell)^2`.
The prescribed boundary is Simpson-exact `phi(midpoint)+h^2/12`, and source is -4. Test mutations deliberately reverse source sign, change boundary to midpoint only, or perturb one observed solution by one float64 ULP.

## Execution in GitHub

Optional workflow: `.github/workflows/m2-ph08-global-assembly.yml`. It uses Python 3.13 and pinned NumPy 2.3.5, runs the bounded synthetic regressions and the known-private-object Git index guard. No archive is mounted. Only public test logs, generic bounded status and file digest evidence are uploaded.

```bash
python -m unittest discover -s tools/validation -p 'test_ph08_uniform_global_assembly.py' -v
python tools/validation/ph08_uniform_global_assembly.py
```

The new code has an explicit allowlist n={2,3,4,8}, no user-supplied matrix payload, and emits no original private operands.

## Original HMM gate still blocked

To connect this limited oracle to **actual** Stage 4 original measurements, a separately approved isolated private run needs SHA-authenticated input, full global A/b and real cell/face assembly, the actual sparse solve provenance, the actual A/D/y of every Recovery stencil, all gradient evaluation sites, and the original complete RMS reduction order. Source-only proof or a few stencil samples is insufficient for whole-RMS certification. Capture must be privacy-reviewed before any publishing; hashes/status/row counts are safer than raw operands. Capturing a float matrix then computing a perfect exact rational residual does **not** bound upstream coefficient/coordinate/source rounding. A small residual is not a forward error bound without a justified inverse/operator bound.

Existing Stage 4 replay runner `ph08_stage4_gateq_replay.py` does verify private archive hashes but needs a *separate, carefully reviewed* task for stronger child-runtime pin and stricter error-output redaction before further private execution. The original Stage 5B replay runner has different controls and must not be conflated with Stage 4.

T/R remains a future-only, preregistered proposal; do not select new thresholds using observed failures. Original Stage 4 quadratic gradient RMS (n=8/16/32) 4.043940694889194e-15, 4.241127033930931e-15, 9.236645226944874e-15 and reduction ratios 0.953506146487913 and 0.45916314091601546 against >=1.8 preserve original **FAIL**. Stage 5B also **FAIL**.

**Immutable classification:** Historical Stage 4/Stage 5B Gate Q **FAIL**; AC-P1 **PENDING**; T/R **PROPOSED_NOT_FROZEN**; PH-08 contract **NOT_FROZEN**; Rust FEM **NO_GO**; Gate M **NOT_VERIFIED**; whole-domain physical error **BOUND_NOT_CERTIFIED**. Qt Issue #46 remains separate/open; Draft PR #35 remains unmerged.

## Next research blockers

1. Privately authenticate that every executed original uniform MMS equation has the PR #70 energy/face/source assumptions.
2. Independent full-domain original assembly and solved-state forward-error budget (not only float residual), with upstream interval/rounding accounting.
3. Original full set of Recovery stencils and gradient/RMS assembly completeness; certified propagation only with an authenticated entire original sample set.
4. Separate preregistered held-out T/R threshold review and independent Gate M finite/open-domain analysis.

Sources: PR #68, #70, #72; docs/validation/m2-g2-ph08-uniform-quadratic-identity.md; Issue #51.
