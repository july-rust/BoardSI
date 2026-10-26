# A_032 / P2-W — limited local ROUTE endpoint connectivity

Status: **COMPLETE — A_032/A_032_1/A_032_2 P2-W limited local source-backed connectivity merged and verified 2026-10-26**

Historical start baseline: `54dc90ccd68a80ec1455c6947ce8a2229924ee26` (A_031 PR #194). Final P2-W integration: `51360acac465cbff394edf63b1363c60b635478c`.
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

Legacy Qt V1–V14 keeps its source-label semantics and must never present *source net alone* as proof of local contact. A_032_2 implements a strictly opt-in and independently validated `BOARDSI_CONTACT_RECORDS_V1` suffix after the legacy END, leaving default V1–V14 byte-identical. Object-level GPU/CPU markers and Cross-Net potential conflict remain bounded local source evidence, never full-board electrical verification. See [A_032_2 evidence ledger](a032-2-contact-viewer.md).

## Final verified closeout (2026-10-26 KST)

**A_032 / A_032_1 / A_032_2: COMPLETE for the bounded P2-W deliverable, not for all P2/M1.** The earlier candidate/pending language below has been reconciled to the following exact evidence:

- Implementation PR: [#196](https://github.com/july-rust/BoardSI/pull/196), final reviewed PR head `8d65fff379de6b64d03631ca48aa5d91fc52c11f`, squash-merged 2026-10-26 03:08:09 UTC.
- Protected `main` merge commit: [`51360acac465cbff394edf63b1363c60b635478c`](https://github.com/july-rust/BoardSI/commit/51360acac465cbff394edf63b1363c60b635478c), GitHub signature **Verified** (`valid`), one parent `54dc90ccd68a80ec1455c6947ce8a2229924ee26`.
- The merged commit's [post-merge checks](https://github.com/july-rust/BoardSI/commit/51360acac465cbff394edf63b1363c60b635478c/checks) completed **17/17 SUCCESS, 0 failed/pending**, matching the 17 required contexts of active main ruleset `#25046595`. At the exact PR head, 23/23 checks passed (including the 17 required native contexts).
- Independent Codex review of exact PR head `8d65fff`: completed, no remaining new findings; all six PR review threads **resolved (6/6)**. Issue [#195](https://github.com/july-rust/BoardSI/issues/195) **CLOSED**.
- [Release 10,000-segment synthetic route benchmark](https://github.com/july-rust/BoardSI/actions/runs/40261462513/job/121374561848): Rust importer 0.6213 / 0.6038 / 0.5814 s; peak child RSS ~57,452 KiB; output 955,611 bytes. **This is not end-to-end Qt GUI latency or real-board performance certification.**
- Synthetic Rust/Qt native tests and V10–V14 opt-in compatibility are covered by the exact-head PR checks. Hardware-GPU reset, VRAM capacity and 1M-object 60 FPS were **NOT_RUN/NOT_VERIFIED**, not release blockers for this scoped local-evidence milestone.
- Remains **NOT_VERIFIED**: whole-board Open/Short, interior/T-junction/trace-width or copper flood/thermal contacts, component Pad/Pin electrical contact, real vendor-export qualification, full SI/RLGC/S-parameters, complete M1/P2. Broad Issues [#9](https://github.com/july-rust/BoardSI/issues/9), [#11](https://github.com/july-rust/BoardSI/issues/11), [#165](https://github.com/july-rust/BoardSI/issues/165) stay open.

This closeout edits validation documentation only. Historical review issues, baseline numbers, and historical pre-merge candidate states are retained as history, not current blockers.

