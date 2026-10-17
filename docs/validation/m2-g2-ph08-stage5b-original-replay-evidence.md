# PH-08 Stage 5B archive preflight and original-replay boundary — 2026-10-17 KST

**Scope:** GitHub versioned public audit implementation and synthetic regression.
**Physical result:** historical Gate Q **FAIL** reproduced by a new, isolated private Stage 5B original HMM replay. The public CI preflight never executes private source. AC-P1 **PENDING**, PH-08 contract **NOT FROZEN**, Rust FEM **NO-GO**, Gate M **NOT VERIFIED**, whole-domain/floating bound **BOUND_NOT_CERTIFIED**.

## Archived provenance established in prior private inspection

An earlier independently recorded inspection of approved private `boardsi-ph08-stage5b-evidence.zip` reported:
- External SHA-256 `f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36` (2,641,874 bytes; 42 ZIP members).
- `stage5b/SHA256SUMS`: 36/36 digest matches, 13 source snapshots covered.
- Five external-SHA-covered but **not internally SHA256SUMS-listed** files: `README-ko.md`, three under `inputs/` (including the Stage 4 nested ZIP), and one under `stage4/`.
- Nested `inputs/boardsi-ph08-stage4-qualification.zip` SHA-256 `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`, 338/338 internal checksums and 20/20 source-snapshot manifest records.
- Archived Stage 5B qualification preserves the previously failed Gate Q. **No original Stage 5B source execution is established by this integrity inspection.**

The prior record is in [PR #60](https://github.com/july-rust/BoardSI/pull/60). The original private archive was separately re-accessed and independently hashed in the follow-up below. Original source and data have **not** been published.

## This PR's new public-only tool

`tools/validation/ph08_stage5b_preflight.py` offers pinned external archive hashing, reuse of the existing ZIP path/symlink/duplicate/decompression and SHA256SUMS auditor, explicit source-snapshot coverage, recognition of the five unlisted external-only members, safe duplicate-key-resistant structural JSON parsing, and independent nested Stage 4 archive auditor delegation in a temporary private directory. No archived source is imported, executed, unpacked into an import path or echoed.

It distinguishes:
- `PASS_INTEGRITY_ONLY_NO_REPLAY`: both archives' hashes and recognized inner digests match; this is **not** HMM replay or physics acceptance.
- `FAIL_CLOSED`: archive unavailable, corrupted, hostile, unexpected layout or missing independent lineage.
- `original_stage5b_replay=NOT_RUN`, `stage5b_qualification_schema=STRUCTURAL_JSON_ONLY_NOT_VERIFIED`, `source_snapshot_coverage=CHECKSUM_COVERED_NOT_SOURCE_ATTESTED`: **never** infer semantic data or physical certification from a SHA match.

The module intentionally does **not** claim to know the executable Stage 5B solver entrypoint or its result schema, which have not been source-audited from raw private archive in this GitHub-only step. Its overrides for synthetic tests do not count as a historical external digest. Default CLI uses the pinned original SHA only. If the documented unlisted-member path layout differs in the actual archive, the tool fails closed pending explicit private source inspection and reviewed change; it must not loosen its checks silently.

## CI / safety boundary

The optional `m2-ph08-stage5b-preflight.yml` workflow runs public synthetic positive and adversarial tests; it uploads only synthetic logs, source digests and GitHub SHA provenance. No Stage 5B private archive exists in public Actions. Required 17 Strict CI contexts are unchanged.

Further original HMM **acceptance**, distinct from successfully reproducing its historical FAIL, still requires:
1. Extend the now source-reviewed Stage 5B qualification entrypoint to independently audited all-cell Recovery A/y/D capture without publishing private source.
2. Bind original input→HMM solve→Recovery A/y/D→observed gradients→saved qualification data through source- and dataset-SHA attestations. Never accept user-supplied reduction rates based on self-asserted dataset IDs.
3. Execute isolated private Stage 5B source only after explicit entrypoint review, with caps on time, CPU/RAM and output; redact all original stdout/stderr and matrix/geometry data from public evidence.
4. Independently compare original Stage 5B saved measurements, rounding, per-cell/multiple-grid data and 120-digit QR/exact captured algebra, keeping input/assembly errors distinct from ex-post exact arithmetic.
5. Freeze a genuinely prospective policy **before** unseen original qualification data; historical Stage 4 and archived Stage 5B FAIL cannot be retroactively made PASS.

**No PH-08 production Rust FEM modifications** are authorized by the integrity helper, synthetic CI, or a successful ZIP hash.

## New independent original Stage 5B private replay — 2026-10-17 KST

An approved private Library ZIP was independently accessed again, with **actual raw bytes** hashed: external Stage 5B SHA-256 matched `f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36`. The embedded Stage 4 ZIP separately matched `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`, and the archived duplicate Stage 4 qualification bytes matched the nested original. All 13 Stage 5B source digests were rechecked against its original preregistration inventory.

After source/import/entrypoint review, the exact Stage 5B `m2_ph08_stage5b_qualification.py` original was re-executed in a temporary **PRIVATE** subprocess with isolated Python invocation, CPU cap 120 s, virtual-address-space limit 2 GiB, bounded output size, one BLAS thread, and automatic source destruction at exit. Executing host: Python 3.13.5, NumPy 2.3.5, SciPy 1.17.0; archived execution had Python 3.12.14 with the same NumPy/SciPy versions. No original source, raw inputs, output arrays, private paths or qualifications were uploaded to public GitHub.

| Check | Private replay result |
| --- | --- |
| Original source snapshot | 13/13 preregistered SHA-256 digests MATCH |
| Stage 5B original MMS rows | 12/12 regenerated |
| Recorded Stage 5B Gate Q / rerun Gate Q | FAIL / **FAIL** |
| Original process exit code | 1, matching documented historical failure exit |
| New smooth/interface MMS | PASS, 0 new MMS failures |
| Affine failures | 0 |
| Negative mutation controls | 3/3 rejection confirmed |
| Historical Stage 4 failures | Retained unchanged |
| Deterministic numerical scalar comparisons | **1761/1761 equal** (maximum observed absolute difference 0) |
| Non-deterministic wall-clock scalar fields | 12 `linear.seconds` entries intentionally excluded from numerical reproducibility comparison |
| Physics acceptance | **NOT CERTIFIED** |

The first full-JSON comparison distinguished the 12 naturally varying timing fields from physics quantities; rerunning the original with timing fields explicitly excluded produced **zero mismatches** across the remaining numerical and structural fields. This is **REPRODUCED_HISTORICAL_GATE_Q_FAIL**, not Gate Q PASS. The status of new T/R registration, certified floating/error propagation, Neumann/Dirichlet finite/open-domain physics, PH-08 Contract, Rust FEM and Gate M is unchanged.

The newly introduced public `ph08_stage5b_preflight.py` remains **integrity-only** and will continue to emit `original_stage5b_replay=NOT_RUN` because that specific executable performs no private HMM run. The separate private original replay above must not be misattributed to GitHub CI.
