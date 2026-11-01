# D_002 — bounded Source COPCUT-aware two-area contact hypothesis

**First audit date:** 2026-11-01 KST. **Tracker:** [Issue #336](https://github.com/july-rust/BoardSI/issues/336). **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.

## Audited differentiation

- C_021 V13: two Source-owned COPCLS rectangular outer rings; zero COPCUT, positive overlap/boundary/disjoint only.
- C_022 V14: 3–6 Source COPCLS rectangles and transitive geometric components; all cutouts refused.
- C_023 V15: multi-area signal trace coverage with no cutouts; no manufactured continuity.
- D_001: Source Pad S/R/RF strict-interior Trace endpoint; does not validate source copper area membership.
- **D_002:** two complete Source-owned same-declared-Net, same-Layer COPCLS rectangles, with at most **one strictly internal same-owner rectangular COPCUT**, exact positive-area overlap after subtracting own cutout. This is a new geometry hypothesis, not an electrical connection.

## Restricted contract and independent source consistency

Opt-in CLI: `boardsi-import-artwork <source.asc> --d002-copcut-contact-v1 RA RB NET_REF 2`.

Rust independently parses complete PADS V9.4 MILS source and checks full inventory exactly two, explicit source Net/Layer, original SHA-256 and Canonical imported outer/cutout segment parity. Reject any unknown third area, wrong Net/Layer, duplicate names, unfilled Pour, multiple or nonrectangular voids and nonrectangular/circular/curved outer geometry. A COPCUT is **only** owned by its enclosing COPPER record; do not infer ownership from proximity.

The exact-source integer-tick algorithm enumerates axis-aligned cell boundaries from both outer rectangles and at most one owned void. Only a cell strictly inside **both remaining** Source copper shapes yields `SOURCE_CONTACT_CANDIDATE`. A fully excised intersection, boundary-only or disjoint case yields `SOURCE_SEPARATION_RISK`. The classification does not assert physical contact nor that an individual PADS Net label is a proven electrical graph node.

Wire remains separate from existing default artwork stdout. Native Qt re-reads Source SHA, Source name/ID, Net, Layer, outer and cutout rings from imported artwork, independently recomputes positive-area inclusion, displays a warning-only diagnostic and invalidates it on changed-source reimport. A stale response is never eligible.

## Explicit physical nonclaims

There is **no independent manufactured-fill/thermal/via-padstack/DRC or actual electrical continuity Oracle** in this task. Consequently the only permitted evidence state is `PHYSICAL_NOT_VERIFIED`; no manufactured-copper `PASS`, global P009 Return Path or Solver promotion is possible. This geometry candidate also does not establish that all surrounding copper remains connected or that an unsupported third object is harmless.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

Original M1 #12 and full U09/U10/U39/T02/T04/T24/T52 remain OPEN/unchecked; PH-08 Gate Q historical FAIL, M0 F1/F6/F7, Draft PR #35 remain unchanged.

## Independently observed bounded code acceptance — 2026-11-01 KST

The **code** PR [#337](https://github.com/july-rust/BoardSI/pull/337) final source HEAD `2ee4735bd7323e5971a542dd198e34c06f20b924` completed **27/27 Check Runs SUCCESS** and **11/11 pull_request workflow runs SUCCESS**. Verified Rust 1.75 formatting, strict Clippy, Debug/Release synthetic regressions, native Qt source COPCUT readback/stale tests on Windows x64, Windows ARM64 and macOS ARM64, Linux Qt 6.8.3 offscreen, plus existing native/FEM and related checks. All three dedicated `D_002 Native Qt COPCUT source contact and stale smoke` native steps were independently observed SUCCESS, including Windows ARM64.

The latest-exact-HEAD automated independent Codex review recorded `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`, **0 unresolved review threads** on commit `2ee4735bd7323e5971a542dd198e34c06f20b924`.

The protected [one-parent Signed Verified Squash main@c70d2309cb4aee1168762b51d7c76eb9d7ca42e4](https://github.com/july-rust/BoardSI/commit/c70d2309cb4aee1168762b51d7c76eb9d7ca42e4) has GitHub REST `commit.verification.verified=true`, `reason=valid`, single parent `2c107dd0f2179295c525654e9255040c93a4ecdd`. The **separate exact merged SHA** [21/21 main Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/c70d2309cb4aee1168762b51d7c76eb9d7ca42e4/checks) and [8/8 event=push workflows SUCCESS](https://github.com/july-rust/BoardSI/actions?query=event%3Apush) were independently read back with exact head SHA, 2026-11-01 KST.

| Bounded code acceptance criterion | Observed result |
| --- | --- |
| Synthetic Rust Source contact/cutout positive and negative tests | **PASS** in PR final-HEAD Numerical Core Debug/Release |
| Original source SHA / Canonical outer+owned void consistency | **PASS**, Rust & native Qt regression |
| Qt Native Windows x64/ARM64, macOS ARM64 D_002 dedicated smoke | **3/3 SUCCESS** |
| Linux Offscreen Qt D_002 CTest and existing Qt runtime suite | **SUCCESS** |
| Final exact PR HEAD checks | **27/27 SUCCESS** |
| All actual PR workflows | **11/11 SUCCESS** |
| Independent same-head Codex and review threads | **completed, findings 0, unresolved 0, threads 0** |
| Protected one-parent Verified Squash | **PASS**, `main@c70d2309...` |
| Independent post-code-main exact SHA Checks | **21/21 SUCCESS** |
| Independent post-code-main exact SHA Push workflows | **8/8 SUCCESS** |

**First bounded code acceptance check date: 2026-11-01 KST. Evidence date: 2026-11-01 KST.** This narrowly closes the Rust/Qt code-test acceptance, not fabricated electrical continuity. The subsequent **docs-only checklist reconciliation PR** is a separate gate; do not mark that docs PR merged, its own exact-HEAD CI/Codex completed or tracker #336 closed until independently verified and posted to Issue #336.

## Physical and product scope remains not accepted

No independent manufactured-fill/thermal/via-padstack/DRC or electrical-physical Oracle was available. `SOURCE_CONTACT_CANDIDATE` is a source geometry condition only. A Source Net label is not an electrical connectivity PASS. The product-wide M1 #12, U09/U10/U39/T02/T04/T24/T52, P009, physical SI, Solver/cache, PH-08 original Gate Q and Draft PR #35 remain unchanged.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No claims about the future documentation PR are recorded here. Its verification and Issue #336 safe closure must be independently documented after the fact.
