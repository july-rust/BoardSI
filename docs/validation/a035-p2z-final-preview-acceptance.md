# A_035 / P2-Z — Developer Preview scoped acceptance and A-series exit gate

**Tracking:** [Issue #207](https://github.com/july-rust/BoardSI/issues/207)  
**Audit baseline (revalidate at integration):** protected `main@6208db7083f4c56f91810e79d74ae22074221ecd` (2026-10-26 KST)  
**Change intent:** evidence reconciliation and accurate Developer Preview product status; no PADS grammar, geometry contract, Qt renderer or physical solver change.

## 1. Existing executable evidence, not new A_035 runtime test results

The audit directly read the **exact baseline SHA** check-runs through the GitHub check-runs API: 21/21 completed SUCCESS. [Commit checks](https://github.com/july-rust/BoardSI/commit/6208db7083f4c56f91810e79d74ae22074221ecd/checks). GitHub protected ruleset [25046595](https://github.com/july-rust/BoardSI/rules/25046595) is active, strict and signed/squash-only, with exactly 17 required contexts (all SUCCESS at baseline); four CodeQL jobs also passed, including `Analyze (python)`. These are **historical executed baseline checks**, not a new physical-board/hardware certification and not evidence that a security alert is fixed.

Native Qt runtime [run 40292003660](https://github.com/july-rust/BoardSI/actions/runs/40292003660), **source SHA 6208db7**:

| Platform | Native Qt job | Result and observed evidence |
| --- | --- | --- |
| Windows x64 | [job 121472238888](https://github.com/july-rust/BoardSI/actions/runs/40292003660/job/121472238888) | SUCCESS; production Rust/QProcess/Qt native source fixture checks, A_034 original local-contact/cross-file receipts, dedicated Qt Quick RHI tests, A_019 interaction and native performance regression, supported Copper regression |
| Windows ARM64 | [job 121472241257](https://github.com/july-rust/BoardSI/actions/runs/40292003660/job/121472241257) | SUCCESS; corresponding native source/import/Quick RHI and Copper tests |
| macOS ARM64 | [job 121472295200](https://github.com/july-rust/BoardSI/actions/runs/40292003660/job/121472295200) | SUCCESS; source/import/RHI frame and picking tests, synthetic A_019 data, supported Copper regression |

**Important distinction:** Several intentionally raster/CPU-only test modes report `a034_local_contact_rhi_frame_scope=NOT_RUN_NO_ACTIVE_RHI`. This is correct for those modes, **not** a global RHI failure or a GPU PASS. Separately, the Qt Quick active RHI test requires `a034_local_contact_cpu_rhi_filter_verified=true`, `a034_local_contact_rhi_pick_scope=PASS`, `a034_local_contact_rhi_frame_scope=PASS` and `a034_cross_file_reimport_scope=PASS` on the actual source-bound frame. Native job logs confirm the checks were executed and passed.

## 2. A_001–A_034 handoff scope matrix

| Function | A-series scoped acceptance | What is **not** proven |
| --- | --- | --- |
| PADS V9.4 MILS ASCII subset | **PASS (restricted public synthetic fixtures)**: declared Trace/Arc/Via/Pad, Board Outline, bounded source-owned Copper/Cutout variants, transforms and source provenance | Arbitrary vendor-generated PADS export/other revisions; full manufacturing-feature and netlist/pin connectivity |
| Rust Importer → Canonical → versioned IPC → Qt Artwork | **PASS (declared subset)**: supported synthetic source geometry, local coordinates, reject/recovery and source ID assertions | Whole-board physical copper qualification, arbitrary input and all T01–T54 acceptance tests |
| Consecutive PCB source re-import | **PASS (native fixture regression)**: route → mirrored pads → route, removal of stale object, Net/layer/contact state | All conceivable PCB combinations and recovery conditions |
| CPU/RHI filter, picking and visible Zoom/Fit | **PASS (native CI scope)**: source local-contact marker and revision-bound presented frame in supported RHI tests | Every end-user GPU/driver, arbitrary device loss, physical precision beyond source contract |
| Limited local contact evidence | **PASS (source-backed local endpoints only)** | Full-board Open/Short, fabricated net continuity, Cu Pour/Thermal connectivity, physical SI |
| 10K/100K/1M synthetic render/interaction | **PASS (native synthetic regression coverage)**: sample generation/interaction, exact picking, repeated tab/fallback; CI Metal/D3D backend receipt as available | **GPU VRAM NOT_MEASURED; guaranteed 1M @ 60 FPS NOT_VERIFIED; independent end-user driver-reset test NOT_RUN** |
| Product Solver | **NOT_RUN / NOT_VERIFIED** for physical PCB RLGC, S, DC/30 GHz and real-board accuracy | Old restricted synthetic M2 FEM coupons do not establish production Solver accuracy |
| Public distributable Developer Preview | **NOT_VERIFIED** | Third-party/license compliance, public installer, clean target PC install acceptance |

Source-ledger: [A_034 integration audit](a034-p2y-integration-stabilization.md), [A_033 source-artwork parity](a033-p2x-source-artwork-parity.md), [A_032 local connectivity](a032-p2w-local-connectivity.md); all limited by their own fixture/model contracts.

## 3. Security and outstanding product gates

- [SEC_001 PR #206](https://github.com/july-rust/BoardSI/pull/206) is merged to protected main as `6208db7083f4c56f91810e79d74ae22074221ecd` and the *latest exact main* `Analyze (python)` job succeeded.
- The live [Code Scanning Alert #1](https://github.com/july-rust/BoardSI/security/code-scanning/1) state cannot be read by the exposed GitHub connector API, **therefore actual Fixed/Dismissed/Open status is NOT_VERIFIED**. Do not close [SEC_001 Issue #205](https://github.com/july-rust/BoardSI/issues/205) without direct UI/API evidence. Successful CodeQL execution alone does not certify alert closure.
- [M1 PADS importer #9](https://github.com/july-rust/BoardSI/issues/9), [M1 artwork viewer #11](https://github.com/july-rust/BoardSI/issues/11), [GPU qualification #165](https://github.com/july-rust/BoardSI/issues/165) stay open. Original M0 legal/platform external gates, PH-08 Gate Q FAIL and product Solver/real-board gates are independent and remain unresolved where recorded.
- Baseline `README.md` claimed only M0/M2 prototypes, insufficiently acknowledging the newly integrated **restricted** Developer Preview Import/Viewer. Update product-facing status precisely without representing full PADS/physical SI acceptance.

## 4. B_001 handoff (proposal, not execution)

**B_001: Stackup & Reference Configuration vertical slice**. Reuse the **same** Canonical Board Model and production Qt Widgets/Rust path instead of a separate mockup. Begin with usable UI fields and fail-closed source/provenance: layer ordering, copper thickness, dielectric thickness, Dk/Df, material source/unknown state and reference conductor mapping. Never silently substitute FR4 or assume missing stackup physical facts. Existing synthetic PADS geometry remains viewable even without qualified stackup; prevent physical analysis Run when mandatory inputs are absent.

Proposed B_001 tests: a public synthetic board with incomplete stackup; input/edit persistence; invalid/unknown material and layer order warnings/errors; selected reference/layer source mapping; re-import invalidation and clear; GUI responsiveness and deterministic canonical/worker boundary; native x64/ARM64/macOS gates. Scope deliberately excludes production RLGC calculations or certified ports.

## 5. A_035 closure decision policy

- **Baseline source/CI audit:** PASS, supported by above exact SHA links and logs.
- **New A_035 PR:** PENDING until all 17 required **PR final-head** contexts pass; independent *same-head* Codex review and review-thread resolution must be verifiable, signed protected squash merge and **post-merge main** checks read separately.
- **Actual Security Alert #1 state:** NOT_VERIFIED and tracked in #205 independently.
- **End-user physical board, distribution/legal, vendor PADS, guaranteed GPU perf and full SI:** NOT_VERIFIED / NOT_RUN (out of A_035 scoped acceptance).
- **A-series scoped exit:** may be marked `COMPLETE / VERIFIED (P2 Developer Preview restricted subset only)` *only* when this A_035 change itself passes all change gates; otherwise `PARTIAL / BLOCKED` with exact outstanding gate listed.

Do not label A_035 or the whole M1/M2 complete from document creation alone.