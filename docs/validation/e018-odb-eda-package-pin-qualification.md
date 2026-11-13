# E_018 — ODB++ EDA PKG/PIN RC Source Qualification

**Status:** IN_PROGRESS until final PR HEAD CI, independent review, protected Verified Squash, and exact merged-main Push checks. This report is a source-only development artifact, **not** vendor/cross-CAD or electrical acceptance.

## Normative grammar and admission

Siemens [ODB++Design 8.1 Update 4](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf), `eda/data` PKG p.112, PIN p.113, outline pp.114–115.

- `PKG <name> <pitch> <xmin> <ymin> <xmax> <ymax>` followed **immediately** by one or more `RC <llx> <lly> <width> <height>` rectangles, optional `ID=<u32>`.
- `PIN <name> S <xc> <yc> 0 E S` with at least one RC rectangle, optional ID; 0-based package and pin ordinal.
- No PKG or PIN attribute lists, package PRP, polarity marker, CR/SQ/contour, non-SMT, through-hole, finished-hole, or component placement in this slice. They return explicit diagnostics rather than being silently reinterpreted.
- Original `NET/SNT/FID` index and design ownership semantics must remain. `$NONE$` never creates copper interconnection.
- Up to 4,096 packages, 65,536 pins and 131,072 rectangles; original 16 MiB logical member / 128 MiB aggregate file-map security limits still apply.
- Coordinates are **exact source 10⁻⁹ ticks** in MM/INCH. Audit exposes rational metres-per-tick; no f64/rounding.
- Package name alone is not an identity; source SHA-256 file entries, exact EDA file path/line and 0-based package/pin ordinal (and optional explicit ID) are retained.

## Architecture / accepted boundary

`odb_eda_packages.rs` is a strict source parser. `odb_eda.rs` validates this grammar before assigning existing Net/FID ownership; `odb_source.rs` exports `audit_unpacked_eda_packages` and `audit_path_eda_packages` for a complete validated board/source snapshot and source SHA witnesses. The EDA importer records SOURCE-only coverage; no fabricated `BoardModel::Component`, `Pin`, Pad, graph edges, plated Via or SI.

Existing single-level E_017 Step Repeat has **no qualified EDA mapping** and must continue returning `ODB_STEP_EDA_NOT_QUALIFIED`; source components are not duplicated into repeated board placements.

## Acceptance tests and evidence (fill only after GitHub observation)

- Rust isolated PKG/PIN positive/negative, exact decimal, multiple packages and ID conflict: PENDING
- `BoardModel` + EDA NET/FID invariant and physical graph absence: PENDING
- MM/INCH exact-coordinate equivalence: PENDING
- Directory / USTAR / GZIP / ZIP source transport identity parity: PENDING
- Existing E_013–E_017, PADS, Qt/IPC regressions: PENDING
- Final PR HEAD + Required Checks / native Win x64, Win ARM64, macOS ARM64: PENDING
- Independent review and zero unresolved threads: PENDING
- Protected Verified Squash commit and exact main Push CI: PENDING

## Not accepted

CMP placement, TOP toeprint, real Copper Pin/Pad attachment, E_019 manufactured plating/via, E_020 Stackup Material, E_021 Qt ODB++, E_022 global connectivity, E_023 independent vendor/cross-CAD oracle, production S parameters, and full issue [#10](https://github.com/july-rust/BoardSI/issues/10) remain **NOT_VERIFIED/OPEN**. Historic #578 was administratively closed independently; do not reopen as a completion workaround.

The only E progress record is [docs/development-checklist.md](../development-checklist.md); no secondary checklist. All dates must be recorded in KST.
