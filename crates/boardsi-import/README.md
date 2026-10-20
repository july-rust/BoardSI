# BoardSI M1 PADS ASCII signature probe (partial)

This directory implements a bounded, **read-only first-line signature check** for PADS PowerPCB and PADS Logic ASCII exports. It **does not** parse board geometry, units, nets, layers, pads, vias or stackup; it does not populate the Canonical Board Model or run an SI calculation. All detections are **SIGNATURE_ONLY**, regardless of reported source revision.

## Usage

```sh
cargo run --locked -p boardsi-import --bin boardsi-import-probe -- /path/to/synthetic.asc
cargo test --locked -p boardsi-import
```

The binary prints one of `PADS_PCB_ASCII_SIGNATURE_ONLY`, `PADS_SCHEMATIC_ASCII`, or `UNKNOWN_OR_UNSUPPORTED`. Schematic/unknown inputs return nonzero. No source bytes, filenames, private net names or geometry are logged. Only the first 513 bytes are read; headers longer than 512 bytes are rejected.

These are public synthetic examples only; do **not** upload company PCB sources to GitHub Actions. The detector tests both `!` and `*` anchored PowerPCB forms and excludes schematic `PADS-POWERLOGIC`/`PADS-LOGIC` and non-PADS `.asc` files.

References:
- [KiCad PADS importer documentation](https://dev-docs.kicad.org/en/import-formats/pads/index.html)
- [KiCad header discriminator](https://docs.kicad.org/doxygen/pads__common_8cpp_source.html)

Roadmap trace: [Issue #8](https://github.com/july-rust/BoardSI/issues/8) (format detection) and [Issue #9](https://github.com/july-rust/BoardSI/issues/9) (future real PADS importer). Product acceptance tests T01/T02/T03/T16/T17/T20/T22 remain NOT_RUN: the new unit tests are limited parser-preflight evidence, **not** the M1 import acceptance gate. No SI accuracy, provenance transformation or redistribution claim.
