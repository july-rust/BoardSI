# E_018 — ODB++ EDA PKG/PIN RC Source Qualification

**Status:** **CODE / SOURCE bounded COMPLETE (2026-11-13 KST)** — verified code PR 30/30, Codex +1/0 unresolved, protected GitHub Verified Squash, and exact code-main `fc096f2d` **event=push 8/8 Workflow SUCCESS**. Final E_018 checklist `COMPLETE` is conditional only on this docs closeout PR #614's own final-head required CI, protected Verified Squash and exact docs-main Push. No vendor / cross-CAD or electrical acceptance. This is not vendor/cross-CAD or electrical acceptance.

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

## Acceptance tests and evidence — exact code PR #613

- **Source PR:** [#613](https://github.com/july-rust/BoardSI/pull/613), final HEAD `75a79dd807a68719d3506bad72586187b6795c73`, 30/30 final-head Check Runs SUCCESS including Windows x64/ARM64 and macOS ARM64 Native Rust/Qt, Linux Qt, CodeQL and docs.
- **Numerical source validation:** `boardsi-import` 333/333 library tests in Debug, 333/333 in Release. Independent E_018 archive-parity integration test PASS in both variants; existing E_013–E_017 and PADS test regression suite also passed.
- **Review:** Codex found P2 stale NET/SNT context after PKG; fixed with fail-closed regression, replied and resolved thread. Codex independent final-head `+1`, 0 unresolved threads.
- **Protected merge:** GitHub Verified single-parent Squash [`fc096f2da66280e43eb43c20bddd1fe076cac35c`](https://github.com/july-rust/BoardSI/commit/fc096f2da66280e43eb43c20bddd1fe076cac35c), verified=true/valid, parent `d0cabad37f6403cabe5042b1dff60573b8194b8e`.
- **Exact code-main event=push:** **8/8 Workflow SUCCESS** for `fc096f2da66280e43eb43c20bddd1fe076cac35c`, 0 failed/skipped workflows, independently from PR checks: [Validate planning documents](https://github.com/july-rust/BoardSI/actions/runs/43349775004), [M2 PH-06 asymmetric stripline FEM independent FVM](https://github.com/july-rust/BoardSI/actions/runs/43349774891), [M2 synthetic G0 reference](https://github.com/july-rust/BoardSI/actions/runs/43349774955), [M2 isolated lossless FEM prototype](https://github.com/july-rust/BoardSI/actions/runs/43349774937), [M2 native FEM regression](https://github.com/july-rust/BoardSI/actions/runs/43349774855), [M2 PH-07 multilayer masked microstrip independent FVM](https://github.com/july-rust/BoardSI/actions/runs/43349775035), [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/43349774918), [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/43349775083).
- **Evidence recording:** 2026-11-13 KST. All historical failed CI attempts and their fixes remain visible in PR #613.

## Automated acceptance matrix

- Rust isolated PKG/PIN positive/negative, exact decimal, multiple packages and ID conflict: PASS (333/333 crate lib suite Debug/Release)
- `BoardModel` + EDA NET/FID invariant and physical graph absence: PASS
- MM/INCH exact-coordinate equivalence: PASS
- Directory / USTAR / GZIP / ZIP STORED / ZIP DEFLATE with features.Z logical source identity parity: PASS
- Existing E_013–E_017, PADS, Qt/IPC regressions: PASS on PR HEAD; exact code-main Push 8/8 Workflow SUCCESS.
- Final PR HEAD + Required Checks / native Win x64, Win ARM64, macOS ARM64: 30/30 SUCCESS
- Independent review and zero unresolved threads: Codex final-head +1, 0 unresolved; P2 corrected
- Protected Verified Squash: `fc096f2da66280e43eb43c20bddd1fe076cac35c` SUCCESS; exact main Push CI: 8/8 Workflow SUCCESS.

## Not accepted

CMP placement, TOP toeprint, real Copper Pin/Pad attachment, E_019 manufactured plating/via, E_020 Stackup Material, E_021 Qt ODB++, E_022 global connectivity, E_023 independent vendor/cross-CAD oracle, production S parameters, and full issue [#10](https://github.com/july-rust/BoardSI/issues/10) remain **NOT_VERIFIED/OPEN**. Historic #578 was administratively closed independently; do not reopen as a completion workaround.

The only E progress record is [docs/development-checklist.md](../development-checklist.md); no secondary checklist. All dates must be recorded in KST.

## Final docs-only closeout gate

- [PR #614](https://github.com/july-rust/BoardSI/pull/614): **PENDING final-head required docs CI, independent review, protected GitHub Verified Squash and exact docs-main event=push workflow confirmation.** This document records verified code-main gates, not evidence of PR #614 acceptance before it occurs.
