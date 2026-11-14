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

## Final exact-commit Qt and merged-main receipt (2026-11-14 KST)

- **Corrective source PR:** [#635](https://github.com/july-rust/BoardSI/pull/635),
  final head `2e683961949f2c4a2303831b1ba0636f54c5cd0f`.
  Exactly **30/30 check-runs SUCCESS** on the final PR head; this includes
  actual applicable Rust and Qt Native Windows x64, Windows ARM64, macOS ARM64,
  and Linux jobs. Codex independent source review **COMPLETED / findings=0 /
  unresolved=0** on that corrective SHA. GitHub protected Verified 1-parent
  Squash main commit: [`9f1829bac92f06dce1eed7e3d904856a6717c76e`](https://github.com/july-rust/BoardSI/commit/9f1829bac92f06dce1eed7e3d904856a6717c76e).
- **Corrective code-main exact `9f1829ba` Push CI: all 8/8 Workflow runs
  `completed / success`.** Seven workflows succeeded on attempt 1;
  [Qt Runtime run #43587722917](https://github.com/july-rust/BoardSI/actions/runs/43587722917)
  was **CANCELLED in attempt 1** during subsequent main activity;
  that cancellation is retained as a historical result, not rewritten as PASS.
  A **retry of failed/cancelled jobs on the same immutable code SHA**
  completed as **attempt 2: SUCCESS**. Qt Native Windows x64, Windows ARM64,
  macOS ARM64, and Linux x86_64 all report `completed / success` in that
  attempt. The other exact-code Push workflow runs are:
  [Numerical #43587722918](https://github.com/july-rust/BoardSI/actions/runs/43587722918),
  [Docs #43587722951](https://github.com/july-rust/BoardSI/actions/runs/43587722951),
  [M2 native FEM #43587723017](https://github.com/july-rust/BoardSI/actions/runs/43587723017),
  [M2 PH-07 #43587723001](https://github.com/july-rust/BoardSI/actions/runs/43587723001),
  [M2 PH-06 #43587722953](https://github.com/july-rust/BoardSI/actions/runs/43587722953),
  [M2 isolated FEM #43587722924](https://github.com/july-rust/BoardSI/actions/runs/43587722924),
  and [M2 synthetic G0 #43587722927](https://github.com/july-rust/BoardSI/actions/runs/43587722927).
- **Prior review closure:** original source PR #633 had one legitimate P2
  finding (KiCad 9 valued Via flags), explicitly corrected in #635;
  the original inline review thread was answered and marked **RESOLVED**.
  [Documentation PR #634](https://github.com/july-rust/BoardSI/pull/634)
  and [Codex review reconciliation #637](https://github.com/july-rust/BoardSI/pull/637)
  also had Codex reviews **COMPLETED / no findings**. #637 final docs PR
  checks **13 SUCCESS / 16 SKIPPED (docs-only paths)**, protected Verified
  Squash [`add89abab321f2fb3cd61b422739c942b15a283c`](https://github.com/july-rust/BoardSI/commit/add89abab321f2fb3cd61b422739c942b15a283c).
- **Exact #637 docs-main `add89aba` Push CI:** **8/8 completed / success**
  independently of the corrective code-main evidence. [Qt runtime
  #43588413071](https://github.com/july-rust/BoardSI/actions/runs/43588413071),
  [Numerical #43588413142](https://github.com/july-rust/BoardSI/actions/runs/43588413142),
  [Docs #43588413128](https://github.com/july-rust/BoardSI/actions/runs/43588413128)
  and the other five code-independent workflows all succeeded.

## Closeout truth boundary

**E_026 SOURCE-LIMITED COMPLETE (initial checkbox/evidence date: 2026-11-14
KST)** after the corrected parser, exact final PR-head tests, completed
independent Codex reviews, resolved P2, protected Verified Squash merges,
and exact corrective code-main plus docs-main 8/8 successful Push workflows.
This checkbox **only accepts the bounded KiCad 9 `20241229` source
Layer/Via inventory and its source-provenance/fail-closed qualification**.

**Explicitly NOT accepted:** full KiCad PCB import, independent physical
KiCad 9 export/native application Oracle, Qt/worker end-to-end board
integration, real plated/barrel electrical contact, stackup material/z,
global open/short, solver-ready connected graph, SI and formal U07/U33/U38/U40
or T16/T18/T19/T20. Those remain **NOT_VERIFIED / NOT_RUN** and parents
#23 / #532 / #520 remain OPEN. Do not promote synthetic parser test evidence
to native vendor acceptance or confuse docs-only skipped workflows with
new Native PASS. E_027 / E_028 are separate limited gates, not residual
P2 corrective work for E_026.
