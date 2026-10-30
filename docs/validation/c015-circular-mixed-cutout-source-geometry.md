# C_015 — PADS Source-exact COPCCO + COPCUT V11 / Qt risk — LIMITED ACCEPTANCE

**Limited-feature acceptance:** **PASS (2026-10-30 KST)** — original Source geometry and local Via contact **candidate only**, not physical Return Path, electrical SI or Solver signoff. **First accepted/check date:** 2026-10-30 KST; evidence date: 2026-10-30 KST. [Issue #280](https://github.com/july-rust/BoardSI/issues/280) · [code PR #281](https://github.com/july-rust/BoardSI/pull/281) · [signed protected Squash commit `273417a7555a1dddc5724b15b062029ddf3b6e52`](https://github.com/july-rust/BoardSI/commit/273417a7555a1dddc5724b15b062029ddf3b6e52). Parent M1 [#12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**.

## Precise accepted scope

- Opt-in Rust `--reference-route-v11` Worker authenticates the same original PADS ASC source SHA, Source Copper Area, Reference Net/Layer, selected Signal Via and Reference Stitch Via, with exact bounded `i128` integer-tick testing (1 tick = 254 nm).
- One original-source rectangle Copper outer per reference layer, zero to eight disjoint, strictly contained explicitly same-owner Source COPCUT rectangles and COPCCO circles, strict unsupported/outside/tangent/malformed shape rejection; signal Via conductive source Pad observes gap, tangent, overlapping Reference copper or missing void without manufacturing Antipad/Thermal.
- `crates/boardsi-model/src/area_contact_circle.rs` Canonical Source local point-inside-copper oracle validates four cardinal exact analytic circular arcs plus mixed rectangular holes (no approximate polygon-derived electrical contact). The local ViaCentre contact evidence supports genuine source V5 and the original V6→V7→V8 chain. **Every circular V11 receipt must contain actual original-source V8**; no synthetic/V4 fallback. Conservative V6/V7 risks remain `UNSUPPORTED` where outside their scope.
- Independent Qt `c015_mixed_cutout_risk.h` checks original imported Copper Area/Net/Layer/Source ID and per-hole circle/rectangle, Signal Via selected pad footprint and risk severity, source Stitch Via centre strictly outside **all** voids and inside the outer copper on both reference layers, nested V8→V7→V6→V5 `CONTACT` Area/source ID bindings and Via indices; original-size CPU/RHI Qt risk marker, stale selection/source invalidation.
- Existing rectangle-only V1–V10 protocol remains unchanged. Scope is *source-local candidate*; it is **not** fabricated electrical Via attachment or true Return Current path verification.

## Verified acceptance ledger (2026-10-30 KST)

| Gate | Evidence | Result |
|---|---|---|
| Code PR original exact HEAD | `95daabc02339aa37a3c5ddb84af7e4798c54e96b`, [PR #281](https://github.com/july-rust/BoardSI/pull/281) | PASS |
| Rust original-source V8 positive/negative fixtures | `crates/boardsi-import/tests/c015_reference_route_v11.rs`, `c015_mixed_cutout_source_geometry.rs`; `numerical` [run 40984559607](https://github.com/july-rust/BoardSI/actions/runs/40984559607) (Debug and Release) | PASS |
| Qt real PADS Importer→V11→independent CPU/RHI stale-risk fixture | `--c015-mixed-cutout-selftest`, `C015_NATIVE_QT_MULTI_CUTOUT=true`; Qt [run 40984559702](https://github.com/july-rust/BoardSI/actions/runs/40984559702), Native Windows x64/ARM64, macOS ARM64, Linux offscreen | PASS |
| Exact-head PR workflows | GitHub Actions PR head `95daabc...` | **12/12 SUCCESS** |
| Independent Codex code review | Exact HEAD `95daabc...`: completed, has_any_findings=false, has_unresolved_findings=false, previously raised P2 Stitch Via Source void/CONTACT vulnerability corrected and review thread resolved | **PASS, 0 unresolved** |
| Protected Squash with signature | [`273417a...`](https://github.com/july-rust/BoardSI/commit/273417a7555a1dddc5724b15b062029ddf3b6e52), GitHub REST `commit.verification.verified=true`, reason=valid, parent `8701b8b...`, active Ruleset `25046595` required signatures/strict status | **PASS / Verified** |
| Independent main post-merge checks | main SHA `273417a...`, GitHub REST `/commits/{sha}/check-runs`, including 17 required contexts and four Analyze contexts | **21/21 SUCCESS** |
| Independent main push workflows | same main SHA `273417a...`, GitHub REST `/actions/runs?head_sha={sha}`; Numerical, Qt, FEM, PH-06, PH-07, Docs, synthetic G0 etc. | **8/8 SUCCESS** |

Evidence counts and hashes above refer to the **code** protected Squash commit. Documentation-only follow-up has its own separate protected PR and must not retroactively change those code results. The **first limited check date** is recorded as 2026-10-30 KST; do not backdate other requirements.

## Scope boundaries (remain UNSUPPORTED)

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No manufactured copper, Antipad/Thermal, true current return, physical DRC, RLGC/impedance/S-parameter SI qualification or product Solver acceptance. Parent M1 Issue #12 and full formal **U09/U10/U39/T02/T04/T24/T52** remain OPEN/unchecked.
