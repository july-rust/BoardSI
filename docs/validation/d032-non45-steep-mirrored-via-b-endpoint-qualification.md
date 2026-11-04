# D_032 — Exact Mirrored Negative Steep 3:4 Non-45° Via-B SOURCE Contact Qualification

**Status:** D_032 bounded SOURCE-geometry qualification **CLOSED / ACCEPTED** (code, documentation, exact-commit CI, protected Verified Squash, and independent main Push confirmed). Tracking [Issue #438](https://github.com/july-rust/BoardSI/issues/438) **CLOSED / completed** on 2026-11-05 KST. **Electrical and product acceptance remain NOT_VERIFIED.**

## Scope: Source geometry only
The bounded PADS V9.4 MILS fixture has exact 254 nm source ticks, Net `NET_A`, L4 Main Trace `(15000,25000)→(35000,25000)`, Via-B `(35000,25000)` and mirrored Branch `(35000,25000)→(44000,13000)`. Its canonical Via-B-to-exterior displacement is `(+9000,-12000)`, an exact negative steep 3:4 non-45° slope. The source-reversed Branch is accepted.

Rust signed i128 and independent Qt source reimport enforce exactly `3*abs(cross)==4*abs(dot)`, nonzero cross/dot, strict Via-B exterior projection beyond the main length squared, and a **negative** canonical cross computed from the shared Via toward exterior. Reversing the Branch source record must not reverse the canonical cross sign. The existing D_030 positive steep 3:4 mode rejects the mirrored negative slope; D_032 rejects D_030 positive slope. No floating-point or 1-source-tick tolerance.

- Fixture: [`d032_non45_steep_mirrored_via_b_endpoint_v94.asc`](../../crates/boardsi-import/tests/fixtures/d032_non45_steep_mirrored_via_b_endpoint_v94.asc)
- Opt-in CLI: `--d032-non45-steep-mirrored-via-b-endpoint-contact-v1 NET_A 1 4 0 1`
- Dedicated witness: `BOARDSI_D032_NON45_STEEP_MIRRORED_VIA_B_ENDPOINT_CONTACT_V1`, `SOURCE_NON45_STEEP_MIRRORED_VIA_B_ENDPOINT_CONTACT_CANDIDATE`.
- Topology: **8 SOURCE nodes, 6 same-layer candidate edges, 3 local components, 0 proven electrical cross-layer edges**.
- Evidence state: **`SOURCE_GEOMETRY_ONLY`**, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`.

## Code PR qualified evidence
- Tracking [Issue #438](https://github.com/july-rust/BoardSI/issues/438); [Code PR #439](https://github.com/july-rust/BoardSI/pull/439).
- Exact final code PR HEAD: [`4ae76653fc94eeb870b314c9cb2e93e12402b6dc`](https://github.com/july-rust/BoardSI/commit/4ae76653fc94eeb870b314c9cb2e93e12402b6dc), **34/34 pull-request Check Runs SUCCESS**, **11/11 pull-request Workflows SUCCESS**.
- Rust Numerical Core [PR run #41916995587](https://github.com/july-rust/BoardSI/actions/runs/41916995587), D_032 direct tests **Debug 3/3 PASS and Release 3/3 PASS**, including branch inversion, strict sign/mode isolation, 1-tick and hostile-source rejection.
- Qt Runtime [PR run #41916995643](https://github.com/july-rust/BoardSI/actions/runs/41916995643): Linux Qt 6.8.3 Offscreen **54/54 CTest PASS**, dedicated D_032 output `D032_NATIVE_QT_NON45_STEEP_MIRRORED_VIA_B_SOURCE true ... NO_ELECTRICAL_PROOF`; dedicated native Qt Windows x64, Windows ARM64 and macOS ARM64 steps in SUCCESS platform jobs.
- Qt directly reimports original PADS SOURCE and independently rejects geometry-hostile inputs even with adversarial SHA/TRACE-altered plausible receipts; no trust escalation from the Rust PASS result.
- Identical final HEAD independent Codex Review `status=completed / has_any_findings=false / has_unresolved_findings=false`, zero unresolved review threads.
- **Historical superseded PR HEAD failures retained**, never retconned: original `369487bd...` `cargo fmt --check` long Rust function and Qt wrapper missing `Receipt &out`; interim `a2b217a...` `cargo fmt --check` long Rust test calls. Fixed on later final HEAD and fully retested.

## Protected code-main merge
- PR #439 protected Squash code-main commit [`2afa5d3fccb4524037ca3f1ccf7e86b123d9a0e9`](https://github.com/july-rust/BoardSI/commit/2afa5d3fccb4524037ca3f1ccf7e86b123d9a0e9), **exactly one parent** `ebf039d25a7217aa87bff859ac8c0a03ed655090` and GitHub `verification.verified=true/reason=valid`.
- **Exact code-main Push independently verified PASS:** GitHub REST restricted to the same `head_sha=2afa5d3fccb4524037ca3f1ccf7e86b123d9a0e9`, `event=push`: **8/8 Workflows SUCCESS and 28/28 Check Runs SUCCESS**, all completed. [Qt runtime Push #41919079083](https://github.com/july-rust/BoardSI/actions/runs/41919079083); [Numerical core Push #41919079100](https://github.com/july-rust/BoardSI/actions/runs/41919079100).
- The exact code PR, signed Squash and post-merge Push are **distinct evidence gates** and were separately verified. No physical electrical or product acceptance is implied.

## Documentation-only closeout — independently verified PASS
- [Docs PR #440](https://github.com/july-rust/BoardSI/pull/440), final HEAD [`ed8af4c9fc268436fb303ff4e915d2f1ddfc5814`](https://github.com/july-rust/BoardSI/commit/ed8af4c9fc268436fb303ff4e915d2f1ddfc5814): only the canonical development checklist and this D_032 validation ledger changed. **8/8 PR Workflows SUCCESS**; all **29/29 PR Check Runs completed (13 SUCCESS, 16 expected docs-only SKIPPED)**. All required checks retained; native compiler matrix was not restarted solely for Markdown edits.
- Independent Codex review at the same exact PR HEAD **completed with no findings or unresolved findings**; **0 unresolved review threads**.
- Protected **Verified one-parent Squash** docs-main commit [`b9546eb59d6b1c6f7d0e01ce01ca63603ac46b5c`](https://github.com/july-rust/BoardSI/commit/b9546eb59d6b1c6f7d0e01ce01ca63603ac46b5c); sole parent `2afa5d3fccb4524037ca3f1ccf7e86b123d9a0e9`, `verification.verified=true/reason=valid`.
- **Exact docs-main Push independently verified PASS:** same head SHA, `event=push`, **8/8 Workflows SUCCESS and all 28/28 Check Runs completed (12 SUCCESS, 16 expected docs-only SKIPPED), 0 failures**. [Qt runtime Push #41921432673](https://github.com/july-rust/BoardSI/actions/runs/41921432673), [Validate planning docs Push #41921432762](https://github.com/july-rust/BoardSI/actions/runs/41921432762).
- Final independent closeout evidence is preserved in the [Issue #438 audit comment](https://github.com/july-rust/BoardSI/issues/438#issuecomment-6643306796); [Issue #438](https://github.com/july-rust/BoardSI/issues/438) was verified **CLOSED / completed**. Stage D **53 checked / 2 pending**, all stages **124 checked / 122 pending**—limited evidence bookkeeping, not a product completion percentage.

## Explicit exclusions
**NOT_VERIFIED:** real fabricated Copper or plated Via Barrel electrical continuity, whole-board Open/Short, reference return path P009/PH-08, RLGC/S solver fidelity, physical Scenario/Cache, M1/M2 formal U/T acceptance or product release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN; legacy PR #35 remains CLOSED/UNMERGED. Acceptance here is a **local SOURCE geometry candidate**, not a fabricated or simulated electrical path.
