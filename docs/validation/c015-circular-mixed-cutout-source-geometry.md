# C_015 — PADS COPCCO + COPCUT source-exact Signal Via clearance V11 / Qt

**Status (2026-10-30 KST):** implementation and verification candidate, NOT YET ACCEPTED. See [Issue #280](https://github.com/july-rust/BoardSI/issues/280) and [Draft PR #281](https://github.com/july-rust/BoardSI/pull/281). Parent M1 [#12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## Scope and original-source ancestry

- Rust `reference_route_v11::encode` is opt-in (`--reference-route-v11`), selects 0–8 same-COPPER explicitly owned rectangle COPCUT or circular COPCCO voids per reference layer, requires two selected Source layers, unique Source Area identity and conductive Source Signal/Stitch Via padstacks.
- Exact bounded `i128` squared circle/rectangle tests classify missing/GAP/tangency/overlap; source tick is 254 nm. Any self-intersection, tangent cutouts, unsupported arc/outer or unowned hole fails closed.
- Original Source-derived `V8` is **mandatory even for circular COPCCO input**. This required a targeted Canonical `area_contact_circle` oracle for rectangle outer, strictly disjoint mixed holes and analytic four-cardinal-arc circles; the V5 ViaCentre contact graph checks exact validated Source geometry and same-layer candidate net, V6/V7 preserve conservative route corridor/PAD risk where circles remain unsupported. Any inability to prove the actual V8 receipt is an error, not a fake or downgraded V4.
- V1–V10 outputs on previously accepted rectangle-only sources must stay unchanged; new circle input may now produce an explicitly qualified **source-local geometry** V8 witness where previously unsupported. No full return-current proof.
- `apps/boardsi-qt-prototype/c015_mixed_cutout_risk.h` independently cross-checks Source SHA, per-layer Area identity, Via identities, Layer/Net, original imported rectangle/circle holes, exact hole bounds, Source Signal Pad gap severity and SOURCE Stitch Via **centre strictly within original copper outside all voids**; it binds V11 to the nested original V8/V5 CONTACT Area IDs. CPU real-size pad rims/RHI risk, source/selection/stale-result invalidation are exercised in native selftest.

## Tests and evidence

- Earlier provisional Source geometry-only candidate passed 11/11 PR Workflows, including Rust and Native Qt on Windows x64/ARM64, macOS ARM64 and Linux Qt offscreen. **This does not qualify the newly expanded circular V8 contract.**
- Current head requires the final newly added Rust `tests/c015_reference_route_v11.rs`, `tests/c015_mixed_cutout_source_geometry.rs`, Source canonical model, Native Qt `--c015-mixed-cutout-selftest`, and all existing A/B/C regression gates to pass on the same final HEAD.
- Independent Codex reviewed an earlier HEAD and found a P2 Qt Stitch Via void-centre/CONTACT-binding vulnerability. Source-geometry independent validation and nested V8/CONTACT ID check were added. **Final same-head Codex re-review and closure of the review thread required.**
- [ ] Final-head fmt, Clippy `-D warnings`, Debug/Release, Native platform matrix and 17 strict required Checks PASS.
- [ ] Final-head Codex review completed with zero unresolved findings; thread resolved with evidence.
- [ ] Signed Verified protected Squash and independent post-merge GitHub Checks/Push Workflow success.
- [ ] Dated `docs/development-checklist.md` limited C_015 acceptance then Issue #280 closure only after evidence.

## Out-of-scope/nonclaims (immutable)

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

The result is not manufactured copper, Antipad/Thermal, electrical/physical DRC, RLGC/impedance/S-parameter qualification, Return Current P009 or product Solver readiness. Formal U09/U10/U39/T02/T04/T24/T52 and parent M1 remain **unchecked/open**.
