# D_012 — PADS Through-Via source span and plating provenance qualification

**First recorded:** 2026-11-02 KST. **Issue:** [#368](https://github.com/july-rust/BoardSI/issues/368). **Code PR:** [#369](https://github.com/july-rust/BoardSI/pull/369). **Scope:** narrow, opt-in **PADS Source geometry and provenance only**; not electrical or physically validated.

## Exact accepted Source-only feature
- The bounded, deterministic `BOARDSI_D012_VIA_SPAN_PROVENANCE_V1` receipt embeds the independently requalified D_011 graph and includes the original SHA-256, Via Net, PADS source Via position, pad/drill dimensions, source first/last layers and selected copper layers.
- The D_010/D_011 prerequisite independently reconciles Source/CANONICAL Via, drill/span, two declared pads, two same-net traces, and four source-owned copper areas. The D_011 graph remains **8 nodes, 6 intra-layer candidate edges, 2 local components and 0 cross-layer electrical edges**.
- The `plated: true` inside `route_preview::into_canonical` is an **IMPORTER ASSUMPTION**, not explicitly source-verified manufacture or a barrel-conduction proof. The D_012 receipt says **`PLATING_IMPORTER_ASSUMED`, `BARREL_NOT_VERIFIED`, `NO_CROSS_LAYER_ELECTRICAL_EDGE`**. It must never be promoted to Source-verified plating or connectivity.
- Bounded 64,000-byte opt-in witness only; existing importer format, old D_001–D_011 commands, public model and ordinary viewer behavior remain unchanged. Qt reconstructs D_011 using **actual accepted imported artwork** and demands exact recomposition of D_012 with matching source hash. Stale source changes, forged span/plating, wrong Via Net/Layer and unsupported geometry are rejected; no risk markers survive a stale reimport.
- CLI opt-in command: `boardsi-import-artwork board.asc --d012-via-span-provenance-v1 RA RB RC RD NET_A 1 4 0`; separate Rust test binary `boardsi-d012-via-span-provenance`; independent native Qt selftest `--d012-via-span-provenance-selftest`.

## Code acceptance — independently checked on 2026-11-02 KST
- [x] [PR #369](https://github.com/july-rust/BoardSI/pull/369) exact final HEAD [`8ecf5fefc83898a2c0696d0fa2bdae63f739b62d`](https://github.com/july-rust/BoardSI/commit/8ecf5fefc83898a2c0696d0fa2bdae63f739b62d/checks): **27/27 PR check runs SUCCESS** and **11/11 independently enumerated event=pull_request workflows SUCCESS** (plus dynamic PR success).
- [x] Rust 1.75 fmt, strict Clippy, D_012 three Rust test cases, existing workspace regressions, Linux Qt Offscreen and actual **Windows x64, Windows ARM64, macOS ARM64 D_012 native Qt smoke** SUCCESS; negative forged plating/span, stale/reimport, bad selection and boundary input included.
- [x] Same exact-HEAD [Codex independent review](https://github.com/july-rust/BoardSI/pull/369) **completed; any findings=false; unresolved findings=false; review threads 0**. An earlier D_012 hardcoded fixture drill expectation was corrected to assert the source-owned Via row; old outdated thread resolved.
- [x] GitHub protected Verified, **one-parent Squash**: [`main@cee3941f977a48c4de16b4837a78597b18918034`](https://github.com/july-rust/BoardSI/commit/cee3941f977a48c4de16b4837a78597b18918034) (`verification.verified=true`, `reason=valid`, sole parent `22dccc35a60dd272901ebfa673de359a91777186`).
- [ ] Independent post-code-main Push CI final acceptance. On initial main SHA, **20/21 check runs** and **7/8 event=push workflows** succeeded; Windows x64 `Numerical core` failed in legacy **D_006 `different_source_bytes_cannot_reuse_sha`** with `Os { code: 2, NotFound }` on the host. Failed Workflow [run #41428236843](https://github.com/july-rust/BoardSI/actions/runs/41428236843) was re-run failed-jobs-only, attempt 2; **must independently recheck final attempt before changing this row to [x]**.

## Excluded / explicitly NOT accepted

```text
pads_source_via_span = QUALIFIED_ON_RESTRICTED_SOURCE_FIXTURE
plating_provenance = IMPORTER_ASSUMED
manufactured_plating = NOT_VERIFIED
via_barrel_electrical_continuity = NOT_VERIFIED
cross_layer_electrical_edges = NOT_PROVEN
manufactured_copper_and_fill = NOT_VERIFIED
global_open_short = NOT_VERIFIED
physical_return_path_p009 = UNSUPPORTED
ph08_and_physical_si = NOT_RUN
solver_ready = false
cache_reuse_eligible = false
formal_m1_u09_u10_u39_t02_t04_t24_t52 = NOT_ACCEPTED
```

Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN; [Draft PR #35](https://github.com/july-rust/BoardSI/pull/35) stays unmerged. Geometry Source witness is **not** a physical plated-barrel validation, nor does an absent candidate edge mean a manufactured open.

## Documentation PR and Issue closure gate
Documentation changes require their **own** exact-final-HEAD required checks, independent Codex zero unresolved, protected Verified one-parent Squash and independent exact post-doc-main push CI. Close Issue #368 **only once both code-main and document-main Push CI acceptance succeed**; publish the final evidence in the Issue and documentation PR without backdating. The present ledger is the gated documentation record, not proof that the subsequent doc PR gates have already occurred.
