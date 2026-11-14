# E_022 — ODB++ Source-qualified local contact / design graph qualification

**Evidence date:** 2026-11-14 KST. **Stage:** E_022. **Scope:** opt-in positive L straight Trace versus positive round P Pad, one native STEP, supported ODB++ 8.1-U4 grammar subset; same physical copper layer. This is a limited SOURCE *design* graph, not a manufactured PCB or SI proof.

## Implemented architecture

- Source: exact ODB++ OdbFileMap -> import_unpacked or import_unpacked_with_eda -> existing format-neutral BoardModel. Unsupported arc, surface, repeated STEP, other pad shapes, fabricated via/drill and uncertain grammar fail closed for this specifically scoped adapter. The general ODB importer remains unchanged.
- One immutable board and file-SHA manifest bound to step, SHA-256, Layer ObjectId, Trace/Pad ObjectId, actual source line and optional EDA FID design Net. The digest returned by a previous import can be passed as an expected digest to reject a mutated/re-imported board.
- Format-neutral representation: preexisting LayeredSourceGraph (CopperNode, GeometryContact/TraceSmdPad, DeclaredNetCheck SAME/DIFFERENT/UNKNOWN) and preexisting electrical_graph_qualification::qualify with SourceCoverage::Partial. A confirmed local positive overlap yields only QualifiedEdgeKind::SourceLocalPositiveArea. Never prepopulate verified_electrical_edges.
- Actual finite-width Straight Trace with round endcaps versus solid Round Pad: strict interior disk/capsule positive overlap. Calculated from relative source-coordinate ticks using conservative binary64 guard. Near tangent, source-tick precision limit, overflow or unsupported frame returns UNKNOWN/no edge. A missing candidate is **not** proof of electrical Open.
- No cross-layer XY contact, Net-name-only contact, plating or barrel qualification. No canonical Graph or SI solver mutation, no global open/short, no actual vendor/export or fabrication oracle.

## Independent synthetic oracle matrix

The code adds these tests inside boardsi-import/src/odb_local_graph.rs:

| Test | Oracle | Required |
| --- | --- | --- |
| independent_2_4_6_8_64_layer_positive_oracle | N strict positive local edges and N connected components for N layers | PASS |
| offcentre_endpoint_and_midspan_contacts | Midspan/off-center/general-angle/endpoint positive contact | PASS |
| separate_and_tangent_are_not_electrical_edges | No positive edge from distinct/tangent copper | PASS |
| same_xy_other_layer_is_not_contact | No layer-crossing connection from identical XY | PASS |
| declared_cross_net_conflict_and_unknown_net | Positive local different-Net conflict, Unknown Net preserved | PASS |
| same_net_does_not_invent_separated_edge | Same Net cannot fabricate contact | PASS |
| stale_sha_reimport_and_bad_eda_fail_closed | Stale manifest and unsupported FID rejected | PASS |
| unsupported_shapes_and_conductors_fail_closed | Arc/unsupported round-line symbol rejected | PASS |
| exact_source_identity_and_repeat_determinism | Source identity and edge order stable | PASS |
| one_instance_step_repeat_must_fail_closed | Reject a translated single-instance panel despite one frame per copper layer | PASS |

Additional required regressions: preexisting PADS graph, ODB Source Import/E_019/E_020, E_021 Qt Artwork/Picking/Import/ReImport and native runtime. All results must be recorded from actual CI, never inferred from this plan.

## CI/review/merge evidence ledger — 2026-11-14 KST

- **Code merged:** [PR #624](https://github.com/july-rust/BoardSI/pull/624) GitHub API returned `merged=true` after normal protected `squash`; merged-`main` SHA [`3dab514179bdb28513236df6c5dfcea938bb8055`](https://github.com/july-rust/BoardSI/commit/3dab514179bdb28513236df6c5dfcea938bb8055). Exact final code PR HEAD `f18824d5c59128a22f24c632fa3d2afbffe6bc80`.
- **Exact HEAD CI:** **9/9 workflow runs SUCCESS**, including [Numerical Core run #43533121289](https://github.com/july-rust/BoardSI/actions/runs/43533121289) and [Qt runtime run #43533121200](https://github.com/july-rust/BoardSI/actions/runs/43533121200). Docs/A032/G0/PH-06/PH-07/Native FEM/Lossless FEM also SUCCESS. Required PR checks were completed before protected squash.
- **Rust tests:** Format/Clippy strict `-D warnings` SUCCESS. Exact Numerical Core Linux log: `boardsi-import` **343/343 unit tests PASS**, including **10/10 E_022 module tests**, zero failed; other workspace/integration tests PASS as observed. No assumption of a manufactured PCB.
- **Native Rust:** Windows x64 SUCCESS, Windows ARM64 SUCCESS, macOS ARM64 SUCCESS **on job rerun attempt 2**. Preserve the historical first macOS attempt **FAILED** on existing unrelated `d006_via_three_area_cli::copper_cutout_blocks_candidate`; E_022's own 10 tests were PASS on that first attempt. Do not rewrite the first failure into success. The same SHA was retried without modifying application code and the latest macOS run returned SUCCESS.
- **Native Qt:** [Qt runtime #43533121200](https://github.com/july-rust/BoardSI/actions/runs/43533121200) Linux x64 offscreen, Windows x64, Windows ARM64 and macOS ARM64 all SUCCESS. These test E_021 regression compatibility; they do not validate actual manufacturing or hardware GPU graphics.
- **Independent review:** [Codex review](https://github.com/july-rust/BoardSI/pull/624) on earlier HEAD `421935ad` reported **two P2 findings**: NX=NY=1 STEP-REPEAT incorrectly admitted and potentially quadratic padstack validation before node-count preflight. Both defects corrected, an explicit single-instance negative fixture added, and **2/2 inline review threads resolved**. Re-review of exact final HEAD `f18824d5` **FAILED due Codex code-review usage quota**; no positive final-HEAD independent review receipt may be asserted.
- **Signature and independent post-merge Push:** Connector merge API confirmed successful Squash and its SHA, but cryptographic GitHub `verification.verified` and separately correlated exact code-`main` `event=push` workflow result were **NOT_VERIFIED** by the available tool access. PR-head 9/9 cannot substitute for independent code-main Push acceptance.
- **Formal bounded verdict:** `SOURCE GRAPH IMPLEMENTED; FINAL CODE PR CI 9/9 PASS; MERGED; LAST-HEAD INDEPENDENT REVIEW AND VERIFIED/POST-MERGE MAIN EVIDENCE NOT_VERIFIED`. This is an implementation closeout with a separate evidence gap, **not a fully verified protected-main Stage E acceptance**.
- **Excluded evidence:** Commercial ODB++ export/cross-CAD oracle, manufactured plating/barrel, global electrical Open/Short, SI readiness, whole Issue #10/#520: `NOT_VERIFIED`/not within the E_022 slice.

Do not change prior time-stamped failures or present unverified gates as PASS.

## Closure boundary

E_022 is only one SOURCE-local Trace/RoundPad positive contact slice. Historic issue #578 is administratively CLOSED; #10 and #520 remain OPEN until their full acceptance gates are met. E_023 independently owns real Vendor fixture + cross-format oracle. E_024 independently owns whole-product ODB gate. E_021 has separate unresolved merged-main CI/signature evidence; do not repair E_021 by claiming E_022 CI also proves E_021.
