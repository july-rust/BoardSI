# BoardSI

**Lightweight PCB & BGA Substrate Signal Integrity Analysis**

**Status: initial planning / architecture validation (M0). This repository does not yet contain a working solver or verified format support.**

BoardSI is planned as an offline-first Qt desktop UI plus Rust signal-integrity calculation core. Inputs will be handled by **separate format importers** (initial priority: PADS Layout ASCII and ODB++Design), normalized to a **format-neutral Canonical Board Model**. The viewer, port validator and solver use this same model. ODB++ is **not** BoardSI's native/internal format.

Planned features: PCB / BGA substrate pad-to-pad analysis; geometry, stackup, reference/port mapping; validated transmission-line RLGC and S-parameters; independent direct and progressive sweep; live curves; Touchstone export; asynchronous workers and recoverable RAM/SSD cache. Full-board 3D Full-wave EM and unvalidated discontinuity models are **out of the initial scope**. Bumps, Cu pillars, solder balls and wirebonds are excluded from substrate pad-to-pad simulation.

**Supported targets (not yet verified):** Windows x64, Windows ARM64, macOS Apple Silicon ARM64.

## Documents

- [Development specification v1.0, 2026-10-14](docs/specification/BoardSI_Development_Spec_v1.0.md)
- [Project management and bootstrap](docs/project-management.md)
- [Roadmap metadata](docs/project-plan.json)
- [Architecture decisions](docs/architecture/README.md)
- [Validation policy](docs/validation/README.md)

## Security and licensing

Do **not** commit proprietary PCB artwork, CAD exports, stackup/net data, confidential Touchstone data, private result files, credentials or personal file paths. Only independent public test fixtures with redistribution rights or synthetically constructed examples may enter CI. Planned operation is **local and offline**; do not upload board data to external services.

No software license has been selected pending Qt/parser/dependency/IP review. Public visibility does **not** grant open-source reuse rights.

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).
