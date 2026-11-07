# BoardSI Rust Worker — Developer Prototype

> **Scope (2026-11-07 KST):** This is the independent Rust computation/IPC process used by BoardSI's limited Qt developer Preview. Its example paths use **public synthetic Canonical Model fixtures**, not actual customer PCB data or a product-qualified signal-integrity solver.

[Project documentation](../../docs/README.md) · [Developer Quickstart](../../docs/developer-quickstart.md) · [Qt Preview](../boardsi-qt-prototype/README.md) · [IPC protocol](../../docs/architecture/worker-ipc-qt-prototype.md) · [Current acceptance status](../../docs/development-checklist.md)

## Responsibility and limits

- The Worker is launched as a separate process from the Qt GUI and communicates over the local framed `boardsi-ipc` protocol. Its standard output is reserved for protocol data; diagnostics are written to standard error.
- The scoped synthetic demonstration computes an idealized **uniform, supplied-parameter scalar RLGC** two-port result. It is **not** an extractor for PCB conductor geometry, physical Via effects, actual measured impedance or validated DC–30 GHz S-parameters.
- Immutable model-v2 A/B fixtures are generated for developer verification. An unrecognized input, bad identity or unsupported format must not silently become a physical PASS.
- CI regression results for the Worker/Qt sample demonstrate only the explicitly exercised synthetic workflow and platform conditions. Packaging and actual PCB Solver acceptance are separate gates.

## Reproducible public synthetic setup

Run from the repository root, using the pinned Rust toolchain and source fixture:

```sh
cargo build --locked -p boardsi-worker
mkdir -p build-qt
cargo run --locked -p boardsi-worker --example m0_model_snapshot_fixture -- \
  build-qt/qt-model-a.frame build-qt/qt-model-b.frame
```

The generated files contain **synthetic test metadata**, not a customer PCB. Run the full Qt/Worker GUI and CTest sequence using [the Developer Quickstart](../../docs/developer-quickstart.md); the GUI needs the matching Worker executable and fixture paths. Do not upload real board layout, Net names, Stackup or Touchstone data to public CI.

## Documentation-only PR and CI

This is a terminal component `README.md` path. Starting with [CI fix #516](https://github.com/july-rust/BoardSI/pull/516), a pull request that **changes only allowed documentation files** is classified on a small Ubuntu runner; heavy F6/F7, native Qt, Rust and FEM jobs are declared but skipped **before native runner allocation**. Changes to Rust, tests, workflow/YAML, JSON, fixtures, renames or unsupported modes retain **full validation**. See [Issue #513](https://github.com/july-rust/BoardSI/issues/513) and [contribution policy](../../CONTRIBUTING.md).
