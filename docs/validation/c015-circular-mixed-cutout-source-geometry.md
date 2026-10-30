# C_015 — Source-exact mixed rectangular COPCUT / circular COPCCO V11 + Qt risk candidate (NOT ACCEPTED)

**Issue:** [#280](https://github.com/july-rust/BoardSI/issues/280) · **PR:** [#281](https://github.com/july-rust/BoardSI/pull/281) (DRAFT; **DO NOT MERGE**) · **Parent M1:** [#12](https://github.com/july-rust/BoardSI/issues/12) **OPEN** · baseline protected `main@8701b8b20548c92e4e429472791f2ce2c3abf57b`.

**Started:** 2026-10-30 KST. **Feature acceptance:** PENDING; no new checklist [x] yet.

## Implemented candidate (not an acceptance claim)

- Rust exact source-grid `i128` circle-vs-pad and rectangle-vs-pad topology, strict same-owner disjoint bounded <=8 Cutouts/Reference layer, source declared net/layer/area identity, malformed and overflow rejection.
- Real PADS V9.4 MILS Source parser fixtures and negative tests; unchanged V1–V10 switching/receipt code.
- Explicit opt-in `--reference-route-v11` Worker and `BOARDSI_REFERENCE_ROUTE_V11` envelope. For all-rectangle inputs, only a **genuinely regenerated original-source V8** is included; for circular input, independently generated original-source V4 is embedded with literal `SOURCE_CONTACT_UNVERIFIED`. The V4 path does **not** prove V5/V6/V7/V8 ancestry and **does not meet the originally requested circle V8 gate**. No synthetic V8, no silent circle removal.
- Qt `c015MixedCutoutMode` validates original source SHA, canonical Source Area ID, imported Via/pad dimensions, per-layer circle and rectangle source geometry, hole uniqueness, integer boundary classification, embedded authentic V4/V8 receipt via appropriate independent Qt decoder. Reuses existing CPU original-size Pad rim and RHI risk marker; stale/source/selection/generation invalidation. New `--c015-mixed-cutout-selftest` included in Native Qt matrix.
- See `crates/boardsi-import/src/reference_route_v11.rs`, `reference_route_v11/geometry.rs`, `tests/c015_reference_route_v11.rs`, `tests/c015_mixed_cutout_source_geometry.rs`, `apps/boardsi-qt-prototype/c015_mixed_cutout_risk.h`, `c005_reference_risk.h`, `preview_p0.cpp` and `.github/workflows/qt-runtime.yml`.

## Verified prior limited evidence versus current outstanding gates

- Initial pure geometry / source-parser fixture unit tests were independently observed passing in earlier Rust Debug/Release runs (2026-10-30 KST). **These are not a substitute for the current final V11/Qt HEAD.**
- Several intermediate C_015 candidate runs failed Rustfmt and one Clippy argument-count gate; targeted fixes were made. A cancelled older run is **not** PASS.
- [ ] Current exact final HEAD: full Rust fmt/Clippy/Debug/Release, added Rust V11 wire tests and all B/C regressions PASS with run IDs.
- [ ] Native Windows x64/ARM64, macOS ARM64, Linux offscreen Qt importer→V11 CPU/RHI and stale selftest PASS.
- [ ] Independent same-HEAD Codex review: completed, findings=0, unresolved=0.
- [ ] Verified Signed protected Squash through Strict Ruleset #25046595, independently checked post-main Checks and Push Workflows.
- [ ] **Original circle V8 ancestry gate:** V5 local source-contact graph and V6/V7 geometry paths currently do not establish authentic V8 receipt for circular Cutout; fallback V4 is deliberately labeled weaker and does not satisfy requirement. Must either implement genuine source-exact circular V5/V6/V7/V8 evidence and Qt legacy verification or explicitly change and separately approve the acceptance contract. Do not silently waive.
- [ ] Dated checklist limited-feature acceptance and closure of Issue #280 **only after every required proof**.

## Boundaries and nonclaims

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

The observed circular pad-to-source void separation is **not manufactured DRC clearance**, actual Copper fill, electrical separation, Antipad/Thermal, Return Current, RLGC, impedance, S-parameter or SI Solver signoff. Whole product M1 and formal U09/U10/U39/T02/T04/T24/T52 stay **unchecked**.
