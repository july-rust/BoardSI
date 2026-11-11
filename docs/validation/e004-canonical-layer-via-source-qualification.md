# E_004 — Canonical Layer/Via Source Provenance & Fail-Closed Qualification

**Status:** **COMPLETE (bounded SOURCE software contract; 2026-11-11 KST).** **First checked:** 2026-11-11 KST. **Evidence date:** 2026-11-11 KST. **Scope:** one read-only SOURCE hole→physical copper-layer envelope observation contract, not board-level Via/electrical/SI acceptance.

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

## Final CI, review, protected merge and independent main Push — verified

- **Implementation PR:** [#586](https://github.com/july-rust/BoardSI/pull/586); exact **final PR HEAD `85996aabb603fa2b63af098a9284e9a0a9e4eee9`**, **38/38 completed Checks SUCCESS**, 0 failed/pending. Includes Rust-format/warnings, model tests, native Windows x64/ARM64 and macOS ARM64, Linux Qt and Qt native platform validations. [PR Checks](https://github.com/july-rust/BoardSI/pull/586/checks).
- **Debug + Release Oracle:** all **six added `source_layer_via::tests` PASS**; observed in [Numerical Core PR run #42971305714](https://github.com/july-rust/BoardSI/actions/runs/42971305714) on code ancestor `aff56884786851124de0fe95f3752198292857a4`; that same code was unchanged in the final PR HEAD, which separately passed its own required CI. Preexisting model/import regressions also passed.
- **Independent review:** final exact HEAD `85996aabb603fa2b63af098a9284e9a0a9e4eee9` review **completed, findings 0** and **unresolved review threads 0**. One prior P2 finding about intervening dielectric uncertainty was fixed in the code and covered by a new synthetic regression. The original review thread was resolved only after correction.
- **Protected merge:** [Verified single-parent Squash commit `de87983a6af2f55423a49881eaacae690fe3b50f`](https://github.com/july-rust/BoardSI/commit/de87983a6af2f55423a49881eaacae690fe3b50f) with GitHub signature `verified=true`, `reason=valid`. This SHA was separately checked against actual `main`.
- **Independent, exact merge-SHA main Push:** **8/8 workflow runs SUCCESS**, 0 failed/cancelled/pending, all `event=push` and `head_sha=de87983a6af2f55423a49881eaacae690fe3b50f`. This is **not** a count of PR checks:
  - [Numerical core #42974419265](https://github.com/july-rust/BoardSI/actions/runs/42974419265)
  - [Qt runtime #42974419276](https://github.com/july-rust/BoardSI/actions/runs/42974419276) — includes Windows x64, Windows ARM64, macOS ARM64, Linux Qt
  - [M2 synthetic G0 #42974419460](https://github.com/july-rust/BoardSI/actions/runs/42974419460)
  - [PH-06 #42974419333](https://github.com/july-rust/BoardSI/actions/runs/42974419333)
  - [Docs #42974419318](https://github.com/july-rust/BoardSI/actions/runs/42974419318)
  - [Native FEM #42974419268](https://github.com/july-rust/BoardSI/actions/runs/42974419268)
  - [Lossless FEM #42974419346](https://github.com/july-rust/BoardSI/actions/runs/42974419346)
  - [PH-07 #42974419296](https://github.com/july-rust/BoardSI/actions/runs/42974419296)

**Bounded E_004 acceptance:** COMPLETE for source-bound Layer/Via envelope observation, provenance validation and fail-closed synthetic regressions. This is **not** whole-board or manufactured Via continuity, native format completeness, verified electrical graph, M0 T21/T51 product acceptance or SI certification.

## Remaining gates

Parent [#4](https://github.com/july-rust/BoardSI/issues/4) and [#520](https://github.com/july-rust/BoardSI/issues/520) retain full-board/cross-format/physical qualification DoD. ODB++ manufacturing-hole→Canonical Via promotion is [E_019](../e-stage-closure-plan.md); Source Graph is E_022. Blocking/dependency criteria remain in [E register](../e-stage-blocked-register.md). Neither parent issue is closed on E_004 alone.
