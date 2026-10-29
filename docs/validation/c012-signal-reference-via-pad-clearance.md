# C_012 — Exact Source Signal Via / Reference Stitch Via circular Pad clearance (V8)

**Tracker:** [Issue #270](https://github.com/july-rust/BoardSI/issues/270) · **Implementation PR:** [#271](https://github.com/july-rust/BoardSI/pull/271) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · **Predecessor:** [C_011](c011-reference-stitch-pad-footprint.md).

**Baseline:** protected `main@647889d21202c22e0dd9a3853365476375e40433`, C_011 #267 code + documentation closed. **Status:** implementation undergoing review/CI; source-geometry limited only, no physical/electrical acceptance. Final dates, HEAD, checks and reviewer findings must be filled **after successful independent gates**, not asserted now.

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

## Required evidence gate (pending until independently observed)

- [ ] Final *exact* PR HEAD required CI complete SUCCESS, Rustfmt, Clippy, Debug/Release, native Qt and previous regressions
- [ ] Independent Codex on that same final HEAD `status=completed, findings=0, unresolved=0`
- [ ] Protected Ruleset #25046595 Strict 17 required, signed Verified Squash-only merge, no bypass
- [ ] Independent post-code-main all required Checks and Workflows SUCCESS
- [ ] Dated `docs/development-checklist.md` first-check, evidence date and final PR/commit/test proofs
- [ ] If documentation has a separate PR, exact-head review/CI, signed Squash and independent post-doc-main verification before Issue #270 closure

**Mandatory non-claims:**

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

This code does not construct manufactured copper fill, antipad/thermal spokes, physical electrical connection, Return Current/P009, RLGC, impedance, S-parameters or full product DRC. No minimum clearance rule is invented. Formal U09/U10/U39/T02/T04/T24/T52, PH-08 and parent M1 #12 remain unchecked/OPEN. No confidential board data is used in public CI.
