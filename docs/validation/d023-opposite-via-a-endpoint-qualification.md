# D_023 — Opposite Via-A collinear endpoint Source contact

- Tracker: [Issue #409](https://github.com/july-rust/BoardSI/issues/409)
- Initial record: 2026-11-04 KST
- Status: **PENDING — NOT ACCEPTED**
- Protected baseline: `29c3b20b11b8df72a108c7f012af1b8c1acc53cc`
- New opt-in: `--d023-collinear-opposite-endpoint-contact-v1 NET_A 1 4 0 1`
- Fixture: `crates/boardsi-import/tests/fixtures/d023_opposite_via_a_endpoint_v94.asc`

## Frozen bounded Source candidate contract

Same layer-4 main Source Trace 15000,25000 → 35000,25000; branch 15000,25000 → 5000,25000. Via-A Pad, Main and Branch all share the exact endpoint. Candidate triangle: Nodes 1/5/7 and extra Via-A Pad–Branch edge 1–7. Expected **8 nodes / 6 layer-local candidate edges / 3 local components / 0 proven cross-layer electrical edges**.

D_022's Via-B mode must reject this Via-A geometry and D_023's Via-A mode must reject D_022's Via-B geometry. 1-tick gaps/offsets, malformed Source, wrong Net/Layer, unknown artwork, stale/forged receipts and electrical promotion fail-closed.

## Evidence — awaiting independent verification

- Code PR final HEAD and required Rust + Qt + 3 native-platform regression: **PENDING**
- Exact final-head Codex review, zero findings and zero unresolved review threads: **PENDING**
- Protected Signed Verified one-parent Squash: **PENDING**
- Exact merged SHA post-main push CI: **PENDING**
- Separate Markdown-only documentation PR, checklist checked, Issue closed: **PENDING**

Only Source-geometry candidates are covered. No manufactured copper, plated barrel electrical continuity, global Open/Short, reference return path, P009, PH-08, RLGC/S solver, scenario/cache, or formal U/T/release acceptance. Parent Issue #12 remains OPEN, Draft PR #35 unmerged.
