# PH-08 Gate Q exact captured-binary64 arithmetic and T/R proposal (2026-10-17 KST)

**Issue:** #51. **Base:** main `1735922aad63c0426efdad9daf3c0dc3e39054b1` (Verified). This PR contains **public-only validation tooling** and unit tests; it is NOT physical acceptance. Gate Q historical **FAIL**; AC-P1 **PENDING**; Contract **NOT FROZEN**; Rust FEM **NO-GO**; Gate M **NOT VERIFIED**.

## Independent private evidence access, scope-separated

- An approved private Project Library copy of `boardsi-ph08-stage5b-evidence.zip` was read for the first time in this follow-up. Exact external archive SHA-256 `f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36`, size 2,641,874 bytes, 42 ZIP members.
- Stage 5B internal `stage5b/SHA256SUMS` has **36** entries, **36/36 MATCH**; all **13** `stage5b/source_snapshot/` entries are covered. Unlisted members are `README-ko.md`, three `inputs/` files and one `stage4/` historical qualification file. These unlisted members are NOT silently treated as signed by the Stage 5B internal manifest; their bytes are covered by the external ZIP SHA.
- Embedded `inputs/boardsi-ph08-stage4-qualification.zip`: **2,301,227 bytes**, exact SHA-256 `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`; its own 338/338 SHA256SUMS match and its `evidence/manifest.json` lists 20 sources. Stage 4 raw bytes were not uploaded.
- `stage5b/qualification.json` itself says `Gate_Q=FAIL` and retains the historic uniform quadratic ratios 0.953506146487913 and 0.45916314091601546. **Stage 5B physical re-execution was NOT RUN in this step.** Stage 5B byte integrity PASS must not be substituted for the mathematical/physical Gate Q qualification. This is a sanitized provenance record: no private source, raw operator matrix, geometry, or other confidential archive members in GitHub.

## Captured arithmetic — mathematical definition

`tools/validation/ph08_gateq_exact_captured.py` converts each already-captured **binary64** A (n×6), y (n), D (2×6), observed coefficients and *independently supplied* observed gradient output into **exact** Python `Fraction` values, *before any reference subtraction*. For full column rank, it computes the unique exact least-squares solution of the already-rounded system through exact rational normal equations (not floating normal equations), and its exact fixed-input transfer map

```text
c* := (A^T A)^(-1) A^T y   [exact rational arithmetic]
g* := D c*
T  := D (A^T A)^(-1) A^T
||T||_inf := max_i sum_j |T_ij|
||g_observed - g*||_inf = exact rational difference
||c_observed - c*||_inf = exact rational difference
```

It separately reports primal residual `||A c_observed - y||_inf`, normal-equations residual `||A^T (A c_observed - y)||_inf`, and relative normal-equations backward residual w.r.t. the exact normal system. **Important:** that backward residual is not a bound on perturbations to A and y in the original least-squares problem. Normal equations are used here solely as an *exact rational reference*, never as a recommended floating LS solver. Rank-deficient cases are rejected; cond₂(A) is NOT certified by this code.

A captured mathematical equality is **not** a priori solver stability, rounding-error, physical-coordinate, PDE-discretization, or input potential error theorem. Exact comparison of a measured float output against exact `Fraction` input operands gives an ex post arithmetic error for **this captured** binary64 system only. In particular it cannot certify the assembly computation that produced those operands, unrecorded intermediate operations, or the physical field. The output is hard-coded to `physical_error_bound=BOUND_NOT_CERTIFIED` and `physical_gate_q=FAIL`.

Constant y-shifts cancel in exact `T` at fixed A,D only if the constant polynomial is represented and full rank; changed A or D requires separate perturbation bounds, and `E=-∇φ` sign convention must be checked against original runtime provenance.

## Proposed T/R decision (NOT FROZEN)

`prereg_hash` canonicalizes the rule JSON and computes a SHA-256. **Hash alone is NOT preregistration**: reviewers must record the rule's exact commit SHA and SHA-256 **before** unseen original validation data are evaluated. The evaluator rejects an edited hash or an unregistered case, and accepts the predeclared `T` or `R` designation only—never makes a regime selection from measured errors.

- `T`: two observed reduction ratios must each be >= 1.8. The historical uniform quadratic ratios fail; future numeric passes are tagged `T_RATE_ONLY_NOT_PHYSICAL_APPROVAL`, never Rust FEM authorization.
- `R`: original operator/source confirmation, exact reference, certified input, assembly, solver, physical-gradient, and transfer bounds, plus independently audited negative controls are **all prerequisites**. With missing certifications `INDETERMINATE`; even caller-provided all-True flags give **NOT VERIFIED**, not PASS, until external independent source-bound audit. Any attempt to send observed reductions into R is a **FAIL**.
- There is **no justified numeric T→R crossover** in this revision. Required next: preregistered and physically dimensioned truncation envelope, certified binary64 forward upper bound from actual original solver and input assembly, exact original mesh/cell-level validation including cubic/graded/skew meshes, independent sign-off, Gate M. Do not infer crossover from ex-post measured error.

## GitHub Actions and security

The new module and synthetic regression test file are imported by the pre-existing optional `test_ph08_gateq_independent.py` suite, so its original `.github/workflows/m2-ph08-gateq-independent.yml` PR path trigger is preserved without adding a required CI. Test assertions cover exact rational forward error before rounding, a 120-digit independently built high-precision reference, non-origin D, rank failure, NaN/Inf and nonbinary input, residuals, existing historical FAIL, T rule pass-only-is-scoped, unknown cases, hash mutation, R post-hoc data leakage, missing and self-asserted certificates. Numeric method tests remain SYNTHETIC, not original HMM.

Do not merge unsigned source commits or bypass main protections. If PR source cannot be published with GitHub-Verified signatures, keep this feature in Draft/open state and record blocker; user SSH keys/localhost signatures are not an option.

**Unresolved independent blockers:** Stage 5B physical replay NOT RUN; original HMM full binary64 error envelope NOT CERTIFIED; observed uniform `-h²/6` NOT DERIVED; Gate Q new R/T criteria PROPOSED; Gate M whole-space physical certification NOT VERIFIED; PH-08 Contract NOT FROZEN; Rust FEM NO-GO.


## Follow-up hardening: GitHub PR #60 (prospective synthetic fixture only)

**Trusted-pinning scope:** `docs/validation/ph08-gateq-tr-synthetic-prereg-v1.json`
is a proposed **synthetic-only** candidate, with *canonical JSON*
SHA-256 `4674bc15a4e796baf44b2ed16ccb473ddeb9a75c684aa678bdbe1eaf73a6630b`.
The independent expected digest is a reviewed source-code constant
`REGISTERED_SYNTHETIC_POLICY_SHA256` in
`tools/validation/ph08_gateq_exact_captured.py`. Unlike the legacy
`judge_tr(config,hash,...)`, `judge_tr_registered(case,...)` does
not accept a replacement digest from the caller; it reads and validates the
separate policy file. Negative tests deliberately modify the JSON *and*
calculate a new digest and still require fail-closed. The legacy direct
interface must **not** be used as proof of independent preregistration.
Changing the policy **and** the code's trusted pin together would represent a
new reviewed version and cannot retrospectively validate an old evaluation.

**Not an original HMM preregistration.** Stage 4/5B outcomes and the failing
rates existed *before* this policy file and GitHub checkpoint. A policy first
introduced in this PR cannot be presented as preregistered for those
historical outcomes; the synthetic policy explicitly rejects original case
IDs. Future original evidence requires a separate versioned, dataset-ID-specific
policy, SHA-256 and **GitHub Verified commit SHA / verified creation
timestamp pinned externally before** the first prospective evaluation,
reviewed independent input/output lineage, and new results without editing old
claims. A GitHub PR, a matching hash, and self-declared certification flags are
not a prior independent attestation. The first review of this prospective
synthetic policy does not constitute approval of numeric T/R crossover.

The `judge_tr` output separates `rate_check_status`,
`arithmetic_check_status`, `capture_provenance_status` and
`certification_status` from its legacy `status` for compatibility.
`status=PASS` in T **only** means the two tested rate reductions passed;
`certification_status=NOT VERIFIED` and `physical_gate_q=FAIL`
remain. For R, caller-supplied booleans never produce certification.
`A,D` perturbation, input potential/PDE/coordinate error, solver forward
bounds and physical acceptance remain independent and **NOT CERTIFIED**.

The exact rational reference is restricted to at most
`MAX_CAPTURE_ROWS=128` samples to limit resource amplification (a tool
safety limit, **not** a physical convergence threshold). For the fixed
captured `A,D`, `T=D(AᵀA)⁻¹Aᵀ` is computed exactly. Constant
potential shift produces `T1=0` only when the captured basis supports a
constant mode and `D` annihilates it; tests also compare exact gradients
before/after an exactly representable uniform binary64 shift. Tests ensure
nonuniform input changes gradient, and distinguish `E=-∇φ` from the
captured `D` evaluator of `∇φ`. The 120-digit mpmath **QR**
overdetermined solve is a second algorithmic crosscheck; high-precision
agreement is not a formal high-precision error certificate.

The normal-equation backward residual refers only to the system
`(AᵀA)c=Aᵀy` evaluated on exact **captured** binary64 values:
it is **not** an original least-squares backward error or a
floating-normal-equation stability result. The apparent numerical accuracy of
an exact input-conditional rational solve is neither a bound on input capture
errors nor a physical truncation bound.

**CI:** the optional workflow now includes the exact module, exact tests,
policy JSON and this document in `pull_request.paths`; it writes test logs,
dependency versions, actual `git rev-parse HEAD`, GitHub PR head SHA,
source/test SHA-256, policy digest and immutable physical-blocker status to
synthetic-only artifacts. The source SHA, Actions run and exact test totals
must be checked after the latest commit (previous 85/85 run was on old HEAD).
No private original archive/geometry is to be included.

**Stage 5B:** file integrity is only an integrity observation. Physical
solver replay, independent convergence and whole-space field acceptance are
NOT RUN/NOT VERIFIED. Historical Stage 4 Gate Q FAIL remains immutable.
