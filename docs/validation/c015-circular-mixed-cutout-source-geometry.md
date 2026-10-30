# C_015 — PADS COPCCO + COPCUT source-exact Signal Via clearance V11 / Qt

**Status (2026-10-30 KST):** LIMITED SOURCE-GEOMETRY ACCEPTED at protected Signed Verified squash `main@273417a7555a1dddc5724b15b062029ddf3b6e52` ([PR #281](https://github.com/july-rust/BoardSI/pull/281), [Issue #280](https://github.com/july-rust/BoardSI/issues/280)). This is **not** a physical SI/return-path qualification. Parent M1 [#12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## Scope and original-source ancestry

- Rust `reference_route_v11::encode` is opt-in (`--reference-route-v11`), selects 0–8 same-COPPER explicitly owned rectangle COPCUT or circular COPCCO voids per reference layer, requires two selected Source layers, unique Source Area identity and conductive Source Signal/Stitch Via padstacks.
- Exact bounded `i128` squared circle/rectangle tests classify missing/GAP/tangency/overlap; source tick is 254 nm. Any self-intersection, tangent cutouts, unsupported arc/outer or unowned hole fails closed.
- Original Source-derived `V8` is **mandatory even for circular COPCCO input**. This required a targeted Canonical `area_contact_circle` oracle for rectangle outer, strictly disjoint mixed holes and analytic four-cardinal-arc circles; the V5 ViaCentre contact graph checks exact validated Source geometry and same-layer candidate net, V6/V7 preserve conservative route corridor/PAD risk where circles remain unsupported. Any inability to prove the actual V8 receipt is an error, not a fake or downgraded V4.
- V1–V10 outputs on previously accepted rectangle-only sources must stay unchanged; new circle input may now produce an explicitly qualified **source-local geometry** V8 witness where previously unsupported. No full return-current proof.
- `apps/boardsi-qt-prototype/c015_mixed_cutout_risk.h` independently cross-checks Source SHA, per-layer Area identity, Via identities, Layer/Net, original imported rectangle/circle holes, exact hole bounds, Source Signal Pad gap severity and SOURCE Stitch Via **centre strictly within original copper outside all voids**; it binds V11 to the nested original V8/V5 CONTACT Area IDs. CPU real-size pad rims/RHI risk, source/selection/stale-result invalidation are exercised in native selftest.

## Tests and evidence

- Earlier provisional Source geometry-only candidate passed 11/11 PR Workflows, including Rust and Native Qt on Windows x64/ARM64, macOS ARM64 and Linux Qt offscreen. **Earlier evidence alone did not qualify the newly expanded circular V8 contract; final-head evidence below now does.**
- Final code HEAD `95daabc02339aa37a3c5ddb84af7e4798c54e96b` includes Rust `tests/c015_reference_route_v11.rs`, `tests/c015_mixed_cutout_source_geometry.rs`, Source canonical model and Native Qt `--c015-mixed-cutout-selftest`; the exact-head Native/Debug/Release and previous A/B/C regression gates passed in the linked PR CI.
- Independent Codex reviewed an earlier HEAD and found a P2 Qt Stitch Via void-centre/CONTACT-binding vulnerability. Source-geometry independent validation and nested V8/CONTACT ID check were added. **Final exact-head Codex review completed at `95daabc02339aa37a3c5ddb84af7e4798c54e96b` with no findings and 0 unresolved review threads; previous P2 corrected and thread resolved.**
- [x] Final code HEAD [`95daabc02339aa37a3c5ddb84af7e4798c54e96b`](https://github.com/july-rust/BoardSI/commit/95daabc02339aa37a3c5ddb84af7e4798c54e96b/checks) **28/28 PR Check Runs SUCCESS**, including all 17 Strict Ruleset #25046595 required checks; Rust fmt/Clippy/Debug/Release and Native Qt Windows x64/ARM64, macOS ARM64, Linux offscreen (2026-10-30 KST).
- [x] [Codex exact HEAD summary](https://github.com/july-rust/BoardSI/pull/281#issuecomment-6512392807) completed, `has_any_findings=false`, `has_unresolved_findings=false`; original P2 review thread resolved (2026-10-30 KST).
- [x] Protected Squash [`273417a7555a1dddc5724b15b062029ddf3b6e52`](https://github.com/july-rust/BoardSI/commit/273417a7555a1dddc5724b15b062029ddf3b6e52) has one parent and `verified=true, reason=valid`. Independent [post-main 21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/273417a7555a1dddc5724b15b062029ddf3b6e52/checks), [Push on main #40987447378](https://github.com/july-rust/BoardSI/actions/runs/40987447378) SUCCESS, and 8/8 push-triggered workflows SUCCESS (2026-10-30 KST).
- [x] `docs/development-checklist.md` C_015 narrow `[x]` added with first-check date 2026-10-30 KST, evidence date 2026-10-30 KST and exact PR/commit/check links; documentation PR must itself pass protected CI, Signed Verified merge and independent post-main checks before Issue #280 closure.

## Out-of-scope/nonclaims (immutable)

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

The result is not manufactured copper, Antipad/Thermal, electrical/physical DRC, RLGC/impedance/S-parameter qualification, Return Current P009 or product Solver readiness. Formal U09/U10/U39/T02/T04/T24/T52 and parent M1 remain **unchecked/open**.
