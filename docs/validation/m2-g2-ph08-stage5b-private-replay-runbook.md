# PH-08 Stage 5B original HMM replay — approved private runbook

**Status:** PUBLIC REPRODUCIBILITY TOOLING ONLY. Original archived and reexecuted Stage 5B Gate Q **FAIL**. No PH-08 physical certification, contract freeze or Rust FEM authorization.

## Why this separate runner exists

PR #62 introduced `ph08_stage5b_preflight.py` for trusted ZIP/nested Stage 4 byte **integrity only**. It cannot and must not claim HMM execution.

`tools/validation/ph08_stage5b_original_replay.py` now provides a separate, explicit original Stage 5B source execution and deterministic result comparison **in a private, approved, network-isolated POSIX environment only**. The executable source and original JSON remain in the private ZIP; GitHub public Actions receives neither.

### Trust chain

1. Stream an approved input ZIP to a size-limited private temporary copy; check the **fixed** external digest `f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36`. The mutable input path is not reopened.
2. Run the existing public Stage 5B preflight on that pinned copy. Require `PASS_INTEGRITY_ONLY_NO_REPLAY`. This includes the separately pinned embedded Stage 4 archive digest `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`, nested 338/338 checksums and 20/20 exact source snapshot coverage; outer 36/36 and 13 covered sources.
3. Re-authenticate the exact immutable in-memory archive bytes that will be extracted/executed, after the preflight's independently verified copy. If the private temporary file changed at any point, fail closed. Verify the Stage 5B `preregistration.json` source digest mapping against the **entire** Stage 5B source snapshot set, exactly 13 source files, including the vetted `m2_ph08_stage5b_qualification.py` entrypoint.
4. Extract only explicitly required SHA-authenticated source files plus immutable preregistration and Stage 4 historical qualification JSON into a private temporary directory. Independently ensure the duplicated Stage 4 qualification matches the nested original.
5. Verify Python 3.13.x, NumPy 2.3.5 and SciPy 1.17.0 under both the caller metadata contract **and the exact sanitized child import environment** (same cwd and source search path, PYTHONPATH removed and user-site disabled) before original execution. Record only sanitized actual child package versions; mismatches fail closed. Execute SHA-pinned original entrypoint once with `PYTHONPATH` removed and a single numerical BLAS thread, 80-second CPU limit, 90-second wall timeout, 2 GiB virtual-address-space limit and 2 MiB private output limit (respecting stricter inherited hard limits). Launch in its own POSIX process group, kill/reap the group after completion or timeout, and sanitize any process setup failure. These caps mitigate resources but **do not constitute aggregate cgroup quotas or sandbox arbitrary Python filesystem/network access**; use a separately approved network-disabled isolated host with per-job aggregate resource policy. No archive source is ever copied into the public repository or CI.
6. Compare saved and freshly generated qualification JSON recursively. Reject a syntactically valid but non-object JSON root before accessing status fields. Only known per-row `rows[i].linear.seconds` timing values are excluded. The 1761 deterministic numeric scalars and all remaining structural/status elements must match exactly. Require exit code 1 and original `Gate_Q=FAIL`, `certified=false`, `physical_solves=0`, `pilot=NOT RUN`. Any unexpected PASS or discrepancy is **NOT VERIFIED**, not physical approval.
7. Emit **sanitized count/status-only JSON**. An unwritable output report path must produce `SANITIZED_REPORT_WRITE_FAILED` without disclosing the private operator path. Never publish private original source, data, manifest records, raw matrix, timings, stderr/stdout, customer geometry or company path. The report is NOT an independent mathematical proof of the float/PDE/open-domain error.

### How to execute privately

Use a locally approved isolated POSIX host with Python 3.13.x, NumPy 2.3.5 and SciPy 1.17.0. Mismatched runtime versions refuse replay as NOT VERIFIED. The original Stage 5B archived computation used Python 3.12.14 with the same NumPy/SciPy versions; differences in libraries/toolchain must be recorded and a discrepancy cannot be waved away.

```bash
# This command must NEVER be placed in public GitHub Actions.
python tools/validation/ph08_stage5b_original_replay.py \
  /approved/private/boardsi-ph08-stage5b-evidence.zip \
  --sanitized-json /approved/private/sanitized-stage5b-replay.json
```

Exit `0` means **historical Gate Q FAIL reproduced**; exit `2` means cannot establish a trusted replay. It NEVER means physical accuracy PASS. Archive missing/untrusted, source drift, malformed qualification, different number of samples, unexpected float values or changed failure semantics must fail closed.

### GitHub CI limitation and next physical gates

The optional `m2-ph08-stage5b-preflight.yml` executes synthetic-only tests for archive preflight and the replay helper's strict comparator, failure statuses, stale/tampered JSON rejection and SHA pin. It **does not** execute any private HMM source. The source-execution phase is limited to private approved environments, and future sanitized evidence must name that provenance accurately.

Previously recorded private source reexecution in Issue #51 compared 12 MMS rows, 1761 deterministic scalars and 12 excluded wall-time scalars, reproducing archived `Gate_Q=FAIL`; even an exact replay is a reproducibility check, not a certified bound.

**Still unresolved:** original A/y/D input/assembly/solver and physical-gradient forward-error enclosure, globally valid T/R crossover and genuinely prospective unseen-data preregistration, Neumann/Dirichlet finite-box applicability and open-domain tails, Gate M, and finite-thickness/trapezoid reference-plane physical model. PH-08 historical Gate Q **FAIL**, AC-P1 **PENDING**, PH-08 Contract **NOT FROZEN**, Rust FEM **NO-GO**, physical bound **BOUND_NOT_CERTIFIED**. Do not change PR #35 or Qt Issue #46 in this branch.
