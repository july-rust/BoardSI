# D_032 — Exact Mirrored Negative Steep 3:4 Non-45° Via-B SOURCE Contact Qualification

**Status:** CODE_PR_ACCEPTED; CODE_MAIN_PUSH_PENDING; documentation evidence under review. **2026-11-05 KST.**

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
- **CODE_MAIN_PUSH_PENDING:** code-main `event=push` CI is still running and **must be independently verified** for exact commit SHA before this evidence can be marked fully accepted. Do not infer from PR 34/34.
- No main Push status is claimed here without completed exact-SHA evidence.

## Documentation-only closeout gates
- Update `docs/development-checklist.md` and this validation ledger only, using existing Markdown-only native skip optimization, preserving required contexts.
- Separate PR exact final HEAD required Checks + independent same-HEAD Codex no findings/0 unresolved threads, protected Verified single-parent Squash, exact post-doc-main Push successful workflow and check-run auditing.
- Record final docs-main SHA and Push outcome administratively in [Issue #438](https://github.com/july-rust/BoardSI/issues/438), avoiding recursive documentation PRs. Close Issue only after all verified gates.

## Explicit exclusions
**NOT_VERIFIED:** real fabricated Copper or plated Via Barrel electrical continuity, whole-board Open/Short, reference return path P009/PH-08, RLGC/S solver fidelity, physical Scenario/Cache, M1/M2 formal U/T acceptance or product release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN; legacy PR #35 remains CLOSED/UNMERGED. Acceptance here is a **local SOURCE geometry candidate**, not a fabricated or simulated electrical path.
