# D_034 — Exact Negative Shallow 4:3 Non-45° Via-B SOURCE Contact Qualification

**Status:** CODE ACCEPTED; exact protected code PR, final-head Codex, Verified Squash and independent post-code-main Push confirmed. Separate docs-only protected PR, docs-main Push and Issue #445 closeout remain **PENDING** until proven. **Evidence / first checked: 2026-11-05 KST.**

## Frozen PADS SOURCE geometry and bounded contract

- PADS V9.4 MILS, one Source tick = **254 nm**; `NET_A`, L4.
- Main `(15000,25000) → (35000,25000)`; Via-B `(35000,25000)`; Branch `(35000,25000) → (47000,16000)`, reverse SOURCE record also accepted.
- Canonical Via-B→exterior vector `(+12000,-9000)`; signed `i128` exact `4*abs(cross)==3*abs(dot)`, cross and dot both nonzero, **canonical cross < 0**, strict exterior projection > main-length²; **zero float/epsilon/one-tick tolerance**.
- Legacy D_028 **positive** shallow 4:3 is preserved with canonical cross > 0 and rejects D_034; D_032 negative steep 3:4 remains distinct, as do Via-A, 45°, collinear, orthogonal.
- [Public synthetic fixture](../../crates/boardsi-import/tests/fixtures/d034_non45_oblique_mirrored_via_b_endpoint_v94.asc); opt-in `--d034-non45-oblique-mirrored-via-b-endpoint-contact-v1 NET_A 1 4 0 1`.
- Wire `BOARDSI_D034_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_V1`; candidate edge `SOURCE_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_CANDIDATE`.
- Exact same-layer SOURCE topology: **8 source nodes / 6 same-layer candidate edges / 3 local components / 0 proven electrical cross-layer edges**.
- Receipt explicitly includes `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`.

## Code PR #446 — verified acceptance

- [Tracking Issue #445](https://github.com/july-rust/BoardSI/issues/445); [code PR #446](https://github.com/july-rust/BoardSI/pull/446).
- Final exact PR source HEAD [`6f97dda85428cb26b0f75beb58685c676dbafb1d`](https://github.com/july-rust/BoardSI/commit/6f97dda85428cb26b0f75beb58685c676dbafb1d/checks): **34/34 pull_request Check Runs SUCCESS**, **11/11 pull_request Workflows SUCCESS**.
- [Numerical workflow #41941929339](https://github.com/july-rust/BoardSI/actions/runs/41941929339): numerical, Windows x64, Windows ARM64 and macOS ARM64 native Rust jobs SUCCESS; Rustfmt/Clippy gate and existing tests, new D_034 3 Rust test functions (source-only, reversal/SHA binding, hostile and cross-mode rejection). Exact per-test Debug/Release total is not separately asserted here without job-log enumeration.
- [Qt workflow #41941929384](https://github.com/july-rust/BoardSI/actions/runs/41941929384): Linux Qt 6.8.3 Offscreen plus actual native Qt Windows x64/Windows ARM64/macOS ARM64 jobs SUCCESS. Dedicated D_034 runner independently reimports PADS SOURCE and reconstructs SHA, TRACE and SOURCE geometry; rejects crafted counterfeit SHA+track, one-tick shifts, signed opposite slope, steep 3:4, malformed SOURCE, stale receipts, and electrical-proof escalation.
- Independent final-HEAD Codex `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; review-thread unresolved count = **0**.
- Historical failures are **preserved**: first candidate `55a016f0...` genuine Rustfmt failure; `a20c20c...` genuine Clippy `-D warnings` dead code in test; `3e668668...` genuine native Qt include-header name mismatch. Fixed before final HEAD `6f97dda...`; any superseded/cancelled runs remain historical, not retroactively PASS.

## Protected main and independent code-main Push — PASS

- [Verified protected one-parent Squash `main@6a72812d3070df068dd6f987503c90fd6505bd20`](https://github.com/july-rust/BoardSI/commit/6a72812d3070df068dd6f987503c90fd6505bd20): sole parent `3e9a0e24a44867c9f076f993c61a5bf171fcf2dc`; GitHub `verification.verified=true`, `reason=valid`.
- Independently filtered exact `head_sha=6a72812d3070df068dd6f987503c90fd6505bd20` and `event=push`: **8/8 complete Workflows SUCCESS**, **28/28 complete Check Runs SUCCESS**. [Qt main Push run #41943601848](https://github.com/july-rust/BoardSI/actions/runs/41943601848), [numerical main Push #41943603125](https://github.com/july-rust/BoardSI/actions/runs/41943603125).
- All this post-code-main verification completed before beginning the separate docs-only branch/PR, avoiding needless native CI duplication.

## Separate docs-only PR and administrative closeout — evidence to follow

- Only two Markdown files changed: this evidence ledger and the [canonical development checklist](../development-checklist.md). No code, workflow, or protection/ruleset modification.
- Existing **docs-only CI classifier** must skip native compilation while maintaining required PR checks; classify intentional SKIPPED separately from SUCCESS.
- Before docs completion: exact final docs PR HEAD mandatory status/check-runs and independent Codex completed/no-findings/zero unresolved, protected Verified single-parent Squash, independent exact docs-main `event=push` evidence. Record final docs PR/SHA/CI and Issue #445 administrative closeout **in Issue comment**, not recursive docs PR.
- Parent Issue #12 stays OPEN; legacy PR #35 remains CLOSED/UNMERGED.

## Scope exclusions — NOT_VERIFIED

Actual manufactured copper, plated Via barrel electrical continuity, entire PCB Open/Short, P009/PH-08 return-current proof, RLGC/S parameter solver fidelity, actual Scenario/Cache, formal U/T and product-release acceptance are **NOT_VERIFIED**. `SOURCE_GEOMETRY_ONLY` is not electrical proof.
