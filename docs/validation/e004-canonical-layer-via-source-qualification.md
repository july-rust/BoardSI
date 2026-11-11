# E_004 — Canonical Layer/Via Source Provenance & Fail-Closed Qualification

**Started:** 2026-11-11 KST. **Scope:** one read-only SOURCE hole→physical copper-layer envelope observation contract, not board-level Via/electrical/SI acceptance.

## Exact contract

`BoardModel::observe_source_via_hole(source, hole)` is implemented in `crates/boardsi-model/src/source_layer_via.rs`. It:

- Refuses an invalid Canonical model or unknown SourceId, unverified zero SHA, ambiguous SHA-to-file mapping, mismatched Layer SourceId, invalid/duplicate Layer order, and invalid Source span.
- Resolves an explicitly declared 1-based physical copper ordinal against the **sorted physical Copper layer set**, never an arbitrary internal layer order or a hard-coded 2/4/6-layer assumption; 2/4/6/8/64 cases share one algorithm.
- Returns logical source file, source line, original source-bound layer object identities, and the **physical envelope**, not connected layers.
- Retains `Unknown`, `SourceDeclaredPlated`, `NonPlated`, and unqualified grammar independently. Existing `FabricationHole::has_verified_manufactured_barrel()` remains false.
- Returns `Missing` or `Assumed` stackup properties separately; examines **all intervening physical layers, including dielectric layers**, not just Copper endpoints. Does not convert missing z/material/thickness to confirmed.
- **Never** creates `Via`, `Drill`, `Pad`, `ConnectivityEdge`, `Via.connected_layers`, or a solver result.

This bounded API is additive; it does not replace existing Via/Drill constructors or assert that all importers call it. `Drill.plated: bool` cannot express manufacturing uncertainty, so **do not copy `SourceDeclaredPlated` into that field without a separately qualified, explicit promotion contract**.

## Synthetic regression matrix

| Scope | Assertions | Expected |
| --- | --- | --- |
| Layer dimensions | 2, 4, 6, 8, 64; full and interior spans | Exact Source-owned layer IDs and envelope length |
| Source identity | wrong SHA, ambiguous identical-SHA files, cross-source layer mapping | `InvalidProvenance` or explicit validation error |
| Span integrity | zero, equal endpoints, reversed, out-of-range, duplicate order | Reject, no fallback |
| Uncertainty | unknown/source-declared plating, unsupported grammar, missing stackup | Separate states preserved; no electrical promotion |
| Intervening dielectric | two Confirmed Copper faces with missing then Assumed dielectric | Stackup Missing then Assumed; never fake Explicit |
| Electrical isolation | same physical span and geometry but no verified copper contacts | No auto-created Via/Drill/Graph edges |

All tests are **synthetic software contracts**. They cannot prove native PADS/ODB++ completeness, actual PCB manufacturing, solder mask, plating reliability, cross-board electrical connectivity, or SI results.

## CI and merge evidence

**Proven before final documentation update:** Code ancestor `aff56884786851124de0fe95f3752198292857a4` passed Rust 1.75 format/warnings and actual `Numerical core` PR CI [run #42971305714](https://github.com/july-rust/BoardSI/actions/runs/42971305714), including all **six new E_004 tests PASS under both Debug and Release** and pre-existing model/import regression. Review on that exact code SHA completed with no additional findings. An earlier independent P2 finding — missing intervening dielectric uncertainty — was **actually corrected in code and the new mutation test**, then the original discussion thread was resolved after new independent review.

**Final release gate is conditional:** PR [#586](https://github.com/july-rust/BoardSI/pull/586) must still have exact final HEAD required checks, unresolved threads zero and Verified protected Squash before this bounded item can be reported COMPLETE. Verify exact merged `main` SHA with **separate** Push CI and record that in [Issue #4](https://github.com/july-rust/BoardSI/issues/4) and [Issue #520](https://github.com/july-rust/BoardSI/issues/520). An ancestor run is not a substitute for final HEAD or main Push.

- Final PR HEAD checks: **PENDING final gate**
- Independent review: **PASS on code ancestor aff568; final HEAD must be rechecked**
- Verified Squash / exact main SHA: **PENDING**
- Separate exact main Push CI: **PENDING**

## Remaining gates

Parent [#4](https://github.com/july-rust/BoardSI/issues/4) and [#520](https://github.com/july-rust/BoardSI/issues/520) retain full-board/cross-format/physical qualification DoD. ODB++ manufacturing-hole→Canonical Via promotion is [E_019](../e-stage-closure-plan.md); Source Graph is E_022. Blocking/dependency criteria remain in [E register](../e-stage-blocked-register.md). Neither parent issue is closed on E_004 alone.
