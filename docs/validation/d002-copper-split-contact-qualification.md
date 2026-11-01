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

## Acceptance gate — pending observed evidence

- [ ] Rust Debug/Release synthetic positive/negative Source-contact and cutout removal tests
- [ ] Native Qt Linux offscreen and Windows x64/ARM64, macOS ARM64 actual Rust importer/readback and stale tests
- [ ] Rust formatting, strict Clippy and all mandatory **17** context checks on exact final code PR HEAD
- [ ] All actual final-head checks and workflows successful, including required native targets
- [ ] Independent latest exact-HEAD Codex review complete with zero unresolved material findings and review threads
- [ ] Protected GitHub-signed Verified one-parent Squash to main, no bypass
- [ ] Independently successful post-main exact merge SHA push checks/workflows
- [ ] Update dated checklist only after verified outcomes; close bounded Issue #336 only when actual gates qualify

The 2026-11-01 date above records work initiation, **not PASS evidence**. No merge, review or CI is preclaimed. Any blocker remains BLOCKED with its exact cause, not retrospectively marked successful.
