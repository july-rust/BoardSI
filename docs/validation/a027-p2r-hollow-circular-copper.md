# A_027 / P2-R — source-owned hollow circular COPCIR Preview

Baseline `main@471168de9255b1686a2fc4f2b5e33f5c493032a3` A_026 PR #184; Issue #185. This document is an implementation contract, not a claim of completed CI.

## Source and scope

- PADS Layout V9.4 MILS `*LINES*`: source-owned `COPCIR` circle as primary piece, followed by one or more same-record `COPCCO` voids on the same declared layer. Source centre and radius come from the exact original diameter coordinates (0.01 mil = 254 nm).
- Rust parser uses source integer-tick circle-in-circle and circle-circle predicates, requiring strict non-contact and bounded geometry; orphan/reused/standalone pour voids are not re-assigned.
- Maximum eight holes, but the existing A_025 combined <=128 source/proxy vertex budget further restricts circle/circle combinations (typically <=2 full 32-vertex circular holes). Insufficient separation fails closed.
- Original Canonical Arc quarter-segments, provenance, source name/label are retained; declared net labels remain *non-electrical*, `net: None`.

## Rendering acceptance contract

- Transport remains V12 `G|C` outer, indexed `J` round voids. No version increment required.
- For hole-bearing circles only: CPU and RHI share exactly the same A_025 independently checked triangle mesh.
- Source circular outer is tessellated as an **inscribed** 32-segment exact 254-nm grid polygon. Every vertex must pass strict integer squared-distance from centre; since the circle is convex, straight edges and the entire triangulated fill cannot exceed the actual source circle. Per-segment centre-to-chord distance independently limits underfill to <1% of source radius.
- Source circular voids use A_026 **circumscribed** 32-sided proxies, each strict segment-to-circle clearance >=1 grid tick, <=1% radial deviation; no rendered copper can enter a real void.
- Overlap, contact, poor approximability, and overly narrow rings are conservatively refused, not silently painted. A_025 checks full mesh area, boundary crossing and triangle overlap.
- Display mesh never replaces analytical source arcs in Canonical; the 32-sided polygon is a Preview artifact, not a fabricated copper or SI solver geometry.

## Test design and limits

Positive fixture: `pads_copper_hollow_circle_v94.asc`: one source COPCIR at (200,200) mil, radius 100 mil; two source-owned COPCCO holes centered at (160,200) and (240,200) mil, radius 15 mil. Synthetic only.

Negative fixtures/mutations: touching inner/outer circles, overlapping voids, bad layer, degenerate diameter, understated owner piece count, orphaned piece, mixed polygon hole. Exact source topology applies; hostile V12 IPC additionally must pass conservative Qt mesh acceptance. Qt native test checks CPU fill and hole pixels, source properties, layer hide/restore, current imported RHI frame, triangle-based RHI picking and reset. A_022–A_026 regressions remain mandatory.

**Evidence status:** New PR CI and independent review are pending. Do not mark PASS without checking exact-HEAD required checks, independent reviewer and merge. A production PADS exporter compatibility certification, electrical connectivity, flood calculation, 2D RLGC/S parameter solver, native GPU VRAM or driver-reset proof, and 1M objects at 60 FPS are NOT_VERIFIED / NOT_RUN.
