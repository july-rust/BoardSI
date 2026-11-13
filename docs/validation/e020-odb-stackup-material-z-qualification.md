# E_020 — ODB++ Stackup Material/Thickness/z SOURCE Qualification

**Scope:** Siemens ODB++Design Release 8.1 Update 4, optional `matrix/stackup.xml`, EDA design `StackupFile/EdaData` only. **Evidence date:** 2026-11-13 KST. **Current status:** IMPLEMENTED_IN_PR / CI_AND_REVIEW_PENDING (do not mark COMPLETE without final receipts).

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

## Final evidence gate — fill from GitHub only after verifying

| Gate | Actual status |
| --- | --- |
| PR | [#617](https://github.com/july-rust/BoardSI/pull/617) |
| Final PR HEAD / required checks | PENDING; record exact SHA and success counts |
| Native Windows x64 / ARM64 / macOS ARM64 | PENDING |
| Independent review and unresolved threads | PENDING |
| Verified protected squash merge SHA | PENDING |
| Independent exact merged-code main push workflows | PENDING |
| Sole checklist and architecture evidence | This report is a pending PR artifact until merged |

**Status must remain IN_PROGRESS until every required completion gate above is verified.** Upper-level [#10](https://github.com/july-rust/BoardSI/issues/10) remains OPEN. Historical [#578](https://github.com/july-rust/BoardSI/issues/578) is administratively CLOSED, not proof of whole-ODB++ acceptance. E_021 Main Qt import, E_022 source-graph, E_023 vendor oracle and E_024 overall acceptance stay unaccepted.

## Sources

- [Siemens ODB++Design Format Specification 8.1 Update 4 (August 2024)](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf), pp.51–61 and Appendix E pp.313–375.
- [BoardSI single development checklist](../development-checklist.md)
- [E_019 source-only drill/via qualification](e019-odb-drill-via-candidate-qualification.md)
