# E_026 — KiCad 9 Layer/Via SOURCE qualification

## Scope and truth boundary

E_026 implements opt-in `boardsi_import::kicad9_source::inspect(&[u8])` for exactly
**KiCad PCB file version 20241229**. It never emits a `BoardModel`, `Via` or
`ConnectivityEdge`, and never asserts fabricated plating, contacted copper, stackup
thickness/z, vendor geometry parity, Qt/worker acceptance, 30 GHz SI or formal U/T PASS.

The parser is limited to root header, board layer declarations, net ordinal declarations,
and top-level track vias; unrelated sections are syntax-scanned but not semantically supported.
Its test fixtures are synthetic source-partial fixtures, **not qualified physical board
or independent KiCad 9 parser acceptance**. Public official v9 specification links:

- https://dev-docs.kicad.org/en/file-formats/sexpr-pcb/
- https://dev-docs.kicad.org/en/file-formats/sexpr-intro/
- https://docs.kicad.org/doxygen/classPCB__WRITER__V9.html
  (`FORMAT_VERSION=20241229`).

## Exact layer identity vs physical order

KiCad 9 stores front copper as ordinal **0**, back copper as **2**, and
internal copper `InN.Cu` as **2N+2**. Sorting numeric IDs would put B.Cu
immediately after F.Cu even on a multilayer board, which would corrupt via spans.
The importer separately retains the exact source ordinal and constructs only the
official **F.Cu → In1.Cu → ... → B.Cu** copper order. Enforce contiguous
inner layers; reject ambiguous, duplicate, missing, nonstandard copper declarations
and board counts outside the KiCad v9 2..32 even-layer range.

Noncopper layers remain source declarations; they are not silently reinterpreted
as copper.

For each via preserve byte offset, UUID/timestamp if present, exact integer-nm
coordinates and sizes (at most six decimal mm places), original start/end names,
typed through/blind/buried/microvia, declared net ordinal, option flags, source layer
ObjectIds and full **envelope** sequence. The envelope is **not** the electrically
connected layer set. In particular, `remove_unused_layers` and
`keep_end_layers` are not promoted to actual pad availability.

Provenance uses `SourceFormat::KiCad`, the file's **raw byte SHA-256**,
`SourceIdentity`, distinct `ObjectId` values and explicit exact extraction
history. Missing UUIDs use stable source-byte-offset identities, not invented UUIDs.
Re-import creates a new file content hash and corresponding object IDs.

## Fail-closed contract

Input maximum 8 MiB, 250k tokens, 100k nodes, depth 64, atom/string
max 8192 bytes, max 20k vias. Reject invalid UTF-8, bad S-expressions,
malformed strings/escapes, duplicate declarations, unknown via fields,
unsupported versions, inconsistent via types or spans, unknown non-zero
net IDs, invalid IDs and unsupported geometry precision or drill sizes.
A single rejection produces no partially published inventory.

Known unsupported cases include:
- Other KiCad version dates, future optional via record formats, padstacks,
  backdrilling and zone or net-class semantics;
- embedded footprints and footprint vias/pads not parsed as board vias;
- z/material thickness and manufacturing plating evidence;
- independent public KiCad 9 board exporter and native GUI confirmation;
- cross-format independent CAD Oracle (tracked separately in E_028).

## Code PR and independent evidence (2026-11-14 KST)

- [Code PR #633](https://github.com/july-rust/BoardSI/pull/633): final implementation HEAD
  `2257b8db7031124e5c1671ed9eb9887e7f1da289` **30/30 PR GitHub check-runs SUCCESS**;
  **0 failure, 0 incomplete** at final-head check.
- [Numerical core run #43571970100](https://github.com/july-rust/BoardSI/actions/runs/43571970100):
  Rust 1.75 `cargo fmt --all -- --check`, strict Clippy, workspace numerical/
  model integrity tests success; all six `kicad9_source::tests` pass in
  both Debug and Release job logs. Synthetic source tests cover 2/4/6/8/32
  Cu layers, through/blind/buried/microvia, file hash mutation and negative
  input/resource caps. No actual KiCad physical Vendor board qualification.
- Exact final HEAD checks: native Rust Windows x64/ARM64, macOS ARM64;
  native Qt Windows x64/ARM64, macOS ARM64 and provisional Linux offscreen;
  FEM/independent-FVM/docs/security workflows applicable to PR **SUCCESS**.
- Protected Squash merged [`main@9e6a98729905529222ca0e3b56472e232ec874a8`](https://github.com/july-rust/BoardSI/commit/9e6a98729905529222ca0e3b56472e232ec874a8)
  (one parent `51eaa0454bfd1c42afacf1c19033d3d85e1fb74b`).
  GitHub commit API `verification.verified=true`, `reason=valid`.
  `refs/heads/main` matched this exact SHA directly after merge.
- **Independent final-head review NOT_VERIFIED**: PR reviews empty and
  review threads 0 (zero threads is not an independent approval).
  Do not claim completed Codex/Copilot approval. Previous E_025 Codex
  requests reported account usage limit. No review bot outcome for E_026.
- **Exact code-main event=push CI: IN_PROGRESS / NOT_YET_VERIFIED** on the
  code merge SHA. Must check [Actions runs for code-main SHA](https://github.com/july-rust/BoardSI/actions)
  independently of PR checks and amend this record with actual eight workflow results.

## Closeout truth boundary

`CODE MERGED / PR CI 30/30 PASS / INDEPENDENT_REVIEW NOT_VERIFIED`.
The main Push Gate remains pending until all corresponding workflow runs finish.
Do not mark E_026 **full gate COMPLETE** or tick the sole canonical checklist
while external independent final-HEAD review is missing. The narrower
implemented source subset is merged and exercised by CI, not a full native
KiCad board importer or SI result. Formal U07/U33/U38/U40 and
T16/T18/T19/T20 remain NOT_RUN; upstream #23/#532/#520 remain OPEN.
