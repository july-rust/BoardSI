# BoardSI

**Lightweight PCB & BGA Substrate Signal Integrity Analysis**

**Status: M0 architecture/runtime prototypes and restricted M2 lossless 2D FEM research coupons (PH-03/05/06/07) are implemented. A product PCB Solver and verified PCB-format import are not implemented.**

PH-07 was merged in [PR #40](https://github.com/july-rust/BoardSI/pull/40), `main@c6dce055`. Synthetic numerical verification does not establish measured-PCB or 30 GHz accuracy. See the [validation audit](docs/validation/m2-validation-audit-hardening.md) for independent checks, platform execution evidence and remaining administrative work.

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
