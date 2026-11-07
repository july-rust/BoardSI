# BoardSI

**Lightweight PCB & BGA Substrate Signal Integrity Program**

BoardSI aims to make **offline, resource-conscious PCB signal-integrity review** practical: import layout data, inspect copper and stackup, identify signal paths and reference nets, configure ports, and eventually calculate validated RLGC, impedance and S-parameters. The design uses a **format-neutral Canonical Board Model**, a Rust core and a Qt desktop UI. **ODB++ is one planned input format, not the internal data model.**

> **Current status — 2026-11-07 KST: Developer Preview / partial functionality.** The repository contains native Rust/Qt prototypes, a **restricted PADS PowerPCB V9.4 MILS source subset** and synthetic artwork/geometry qualification. It is **not** a production PADS converter, complete electrical PCB connectivity analyzer, shipping installer or physically validated PCB SI solver. [What is actually verified?](docs/development-checklist.md)

[**Documentation hub / 문서 시작**](docs/README.md) · [Developer Quickstart](docs/developer-quickstart.md) · [Specification v1.0](docs/specification/BoardSI_Development_Spec_v1.0.md) · [Development checklist](docs/development-checklist.md) · [Validation](docs/validation/README.md)

## What works, what is planned

| Area | Implemented and scope-tested today | Not yet product-qualified |
| :--- | :--- | :--- |
| **Import & Artwork** | Restricted V9.4-MILS PADS SOURCE parser, selected straight/arc traces, pads, vias and copper geometry, native Qt Artwork Preview, public synthetic fixtures | Complete PADS/vendor exports, ODB++, IPC-2581 and other planned importers, lossless full-board normalization |
| **Copper contact** | Source-local finite-width trace-to-trace and off-centre trace-to-Via conductive annulus / circular, square, rectangular SMD-pad **geometry candidates** ([D_054/D_055 evidence](docs/validation/d054-d055-general-trace-contact-qualification.md)) | Proven whole-board electrical Open/Short, thermal/pour/plating completeness, new-candidate integration into the actual Qt global connectivity graph |
| **Execution** | Rust Canonical integrity model, isolated Worker IPC, Qt Widgets and optional GPU-first artwork backends; native regression evidence on supported platforms | Complete Import→Port→Solver→Results user journey, resilience/packaging/clean-system redistributable release |
| **Numerics** | Mathematical **supplied scalar** RLGC→uniform 2-port S; research-only restricted PEC/lossless 2D FEM coupons | Accurate board-geometry→frequency-dependent physical RLGC, multiconductor coupling, 30 GHz PCB validation, final Touchstone workflow |
| **PH-08 physical gate** | Historical failure reproduced in controlled evidence | Gate Q **FAIL**; original physical acceptance **not approved** |

**The planned product** targets Windows x64, Windows ARM64 and macOS Apple Silicon (arm64); Linux is also used for provisional native CI. Successful source build or synthetic CI is **not** an installer, release, physical qualification or formal acceptance of U01–U48/T01–T54. See the [M0 completion gate](docs/validation/m0-completion-gate.md).

## Developer starting point

Requirements for the documented source baseline: Rust **1.75.0** (repository toolchain), C++17, CMake **≥3.21**, Qt **≥6.5** (tested configurations document Qt 6.8.3). From the repository root:

\`\`\`sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace

# Restricted PADS signature / geometry examples (public synthetic only).
cargo run --locked -p boardsi-import --bin boardsi-import-probe -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc
cargo run --locked -p boardsi-import --bin boardsi-import-artwork -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc \
  --trace-pad-via-contacts-v1
\`\`\`

The last command reports **local copper-area candidates only**, not electrical network connectivity or SI. For **Qt Widgets/Worker build instructions, Windows developer launcher, model fixtures and troubleshooting**, use the [complete Quickstart](docs/developer-quickstart.md). The [importer](crates/boardsi-import/README.md) and [Qt Preview](apps/boardsi-qt-prototype/README.md) READMEs provide component-specific details.

## Project structure and documentation

| Path | Responsibility |
| :--- | :--- |
| \`crates/boardsi-import/\` | PADS signature, restricted source parsing, geometry/provenance and local contact candidates |
| \`crates/boardsi-model/\` | Shared Canonical Board Model and structural validation |
| \`crates/boardsi-numerics/\` | Supplied scalar RLGC and single-ended 2-port mathematics |
| \`crates/boardsi-ipc/\` | Versioned local Worker IPC contracts |
| \`apps/boardsi-worker/\` | Isolated Rust Worker and synthetic model evidence |
| \`apps/boardsi-qt-prototype/\` | Qt Widgets Preview, Artwork viewer, runtime/harness |
| \`docs/specification/\` | **Normative** approved v1.0 product requirements and U/T definitions |
| \`docs/architecture/\` | Architecture decisions and scope-bound contracts |
| \`docs/validation/\` | Per-test executed evidence, failures, accepted limits |
| \`docs/development-checklist.md\` | **Only** authoritative live A/B/C/D and U/T implementation ledger |
| \`docs/audits/\` | Date-pinned audits; historical snapshots, not guaranteed current truth |

[Architecture map](docs/architecture/README.md) · [Validation index](docs/validation/README.md) · [Project management](docs/project-management.md) · [Preview-first development policy](docs/preview-delivery-playbook.md) · [Recent documentation audit](docs/audits/2026-11-07-documentation-audit.md).

## Product boundary and evidence integrity

The v1.0 *target workflow* is **Import → Artwork → Stackup/Materials → Reference Nets → Signals/Ports → Frequency & Resources → Run/Results**. This **is not** a declaration that the whole workflow currently works. The initial product excludes whole-board 3D full-wave EM and unvalidated discontinuity models; BGA/Substrate support does not imply full bump, solder-ball, Cu-pillar or wirebond physics.

Historical PH-08 Stage 4 and authorized private Stage 5B original HMM replay preserved **Gate Q FAIL**; synthetic mathematical CI does not overturn it. AC-P1 remains pending and original PH-08 physical error bounds remain unqualified. [PH-08 evidence and limitations](docs/validation/README.md).

## Security, licensing and contributing

- **Offline/confidential first:** never commit or upload employer PCB artwork, design exports, netlists, stackup, Touchstone data, customer results, credentials or private filesystem paths. CI uses public-licensed or synthetic fixtures only.
- **No implied distribution grant:** public source visibility does not establish open-source reuse rights or satisfy Qt/third-party distribution obligations. Production installers and licensing remain unqualified.
- **Strict verification:** keep source SHA, platform, test scope, evidence date and negative/uncertain outcomes. A green build is not physical model validation.
- **Efficient documentation:** true Markdown-only PRs run the required documentation check and skip native platform jobs; requirements and historical validation contracts are not changed by a docs cleanup.

[Contributing guide](CONTRIBUTING.md) · [Security policy](SECURITY.md) · [Documentation conventions](docs/README.md).
