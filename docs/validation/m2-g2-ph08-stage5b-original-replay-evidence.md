# PH-08 Stage 5B archive preflight and original-replay boundary — 2026-10-17 KST

**Scope:** GitHub versioned public audit implementation and synthetic regression.
**Physical result:** historical Gate Q **FAIL**; original Stage 5B PDE/HMM replay **NOT RUN** in this task. AC-P1 **PENDING**, PH-08 contract **NOT FROZEN**, Rust FEM **NO-GO**, Gate M **NOT VERIFIED**, whole-domain/floating bound **BOUND_NOT_CERTIFIED**.

## Provenance established in previously recorded private inspection (not re-accessed here)

An earlier independently recorded inspection of approved private `boardsi-ph08-stage5b-evidence.zip` reported:
- External SHA-256 `f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36` (2,641,874 bytes; 42 ZIP members).
- `stage5b/SHA256SUMS`: 36/36 digest matches, 13 source snapshots covered.
- Five external-SHA-covered but **not internally SHA256SUMS-listed** files: `README-ko.md`, three under `inputs/` (including the Stage 4 nested ZIP), and one under `stage4/`.
- Nested `inputs/boardsi-ph08-stage4-qualification.zip` SHA-256 `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`, 338/338 internal checksums and 20/20 source-snapshot manifest records.
- Archived Stage 5B qualification preserves the previously failed Gate Q. **No original Stage 5B source execution is established by this integrity inspection.**

These facts derive from the previously recorded private-byte audit in [PR #60](https://github.com/july-rust/BoardSI/pull/60), not an independent reinspection of private bytes in this PR. Original source and data have **not** been published.

## This PR's new public-only tool

`tools/validation/ph08_stage5b_preflight.py` offers pinned external archive hashing, reuse of the existing ZIP path/symlink/duplicate/decompression and SHA256SUMS auditor, explicit source-snapshot coverage, recognition of the five unlisted external-only members, safe duplicate-key-resistant structural JSON parsing, and independent nested Stage 4 archive auditor delegation in a temporary private directory. No archived source is imported, executed, unpacked into an import path or echoed.

It distinguishes:
- `PASS_INTEGRITY_ONLY_NO_REPLAY`: both archives' hashes and recognized inner digests match; this is **not** HMM replay or physics acceptance.
- `FAIL_CLOSED`: archive unavailable, corrupted, hostile, unexpected layout or missing independent lineage.
- `original_stage5b_replay=NOT_RUN`, `stage5b_qualification_schema=STRUCTURAL_JSON_ONLY_NOT_VERIFIED`, `source_snapshot_coverage=CHECKSUM_COVERED_NOT_SOURCE_ATTESTED`: **never** infer semantic data or physical certification from a SHA match.

The module intentionally does **not** claim to know the executable Stage 5B solver entrypoint or its result schema, which have not been source-audited from raw private archive in this GitHub-only step. Its overrides for synthetic tests do not count as a historical external digest. Default CLI uses the pinned original SHA only. If the documented unlisted-member path layout differs in the actual archive, the tool fails closed pending explicit private source inspection and reviewed change; it must not loosen its checks silently.

## CI / safety boundary

The optional `m2-ph08-stage5b-preflight.yml` workflow runs public synthetic positive and adversarial tests; it uploads only synthetic logs, source digests and GitHub SHA provenance. No Stage 5B private archive exists in public Actions. Required 17 Strict CI contexts are unchanged.

A future approved private original **replay** requires, in a separate reviewed step:
1. Read the SHA-pinned original Stage 5B source layout and record exact solver/qualification entrypoints and dependency versions without publishing private source.
2. Bind original input→HMM solve→Recovery A/y/D→observed gradients→saved qualification data through source- and dataset-SHA attestations. Never accept user-supplied reduction rates based on self-asserted dataset IDs.
3. Execute isolated private Stage 5B source only after explicit entrypoint review, with caps on time, CPU/RAM and output; redact all original stdout/stderr and matrix/geometry data from public evidence.
4. Independently compare original Stage 5B saved measurements, rounding, per-cell/multiple-grid data and 120-digit QR/exact captured algebra, keeping input/assembly errors distinct from ex-post exact arithmetic.
5. Freeze a genuinely prospective policy **before** unseen original qualification data; historical Stage 4 and archived Stage 5B FAIL cannot be retroactively made PASS.

**No PH-08 production Rust FEM modifications** are authorized by the integrity helper, synthetic CI, or a successful ZIP hash.
