# E_004 — Canonical Layer/Via Source Provenance & Fail-Closed Qualification

**Started:** 2026-11-11 KST. **Scope:** one read-only SOURCE hole→physical copper-layer envelope observation contract, not board-level Via/electrical/SI acceptance.

## Exact contract

`BoardModel::observe_source_via_hole(source, hole)` is implemented in `crates/boardsi-model/src/source_layer_via.rs`. It:

- Refuses an invalid Canonical model or unknown SourceId, unverified zero SHA, ambiguous SHA-to-file mapping, mismatched Layer SourceId, invalid/duplicate Layer order, and invalid Source span.
- Resolves an explicitly declared 1-based physical copper ordinal against the **sorted physical Copper layer set**, never an arbitrary internal layer order or a hard-coded 2/4/6-layer assumption; 2/4/6/8/64 cases share one algorithm.
- Returns logical source file, source line, original source-bound layer object identities, and the **physical envelope**, not connected layers.
- Retains `Unknown`, `SourceDeclaredPlated`, `NonPlated`, and unqualified grammar independently. Existing `FabricationHole::has_verified_manufactured_barrel()` remains false.
- Returns `Missing` or `Assumed` stackup properties separately; does not convert missing z/material/thickness to confirmed.
- **Never** creates `Via`, `Drill`, `Pad`, `ConnectivityEdge`, `Via.connected_layers`, or a solver result.

This bounded API is additive; it does not replace existing Via/Drill constructors or assert that all importers call it. `Drill.plated: bool` cannot express manufacturing uncertainty, so **do not copy `SourceDeclaredPlated` into that field without a separately qualified, explicit promotion contract**.

## Synthetic regression matrix

| Scope | Assertions | Expected |
| --- | --- | --- |
| Layer dimensions | 2, 4, 6, 8, 64; full and interior spans | Exact Source-owned layer IDs and envelope length |
| Source identity | wrong SHA, ambiguous identical-SHA files, cross-source layer mapping | `InvalidProvenance` or explicit validation error |
| Span integrity | zero, equal endpoints, reversed, out-of-range, duplicate order | Reject, no fallback |
| Uncertainty | unknown/source-declared plating, unsupported grammar, missing stackup | Separate states preserved; no electrical promotion |
| Electrical isolation | same physical span and geometry but no verified copper contacts | No auto-created Via/Drill/Graph edges |

All tests are **synthetic software contracts**. They cannot prove native PADS/ODB++ completeness, actual PCB manufacturing, solder mask, plating reliability, cross-board electrical connectivity, or SI results.

## CI and merge evidence

To be filled with **observed** results after final PR HEAD and protected merge; do not prelabel unexecuted checks as PASS.

- PR and final exact head: **PENDING**
- Rust/Native CI outcomes by workflow: **NOT_RUN / PENDING**
- Review and unresolved threads: **PENDING**
- Verified Squash and exact main SHA: **PENDING**
- Separate main Push CI: **PENDING**

## Remaining gates

Parent [#4](https://github.com/july-rust/BoardSI/issues/4) and [#520](https://github.com/july-rust/BoardSI/issues/520) retain full-board/cross-format/physical qualification DoD. ODB++ manufacturing-hole→Canonical Via promotion is [E_019](../e-stage-closure-plan.md); Source Graph is E_022. Blocking/dependency criteria remain in [E register](../e-stage-blocked-register.md). Neither parent issue is closed on E_004 alone.
