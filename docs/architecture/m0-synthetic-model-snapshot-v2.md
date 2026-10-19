# M0 F4-1 — Content-bound synthetic Model Snapshot v2 (provisional ADR)

Date: 2026-10-19 KST. Status: **IMPLEMENTED AS AN EXPERIMENT; NOT FULL F4 ACCEPTANCE**.
Related: Issues #3, #4, #5, #6, #46 and #78. Based on main `f4cde32ac686ea531c90d9ec7f136a979cffe294`.

## Why a separate v2 experiment

Existing IPC v1 remains unchanged: `Snapshot {points,delay_ms}` carries no BoardModel data and the legacy Qt Widgets prototype still uses that protocol. Reusing its identity string as though it authenticated model geometry would be unsafe. This PR adds an **explicit opt-in** `boardsi-worker --model-snapshot-v2` mode that receives one bounded binary-length-prefixed canonical JSON request and responds with a **synthetic identity receipt**, then exits. It does not run an electromagnetic or transmission-line computation.

The Rust client must first construct and structurally validate an M0 synthetic BoardModel, then capture its canonical payload into an owned private byte vector. The original source can change after capture without changing those bytes. New captures produce different model digests when supported model content changes. The separate OS worker rehashes the actual transported bytes and refuses a mismatched model/snapshot identity; receipt identity is checked again by the client.

## Explicit limited scope and fail-closed unsupported content

This provisional canonical subset supports exactly one synthetic PCB source, one frame, one copper layer, one confirmed material, one net and one line-segment outline. It serializes: source identity (file **claimed** SHA marker, source version, paths, import conditions and capability states), model scope, frame unit/transform/rational coordinates and metadata, layer order/kind/material and property provenance, confirmed dielectric/conductor parameters and frequency band, net/role, ordered outline vertices and simple metadata provenance.

Components, pins, traces, arcs, copper areas, holes, vias, padstacks, cutouts, multiple objects of a type, non-orthogonal transforms and conversion history are **rejected** rather than omitted. This is an intentionally restricted synthetic-only proof; it does not serialize or qualify a real PCB. Some synthetic import-capability flags may be Missing and are encoded as such. SourceFile SHA bytes are importer/fixture-supplied; they are **not** independently authenticated CAD source hashes.

Successful Worker receipt only establishes bounded canonical-payload transfer and content comparison for the restricted fields. The worker does not repeat all host BoardModel semantic validations, and cannot establish physical readiness or user-acceptance PASS.

## Exact identity and frame policy

Canonical representation is UTF-8 compact JSON serialized by pinned serde_json 1.0.114. Objects are emitted in deterministic sorted-key order (serde_json default Map), arrays preserve meaningful model order. Confirmed finite binary64 numbers are emitted by pinned serializer, minus-zero normalizes to plus-zero. Exact Rational values are decimal numerator/denominator string pairs; no floating conversion of geometric source ticks. Source ID and object metadata are validated before serialization; nonempty conversion history is unsupported.

- Model digest: SHA-256 of ASCII domain `BoardSI M0 synthetic model v2\0` followed by exact canonical model JSON UTF-8 bytes.
- Snapshot digest: SHA-256 of ASCII domain `BoardSI M0 synthetic run snapshot v2\0` followed by deterministic JSON containing model digest, points and delay_ms.
- Run ID: distinct client-generated execution identity; NOT a model hash. Bounded 1..128 bytes in this phase.
- Envelope: explicit schema=2, kind=SyntheticSnapshotV2, run ID, snapshot ID, model digest, model_json, points, delay_ms. One length-prefixed UTF-8 JSON frame, 1..65536 bytes; canonical model JSON max 32768 bytes. Unsupported versions, malformed/truncated frames, mismatches and nonfinite properties fail closed.
- Response: one `SyntheticModelReceipt` with schema/run/model/snapshot identities and synthetic=true. Not numerical SI output; not a Completed/Cancelled event contract.

The SHA-256 routine is embedded solely to avoid introducing a new dependency into this pinned Rust 1.75 build; it is tested against published known-answer vectors. **A digest is not a signature or authentication of the source data**. Review must assess the implementation and error cases before any reuse in production.

## Gates remaining before M0 F4 can close

1. Freeze a full Canonical Board Model schema that includes **every** solver-relevant geometry/material/net/provenance field and clean revision migration, rather than this narrow subset.
2. Transfer full immutable model/run payload to the independent Worker with version negotiation and rejection of unsupported model types; independent semantic validation and real numerical execution.
3. Complete nonblocking Qt/QProcess v2 path, cancellation/heartbeat/progress/result/OS exit convergence, stale and cross-run rejection and load/backpressure behavior.
4. Run native release/debug tests on Windows x64/ARM64, macOS Apple Silicon (Linux supplemental), retain original Qt #46 intermittent FAIL history and unchanged 1000ms global timer gate.
5. Qualify redistributable test fixtures, package/license/installer and baseline PC separately.

Prior PH-08 original Stage4/Stage5B Gate Q FAIL, BOUND_NOT_CERTIFIED, PH-08 NOT_FROZEN and the prohibited Draft PR #35 are unchanged. U/T acceptance statuses remain NOT_RUN unless independently qualified. A passing SHA here is **PARTIAL F4 transport evidence only**.
