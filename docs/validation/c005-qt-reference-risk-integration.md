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

## Acceptance evidence — 2026-10-28 KST

### PR #249: source-bounded Rust/Qt implementation

- [x] Code [PR #249](https://github.com/july-rust/BoardSI/pull/249), exact final HEAD [`43204eab6c6b204a93a7e36ad957bad772187fd1`](https://github.com/july-rust/BoardSI/commit/43204eab6c6b204a93a7e36ad957bad772187fd1/checks): **11/11 GitHub PR workflows SUCCESS and 22/22 jobs SUCCESS**. Rustfmt/Clippy/Debug/Release regression gates passed. The dedicated Rust `c005_reference_risk_wire` integration ran **5/5 synthetic tests in each Debug and Release profile**; do not confuse these with unrelated 12-case C_003 tests.
- [x] Qt Linux CTest `qt-preview-c005-reference-risk` PASS, actual Rust importer subprocess and representative split risk, source SHA pin, stale marker and Properties removal, external source-file edit invalidation. CTest suite 12/12 PASS on Linux. Existing Windows x64/ARM64 and macOS ARM64 Qt Native CI gates and source-only RHI variants pass; **the PR #249 Native workflows did not yet call the dedicated C_005 smoke**, and that test-coverage gap is separately closed by PR #250.
- [x] Exact-head independent Codex review `43204ea` **completed**, `has_any_findings=false`, `has_unresolved_findings=false`; three historical P2 findings fixed, all three review threads resolved. Existing PADS non-Unicode source path compatibility retained.
- [x] Protected signed **Verified Squash** merge [`main@338f5c5f9d8590ba84d56803036590158a8f87c4`](https://github.com/july-rust/BoardSI/commit/338f5c5f9d8590ba84d56803036590158a8f87c4), GitHub REST commit `verification.verified=true`, `reason=valid`.
- [x] Independent post-main [`338f5c5` GitHub checks](https://github.com/july-rust/BoardSI/commit/338f5c5f9d8590ba84d56803036590158a8f87c4/checks): **21/21 Check Runs SUCCESS** including required 17 contexts and [Push on main #40684417663](https://github.com/july-rust/BoardSI/actions/runs/40684417663) SUCCESS.

### PR #250: direct C_005 Native Qt source-risk tests

- [x] Follow-up [PR #250](https://github.com/july-rust/BoardSI/pull/250), exact final HEAD [`b7f6e550f1cd9bf8d6ca32ad8bdf43ae7e8222f2`](https://github.com/july-rust/BoardSI/commit/b7f6e550f1cd9bf8d6ca32ad8bdf43ae7e8222f2/checks): **25/25 GitHub Checks SUCCESS**, **9/9 PR workflow runs SUCCESS**. The added `--c005-risk-selftest` executed and passed independently in **Windows x64, Windows ARM64, macOS ARM64** native Qt jobs; Linux Qt CTest succeeded. These demonstrate public synthetic risk-only geometry and stale-state lifecycle, not physical plane/Return Path.
- [x] Exact-head independent Codex `b7f6e55` completed: 0 findings, 0 unresolved review threads.
- [x] Protected Signed **Verified Squash** merge [`main@0c653bb6c01eba10bc0d1e145bc27d5228bd24f3`](https://github.com/july-rust/BoardSI/commit/0c653bb6c01eba10bc0d1e145bc27d5228bd24f3), GitHub `verification.verified=true`, `reason=valid`. Ruleset #25046595 strict, required 17 and no bypass.
- [ ] **Independent post-main CI** for `0c653bb6c01eba10bc0d1e145bc27d5228bd24f3`: [commit checks](https://github.com/july-rust/BoardSI/commit/0c653bb6c01eba10bc0d1e145bc27d5228bd24f3/checks), [Push on main #40686719331](https://github.com/july-rust/BoardSI/actions/runs/40686719331), [Qt runtime #40686719649](https://github.com/july-rust/BoardSI/actions/runs/40686719649). Still running when this document revision was prepared; do not mark PASS until exact-SHA 21/21 and Push on main SUCCESS are independently observed.

### Date and status controls

- C_005 only: **2026-10-28 KST** is the prospective first checklist check date and evidence date, contingent on the last post-main step and documentation PR protection. Existing historical first-check dates are immutable.
- [ ] Full scoped C_005 checklist completion still awaits PR #250 post-main CI and documentation integration.
- Parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) was accidentally auto-closed by a text keyword in PR #249, **reopened**, and remains OPEN because its actual physics and U/T gates are unresolved. Do not auto-close the parent.
- [ ] P009 physical Return Path, electrical conductor continuity, physically validated skin/proximity/roughness loss, full RLGC/S, signal Port model, physical Scenario/cache, arbitrary PADS/ODB++, real PCB measurement and full U09/U10/U39 / T02/T04/T24/T52. **NOT RUN / NOT ACCEPTED**.

**Current limited verdict: implementation and Native smoke merged; post-main PR #250 check pending.** No claim of a production physical SI solver.
