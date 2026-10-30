# C_016 — Source-Exact Reference Stitch Via Full Pad / Mixed Cutout Clearance V12

**Tracker:** [Issue #284](https://github.com/july-rust/BoardSI/issues/284) · **Code:** [PR #285](https://github.com/july-rust/BoardSI/pull/285) · **Parent:** [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12), **OPEN**.

**Verdict: LIMITED PASS — code acceptance for original-source bounded geometry and preview risk visualization ONLY.** No physical PCB, Return Path, manufactured DRC or SI acceptance.

**First checked (KST):** 2026-10-30 · **Evidence date (KST):** 2026-10-30.

## Exact evidence / acceptance

| Gate | Evidence | Result |
|---|---|---|
| Original implementation exact PR HEAD | [`21dc6f8347568e777ad1182f38c00ae41f0c4436`](https://github.com/july-rust/BoardSI/commit/21dc6f8347568e777ad1182f38c00ae41f0c4436/checks) | **27/27 Check Runs SUCCESS**, **12/12 PR Workflows SUCCESS** |
| Rust strict integer Source tick exact geometry and bounded negative/positive cases | `crates/boardsi-import/src/reference_route_v12/geometry.rs`, `tests/c016_reference_route_v12.rs`; exact PR HEAD Numerical core, Native Debug/Release | **PASS** |
| Qt genuine importer → V12 original Source receipt → independently checked Preview risk + stale invalidation | `apps/boardsi-qt-prototype/c016_stitch_pad_risk.h`, `--c016-stitch-pad-selftest`, `.github/workflows/qt-runtime.yml` | **PASS**, Windows x64/Windows ARM64/macOS ARM64, Linux offscreen |
| Same-head Codex independent code review | [PR #285](https://github.com/july-rust/BoardSI/pull/285): `commit_sha=21dc6f8347568e777ad1182f38c00ae41f0c4436`, `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; all **3/3** previous inline finding threads resolved | **PASS** |
| Strict Ruleset and Squash-only protected main | [PR #285 merged](https://github.com/july-rust/BoardSI/pull/285), [`main@4b5cb04ca537f8d9b4400978dfb3104ef744c59f`](https://github.com/july-rust/BoardSI/commit/4b5cb04ca537f8d9b4400978dfb3104ef744c59f), `verified=true`, `reason=valid`, **one parent** | **PASS** |
| Independent post-code-main exact SHA (not PR checks) | [main merge SHA Checks](https://github.com/july-rust/BoardSI/commit/4b5cb04ca537f8d9b4400978dfb3104ef744c59f/checks); GitHub `event=push` workflow runs | **21/21 Check Runs SUCCESS**, **8/8 Push Workflows SUCCESS** |
| Living checklist | [`docs/development-checklist.md`](../development-checklist.md), scoped checkbox with original first-checked date, code PR/review and post-main evidence | **Limited code scope checked** |

The first CI attempts had formatting and test assertion errors; an independent Codex review identified Source Area hex decoding and Qt marker-selection errors. These were corrected **before** the final HEAD. Intermediate failed/cancelled attempts are **not** represented as a final PASS.

## Supported geometry / provenance contract

- Opt-in `--reference-route-v12`; original verified Source PADS V9.4 MILS *subset*, selected two distinct Reference Layers and one explicitly selected plated Reference Stitch Via. Existing V1–V11 command outputs remain separate.
- Every selected Source Layer has exactly one owner-bound rectangular Reference Copper area. Its 0–8 Source-owned, mutually disjoint, strictly enclosed rectangular `COPCUT` / circular `COPCCO` holes are checked using integer Source ticks (0.01 mil = 254 nm) and analytic round-Pad distances. No polygon tessellation is used as the predicate.
- Full circular **conductive Stitch Via Pad** versus outer Copper rectangle and every inner void, with distinct `STITCH_PAD_FULL_COPPER_FOOTPRINT_CANDIDATE`, `STITCH_PAD_CUTOUT_TANGENCY_RISK`, `STITCH_PAD_CUTOUT_INTERSECTION_RISK`, `STITCH_PAD_OUTER_TANGENCY_RISK` and `STITCH_PAD_OUTER_CROSSING_RISK` classifications. No manufacturing clearance threshold is inferred.
- The original **byte-exact V11** is embedded in V12, including actual V8 Source-local candidate ancestry. The Qt side independently verifies original SHA, Source Area ID (hex decoded), Net, Layer, selected stitch and circular pad, V11 Receipt and source Copper geometry. Risk markers use Source-sized round Pad rims in the existing CPU/RHI artwork rendering paths. Source reimport/change, selection/settings changes and stale IPC invalidate results.
- Unsupported/ambiguous source geometry, incorrect Source identifiers, unknown pad shape, stale ancestry, illegal void contacts or invalid coordinates **fail closed**, not as safe Copper.

## Explicitly unaccepted

- Actual copper flood continuity, electrical connection, return-current distribution, thermal/antipad implementation, manufactured DRC clearance, short/open proof, substrate stackup, impedance, RLGC, S parameters, or validated physical SI.
- Arbitrary PADS geometries, other ASCII revisions and all externally fabricated PCB samples. Synthetic regression cannot be treated as manufacturing qualification.
- The C_016 source-only *limited* code verdict does **not** close any formal U09/U10/U39/T02/T04/T24/T52 or parent M1 Issue #12 requirements.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

This evidence ledger records **code and post-code-main** proof. The documentation PR has its **own** exact-head, independent review, protected signed Squash and post-document-main gates; those are verified separately on the GitHub PR/Issue and are not pre-certified by this code verdict.
