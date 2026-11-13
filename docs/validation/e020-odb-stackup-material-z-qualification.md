# E_020 — ODB++ Stackup Material/Thickness/z SOURCE Qualification

**Scope:** Siemens ODB++Design Release 8.1 Update 4, optional `matrix/stackup.xml`, EDA design `StackupFile/EdaData` only. **Evidence date:** 2026-11-13 KST. **Current status:** CODE_MERGED / EVIDENCE_QUALIFIED — both squash commits are confirmed cryptographically Verified; exact code-main Push Numerical Core retry 2 is SUCCESS, Qt Runtime retry 2 remains IN_PROGRESS. E_020 is **NOT YET COMPLETE** until Qt finishes on the exact code-main SHA and final docs closeout is verified.

## Contract

- New opt-in `boardsi_import::odb_stackup::{audit_unpacked_stackup,audit_path_stackup}` reads ODB++ bounded FileMap and reuses existing geometry import for selected step and source identity.
- `StackupFile` XML parsing is explicitly bounded (2 MiB input, 8192 nodes, depth 24); DTD, XInclude, processing instructions other than initial XML declaration and all external entities are rejected. Only five named XML character entities are decoded; full W3C XML conformance and unrestricted vendor interoperability are **NOT_CLAIMED**.
- The SOURCE-only sidecar retains the verified ODB source identity and separate SHA-256 of `matrix/matrix` and `matrix/stackup.xml`; each layer retains its source line, names, type, nominal/finished thickness, optional frequency-specific Dk/Df, and exact Rational top-face z derived from complete nominal layer thicknesses.
- Canonical z increases from **top to bottom** (`boardsi-model::Layer::z_m`), with top face z=0. This is a derived *internal relative* coordinate convention, not source-asserted absolute z or measured fabrication thickness. Previous user-facing draft text proposing a bottom-origin z was inconsistent with the existing Canonical model and is intentionally not implemented.
- EDA material specification references and ordered Copper names are cross-checked against `matrix/matrix`, which takes precedence per the Siemens spec (2024, pp.51–53); SupplierData is not used to fabricate EDA facts. Unqualified material/property ambiguity and unsupported XML structures fail closed.
- The existing `BoardModel` is unchanged: no confirmed property, fabricated material, conductive Via, manufactured barrel, global connectivity edge, Solver or SI result is produced.

## Focused synthetic regression matrix

`crates/boardsi-import/tests/e020_odb_stackup.rs` covers 2/4/6/8/64-Copper layers; exact SI rational thickness and z; FrequencyVal/GHz Dk/Df; optional absent XML and Supplier-only nonpromotion; hostile malformed XML; invalid references; thickness/precision and unit validation; source SHA changes; and electrical graph nonpromotion. Existing D_056_08/E_013–019 and PADS regressions are preserved.

No independent real ODB exporter fixture, manufacturing stackup, material measurement, dielectric dispersion model, full XML schema/XSD engine, design-to-supplier material reconciliation, formal product U/T or Solver input qualification is accepted.

## Final code PR evidence — 2026-11-13 KST

| Gate | Independently observed evidence |
| --- | --- |
| Code PR | [#617](https://github.com/july-rust/BoardSI/pull/617), **MERGED** |
| Final code HEAD | `d03e64ae19c90704bb79a6a4d8340a5b6dcc2c0e` |
| Final HEAD Actions | **9/9 GitHub workflows SUCCESS, 25/25 jobs SUCCESS, 0 failed, 0 skipped**; [Numerical #43386906295](https://github.com/july-rust/BoardSI/actions/runs/43386906295), [Qt #43386906273](https://github.com/july-rust/BoardSI/actions/runs/43386906273), [Planning #43386906396](https://github.com/july-rust/BoardSI/actions/runs/43386906396); other M2/A032 workflows confirmed on exact HEAD |
| Native source regressions | Linux numerical; Windows x64, Windows ARM64, macOS ARM64 Rust; Linux Qt offscreen and all three native Qt targets success on exact HEAD |
| Independent Codex review | Initial automated Codex review on earlier HEAD `4ec3acb874` found **2 P2**; nested Group and multi-Property thickness ambiguity fixed, each separately tested, **2/2 threads resolved, 0 unresolved**. Later final-HEAD Codex review requested but no final-HEAD response captured; do **not** claim final-HEAD independently approved |
| Protected Squash merge | GitHub accepted protected single-parent `squash` merge; exact main SHA [`d96e3b9f4be443a634aa964ad9fbb1894e6c8147`](https://github.com/july-rust/BoardSI/commit/d96e3b9f4be443a634aa964ad9fbb1894e6c8147). GitHub REST `/git/commits/{sha}` **verification.verified=true; reason=valid; single parent** independently confirmed |
| Exact merged-code main `event=push` | Original code-main `d96e3b9f` eight Push runs: six SUCCESS, Numerical Core and Qt Runtime CANCELLED by later docs-main Push. Independently verified via GitHub REST `actions/runs?event=push&head_sha=`. **Numerical retry attempt 2 [#43388890660](https://github.com/july-rust/BoardSI/actions/runs/43388890660): SUCCESS, 5/5 jobs**. **Qt retry attempt 2 [#43388890573](https://github.com/july-rust/BoardSI/actions/runs/43388890573): IN_PROGRESS**, do not claim eight successes yet |
| Documentation closeout | [PR #618](https://github.com/july-rust/BoardSI/pull/618) MERGED, docs-main [`0eeeb21824646b0078e6019c8edc48fa678cd525`](https://github.com/july-rust/BoardSI/commit/0eeeb21824646b0078e6019c8edc48fa678cd525) `verification.verified=true; reason=valid; single parent`, exact SHA docs-only `event=push` 8/8 workflows SUCCESS, 8 jobs successful + 16 intentionally SKIPPED |

Actual test log attempts with format/Clippy failures or cancellations before the final green HEAD are preserved on GitHub. Source-only software acceptance is implemented and merged, but **E_020 remains EVIDENCE_QUALIFIED** until the outstanding exact code-main Qt Runtime Push rerun SUCCESS and guarded final docs closeout are independently checked. Initial independent Codex review (earlier HEAD) and resolved P2 findings are preserved; a fresh final-code-HEAD Codex review receipt was requested but not received, and must not be claimed. Do not promote these missing process gates to PASS.

**Status remains EVIDENCE_QUALIFIED / NOT_FINAL_ACCEPTED until every required completion gate above is verified.** Upper-level [#10](https://github.com/july-rust/BoardSI/issues/10) remains OPEN. Historical [#578](https://github.com/july-rust/BoardSI/issues/578) is administratively CLOSED, not proof of whole-ODB++ acceptance. E_021 Main Qt import, E_022 source-graph, E_023 vendor oracle and E_024 overall acceptance stay unaccepted.

## Sources

- [Siemens ODB++Design Format Specification 8.1 Update 4 (August 2024)](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf), pp.51–61 and Appendix E pp.313–375.
- [BoardSI single development checklist](../development-checklist.md)
- [E_019 source-only drill/via qualification](e019-odb-drill-via-candidate-qualification.md)
