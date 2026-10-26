# A_032 / P2-W — limited local ROUTE endpoint connectivity

Status: **A_032_2 IMPLEMENTATION CANDIDATE — EXACT-HEAD CI / INDEPENDENT REVIEW / MAIN MERGE PENDING**

Baseline protected main: `54dc90ccd68a80ec1455c6947ce8a2229924ee26` (A_031 PR #194).
Tracking: [Issue #195](https://github.com/july-rust/BoardSI/issues/195).

## What is source-backed

- `*ROUTE* *SIGNAL*` supplies **source-declared net membership** for supported V9.4 MILS Trace, Arc and Via geometry, not by itself a physical connection.
- Only co-located **exact original source tick endpoint pairs on the same layer**, with equal *canonical* source Net IDs and different source conductors, generate a `SameLayerContact` evidence edge. Trace-Trace, Trace-Arc, Arc-Arc, Trace-Via, Arc-Via are in scope. Via-Via coincidence does not infer contact.
- An explicitly plated Through Via with `Conductive` padstack entry on the corresponding layer has that layer added to `connected_layers` **only after** a real supported local trace/arc endpoint-to-via centre contact is found. Only two or more independently confirmed connected layers yield `ViaBarrel` evidence edges between the Via's own contact nodes.
- Different declared net IDs at the same supported endpoint create a **potential conflict counter** only; do **not** produce any `ConnectivityEdge` or a certified Short.
- The graph uses stable SourceIdentity-scoped Object IDs, deterministic ordering, bounded per-coordinate buckets and an edge-count cap. Canonical geometry, source provenance and original Net assignment are unchanged.
- Existing `BoardModel::validate()` continues checking referential, net, layer, position, via-padstack and plated-barrel evidence invariants.

## Deliberately NOT proven

| Scope | Status / reason |
| --- | --- |
| Near (one tick) endpoint, unrelated layers | No connection; there is no merge tolerance |
| Interior-line crossing, T-junction, width-overlap | NOT_EVALUATED, not a proved Open |
| SMD component Pad/Pin contact | NOT_EVALUATED; Pad and Pin Net still unverified |
| COPCLS/COPCIR source-filled copper and holes | NOT_EVALUATED; area contact needs a separate analytic proof |
| POUROUT, real plane flood, thermal relief, arbitrary plated stack | UNSUPPORTED / NOT_VERIFIED |
| Complete-board open/short, isolated copper island | NOT_VERIFIED |
| Vendor-export real-data qualification, 2D RLGC/S, GPU VRAM/reset/1M at 60 FPS | NOT_RUN |

Legacy Qt V1–V14 keeps its source-label semantics and must never present *source net alone* as proof of local contact. A_032_2 proposes a strictly opt-in and separately gated `BOARDSI_CONTACT_RECORDS_V1` suffix after the legacy END, leaving default V1–V14 byte-identical. Object-level GPU/CPU markers and Cross-Net potential conflict remain bounded local source evidence, never full-board electrical verification. See [A_032_2 evidence ledger](a032-2-contact-viewer.md).

## Acceptance requirements (not yet evidence)

- Rust `cargo fmt --check`, import/model/numerical tests and clippy clean.
- Native Qt Linux x64, Windows x64, Windows ARM64, macOS ARM64 compatibility, all older import/copper regressions.
- Exact-final-head independent Codex review, zero unresolved P1/P2 findings/threads.
- 17/17 required PR checks SUCCESS, unchanged ruleset #25046595, GitHub Verified Squash merge.
- Independent protected-main 17/17 post-merge checks and merge signature verification.

Any missing evidence must remain `NOT_RUN` or `BLOCKED`. Broad Issues #9/#11/#165 remain OPEN.
