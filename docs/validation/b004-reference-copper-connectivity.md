# B_004 — 제한적 단순 Copper Area 국소 접촉 (검증 원장)

Date: 2026-10-27 KST. [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**.
[PR #226](https://github.com/july-rust/BoardSI/pull/226) is the bounded B_004 candidate.
Refer to [development checklist](../development-checklist.md) and
[B_003 accepted scope](b003-b001-reference-topology.md).

## Exact implemented candidate scope

- Only original PADS V9.4 MILS *LINES* COPPER with an explicit hole-free,
  closed, straight **COPCLS** Polygon, bound to its real Canonical source Area ID,
  declared source Net label and accepted ROUTE Net ID.
- Trace **endpoints** and source-declared conductive SMD **Pad centres**:
  same Layer, same declared Net and strictly inside the source polygon in
  integer PADS ticks. Boundary, width/body-only contacts and one-tick misses
  are not invented.
- Source-grid bounded candidate lookup per Layer and X interval; B_003
  original local contact graph/legacy opt-in Qt V1 format remains unchanged.
- Separate `ContactSite::AreaInterior` is validated by the Canonical Model;
  legacy/general `AreaWitness` remains unchanged for pre-existing synthetic
  contracts. Different sites inside one same-source Area can use a typed
  `AreaContinuity` edge. Separate Area IDs do not connect merely from names.
- Source-unknown/conflicting Net, cutout/hole, circle/arc/curved geometry:
  `UNSUPPORTED` / no new electrical edge. Report for a selected Net is
  `LOCAL_CONTACT_VERIFIED` only with valid Canonical model and local evidence;
  otherwise `UNSUPPORTED`.

**Out of scope and NOT_VERIFIED:** full Copper flood/Plane connectivity,
general Pad body and Trace-width touch, internal Trace intersections, complex
islands, Via-to-Plane/thermal/anti-pad, actual continuous high-frequency
return path, physical P009, SI solver, immutable physics cache scenario,
M1 full U09/U10/U39 or T02/T04/T24/T52 acceptance. Preserve
`return_path_p009=UNSUPPORTED`, `solver_ready=false`,
`physical_si=NOT_RUN`, `cache_reuse_eligible=false`.

## Regression matrix (evidence must be attached before checking)

| Synthetic regression | Expected |
| --- | --- |
| One Trace endpoint strictly inside same-Net polygon | 1 local contact |
| Two distinct Trace endpoint witnesses on one Area | same Area reachability |
| Source-declared SMD Pad centre strictly inside polygon | local contact |
| Boundary/exterior/1 tick miss/other Layer | no contact |
| Cross-Net coincidence | diagnostic only, no edge |
| Same-named disconnected Areas | remain disconnected |
| Missing/unrecognized Area Net, owned COPCUT | no contact |
| Forged/modified AreaInterior node | Canonical validation ERROR, report UNSUPPORTED |
| Existing 27 synthetic model-integrity tests | unchanged PASS |
| Existing B_003 contacts / legacy V1 wire | unchanged PASS |
| Windows x64, Windows ARM64, macOS ARM64, Linux Qt | Native CI PASS |
| Rust fmt/clippy/unit/integration, CI 17 required | exact HEAD all SUCCESS |

## Execution and review history — not evidence of final acceptance

- First bounded candidate authored in PR #226.
- Initial Rustfmt errors corrected from Numerical CI log; Clippy no-effect
  regression fixed.
- Independent Codex P1 identified an incorrect previous Polygon segment-start
  comparison; fixed by checking prior segment **end**.
- Independent Codex P2 identified a local Reference report that trusted mutated
  evidence; fixed by refusing reports when Canonical validation has errors.
- Strict-check-on-all-existing-`AreaWitness` caused legacy synthetic Model
  regression; fixed by introducing B_004-only `AreaInterior`, keeping the
  legacy general witness contract.
- An added public synthetic SMD pad-source regression exercises actual
  `part_preview::augment_source_bound` then Copper graph creation.
- Source code HEAD `7c68f9523b65999271fff5b510e984ec30dac72a`:
  **12/12 GitHub Actions workflows SUCCESS, 23/23 recorded jobs SUCCESS**,
  including Rust fmt/clippy/numerical and Rust Native Windows x64/ARM64,
  macOS ARM64, Linux, native Qt Windows x64/ARM64, macOS ARM64 and Linux.
  [Exact source HEAD CI](https://github.com/july-rust/BoardSI/commit/7c68f9523b65999271fff5b510e984ec30dac72a/checks).
- Independent Codex review against that same source HEAD:
  `status=completed`, `has_any_findings=false`,
  `has_unresolved_findings=false`. Four historical review threads
  were resolved after code remediation. [Review evidence PR #226](https://github.com/july-rust/BoardSI/pull/226).
- New public synthetic PADS regression additionally verifies forward
  collinear Copper vertices, declared component SMD Pad centre contact,
  unsupported Cutout, source-grid one-tick miss, disjoint same-Net Area
  isolation, forged witness, legacy Qt V1 compatibility.
- The documentation and checked-date commit following the above **source
  HEAD** is a different PR HEAD and MUST rerun CI/review gates before merge.
  A signed protected squash and post-merge `main` result are **not yet
  asserted** by this ledger.

**First scoped checklist check: 2026-10-27 KST. Evidence date: 2026-10-27 KST.**
Only the bounded B_004 Rust local Copper contact behavior is checked.
No product physics qualification or full B_001 completion is claimed.
Issue #12 stays **OPEN**.
