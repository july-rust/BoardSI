# C_012 — Exact Source Signal Via / Reference Stitch Via circular Pad clearance (V8)

**Tracker:** [Issue #270](https://github.com/july-rust/BoardSI/issues/270) · **Implementation PR:** [#271](https://github.com/july-rust/BoardSI/pull/271) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · **Predecessor:** [C_011](c011-reference-stitch-pad-footprint.md).

**Baseline:** protected `main@647889d21202c22e0dd9a3853365476375e40433`, C_011 #267 completed. **2026-10-29 KST code acceptance:** Source-geometry V8 implementation only. Code final PR HEAD [`5874567053f2f7320e69fc674f0260808bc1deaa`](https://github.com/july-rust/BoardSI/commit/5874567053f2f7320e69fc674f0260808bc1deaa/checks) passed **27/27 Checks SUCCESS** and **11/11 PR Workflows SUCCESS**, including Rustfmt, Clippy, Debug/Release, previous B/C regressions, Windows x64/ARM64/macOS ARM64 Qt Native and Linux offscreen. [Codex latest exact HEAD review](https://github.com/july-rust/BoardSI/pull/271#issuecomment-6494874536) reports `status=completed, has_any_findings=false, has_unresolved_findings=false`; historical P1/P2 threads **2/2 resolved**. [Protected code Squash PR #271](https://github.com/july-rust/BoardSI/pull/271) merged to [`main@ae187f1f4437181676239106260b64238bc0afd8`](https://github.com/july-rust/BoardSI/commit/ae187f1f4437181676239106260b64238bc0afd8), one parent and GitHub `verification.verified=true, reason=valid`. Active [Ruleset #25046595](https://github.com/july-rust/BoardSI/rules/25046595) is Strict with 17 required checks, Squash-only, signed commits and zero bypass actors.

**Code post-main accepted 2026-10-29 KST:** Independently verified [`main@ae187f1f` **21/21 Checks SUCCESS**](https://github.com/july-rust/BoardSI/commit/ae187f1f4437181676239106260b64238bc0afd8/checks) and **9/9 associated push Workflows SUCCESS** (eight push workflows plus dynamic `Push on main`), including Windows x64/ARM64, macOS ARM64 native Qt, Linux Qt offscreen and Rust regression. **Remaining separate gate:** acceptance documentation PR #272 exact-final-doc-HEAD CI/review, protected signed Verified Squash, and independent post-doc-main checks. Issue #270 and parent M1 #12 remain OPEN until the separate documentation closure gate is evidenced.

## Contract

- Public synthetic PADS Layout ASCII V9.4 MILS and the existing C_011 constrained two-layer route. Input is exact original bytes, trace indices/split, two Reference layers and explicit Stitch Via index. The unique Signal transition Via is independently found from the ordered source route's same-Net joint, **not** by nearest-neighbor or arbitrary Net names.
- Additive opt-in `--reference-route-v8`; V7 source-area/pad validation is called independently and embedded **byte-for-byte**, including its prior V6/V5/V4/V2 contracts. Legacy opt-ins unchanged.
- Require distinct Signal and Reference Net, source-canonical Via identity, explicit plated through-drill and circular conductive Pad on **each** selected Reference layer; source geometry and Layer span must agree. Fail closed on missing/ambiguous data.
- Compare original Source-Tick centre distances: `4*(dx²+dy²)` versus `(diameter_signal+diameter_reference)²`, with checked integer products. Less is `SOURCE_PAD_OVERLAP_RISK`, equal `SOURCE_PAD_TANGENT_RISK`, greater `SOURCE_PAD_SEPARATE_OBSERVED`. If embedded V7 is not contained, the overall is `V7_REFERENCE_FOOTPRINT_RISK`. **Separation is NOT a DRC clearance PASS.**
- The receipt carries Source SHA, Source Via indices, coordinates, Pad/Drill sizes per layer, and exact classification; Qt independently recomputes the geometric classification and verifies local imported identities. A signed edge gap in mm is **approximate display only**, not an acceptance predicate.
- Qt opt-in V8 CPU/RHI displays both source-sized round Pad rims on each Reference layer, provenance/Net/Layer/size/approximate gap in Properties, and clears stale results on changed selection, options, source SHA/watcher, reset, reimport, process generation and late response.

## Public regression

- Rust: `crates/boardsi-import/tests/c012_signal_stitch_via_pad_v8.rs`, exact separation, tangent, overlap, one Source Tick difference, SHA mutation and fail-closed malformed Source.
- Qt: `apps/boardsi-qt-prototype/preview_p0.cpp --c012-clearance-selftest` through actual Rust importer to native Qt CPU/RHI; CMake Linux offscreen and workflow Windows x64/ARM64 + macOS ARM64.
- Previous C_011/V7 and all B/C opt-ins must remain byte compatible.

## Code evidence / final documentation gates

- [x] **Exact final code HEAD 58745670:** 27/27 PR Checks, 11/11 PR Workflows SUCCESS; Rustfmt, Clippy, Debug/Release, Native Qt Windows x64/ARM64, macOS ARM64 and Linux offscreen. **First checked 2026-10-29 KST; evidence date 2026-10-29 KST.**
- [x] **Independent same-head Codex:** `status=completed, has_any_findings=false, has_unresolved_findings=false`; two historical P1/P2 review threads resolved. **First checked 2026-10-29 KST; evidence date 2026-10-29 KST.**
- [x] **Protected code merge:** PR #271 Squash to signed Verified `main@ae187f1f4437181676239106260b64238bc0afd8`; exact Ruleset #25046595 remains enforced. **First checked 2026-10-29 KST; evidence date 2026-10-29 KST.**
- [x] **Independent code post-main Checks and Workflows:** [`main@ae187f1f` 21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/ae187f1f4437181676239106260b64238bc0afd8/checks), **9/9 associated push Workflows SUCCESS**, including Qt Windows x64/ARM64 and macOS ARM64 Native and Linux offscreen. **First checked 2026-10-29 KST; evidence date 2026-10-29 KST.**
- [ ] **Separate documentation closeout:** dated checklist, final acceptance document PR, exact-final-doc-HEAD required CI/Codex, protected Verified Squash and post-doc-main independent checks. Issue #270 must remain OPEN until all evidence is final.


**Mandatory non-claims:**

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

This code does not construct manufactured copper fill, antipad/thermal spokes, physical electrical connection, Return Current/P009, RLGC, impedance, S-parameters or full product DRC. No minimum clearance rule is invented. Formal U09/U10/U39/T02/T04/T24/T52, PH-08 and parent M1 #12 remain unchecked/OPEN. No confidential board data is used in public CI.
