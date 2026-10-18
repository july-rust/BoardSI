# M0 — Canonical Model-gated synthetic Rust Worker handoff

Status: **PARTIAL IMPLEMENTATION; NOT PRODUCT ACCEPTANCE**.

The new `boardsi_worker::model_handoff::ModelBoundSyntheticClient` carries an immutable borrow of an M0 `BoardModel` while its real, separately spawned Rust Worker runs. Before the Worker starts, the M0 gate requires:

- one explicitly `Synthetic` source and nonempty layers/nets;
- no structural validation errors or missing required properties;
- a valid IPC v1 synthetic `Snapshot` and bounded run ID.

The actual integration test launches the existing OS process, polls framed IPC and verifies completion, five synthetic results and clean process exit. Negative tests reject duplicate object IDs, missing layer material, non-synthetic source, empty model and invalid Snapshot **before** spawning.

## Strict limits

1. This is **not** a PCB/ODB/PADS importer. Source SHA markers in synthetic tests do **not** imply measured digest verification.
2. The IPC v1 `Snapshot { points, delay_ms }` still has **no canonical geometry/net/material payload or content hash**. The Worker uses the original fixed synthetic uniform-line reference, **not** model-derived RLGC.
3. This prototype does **not** integrate the Qt GUI with `BoardModel`; existing Qt UI still starts its original synthetic test Snapshot.
4. Structural validation and source provenance are distinct from physical readiness, material certification, accuracy and completed T acceptance.
5. This work must not close Issue #5 or M0 until canonical content identity, immutable versioned run handoff, Qt integration and native regression gates are verified separately.

No IPC protocol version/field changes, PH-03/05/06/07 Frozen Contract changes or PH-08 physics status changes are made.

Related: Issues #3, #4, #5, #6 and #78. Follow-up: model content-bound immutable run snapshot and Qt handoff without arbitrary model mutability or result identity confusion.
