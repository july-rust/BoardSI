# D_056_08 — ODB++Design source-evidence adapter (bounded initial slice)

**First implemented:** 2026-11-10 KST. **Status:** code [PR #562](https://github.com/july-rust/BoardSI/pull/562) in progress. **Acceptance:** D_056_08 NOT COMPLETE; this document does not assert whole-board ODB++ support, Qt import, electrical connectivity or SI.

Parent [#520](https://github.com/july-rust/BoardSI/issues/520) · Multi-format gate [#532](https://github.com/july-rust/BoardSI/issues/532) · [Source contract](d056-01-source-evidence-audit.md) · [Single checklist](../development-checklist.md#d_056).

## 1. Authoritative format evidence

Siemens EDA, [ODB++Design Format Specification Release 8.1 Update 4, August 2024](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf). This implementation independently parses a limited record subset; it does not embed or redistribute the specification or imply Siemens validation or license.

- Product model pp.43–46: product root / steps / layer containers.
- matrix/matrix pp.62–66: STEP and LAYER blocks; ROW, NAME, TYPE, CONTEXT, POLARITY.
- Layer features pp.164–172: UNITS, symbol table, F, L, P, A records, optional ID and line provenance.
- Standard symbols pp.187–189: round, square, rectangle; explicit I (mils) and M (micrometres) symbol units or source-file unit.
- Public synthetic grammar tests are **not** independent real CAD exporter evidence.

## 2. Runtime entrypoints and limits

Rust source: crates/boardsi-import/src/odb_source.rs. Public library functions are import_unpacked(OdbFileMap, step) and import_directory(Path, step). The diagnostic CLI boardsi-odb-import accepts an unpacked product directory and explicit step. Not a main Qt Import yet.

- Bounded product-root-relative file map; no symlink, nonregular-file or unsafe path traversal.
- File size at most 16 MiB; cumulative at most 128 MiB; at most 8192 files; directory depth at most 16.
- Exact decimal coordinates up to nine fractional digits. Metric frame is 1e-12 metre per tick, inch frame 2.54e-11 metre per tick, retaining exact rational transforms.
- Source SHA-256 per file and logical relative source path:line provenance for Canonical objects.
- Positive L lines with round symbols -> Trace; positive P with round/square/rectangle -> Pad/Padstack; positive A with round symbol and common exact radius -> analytic Arc.
- Matrix BOARD Copper Layer order accepts SIGNAL, POWER_GROUND and MIXED for 2–64 layers.
- No fabricated net, material, connected-layer evidence, manufacture connectivity or S-parameter is produced.

ImportCapability: geometry PARTIAL, net MISSING, pin-pad MISSING, stackup PARTIAL (layer order only; no z/thickness), reference MISSING, model MISSING. All Graph nodes/edges remain empty.

## 3. Explicitly unqualified / unsupported

- Archive .tgz/.tar.gz/ZIP, compressed features.Z or compressed native ODB++ data.
- Nested Step Repeat / flip/mirror beyond the supported pad-local orthogonal transform, custom symbols, arbitrary-angle/scaled pads.
- Subtractive negative copper, S/OB/OS/OE/SE surface contours, copper text/barcodes and unsupported Copper features: source import **fails closed**.
- EDA/Net mapping, Feature-to-Net mapping, Component/Pin, PTH/NPTH drill, blind/buried/microvia manufacturing information and plated barrels.
- Actual layer z/material/Dk/Df/thickness, whole-board electronic connectivity, verified open/short and SI.
- Main Qt Artwork IPC / CPU / RHI / picking and vendor-export qualification.
- Concurrent filesystem mutations are NOT atomically snapshotted; TOCTOU race and adversarial mount changes remain NOT_VERIFIED.

This is a strict importer for **selected supported Copper records**, not a complete import of all objects present in a CAD product. Any unparsed evidence must not be promoted to COMPLETE.

## 4. Test coverage and acceptance

Implemented tests: 2/4/6/8/64-layer source maps; round line, rotated rectangle pad, CW/CCW arc and canonical coordinates; SHA mutation binding; missing source file; bad feature count; unsupported/negative copper; wrong arc radius; bad pad rotation; unsafe path; decimal overflow; wrong step; real unpacked directory loading and Unix symlink rejection.

The exact PR-head CI and protected post-Squash main Push results must be recorded separately after verification. Code tests alone do **not** qualify real independent CAD exports or physical connection.

Remaining D_056_08 gates: external valid ODB++ fixture + reference oracle, full supported geometry inventory, EDA/Net/Part/Drill/Surface mappings, safe archive handling, Main Qt/IPC integration and source-qualified complete electrical Graph behavior. D_056_08 remains OPEN until its agreed acceptance contract is actually met; Issue #532 and parent #520 remain OPEN through later format stages.

Avoid filling the sole checklist with unverifiable success claims.
