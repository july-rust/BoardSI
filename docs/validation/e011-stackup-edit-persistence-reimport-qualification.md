# E_011 — Main Qt Canonical Stackup Editing, Persistence, ReImport Invalidation & Fail-Closed Qualification

> Work/evidence date: **2026-11-12 KST**. Scope: one publicly reproducible synthetic PADS V9.4 4-Copper-layer Through Via source, **Source-bound Qt/Rust software qualification only**. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**. This report does **not** accept real vendor PCB stackup, manufactured connectivity, P009 physical Return Path, validated loss/roughness, physical Scenario/Solver Cache, RLGC/S or formal U09/U10/U39 and T02/T04/T24/T52.

## Starting evidence and actual change

- Starting protected `main@b245d6044e5421799b13fe32575b91dcbd8f1178`; E_010 source-only Qt Canonical Import/IPC/Picking/ReImport qualified in [E_010](e010-main-qt-import-picking-qualification.md).
- Existing `B001StackupEditor` supports source SHA-bound JSON v1/v2 user overlays, explicit Assumed/UNKNOWN material, `boardsi-stackup-config` Rust audit and session-scoped audit revision. Previously, real main `PreviewWindow` status handler initialized the Stackup editor only on legacy `PARTIAL` success; accepted `CANONICAL SOURCE GEOMETRY` silently skipped Stackup initialization.
- [PR #597](https://github.com/july-rust/BoardSI/pull/597) adds a distinct **Open Canonical PCB** action, routes actual accepted Canonical SHA/path/Layer count into the same editor, and clears source-scoped Stackup/Audit state on RUNNING/FAILED/CANCELLED or unsupported status. It keeps legacy `Open PCB` and old B_001 behavior intact. Canonical IPC conveys no Source-qualified Copper/Net Reference inventory: labels/artwork cannot become verified Return Path.
- `resetForImport(path, layerCount, acceptedSourceDigest)` rejects a disk SHA mismatch with the accepted importer receipt. It also clears audit frequency on every reImport rather than carrying the old board's setting. Manual local Source assumptions are never promoted to imported Source facts.
- Actual main executable test `--e011-stackup-selftest` calls `PreviewWindow::openCanonicalCandidate`, the real Canonical Rust importer and material-validator Rust subprocess on synthetic `pads_via_round_through_v94.asc` bytes, including distinct Source A → B → A in one controller. It writes a small `qt-e011-stackup-results.json` receipt.

## Bounded negative/positive test matrix

| Probe | Actual expected assertion | Acceptance |
| :--- | :--- | :--- |
| Source A Canonical import | 4 Layer rows; kind UNKNOWN, Dk/Df blank, no implicit FR-4 | Source facts only |
| Incomplete user overlay | Atomic JSON save/load allowed; physical RUN remains BLOCKED | Local persistence only |
| Explicit user Stackup edits | Copper conductivity, Core Dk/Df, thickness, Source document, Reference hint | Assumed property; not Source-measured |
| Rust material audit | Valid explicit inputs produce scoped SHA revision; Reference and SI remain unsupported | UI audit only |
| Edit, save, load, re-audit | Previous success is stale on edit/load; re-audit is required, original semantic revision can be reproduced | No Solver cache |
| Malformed JSON, negative Df | Reject corrupted file transactionally; invalid material blocked | Fail-closed |
| Source A → B (same Layer count) | Old overlay SHA mismatch rejects load, Reference/Material/Audit cleared | No cross-board bleed |
| Source B → A | Explicit reload and new Rust audit restore approved user assumptions only | Requalification required |
| Mutated Source A on disk | Old session receipt, save, load and Rust audit rejected | SHA pinned |
| Forged accepted SHA and PADS V9.3 header | Stackup cleared; unsupported Source import rejected | Fail-closed |
| Existing B_001/B_002/E_010 regressions | Existing Qt and Rust tests retained | Non-regression in current CI |

The test uses local copies of the repository synthetic fixture. It is not an independent Vendor Export, not measured Material, and not a proof of electrical connectivity. The Canonical subset is limited to supported 4-layer Through Via geometry; arbitrary material/layer mapping must not be inferred from this result. Asynchronous Worker cancellation and physical result-cache semantics remain separately scoped.

## Exact code PR and CI evidence

| Gate | Evidence / current decision |
| :--- | :--- |
| Source implementation | [PR #597](https://github.com/july-rust/BoardSI/pull/597), exact final HEAD `f1ae06e580d38ac9ee01f51926b024566542af6d` |
| Final HEAD Actions | **35/35 checks SUCCESS** (includes rerun outcome described below) |
| Qt execution | Linux Qt 6.8.3 offscreen, Windows x64, Windows ARM64, macOS ARM64: **SUCCESS** on exact PR HEAD, including actual main-Qt E_011 test |
| Independent Codex | Exact `f1ae06e580`: **Didn't find any major issues**; 0 unresolved review threads |
| Protected merge | [Verified single-parent Squash `da880d29a4ada85f7f1da77b744a3700b64c1e48`](https://github.com/july-rust/BoardSI/commit/da880d29a4ada85f7f1da77b744a3700b64c1e48), `verification.verified=true`, `reason=valid` |
| Exact merged-code main Push | **PASS** — exact merged-code `da880d29a4ada85f7f1da77b744a3700b64c1e48`: **8/8 event=push workflows SUCCESS, 28/28 check-runs SUCCESS**; Qt Linux/Windows x64/Windows ARM64/macOS ARM64 jobs SUCCESS; [exact code-main Push checks](https://github.com/july-rust/BoardSI/commit/da880d29a4ada85f7f1da77b744a3700b64c1e48/checks). Separately reported dynamic security-trigger workflow succeeded but is not counted as event=push. These outcomes are independent of the PR HEAD CI |
| Documentation closeout | [PR #598](https://github.com/july-rust/BoardSI/pull/598) docs-only closeout: this report, [single checklist](../development-checklist.md), [E closure plan](../e-stage-closure-plan.md) and validation index; final exact-HEAD docs CI/independent review, Verified Squash merge and exact docs-main Push to be verified before final declaration |

**Historical FAIL retained:** An earlier PR-head `M2 isolated lossless FEM prototype / rust-fem` attempt failed an existing, unrelated `different_source_bytes_cannot_reuse_sha` test (`crates/boardsi-import/tests/d006_via_three_area_cli.rs`, `D006_FILE`). A retry of that same job and same HEAD succeeded. The previous FAIL is not relabeled PASS, and no root-cause fix for that intermittent D006 condition is claimed.

## Product boundary and closeout contract

- `canonical_stackup=ASSUMED` and scoped Rust Audit Revision are not Source-certified material or Solver input acceptance.
- `reference_continuity=UNSUPPORTED_NO_COPPER_NET_EVIDENCE`, `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, and unavailable physical cache/return-path claims remain unchanged.
- Entire [Issue #12](https://github.com/july-rust/BoardSI/issues/12), PH-08 Gate Q, E_012 hardware GPU and unrelated #46 remain outside E_011.
- **Code gate is verified:** exact merged-code main Push 8/8 workflows and 28/28 checks SUCCESS. **Final bounded COMPLETE requires** [docs-only PR #598](https://github.com/july-rust/BoardSI/pull/598) exact final-head required CI, same-head independent review with 0 unresolved threads, protected Verified Squash, and exact docs-main Push. The checklist's COMPLETE entry is effective only once these final gates are independently observed. This draft must not be used to claim already-merged docs.
