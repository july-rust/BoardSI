# D_015 — PADS same-layer perpendicular T-Junction SOURCE contact candidate qualification

**Recorded:** 2026-11-03 KST. **Tracker:** [Issue #378](https://github.com/july-rust/BoardSI/issues/378). **Code PR:** [#379](https://github.com/july-rust/BoardSI/pull/379). **Parent M1:** [#12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN.

## Precisely bounded SOURCE-only proof

A dedicated public synthetic PADS V9.4 MILS fixture declares exactly four Copper layers, two named Through-Via placements spanning Layer 1→4, and four straight same-Net Traces. The three D_014 chain segments remain intact: Layer-1 arm A to Via A, Layer-4 centre line Via A to Via B, and Layer-1 arm B. The fourth straight Layer-4 Trace terminates perpendicularly at the strict centreline interior of the Layer-4 Via-A-to-Via-B segment. Source coordinates are 0.01-mil ticks, converted to Canonical geometry; decimal-equivalent source MILS lexical coordinates retain separate SHA.

Rust opt-in `boardsi-import-artwork <source.asc> --d015-t-junction-v1 NET_A 1 4 0 1` validates complete Source section grammar (no extra `*LINES*` Copper/COPCLS, PART, Arc or unmodeled sections), exactly one named VIA definition with two supported padstack rows, complete Net/Layer/Trace/Via inventory, source-tick geometry and Canonical consistency. It emits SHA-256-bound SOURCE candidate receipts. Qt independently reimports the actual source file, validates Source and imported Via/Trace geometry and all witness rows, refuses malformed/forged/stale evidence, and chains the native D_015 check into existing D_012→D_014 smoke. The ordinary Import wire and existing Github Actions workflow files are not changed.

The bounded candidate graph contains **8 nodes, 5 SAME-LAYER CONTACT CANDIDATE edges, 3 local disconnected components, and 0 proven electrical cross-layer edges**. The two vias are *not* connected across layers by electrical barrel edges. Source-declared plating and Net labels are not fabrication evidence.

Adversarial cases include Source replacement and SHA rebind, wrong Net/Layer/Via, one-tick misses, perpendicular crossing/overrun, parallel centreline, Via-centre interior false contacts, missing/extra objects, unsupported COPCLS, additional/duplicate VIA definitions, duplicate via pad rows, explicit INNER pad, and forged Qt receipt promotion. Decimal `80.00` MILS is accepted without changing Canonical contact geometry, while invalid sub-tick notation is rejected.

## Code acceptance

- [x] Exact final code PR [#379](https://github.com/july-rust/BoardSI/pull/379) HEAD `d87a19658eaae82a7907e32502215ecf8a34b9de`: **27/27 GitHub PR Check Runs SUCCESS**; **11/11 PR Workflows SUCCESS**, 22/22 workflow jobs SUCCESS. Rust 1.75 `cargo fmt`, Clippy, Native Windows x64/ARM64 and macOS arm64; Qt Offscreen Linux and actual Windows x64/ARM64/macOS arm64 test jobs success, including new D_015.
- [x] Independent same HEAD Codex `completed/has_any_findings=false/has_unresolved_findings=false` and **zero unresolved review threads**. Reviewer identified three material deficiencies (extra Source Copper, duplicate VIA definition, decimal MILS coordinate restrictions), which received production Rust/Qt remediations and explicit regressions before final approval.
- [x] Protected GitHub **Verified one-parent Squash** at [`main@ae89e46a8fe0568a113b5501d870d8d0711a0f10`](https://github.com/july-rust/BoardSI/commit/ae89e46a8fe0568a113b5501d870d8d0711a0f10): sole parent `ece21f0198213ed43a0924aaa8ae846e097aed73`, `verification.verified=true`, `reason=valid`.
- [ ] **Independent code-main post-push** exact merge SHA `ae89e46a8fe0568a113b5501d870d8d0711a0f10`: require all 21/21 distinct Check Runs and all 8/8 `event=push` workflows SUCCESS, no missing, pending, skipped, cancelled or head mismatch. This gate is **PENDING until independently verified**.

## Non-accepted electrical and solver capabilities

```text
source_t_junction = SAME_LAYER_CONTACT_CANDIDATE_ONLY
plating_provenance = PLATING_IMPORTER_ASSUMED
manufactured_plating = NOT_VERIFIED
barrel_electrical_continuity = NOT_VERIFIED
cross_layer_electrical_edges = NOT_PROVEN
global_open_short = NOT_VERIFIED
physical_return_path_p009 = UNSUPPORTED
solver_ready = false
cache_reuse_eligible = false
physical_si = NOT_RUN
formal_m1_u_t_acceptance = NOT_ACCEPTED
```

Parent Issue #12 remains **OPEN**; Draft PR #35 remains unchanged. No solver Scenario/cache, manufactured Copper conductivity, broad graph open/short, P009, PH-08, or formal M1 U09/U10/U39 and T02/T04/T24/T52 closure is claimed.

## Separate documentation and tracker acceptance

- [ ] This document's independent PR must pass its **own exact-final-HEAD** CI, completed Codex with zero unresolved findings/review threads, protected Verified single-parent Squash, and an **independent post-document-main event=push** exact SHA success.
- [ ] Only after documentation-main CI succeeds may Issue #378 be CLOSED/completed and the final `docs/development-checklist.md` D_015 documentation/Issue acceptance box be marked complete. A later evidence-only protected docs reconciliation is appropriate to avoid claiming post-merge validation in advance.
