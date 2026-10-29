# C_011 — Source-exact Reference Stitch Via circular Pad Footprint risk (V7)

**Issue:** [#267](https://github.com/july-rust/BoardSI/issues/267)  
**Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) — OPEN  
**Predecessor:** [C_010 #264](https://github.com/july-rust/BoardSI/issues/264)  
**Status:** IMPLEMENTATION CANDIDATE, NOT ACCEPTED until final exact-HEAD native CI/review/protected merge/post-main verification.

## Bounded engineering contract

- Public synthetic PADS Layout ASCII V9.4 MILS only, existing C_010 Signal route and exact source COPCLS/COPCUT Reference Area ownership.
- Opt-in `--reference-route-v7` reparses original bytes and computes exact source-grid circular pad-to-polygon-edge containment on both selected reference layers.
- Strict equality with source Via/Padstack/Drill round Conductive geometry. Pad centre inside Copper is necessary but **insufficient**. Boundary tangent and Cutout invasion fail closed as risk.
- Use checked source-coordinate `i128` cross products and exact squared radius comparison, without float epsilon for acceptance.
- Existing V6 source proof, V5 strict ViaCentre→AreaInterior, V4/V2 and V1–V6 wire bytes remain unchanged. Same-Net disconnected Copper Source Area identity cannot become electrical continuity.
- Qt CPU/RHI risk marker shows source area, pad/drill dimensions, status and unsupported physical claim. Invalidated on reimport, Source SHA change, selection/generation changes, late IPC. Native real Rust→Qt smoke in Windows x64/ARM64, macOS ARM64 and Linux offscreen.

## Status values

`PAD_FOOTPRINT_CONTAINED_CANDIDATE`, `PAD_BOUNDARY_RISK`, `PAD_CUTOUT_RISK`, `V6_CORRIDOR_RISK`, and error/unsupported fail closed.

A geometric candidate does **not** imply electrical contact or stitched return path. It explicitly excludes thermal spokes, antipad/clearance synthesis, fabricated poured copper, 3D RF effects, physical Return Current P009, RLGC/S and product Solver/Cache.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

## Verification matrix (unverified until checked in GitHub)

- [ ] Rust fmt/Clippy; Debug+Release V7 public positives and negative tangent/cutout/island/unsupported tests.
- [ ] Legacy B/C regression and V1–V6 bytes unchanged.
- [ ] Qt actual importer subprocess and CPU/RHI source risk marker, stale input/reimport/regeneration.
- [ ] Windows x64/ARM64, macOS ARM64, Linux offscreen native CI on **exact final PR HEAD**.
- [ ] Independent Codex same final HEAD review completed, findings 0/unresolved 0.
- [ ] All protected Ruleset #25046595 checks passed, signed Verified Squash; no bypass.
- [ ] Independent exact post-main Checks and Workflow Runs SUCCESS.
- [ ] Issue closure and checklist first-check/evidence date KST only after passing all gates.

Formal U09/U10/U39, T02/T04/T24/T52, PH-08 and M1 #12 remain unchecked/open. No confidential files or original proprietary PCB included.
