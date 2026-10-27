# B_005 — Restricted Reference geometry coverage audit

Status: **IMPLEMENTATION CANDIDATE / NOT ACCEPTED** (2026-10-27 KST).
Tracking: [Issue #12](https://github.com/july-rust/BoardSI/issues/12).
Baseline: protected `main@cc673ab285af6c8f11314e1b4677d543b5c67415`.
Before implementation, the [development checklist](../development-checklist.md) was read.

## Intent and verified boundary

Add a Rust-only API `reference_geometry::audit` for an explicitly selected
nonbranching ordered single-Copper-Layer straight Trace path and a user-named
Reference Net on an adjacent **imported source-order** Copper Layer. A declared
source label is a geometry filter, NOT an electrical connection proof.

- Recheck Canonical validation, same declared Signal Net and Layer,
  B_003 source-grid endpoint graph join and no duplicate/backtracking path.
- Match imported source Area ID, Net label, Layer and every polygon segment
  against the Canonical representation. Require exactly one supported,
  hole-free straight source COPCLS Area containing the **entire** selected
  path centreline, strict interior, including concave polygons and boundary touches.
- Preserve the previous Connectivity Graph and Qt V1 protocol unchanged.
- Reject unsupported area geometry, missing Net, gap, split, cross-layer
  or ambiguous candidate with an explicit `UNSUPPORTED` reason.
- Output `SOURCE_ORDER_GEOMETRY_ONLY` rather than physical continuity PASS.
  Retain `return_path_p009=UNSUPPORTED`, `solver_ready=false`,
  `physical_si=NOT_RUN`, `cache_reuse_eligible=false`.

**Deliberate limitations:** layer ordering is from the PADS importer only;
there is no measured z/thickness or physical Stackup proof. Trace width,
wide-pad overlap, unsupported vias/cutouts, electric-plane stitching, physical
Return Path and frequency-dependent current distributions are NOT_VERIFIED.
B_001 overall remains OPEN. U09/U10/U39/T02/T04/T24/T52 stay formally
unchecked.

## Review remediation (before acceptance)

- Initial PR HEAD `6c0cd6f2` failed Rustfmt; formatted with exact CI diff at `3c2dde1`.
- Independent Codex found incorrect synthetic COPPER source headers,
  collinear backtracking and unselected interior branch false positives.
  Corrected COPPER piece/layer fields and added full-path branch/overlap
  rejection plus negative regression tests. CI and re-review remain required.

## Mandatory acceptance gates — update after actual CI

- [ ] Exact HEAD Rust fmt, Clippy, numerical/importer tests
- [ ] Native Windows x64, ARM64, macOS ARM64, Linux Qt regression
- [ ] Previous B_003/B_004 contact + Qt V1/GPU Artwork regression
- [ ] Independent Codex exact-HEAD review and no unresolved blocking finding
- [ ] All required protected branch checks
- [ ] Verified signed Squash Merge, independent post-main push CI
- [ ] Record first checked KST date/evidence date in development checklist
- [ ] Issue #12 remains OPEN; formal U/T unchanged

**No positive acceptance is recorded before these gates.**

## Additional source-proof remediation (pending CI)

Codex identified detached source Net label spoofing, canonical Trace geometry
mutation, and source Arc branches. B_005 now takes the actual original PADS
bytes, rechecks original Source SHA and reparsed Copper inventory, independently
reparses each selected ROUTE Trace ID/Layer/Net/endpoints, and rejects established
same-Net Arc contact at selected path. Added three negative regression cases.
These changes require fresh exact HEAD CI and independent Codex review.

- Original layer ordering is revalidated against reparsed source Layer IDs/order/kind; user-mutated layer adjacency must return `ORIGINAL_LAYER_ORDER_MISMATCH`. Added anti-spoof regression (still CI/review pending).

- Latest Codex P2: canonical source Net label/ID spoof and omitted unselected source branch. Added strict complete Net/Trace/Arc source inventory matching (ID, layer, net, coordinates, widths) and two negative tests. Gate remains NOT_ACCEPTED pending CI and latest-head Codex.

- Latest Codex P2: original source Via touching selected straight Trace could make a through-layer branch. Reject ANY reparsed source Via centre on selected path (`SOURCE_VIA_CONTACT_UNSUPPORTED`) without inventing a physical Via/Reference model. Added synthetic Via negative test; verification pending.
