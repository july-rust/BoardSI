# BoardSI

**Lightweight PCB & BGA Substrate Signal Integrity Analysis**

**Status: Native Qt/Rust Developer Preview with a restricted, source-validated public-synthetic PADS Layout ASCII V9.4 MILS import subset and CPU/Qt Quick-RHI Artwork Viewer. This is not a qualified general-purpose PADS importer, full-board electrical connectivity tool, or physical PCB SI Solver. Historical M0 architecture/runtime prototypes and restricted M2 lossless 2D FEM research coupons (PH-03/05/06/07) remain distinct from product SI verification.**

PH-07 was merged in [PR #40](https://github.com/july-rust/BoardSI/pull/40), `main@c6dce055`. Synthetic numerical verification does not establish measured-PCB or 30 GHz accuracy. See the [validation audit](docs/validation/m2-validation-audit-hardening.md) for independent checks, platform execution evidence and remaining administrative work.

BoardSI is planned as an offline-first Qt desktop UI plus Rust signal-integrity calculation core. Inputs will be handled by **separate format importers** (initial priority: PADS Layout ASCII and ODB++Design), normalized to a **format-neutral Canonical Board Model**. The viewer, port validator and solver use this same model. ODB++ is **not** BoardSI's native/internal format.

Planned features: PCB / BGA substrate pad-to-pad analysis; geometry, stackup, reference/port mapping; validated transmission-line RLGC and S-parameters; independent direct and progressive sweep; live curves; Touchstone export; asynchronous workers and recoverable RAM/SSD cache. Full-board 3D Full-wave EM and unvalidated discontinuity models are **out of the initial scope**. Bumps, Cu pillars, solder balls and wirebonds are excluded from substrate pad-to-pad simulation.

**Target platforms:** Windows x64, Windows ARM64, macOS Apple Silicon ARM64. Native numerical, Worker/Qt and limited FEM **prototype tests** have passed GitHub Actions; the full product, installers, real-board workflow and distributed binaries remain **NOT VERIFIED**.

## Documents

- **[Development completion checklist (A_036; continuously updated)](docs/development-checklist.md)** — U01–U48, T01–T54, finished/partial/unverified with evidence
- [Development specification v1.0, 2026-10-14](docs/specification/BoardSI_Development_Spec_v1.0.md)
- [Project management and bootstrap](docs/project-management.md)
- [Roadmap metadata](docs/project-plan.json)
- [Architecture decisions](docs/architecture/README.md)
- [Validation policy](docs/validation/README.md)
- [A_035/P2-Z limited Developer Preview acceptance and B-series handoff](docs/validation/a035-p2z-final-preview-acceptance.md)
- [Pre-PH08 complete-project baseline audit](docs/audits/2026-10-18-pre-ph08-project-audit.md)
- [Official icon B selection and source status](docs/branding/boardsi-icon-decision.md)

## Security and licensing

Do **not** commit proprietary PCB artwork, CAD exports, stackup/net data, confidential Touchstone data, private result files, credentials or personal file paths. Only independent public test fixtures with redistribution rights or synthetically constructed examples may enter CI. Planned operation is **local and offline**; do not upload board data to external services.

No software license has been selected pending Qt/parser/dependency/IP review. Public visibility does **not** grant open-source reuse rights.

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).

## Original PH-08 Gate Q evidence (2026-10-18)

Stage 4 HMM Gate Q **FAIL** remains unchanged. An **authorized private** Stage 5B original HMM replay reproduced its archived **FAIL** (12/12 rows, 1761/1761 deterministic scalars identical); public CI is *synthetic-only*. AC-P1 PENDING; T/R PROPOSED_NOT_FROZEN; PH-08 contract NOT_FROZEN; Rust FEM NO_GO; Gate M NOT_VERIFIED; physical error BOUND_NOT_CERTIFIED. See [original replay evidence](docs/validation/m2-g2-ph08-stage5b-original-replay-evidence.md), [Issue #51](https://github.com/july-rust/BoardSI/issues/51). Owner explicitly **approved the existing option B SVG as the visual design master** (2026-10-18); original earlier PNG byte identity and final platform icon packaging remain **NOT VERIFIED**. Do not alter or substitute the SVG.
