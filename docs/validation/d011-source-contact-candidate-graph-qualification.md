# D_011 — PADS dual-layer source contact candidate graph qualification

**First recorded:** 2026-11-02 KST. **Code acceptance:** VERIFIED for restricted Source geometry candidate graph only. **Issue:** [#365](https://github.com/july-rust/BoardSI/issues/365). **Code PR:** [#366](https://github.com/july-rust/BoardSI/pull/366).

## Scope and model

The graph is an opt-in, deterministic derivative of the **actual PADS V9.4 MILS imported geometry re-qualified by D_010**, not an electrical connectivity extractor. It requires a single declared plated Through-Via with two distinct conductive circular pads on layers A/B; four source-owned rectangular COPCLS areas A/C and B/D (each with optional owned rectangular COPCUT); and exactly two straight same-net traces, one on each layer, with strictly internal and independently checked endpoints.

- Layer A graph: **ViaPad_A, Area_A, Trace_A, Area_C**, with local candidate edges ViaPad_A↔Area_A, ViaPad_A↔Trace_A and Trace_A↔Area_C.
- Layer B graph: **ViaPad_B, Area_B, Trace_B, Area_D**, with local candidate edges ViaPad_B↔Area_B, ViaPad_B↔Trace_B and Trace_B↔Area_D.
- Exactly **8 source-bound nodes / 6 layer-local SOURCE_CONTACT_CANDIDATE edges / 2 layer-local components / 0 cross-layer electrical edges**; canonical area source IDs are retained, while traces use deterministic SHA-bound layer/net/geometry-derived keys.
- Receipt embeds D_010's independent Source-only witness, source SHA, exact bounded topology and explicit NOT ELECTRICAL end marker. This is a geometrical contact *candidate*, and an absent edge does not establish an actual manufactured electrical open.
- Input: `boardsi-import-artwork board.asc --d011-source-contact-graph-v1 RA RB RC RD NET_A 1 4 0`. Dedicated test binary: `boardsi-d011-source-contact-graph`. Default artwork protocol and earlier D_001–D_010 contracts remain unchanged.

D_011 declines a graph when D_010 reports noncandidate geometry or rejects missing Via pad, wrong net/layer/source ownership, COPCUT/boundary, unsupported geometry, stale source SHA and unbounded or forged data. Qt does not trust graph rows: it independently recomputes the D_010 Via/Trace/Area contacts from the actually accepted imported Source, then re-materializes and exactly compares the supplied graph. Re-import clears obsolete risk markers; negative inputs do not leave a false accepted graph.

## Reproducible verified code evidence — 2026-11-02 KST

- [x] **Code PR [#366](https://github.com/july-rust/BoardSI/pull/366) exact final HEAD `d1fed0ca9b8a93ebc8eea7a801b1f6642db52591`: 27/27 PR Check Runs SUCCESS and 11/11 independently enumerated `event=pull_request` workflows SUCCESS**. Rust 1.75 fmt and strict Clippy, Debug/Release dedicated D_011 integration tests 3/3 each, D_001–D_010 compatibility retained.
- [x] **Qt Linux x86_64 Qt 6.8.3 Offscreen CTest 34/34 PASS** including `qt-preview-d011-source-contact-graph` and logged `D011_NATIVE_QT_SOURCE_GRAPH true true true true true true true NO_ELECTRICAL_BARREL_EDGE`. Independent D_011 actual imported Source smoke step SUCCESS on **Windows x64, Windows ARM64, and macOS ARM64**.
- [x] **Independent exact-HEAD Codex review [PR #366](https://github.com/july-rust/BoardSI/pull/366)**: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; open review threads = 0.
- [x] **Protected GitHub Signed Verified one-parent Squash**: [`main@53efa8b64681eccf0c6ba0309206a3871752a93d`](https://github.com/july-rust/BoardSI/commit/53efa8b64681eccf0c6ba0309206a3871752a93d), single parent `e3bd0e02ebd68c9de49083d26916686c226c5d78`, `verification.verified=true`, `reason=valid`.
- [x] **Independent exact post-code-main SHA** [21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/53efa8b64681eccf0c6ba0309206a3871752a93d/checks) and **8/8 `event=push` workflows SUCCESS**, all `head_sha=53efa8b64681eccf0c6ba0309206a3871752a93d`. The four CodeQL language analyses finished successfully as part of these checks.

## Excluded / explicitly NOT accepted

```text
source_contact_candidate_graph = QUALIFIED_IN_NARROW_FIXTURE
manufactured_copper_and_fill = NOT_VERIFIED
via_barrel_electrical_continuity = NOT_VERIFIED
cross_layer_electrical_edges = NOT_PROVEN
global_open_short = NOT_VERIFIED
physical_return_path_p009 = UNSUPPORTED
ph08_and_physical_si = NOT_RUN
solver_ready = false
cache_reuse_eligible = false
formal_m1_u09_u10_u39_t02_t04_t24_t52 = NOT_ACCEPTED
```

These are **not** formal M1 or physical SI results. [Parent Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN and [Draft PR #35](https://github.com/july-rust/BoardSI/pull/35) remains unchanged and unmerged.

## Separate documentation and issue gate

The documentation PR that introduces this ledger and the development checklist update requires its own exact-final-HEAD required checks, independent Codex zero unresolved findings, protected Verified one-parent Squash and independent exact-document-main `event=push` CI. **Do not close Issue #365 until those final independent gates pass.** Record verified documentation acceptance in the documentation PR and Issue #365 comments without backdating the verification.
