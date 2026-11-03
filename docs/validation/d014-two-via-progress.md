# D_014 — two-Through-Via Source-local contact chain qualification

**First recorded:** 2026-11-03 KST. **Issue:** [#374](https://github.com/july-rust/BoardSI/issues/374). **Code:** [PR #375](https://github.com/july-rust/BoardSI/pull/375). This is an opt-in, SHA-bound **Source contact candidate** qualification for PADS V9.4 MILS. It is **not** electrical proof of a manufactured plated barrel or a whole-board opens/shorts check.

## Source fixture, geometry, and fail-closed result

- Synthetic four-layer PADS ASC: **two Through Vias** on the same net, **three straight Traces**, Layer 1 arm A → Via A, Layer 4 Trace A→B, Layer 1 arm B ← Via B. Exact Source Via-centre to Trace-endpoint matching at four selected layer-local contacts.
- The deliberately **bounded candidate graph** emits **7 nodes, 4 layer-local candidate edges, 3 disconnected local components, 0 proven electrical cross-layer edges**. This is **not** a physical barrel connection between Layer 1 and Layer 4, and a missing Source candidate does not prove a manufactured open.
- Strict Source-only Rust witness: file SHA-256, selected Net/Layer/Via index, Via count and span, drill/pad/source-defined via name, Canonical Via/Trace/stackup inventory and geometry, and a fixed receipt with fail-closed statuses. The parser must not silently tolerate skipped/unmodeled Source objects.
- Specific fail-closed adversaries: missing/extra Via or Trace, wrong Net/Layer or endpoint, unexpected inner Via pad definition, additional analytic Arc, additional opposite-Via endpoint contact, **ViaCentre-to-TraceInterior contact**, overlapping/crossing Layer-1 arms, stale Source reimport, SHA-rebound forged receipt, and forged electrical/barrel promotion. Exact integer tick geometry is used for the bounded segment contact gate.
- Independent Qt validation reimports actual Source from the file, checks the local ImportedVia and ImportedTrack geometry and zero imported Arcs, checks Source Via padstack row restrictions, accepts harmless PADS `*REMARK*` records without losing section state, then independently reconstructs the expected SHA-bound witness rather than trusting Rust receipt text.

## Code acceptance — verified 2026-11-03 KST

- [x] Final exact [code PR #375](https://github.com/july-rust/BoardSI/pull/375) HEAD `ee1ce7fab79216866a1f6ce471f7c6fe98cf025c`: **27/27 pull-request Check Runs SUCCESS** and **11/11 PR Workflows SUCCESS**. Rust 1.75 fmt/Clippy and native Windows x64/ARM64, macOS arm64; Linux Qt offscreen **37/37 CTest PASS**, including `qt-preview-d014-two-via-source-chain`. Existing D_012→D_013→D_014 chained native Qt smoke validated on Windows x64/ARM64 and macOS arm64. No new GitHub Actions workflow.
- [x] Independent Codex review on that **same exact HEAD** `completed/has_any_findings=false/has_unresolved_findings=false`; **all 5 historical review threads resolved**. The review found and the implementation addressed additional endpoints, Via inner pad, extra analytic Arc, harmless remarks, and interior Via/Trace or arm crossing contacts.
- [x] GitHub-protected **Verified one-parent Squash** `main@03203494cb99b83291e3c80cfb22bd5c98b1c4f4`, sole parent `35cf89a0616edab57c9db3b65c55af6669bf04c4`, `verification.verified=true`, `reason=valid`.
- [x] **Independent post-code-main `event=push`** on exact code merge SHA `03203494cb99b83291e3c80cfb22bd5c98b1c4f4`: **21/21 Check Runs SUCCESS** and **8/8 event=push Workflows SUCCESS**, all matching that SHA. [Commit check evidence](https://github.com/july-rust/BoardSI/commit/03203494cb99b83291e3c80cfb22bd5c98b1c4f4/checks).
- [x] In the final code PR Numerical Core, the existing **D_006** macOS arm64 `different_source_bytes_cannot_reuse_sha` test initially failed with a transient executable `NotFound` error; **only that failed job was rerun at the identical final SHA**, and the final Numerical Core workflow finished SUCCESS. This is not evidence of a D_014 functional failure or a waived required check.

## Explicit scope exclusions

```text
source_local_contact = QUALIFIED_WHEN_PROVEN_FOR_RESTRICTED_SOURCE
two_via_source_chain = CANDIDATE_ONLY
plating_provenance = IMPORTER_ASSUMED
manufactured_plating = NOT_VERIFIED
barrel_electrical_continuity = NOT_VERIFIED
cross_layer_electrical_edges = NOT_PROVEN
global_open_short = NOT_VERIFIED
physical_return_path_p009 = UNSUPPORTED
solver_ready = false
cache_reuse_eligible = false
physical_si = NOT_RUN
formal_m1_u_t_acceptance = NOT_ACCEPTED
```

D_013 `legacy_v1_barrels` remains a **compatibility projection**, never a verified barrel electrical edge. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN and Draft [PR #35](https://github.com/july-rust/BoardSI/pull/35) stays unchanged.

## Separate documentation acceptance — verified 2026-11-03 KST

- [x] Separate [documentation PR #376](https://github.com/july-rust/BoardSI/pull/376) final exact HEAD `52d4f5cccf5dbcbc054bb20da5b9c3af37de186d`: **22/22 PR Check Runs SUCCESS**, **8/8 PR Workflows SUCCESS**, independent Codex `completed/has_any_findings=false/has_unresolved_findings=false` and **0 unresolved review threads**.
- [x] GitHub-protected **Verified one-parent Squash** at [`main@741da21d46811bd09f7e4211005006a5d7059249`](https://github.com/july-rust/BoardSI/commit/741da21d46811bd09f7e4211005006a5d7059249), `verification.verified=true`, `reason=valid`; sole parent `03203494cb99b83291e3c80cfb22bd5c98b1c4f4`.
- [x] **Independent post-document-main** exact SHA `741da21d46811bd09f7e4211005006a5d7059249` **21/21 Check Runs SUCCESS** and **8/8 `event=push` Workflows SUCCESS**; [commit checks](https://github.com/july-rust/BoardSI/commit/741da21d46811bd09f7e4211005006a5d7059249/checks).
- [x] [Issue #374](https://github.com/july-rust/BoardSI/issues/374) actually **CLOSED/completed** at GitHub `closed_at=2026-11-03T00:31:09Z` **after** independent documentation-main push CI acceptance. This ledger and the D_014 checklist's final `[x]` are a *subsequent separate protected documentation follow-up*, not evidence that was presumed before validation.

These gates accept only the **bounded SOURCE-local candidate** model; manufacturing plating, cross-layer electrical barrel connectivity, whole-board open/short, return-path P009, PH-08 and SI solver/M1 U/T qualifications remain unproven.
