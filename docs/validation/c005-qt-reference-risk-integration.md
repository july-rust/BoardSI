# C_005 — Qt Source Reference Risk Overlay and Solver Block

Started: 2026-10-28 KST. Tracker: [Issue #248](https://github.com/july-rust/BoardSI/issues/248). Parent: [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. Source baseline: protected `main@5046538d368c73c73d67046787ed437c15166a25`.

## Scope and implementation candidates

- Opt-in `boardsi-import-artwork PATH --reference-risk-v1 TRACE_INDEX REF_NET REF_LAYER [SECOND_TRACE_INDEX SECOND_REF_LAYER]` reuses `boardsi_import::reference_topology::audit` from C_002. It independently reparses original PADS bytes, builds restricted Canonical Source-owned Copper/Trace geometry, and verifies exact source identity and topology. Existing ordinary artwork CLI stays unchanged.
- Bounded `BOARDSI_REFERENCE_RISK_V1` transport carries a SHA-256 pinned Source identity, strict risk state/reason, selected source-order segment indices **plus complete Source-backed ObjectId component strings**, exact reference Trace endpoints and at most one source-grid risk location. Qt separately compares accepted ImportedTrace geometry, declared Net and Layer to the returned source witnesses.
- `C005ReferenceRiskController` asynchronously runs the installed Rust artwork import executable in audit-only mode. It refuses stale Source hashes, edited inputs, failed/malformed receipts, and importer changes; late old process responses are rejected through generation tracking.
- `ImportedArtworkPanel` renders risk-only marker hints in CPU GraphicsView and Qt Quick/RHI using shared source nanometre anchors, with selection, net and layer visibility and Properties. SOURCE_GEOMETRY_ONLY is **never** a successful electrical reference proof; UNSUPPORTED has no positive marker.
- `physical_si=NOT_RUN`, `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `cache_reuse_eligible=false` remain hard boundaries; existing Worker Run stays explicitly synthetic.

## Release/physics exclusions

Not implemented: physical AC return current, stitching/return Via evaluation, whole-board Net/Plane connectivity, arbitrary PADS versions and profiles, complete Canonical physical scenario/Port snapshot, frequency-dependent conductor/dielectric loss, validated RLGC/S and caches. The C_002 split witness is a representative Trace midpoint, **not the actual split intersection**. No U09/U10/U39 or T02/T04/T24/T52 formal product signoff. Parent #12 OPEN; B_007 original exact-head review remains a separate gate.

## Acceptance evidence (pending until independently verified)

- [ ] Exact final C_005 code PR HEAD SHA and complete Rustfmt/Clippy/Debug/Release.
- [ ] Rust `c005_reference_risk_wire` public synthetic tests; unchanged B_003..B_009/C_001..C_004 suites.
- [ ] Native Qt `qt-preview-c005-reference-risk` on Windows x64, Windows ARM64, macOS ARM64 and Linux with Rust subprocess, risk witness, CPU marker and stale invalidation; RHI marker rendering/picking where backend is available.
- [ ] Independent Codex review **completed on exact HEAD**, 0 findings, 0 unresolved review threads.
- [ ] Protected Signed Verified Squash Merge with GitHub commit verification.
- [ ] Independent post-merge exact `main` SHA workflow/check evidence.
- [ ] Dated live checklist limited C_005 box checked only with provenance, old first-check dates unchanged.

**Current verdict: NOT_ACCEPTED until above evidence passes.** This document must be updated with actual HEAD SHA, links, dates and real PASS/FAIL before closure.
