# D_046 — PADS Same-Layer Oblique Finite-Width SOURCE Contact Qualification

**First checked: 2026-11-06 KST; evidence date: 2026-11-06 KST.** [Issue #481](https://github.com/july-rust/BoardSI/issues/481) · [Code PR #482](https://github.com/july-rust/BoardSI/pull/482) · [Canonical checklist](../development-checklist.md).

## 1. Boundary, fixed SOURCE and geometry

This is an opt-in, narrowly bounded **SOURCE_LOCAL_FINITE_WIDTH_ONLY** diagnostic, not a general PCB connectivity engine or manufactured electrical proof.

The public [D_046 PADS PowerPCB V9.4 MILS fixture](../../crates/boardsi-import/tests/fixtures/d046_oblique_width_v94.asc) uses 0.01 mil = **254 nm per SOURCE Tick**, Layer 1, NET_A/NET_B, four verified SMD Pads and four 10 mil vertical traces from D_040. The fifth trace is an **oblique round-capped bridge** from **(224,350) to (236,357) mil**, width **42 mil**, nonparallel and nonperpendicular to the existing vertical rails. Centerlines do **not** touch or intersect.

| Existing vertical trace | Minimum centerline distance | Combined half-width | Result |
| :--- | ---: | ---: | :--- |
| Left upper | 24 mil | 26 mil | Positive-area copper candidate |
| Left lower | 24 mil | 26 mil | Positive-area copper candidate |
| Right upper | 25 mil (exact 7-24-25) | 26 mil | Positive-area copper candidate |
| Right lower | 24 mil | 26 mil | Positive-area copper candidate |

**Expected local graph:** 9 SOURCE Nodes / 10 same-layer geometry Candidate Edges / 1 Local Component / 0 proven Electrical Cross-Layer Edges. Four new POSITIVE_FINITE_WIDTH_OBLIQUE_CONTACT candidates are added to the six baseline local edges. These are SOURCE-local candidates, not real-board electrical shorts/opens or arbitrary contact inference.

The first design proposal, (224,350)→(236,360) mil with width 44 mil, would overlap a nearby SMD Pad envelope under the conservative Pad safety gate. It was **not** accepted as the fixture. The revised source above prevents such unintended Pad contact and provides an exact 40 mil tangent case.

## 2. Exact bounded distance oracle and strict Fail-Closed

The [Rust implementation](../../crates/boardsi-import/src/d046_oblique_width.rs) works in SOURCE Tick **i128** arithmetic. The [Native Qt implementation](../../apps/boardsi-qt-prototype/d046_oblique_width_qt.h) independently reconstructs from ImportedArtwork using nm-to-Tick normalization and explicitly bounded **qint64** products. They do not trust one another's contact edge list.

For finite, nonintersecting 2D segments A and B, the minimum separation is the minimum of their four endpoint-to-segment distances. For an endpoint P and segment S→T with v=T−S, w=P−S, d=v·v:

- If w·v ≤ 0, compare squared distance |P−S|².
- If w·v ≥ d, compare squared distance |P−T|².
- Otherwise compare the *exact rational* squared distance cross(v,w)² / d.
- Accept only if **4 × distance² < (width_A + width_B)²**. Equality is boundary-only and is **rejected**.

Degenerate width/length, parallel/perpendicular or centerline contact, unsupported geometry/sections, wrong owner/pin pair/net/layer, overflow-risk boundaries, missing/duplicated source, forged graph/receipt and stale SHA all fail closed. Bridge endpoints must be in the bounded SOURCE source inventory; a broad geometry or 3D EM guarantee is not implied.

### Required adversarial evidence

- **40 mil bridge:** combined half-width 25 mil equals the right-upper 25 mil separation. Three contacts can remain positive, but **full four-edge qualification rejects**.
- **38 mil bridge:** combined half-width 24 mil; right-upper 25 mil produces a strict gap and other contacts can be tangent. Reject.
- **Corner/AABB false-positive:** the exact squared distance comparison rejects candidates with only bounding-box contact; the dedicated Rust geometry test asserts an AABB-overlapping-but-separated oblique case.
- Reversed bridge direction, NET_B, supported PADS header variations, CRLF and tab SOURCE variations are independently tested within supported limits.
- Native Qt [runner](../../apps/boardsi-qt-prototype/d046_qt_runner.cpp) reimports the actual hostile source and verifies rejected reconstruction on tangent/gap/corner/owner/layer/missing inventory; Rust CLI failure alone is not Qt proof.
- Fabricated edge, stale source digest and incomplete receipts are rejected against the exact SHA-bound independently rebuilt receipt.

## 3. Verified code evidence and chronology

- **Final code PR HEAD** [ea5225fe9886b998ca153f6023cf311b85f0e501](https://github.com/july-rust/BoardSI/commit/ea5225fe9886b998ca153f6023cf311b85f0e501/checks): **34/34 PR Check Runs SUCCESS; 11/11 pull_request Workflows SUCCESS**. Independent exact-HEAD Codex code review completed with findings=false and unresolved=false; review thread count **0**.
- Rustfmt, Clippy and D_046 Rust Debug/Release tests PASS. Actual Linux Qt offscreen **68/68 CTest PASS**, including d046 native reimport; Windows x64, ARM64 and macOS ARM64 actual Native Qt CI SUCCESS.
- Initial code HEAD c05e6f2 had a Rustfmt check failure. The replacement HEAD ea5225f includes formatting and correct Qt 40 mil tangent / 38 mil gap negative fixtures, then received a **fresh complete matrix**. Historical failure is not rewritten.
- Protected **Verified one-parent Squash** [code-main 11267c7494f9d7b82a06220402b4fa158fed4c29](https://github.com/july-rust/BoardSI/commit/11267c7494f9d7b82a06220402b4fa158fed4c29); sole parent b64f2e79fefeb4b255133552add112b3c6dd4d08, REST verified=true/reason=valid and exact main ref match.
- Independent **exact code-main event=push 8/8 Workflows SUCCESS and 28/28 Check Runs SUCCESS**, not inferred from PR checks.
- Separate Markdown-only docs PR, its Docs-only checks, protected merge and exact docs-main Push require independent verification. Record terminal evidence, closing chronology and administrative completion in [Issue #481](https://github.com/july-rust/BoardSI/issues/481); do not create recursive documentation PRs.

## 4. Non-acceptance and checklist semantics

D_046 adds **one** checked bounded Stage D diagnostic: Stage D **66 → 67** checked and 2 pending unchanged; overall **137 → 138** checked and 122 pending unchanged. These evidence-item counts are **not** percent of BoardSI product completion.

**NOT_VERIFIED:** manufactured Copper continuity, whole-board Open/Short, arbitrary multilayer Via/Pour/Thermal topology, P009/PH-08 Return Path, physical Scenario/Cache, RLGC/S-parameter solver, formal U01–U48/T01–T54 tests or product release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.
