# B_003 — B_001 source-backed Trace/Pad/Via connectivity continuation

Date: 2026-10-27 KST. Tracking: [Issue #12](https://github.com/july-rust/BoardSI/issues/12). Code: [Draft PR #224](https://github.com/july-rust/BoardSI/pull/224).

## Baseline and bounded candidate

Starting protected `main@fca3ffda5fcd03a44510c57a4a253caa603d991c`, verified signed. The [development checklist](../development-checklist.md) was read before coding. Existing [A_032 exact endpoint graph](a032-p2w-local-connectivity.md) and [B_002 limited audit revision](b002-b001-completion.md) were reused.

First implementation increment: strictly collinear original PADS source-tick straight-trace T-junction witnesses, validated again by the Canonical model using i128 cross and dot products. Same Layer plus equal source-declared Net IDs are still required for a contact edge; a cross-Net coincidence adds only a potential-conflict count and never an edge. One tick off centreline must not connect. This evidence does not validate trace widths, Pad contacts, arbitrary crossings, Area/Plane, reference return path or whole-board Open/Short.

The new source-bound production CLI path derives *declared* component Pad/Pin Net only from accepted PADS ROUTE SIGNAL pin-pair declarations and the real importer Net inventory. No matching declaration => `Pad.net=None`; conflicting declarations fail closed. New geometric witnesses qualify **only exact Pad-centre points on straight Trace centrelines on the same conductive Layer and declared Net**, not Pad body/width overlap or full physical electrical connectivity. The original `part_preview::augment` remains unknown-Net by default for legacy callers. The existing opt-in Qt V1 contact records remain endpoint-only and omit newer Pad/interior evidence to avoid mislabeling.

## Verification ledger — do not pre-accept

- [ ] Exact HEAD Rust fmt/clippy and importer tests: **PENDING**.
- [ ] Native Qt Windows x64, Windows ARM64, macOS ARM64, Linux: **PENDING**.
- [ ] 10k route and previous A_032 regression: **PENDING**.
- [ ] Independent Codex same-HEAD review: **NOT_RUN**.
- [ ] Protected signed squash merge + independent post-main checks: **NOT_RUN**.
- [ ] Re-read checklist after test completion, add only accepted scoped subitem with first-check KST and evidence date; otherwise leave formal boxes unchanged.

**State:** B_003 PARTIAL / B_001 PARTIAL. Issue #12 stays OPEN. T02/T04/T24/T52 and U09/U10/U39 are NOT accepted; `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN` remain unchanged.

## Additional work remaining for bounded B_003

1. Review and qualify source-declared Pad/Pin Net binding and exact-centre geometry cases against positive, negative and contradictory fixtures. These are **candidates pending final CI**.
2. Trace-to-Pad width/body overlap and source-net-complete general Pad binding are **NOT_IMPLEMENTED**.
3. Via-to-Trace contacts currently require supported exact endpoint/centre contacts; broader Via shape interior intersection remains **NOT_IMPLEMENTED**, subject to B_003 declared scope review.
4. Verify same-HEAD native CI, independent review, protected merge, post-main checks and then update dated checklist.

No code capability is checked complete until exact-head CI and independent review pass.

## Final accepted bounded B_003 implementation — 2026-10-27 KST

The above candidate/pending sections are retained as historical stages; the following is the later, controlling evidence.

- Source PR [#224](https://github.com/july-rust/BoardSI/pull/224) final HEAD `e211b2e626afd89ac7a880eaeace40e91c8abc50` completed **28/28 GitHub checks SUCCESS**, including Rust numerical/fmt/clippy, Windows x64, Windows ARM64, macOS ARM64, Linux Qt and existing Artwork/IPC regression.
- Independent Codex zero-major review on prior commit `e40d45d157277d81ac8f50afef4a6b038c63f7bc` and resolved six historic threads. Git Trees of `e40d45d1` and final `e211b2e6` are **identical**: `aa5ac263f50044c79d97f27585e883010cab5633`; intervening commit did not change code content. A second review was requested at exact final SHA; its receipt, if available, should be recorded separately rather than assumed.
- Protected GitHub Verified signed squash [`main@1c11d08be6e7c83d3f6b697f515f61a8c1a43ba8`](https://github.com/july-rust/BoardSI/commit/1c11d08be6e7c83d3f6b697f515f61a8c1a43ba8) completed, signature reason `valid`.
- Merge automatically closed Issue #12; it was immediately explicitly reopened because B_001 full scope remains incomplete.
- **First scoped checklist acceptance: 2026-10-27 KST; evidence date: 2026-10-27 KST.**
- Post-source-main check results are **NOT YET RECORDED** in this ledger and must be verified independently; this evidence follow-up PR also requires its own checks/review/merge.

### B_003 acceptance boundary

**B_003 code subset: COMPLETE** only for source-grid exact restricted PADS Trace/Pad/Via contacts and test coverage described in PR #224. B_001 whole Issue #12 **PARTIAL / OPEN**.

Still NOT_IMPLEMENTED/NOT_VERIFIED: arbitrary SMD pad-body overlap and non-centre contact, unsupported PADS Pin/Net associations, arbitrary Via anti-pad/thermal/blind/buried spans, whole-board Copper Area/Plane Split/Cutout geometry, quantitative actual Reference Return Path, P009 proof, physical Scenario/Cache, Material loss/roughness, formal full U09/U10/U39/T02/T04/T24/T52. Keep `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`.
