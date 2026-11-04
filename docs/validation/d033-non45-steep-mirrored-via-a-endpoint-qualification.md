# D_033 — Exact Mirrored Negative Steep 3:4 Non-45° Via-A SOURCE Contact Qualification

**Status:** D_033 bounded SOURCE-geometry code qualification **ACCEPTED** after exact PR checks, independent Codex review, protected Verified one-parent code Squash, and exact code-main Push CI. Documentation-only closeout and Issue #442 administrative closure remain **PENDING** until their separate checks complete. **Date of initial bounded code evidence: 2026-11-05 KST.**

## Frozen SOURCE geometry and mathematical proof scope

- PADS V9.4 MILS, one exact Source Tick = **254 nm**; Net `NET_A`; Layer L4.
- Main Trace: `(15000,25000)→(35000,25000)`. Via-A: `(15000,25000)`.
- Distinct Branch: `(15000,25000)→(6000,13000)`; Via-A-to-exterior vector **(-9000,-12000)**. Exact reversed Branch SOURCE record accepted.
- The signed SOURCE-domain `i128` test requires `3*abs(cross)==4*abs(dot)`, both nonzero, strict **negative canonical cross** from shared Via-A toward exterior, and strictly negative external projection relative to Via-A. Reverse record order cannot change the canonical orientation.
- Existing D_031 mode now explicitly requires positive canonical cross; D_033 negative case is rejected by D_031, while original positive [D_031](d031-non45-steep-via-a-endpoint-qualification.md) source remains accepted there. The D_030 positive Via-B and D_032 mirrored Via-B candidates remain separate. No floating-point/epsilon/one-tick geometry tolerance.

**Frozen public fixture:** [`d033_non45_steep_mirrored_via_a_endpoint_v94.asc`](../../crates/boardsi-import/tests/fixtures/d033_non45_steep_mirrored_via_a_endpoint_v94.asc).

**Explicit opt-in:** `--d033-non45-steep-mirrored-via-a-endpoint-contact-v1 NET_A 1 4 0 1`.

**Dedicated wire mode:** `BOARDSI_D033_NON45_STEEP_MIRRORED_VIA_A_ENDPOINT_CONTACT_V1`; candidate kind `SOURCE_NON45_STEEP_MIRRORED_VIA_A_ENDPOINT_CONTACT_CANDIDATE`.

**Qualified SOURCE topology:** **8 nodes, 6 same-layer candidate edges, 3 local components, 0 proven electrical cross-layer edges**; `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`.

## Exact code PR evidence — PASS

- Tracking [Issue #442](https://github.com/july-rust/BoardSI/issues/442); [code PR #443](https://github.com/july-rust/BoardSI/pull/443).
- **Final PR HEAD [`39afdf94cb40cf8380b4d8d9be27ee0096093b8f`](https://github.com/july-rust/BoardSI/commit/39afdf94cb40cf8380b4d8d9be27ee0096093b8f/checks)**: **34/34 pull_request Check Runs SUCCESS, 11/11 pull_request Workflows SUCCESS**, with unchanged protected mandatory checks.
- [Numerical Core PR #41933784234](https://github.com/july-rust/BoardSI/actions/runs/41933784234): Rust 1.75 fmt/Clippy and tests passed; D_033 exact mode/hostile regression **Debug 3/3 and Release 3/3 PASS**; native Windows x64/ARM64 and macOS ARM64 native Rust jobs SUCCESS.
- [Qt Runtime PR #41933784378](https://github.com/july-rust/BoardSI/actions/runs/41933784378): Linux Qt 6.8.3 Offscreen **55/55 CTest PASS**, dedicated `D033_NATIVE_QT_NON45_STEEP_MIRRORED_VIA_A_SOURCE true ... NO_ELECTRICAL_PROOF`; actual D_033 native Qt steps on Windows x64, Windows ARM64, macOS ARM64 all **SUCCESS**; full Qt workflow SUCCESS.
- Rust and Qt independently validate the imported SOURCE, SHA-bound geometry inventory, graph construction, reversed Branch records, and negative cases (one-tick gap/offset/slope, positive D_031 mirror, Via-B D_030/D_032 modes, 4:3, 45°, collinear, orthogonal, malformed SOURCE, wrong Via/Net/Layer, extra/missing objects, forged/stale receipts and electrical-proof escalation). Qt separately reimports hostile SOURCE with matching counterfeit SHA and TRACE data, testing the actual geometry predicate.
- Independent Codex review at the **same final HEAD**: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; zero unresolved review threads.

### Historical failed/superseded evidence — retained

- The **first code HEAD `69dae644a68055cf2c9b365e0a8875a3e2334fe4`** produced a real `cargo fmt --all -- --check` FAIL in [Numerical Core #41933139544](https://github.com/july-rust/BoardSI/actions/runs/41933139544). Fixed in a later commit and rerun on the final HEAD. Superseded runs/jobs cancelled by concurrency are not retroactively called successful.
- Independent Codex at interim HEAD `6affb1bce710095ecdc49f4ca031c5471513f4c8` reported a **P2**: the Qt forged TRACE did not match certain hostile SOURCE coordinates, making a geometry predicate test non-probative. This was corrected by exact matching SOURCE/TRACE and regression checks on final HEAD `39afdf94...`. Original [review discussion](https://github.com/july-rust/BoardSI/pull/443#discussion_r4474628173) retains the finding and corrective reply; thread resolved. Later independent final-HEAD Codex no findings.

## Protected code-main merge and independent Push — PASS

- Code PR #443 merged as GitHub **Verified one-parent Squash [`main@91121f399b1d8b177a2c9ddaef92bd945488ec89`](https://github.com/july-rust/BoardSI/commit/91121f399b1d8b177a2c9ddaef92bd945488ec89)**; sole parent `1e465b76df4493b7a58d88c3e62a6dc3e280b058`; `verification.verified=true`, `reason=valid`.
- Independently filtered exact `head_sha=91121f399b1d8b177a2c9ddaef92bd945488ec89`, `event=push`: **8/8 Workflows SUCCESS and 28/28 Check Runs SUCCESS**, all completed. This post-code-main gate was completed **before** the separate documentation PR began, to avoid concurrency cancellation.
- These are narrow SOURCE geometry results; no electrical or product integration gate is silently upgraded.

## Separate documentation PR and administrative closeout — awaiting final evidence

- This ledger and the canonical [`docs/development-checklist.md`](../development-checklist.md) are the **only two Markdown files** permitted for the closeout PR. Existing docs-only CI optimization must skip native compilation while preserving all mandatory checks.
- Before claiming documentation complete: require exact docs PR HEAD CI, independent same-HEAD Codex no findings/zero unresolved threads, protected Verified single-parent docs-main Squash, and independent completed exact `event=push` docs-main CI (record successes and intentional skips separately).
- Record verified final document PR/SHA/CI and close [Issue #442](https://github.com/july-rust/BoardSI/issues/442) **only after** the final gates pass, in its administrative evidence comment **without another recursive documentation PR**.

## Explicit exclusions

**NOT_VERIFIED / NOT_RUN:** fabricated Copper or plated Via Barrel electrical continuity, whole-board Open/Short, P009/PH-08 physical return path, RLGC or S-parameter solver fidelity, physical Scenario/Cache, formal M1/M2 U/T and product release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN; legacy [PR #35](https://github.com/july-rust/BoardSI/pull/35) stays CLOSED/UNMERGED. A SOURCE contact candidate is not physical conductor or SI solver proof.
