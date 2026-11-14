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
- **Original code-merge review snapshot, preserved:** before 2026-11-14 Codex retry,
  the final-head independent review was NOT_VERIFIED (initial usage-limit
  failure; zero review threads was not proof of review).
- **Actual post-merge Codex review completed:** bot reviewed original
  merge SHA `9e6a98729905529222ca0e3b56472e232ec874a8` on PR #633,
  finding **one P2**: `(locked yes/no)`, `(free yes/no)`,
  `(remove_unused_layers yes/no)` and `(keep_end_layers yes/no)` had
  been incorrectly rejected as `BAD_VIA_FLAG`. This is a genuine
  source parser compatibility bug; original CI did not cover valued flags.
  Source cross-check: KiCad 9 upstream `parseMaybeAbsentBool(true)` for
  all four fields.
- **P2 correction PR #635**: final HEAD
  `2e683961949f2c4a2303831b1ba0636f54c5cd0f`; accepts bare/yes/no
  forms with typed rejection of invalid/multiple flags; preserves effective
  `locked` and `free` SOURCE flags. Adds positive and mutation regressions.
  **30/30 PR checks SUCCESS** including Native Rust/Qt Windows x64/ARM64,
  macOS ARM64 and Linux. **Independent Codex review COMPLETED, findings=0,
  unresolved=0** for exact corrective HEAD. Protected Verified one-parent
  Squash `9f1829bac92f06dce1eed7e3d904856a6717c76e`.
  Code-main Push check is recorded separately below when completed.
- **Documentation PR #634** was also independently reviewed after merge
  by Codex at `d73de8de`: COMPLETED, no findings.
- **Exact code-main event=push CI: 8/8 WORKFLOWS SUCCESS** on
  `9e6a98729905529222ca0e3b56472e232ec874a8` verified separately from PR checks.
  [Numerical core run #43573232613](https://github.com/july-rust/BoardSI/actions/runs/43573232613)
  and [Qt runtime run #43573232555](https://github.com/july-rust/BoardSI/actions/runs/43573232555)
  include Native Rust/Qt Windows x64, ARM64, macOS ARM64 and Linux GUI;
  other six runs: docs #43573232547, M2 isolated FEM #43573232533,
  synthetic G0 #43573232558, M2 Native FEM #43573232510,
  PH07 FVM #43573232512 and PH06 FVM #43573232499. All eight exact
  merge-SHA event=push runs report `completed / success` (2026-11-14 KST).

## Closeout truth boundary

**Code scope:** KiCad 9 `20241229` Layer/Via SOURCE-only inventory.
Original #633 + post-merge Codex P2 correction #635 are both protected
Verified Squash merged. Original review **COMPLETED/P2 found 1**; corrective
final-HEAD review **COMPLETED/no findings**. Main Push receipt for corrective
merge SHA `9f1829ba` is checked separately and must not be claimed successful
until observed. The earlier #633 code-main 8/8 push SUCCESS remains valid.

**No product promotion:** This does not establish full native KiCad
readability, independently exported KiCad 9 real design Oracle, Qt import,
physical via contact, stackup material/z, net connectivity or SI. Formal
U07/U33/U38/U40 and T16/T18/T19/T20 remain NOT_RUN. Parent
#23/#532/#520 remain OPEN. Track final documentation PR independently;
avoid describing docs-only SKIPPED Native jobs as new Native PASS.
