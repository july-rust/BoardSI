# B_001_2 — Rust Canonical Stackup / Material audit slice

**Date:** 2026-10-27 KST · **Issue:** [#12](https://github.com/july-rust/BoardSI/issues/12) · **PR:** [#215](https://github.com/july-rust/BoardSI/pull/215) · [Checklist](../development-checklist.md) · [Development spec §6](../specification/BoardSI_Development_Spec_v1.0.md).

## Actual bounded integration

- `crates/boardsi-model/src/stackup_configuration.rs` converts explicitly user-entered Layer/Material/Source evidence into the existing **Rust Canonical BoardModel**. Layer z is computed top-to-bottom, all values are `Property::Assumed` and `SourceIdentity` is bound to source SHA. No FR4/material property is auto-filled; no `Confirmed` properties are invented.
- Native `boardsi-stackup-config --audit-stdin-v1` accepts only schema `boardsi-b001-stackup-overlay-v2` as bounded UTF-8 JSON, enforces exact ordered source Layer IDs, integer micrometre thickness, finite Dk/Df/conductivity, source document/revision/method, frequency/temperature, explicit material valid band, distinct copper reference layer ID and valid analysis frequency. Out-of-band is **rejected**; never silently extrapolates.
- Qt editor displays material provenance and band columns with explicit analysis frequency input. A user-invoked button runs the **native Rust executable**, parses its strict audit receipt, verifies source digest and layer count, and reports **CANONICAL STACKUP ASSUMED / REFERENCE UNSUPPORTED / SI NOT_RUN** on success. No physical solver enablement. Qt status is invalidated when edits or audit frequency change.
- Original 10-column overlay schema v1 can be loaded **as incomplete input**, without upgrading provenance, and saved to v2. Reimport/source mutation invalidates previous overlay as in B_001. Source never uploaded.
- CI builds the actual native companion on Windows x64/ARM64, macOS ARM64 and Linux; Qt selftest exercises Qt→Rust successful assumed material audit and out-of-band failure. Rust cargo tests independently test missing/invalid/cross-frequency inputs and exact z calculation.

## Explicit limits / no overclaim

The parser-to-Canonical step currently constructs a bounded **material-only model** using the imported file SHA and manually supplied layer IDs; there is no full PADS Copper/Net/Pad/Port Canonical object import into this audit. An absent Board Outline is acknowledged as a separate full-board missing property, **not** a material error; all actual material misses/errors remain fatal. A reported material audit success cannot be passed to a solver. Reference status always **UNSUPPORTED_NO_COPPER_NET_EVIDENCE**, because a layer name or role hint does **not** establish return-path geometry, split/cutout continuity or a valid signal reference.

Remaining to close Issue #12: full imported Copper/Net/Area object mapping, reference return continuity (P009, T04, T24) and true source/reimport scenario cache invalidation, material loss/roughness/model validity and complete U09/U10/U39/T02/T04/T24/T52 acceptance, including independent real/qualified data. No frequency-domain SI, DC conductivity claim, 30 GHz result, proprietary input qualification or vendor certification is made.

## Validation ledger

| Scope | State | Evidence |
| --- | --- | --- |
| Rust Compiler/Clippy/fmt tests | **PENDING latest HEAD CI** | [PR #215 checks](https://github.com/july-rust/BoardSI/pull/215/checks) |
| Qt native Windows x64, Windows ARM64, macOS ARM64 | **PENDING latest HEAD CI** | [PR #215 checks](https://github.com/july-rust/BoardSI/pull/215/checks) |
| Linux CTest | **PENDING latest HEAD CI** | [PR #215 checks](https://github.com/july-rust/BoardSI/pull/215/checks) |
| Independent Codex review | **PENDING latest HEAD** | [PR #215](https://github.com/july-rust/BoardSI/pull/215) |
| Real Reference Copper continuity and production Solver | **NOT_IMPLEMENTED / BLOCKED** | [Issue #12](https://github.com/july-rust/BoardSI/issues/12) |
| Entire specification U09/U10/U39, T02/T04/T24/T52 | **NOT_QUALIFIED** | [Development checklist](../development-checklist.md) |

Do not promote tests from a stale head SHA or label a Material-only audit as physical return-path success.
