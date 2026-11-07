# Developer Quickstart — BoardSI

[← Documentation hub](README.md) · [Qt Preview details](../apps/boardsi-qt-prototype/README.md) · [PADS Import details](../crates/boardsi-import/README.md)

**Applies to:** source-tree developer Preview, restricted public synthetic examples. **Not a supported installer, generic PADS conversion, measured-PCB electrical analysis or physical SI solver.**

## 1. Prerequisites and supported evidence

| Item | Source-tree baseline | Clarification |
| :--- | :--- | :--- |
| Rust | [`rust-toolchain.toml`](../rust-toolchain.toml): Rust **1.75.0** with rustfmt/clippy | Pinned **provisional M0** baseline, not a final product support policy |
| C++ | C++17; CMake **minimum 3.21** | Declared in [Qt CMake project](../apps/boardsi-qt-prototype/CMakeLists.txt) |
| Qt | Qt **6.5 minimum**, Qt **6.8.3** used in documented native CI | Core, Gui, Widgets required; Quick/RHI and OpenGLWidgets capability dependent |
| Target machines | Windows x64, Windows ARM64, macOS Apple Silicon ARM64 (native CI); Linux x86_64 (provisional test runner) | Passing a test on these hosts **does not** certify a self-contained redistributable build |
| Data | Public synthetic `*.asc` fixtures committed to the repository | Never upload an employer's PCB, internal paths, credentials or results |

All commands below run from the **repository root** unless a command explicitly changes directory.

## 2. Validate the Rust workspace

```sh
cargo --version
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Tests validate their **own documented scopes**, not full product U01–U48/T01–T54 acceptance. An independent on-device run is not equivalent to a recorded GitHub PR-head CI run.

## 3. Check a public synthetic PADS source

The *signature probe* and the *partial artwork importer* are **different executables**:

```sh
# Read-only format/signature classification; does not parse PCB geometry.
cargo run --locked -p boardsi-import --bin boardsi-import-probe -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc

# Restricted SOURCE geometry -> partial canonical/artwork protocol.
cargo run --locked -p boardsi-import --bin boardsi-import-artwork -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc
```

The second stdout is a **versioned machine-readable Qt protocol**, not a human-friendly report and **not S-parameter data**. PADS input is restricted to supported V9.4 MILS records; unsupported objects must not be silently interpreted as full geometry.

### D_054 and D_055 local copper-candidate diagnostics

```sh
# Trace-to-trace: straight and supported CW/CCW circular-arc copper.
cargo run --locked -p boardsi-import --bin boardsi-import-artwork -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc \
  --general-trace-contacts-v1

# Trace-to-via annular copper and supported SMD circular/square/rectangle pads.
cargo run --locked -p boardsi-import --bin boardsi-import-artwork -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc \
  --trace-pad-via-contacts-v1
```

The result is SOURCE-SHA-bound **local positive-area geometric contact evidence**. It is **not** a verified PCB-wide net graph, actual Open/Short decision, manufactured contact test or solver output. D_055 uses the actual Via outer pad and drilled hole and can detect off-centre, side, corner and midspan contacts; unknown PADS objects and uncertain tangencies fail closed.

For **complete inventory and evidence boundaries**, consult the [D_054–D_055 qualification](validation/d054-d055-general-trace-contact-qualification.md) and [single development checklist](development-checklist.md).

## 4. Build the developer Qt Widgets/Worker Preview

On a Unix-like system with the required Qt SDK and compiler installed:

```sh
cargo build --locked -p boardsi-worker
cargo build --locked -p boardsi-import --bin boardsi-import-artwork
cmake -S apps/boardsi-qt-prototype -B build-qt \
  -DCMAKE_PREFIX_PATH=/absolute/path/to/Qt/prefix \
  -DBOARDSI_WORKER=/absolute/path/to/target/debug/boardsi-worker
cmake --build build-qt --parallel 2

# Generate the two test-only immutable synthetic Canonical Model frames.
cargo run --locked -p boardsi-worker \
  --example m0_model_snapshot_fixture -- \
  build-qt/qt-model-a.frame build-qt/qt-model-b.frame

# Linux headless testing; not a benchmark of live GPU rendering.
QT_QPA_PLATFORM=offscreen ctest --test-dir build-qt --output-on-failure
```

On an available desktop (actual paths and app bundle locations vary per host):

```sh
./build-qt/boardsi-preview-p0 \
  /absolute/path/to/boardsi-worker \
  /absolute/path/to/qt-model-a.frame \
  /absolute/path/to/qt-model-b.frame \
  /absolute/path/to/target/debug/boardsi-import-artwork
```

**Windows x64:** after building the Rust Worker, Importer, Qt executable and synthetic frames with the appropriate SDK, you can use the [developer launcher](../scripts/run-boardsi-preview-windows-x64.ps1):

```powershell
$env:QT_PREFIX = "C:\path\to\Qt\6.8.3\msvc2022_64"
.\scripts\run-boardsi-preview-windows-x64.ps1
```

The launcher is **Windows x64 only**; do not use it as a Windows ARM64 launch recipe. On Windows use PowerShell environment-variable syntax (for example `$env:QT_QPA_PLATFORM = "offscreen"`) rather than Linux's inline assignment. Separate Qt SDK/runtime and redistributable package dependencies must be audited before any distribution.

The executable is still called **`boardsi-preview-p0`** for historical compatibility; the current source tree contains additional P1/P2 Artwork features. Historical notes that *P0's* Open PCB button only selected a filename **do not describe every later Preview mode**. [Qt Preview README](../apps/boardsi-qt-prototype/README.md) separates the dated milestone behavior.

## 5. Troubleshooting and honest interpretation

| Symptom | Check first | What not to infer |
| :--- | :--- | :--- |
| Qt not found | Correct Qt SDK prefix; CMake uses Qt6 Core/Gui/Widgets | No need to modify the Rust solver |
| Worker/fixtures missing | Absolute Worker executable and regenerated `qt-model-*.frame` | Do not label the UI `SI_VALIDATED` |
| No GPU viewport | Qt Quick/OpenGLWidgets availability and actual renderer state | GPU-requested is **not** GPU-active |
| PADS parser rejects a source | Header, V9.4 supported grammar, file size, object inventory, unsupported shapes | A rejected object must **not** become a fabricated straight trace or pad |
| Copper candidate absent | Source provenance, Pad shape/layer, Via drill/copper, Net ownership, uncertainty limit | Geometric absence is **not** proof of a manufactured electrical open |
| Green PR CI | Inspect **exact PR-head SHA, job, OS and test** | Build/test PASS is not full product acceptance |

## 6. Release, data and validation policy

- **No external upload:** BoardSI source-run workflows are intended to be local/offline; public CI uses only public synthetic examples.
- **No invented physical accuracy:** supplied scalar RLGC-to-S numerical mathematics and lossless field coupons do **not** establish PCB geometry-to-RLGC extraction or 30 GHz accuracy.
- **No implied licensing:** source/build test artifacts are not automatically approved distributable installers. See [SECURITY](../SECURITY.md) and [Preview delivery policy](preview-delivery-playbook.md).
- **Historical PH-08:** original Gate Q **FAIL** remains unchanged; do not use synthetic PASS to turn it into a physical approval. [Validation evidence](validation/README.md).

**Next reading:** [Architecture](architecture/README.md) → [Validation](validation/README.md) → [Canonical checklist](development-checklist.md).
