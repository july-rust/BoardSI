# C_025_03 — Legacy PADS Import PR reconciliation and safe closure

**Audit date:** 2026-10-31 KST  
**Immutable start baseline:** protected `main@d17b562a6932355438fa9bd80ca740d88eabe20f` ([commit](https://github.com/july-rust/BoardSI/commit/d17b562a6932355438fa9bd80ca740d88eabe20f)).  
**Scope:** historical PR #115, #117 and #118; only repository history, old/new source identity, obsolete workflow disposition and open follow-up issues. **No application code changes, no independent PADS qualification, no solver or release gate changes.**

## Decision matrix — old PRs are closed **without merge**

| PR | Historic HEAD | Verified source/result | Disposition |
| --- | --- | --- | --- |
| [#115](https://github.com/july-rust/BoardSI/pull/115) | `131e91d89ea4a52c94c6a6ce88e8021925d9b156` | Original 184 lines of `crates/boardsi-import/src/lib.rs` match the beginning of present `main` verbatim, with new modules appended. Original `boardsi-import-probe.rs` blob and both signature-fixture blobs are **exactly identical** in `main`. | **SUPERSEDED — CLOSED UNMERGED.** [Closure comment](https://github.com/july-rust/BoardSI/pull/115#issuecomment-6541801525) |
| [#117](https://github.com/july-rust/BoardSI/pull/117) | `c807a3d5cd60ba4254d58b8fc3bfa990dc6557ac` | Same original bounded Signature Probe as #115; current `main` retains core signature logic and exact matching CLI blob. This is *not* proof of identical complete workspace tree. | **SUPERSEDED — CLOSED UNMERGED.** [Closure comment](https://github.com/july-rust/BoardSI/pull/117#issuecomment-6541802248) |
| [#118](https://github.com/july-rust/BoardSI/pull/118) | `8f652e5af9d46af799f6b6bb888abf2a53f3f540` | Old `import_shell.cpp` / CMake `boardsi-qt-import-shell` / `IMPORT_SHELL.md` are absent from current `main`. Current Qt `Open PCB` / `PreviewImportPanel` provides an evolved **different** asynchronous, limited route-import UX. Exact standalone signature-shell behavior and native selftests were **not** proven equivalent. | **ARCHITECTURE SUPERSEDED / DEDICATED SHELL NOT ADOPTED — CLOSED UNMERGED**, with test and format-classification gaps tracked under [Issue #8](https://github.com/july-rust/BoardSI/issues/8). [Closure comment](https://github.com/july-rust/BoardSI/pull/118#issuecomment-6541803015) |

All three PRs were re-read through the connected GitHub API after each close operation: `state=closed, merged=false`. No historical branch was rebased, merged, rewritten or deleted. GitHub historical review and signing records remain accessible.

## Evidence ledger

- **Signature Rust core:** old [PR #115 tree](https://github.com/july-rust/BoardSI/blob/131e91d89ea4a52c94c6a6ce88e8021925d9b156/crates/boardsi-import/src/lib.rs) blob `8fc1fd58d5ddd310c37043cf37a0565fe4e3bf20` (184 lines); present `main` blob `093fcffcf98048504a116c742f0b9fc80db38904` (222 lines). The entire 184-line old source appears unmodified as its initial lines; new module declarations follow. This is **source identity within that bounded core**, not whole-product equivalence.
- **Signature Rust CLI:** old and present `crates/boardsi-import/src/bin/boardsi-import-probe.rs` share blob `f6d7b787f4f7e8d5048eaf6b16015a8a4d6a3577`.
- **Original fixtures:** `tests/fixtures/not_a_pcb.asc` has old/current blob `1b8f15bb88a360a7d7ecaa54171ea9d87b985a1c`; `tests/fixtures/pads_header_only.asc` has `e65441b97300f6c1266867518c90388702968da0`.
- **Expanded tests:** current `crates/boardsi-import/tests/cli.rs` blob `883b0a31171e2b6f06180bca0b4858410ee67a71` is different from original `4afaace9aec21bb6c71199aed74aa484a880ca11`; current file contains additional route/artwork tests. This is not a claim that the old file and the new file are byte-identical.
- **Current Qt UX:** [`preview_p0.cpp`](https://github.com/july-rust/BoardSI/blob/main/apps/boardsi-qt-prototype/preview_p0.cpp) sets `Open PCB` and local `QFileDialog::getOpenFileName`, uses [`preview_import.h`](https://github.com/july-rust/BoardSI/blob/main/apps/boardsi-qt-prototype/preview_import.h) `QProcess` with asynchronous completion, cancellation, input/error inspection and a 10-second timeout. CMake registers `qt-preview-p0-smoke`, `qt-preview-p2a-partial-import` (when `BOARDSI_IMPORTER` is configured), and other synthetic Qt/Import checks.
- **Obsolete Qt UX:** [old `import_shell.cpp`](https://github.com/july-rust/BoardSI/blob/8f652e5af9d46af799f6b6bb888abf2a53f3f540/apps/boardsi-qt-prototype/import_shell.cpp) uses `boardsi-import-probe`, 5-second deadline, 1024-byte output cap, strict expected `PCB/UNKNOWN/SCHEMATIC` selftest status and a *signature-only* result. Old PR #118's independent native selftest was historically unverified; the Codex P1 test-gap finding was not accepted as PASS. Existing Qt route-import tests **do not independently prove** old signature-only shell classification scenarios or its failure-path equivalence.

### Validation truth table

| Check | Result / exact limitation |
| --- | --- |
| Original bounded signature detector and probe CLI present in `main` | **PASS — source/Blob SHA comparison** (no local execution as part of this closure audit) |
| Old #115 and #117 obsolete branch merge requirement | **NOT REQUIRED**; old branches diverged from current `main`, so do not merge |
| #118 new GUI entrypoint availability | **PASS — static source and test registration audit only** |
| Current GUI's direct equivalence with old standalone shell `PCB/UNKNOWN/SCHEMATIC` tests | **NOT VERIFIED**; the old 5-second probe-only binary was never promoted as a separate product target |
| Fresh `cargo test --locked -p boardsi-import` and actual Qt native runtime invocation as part of C_025_03 | **NOT RUN locally**; no connected tool provides direct native build execution. Required exact-HEAD PR checks must be read independently after the audit PR is opened |
| Formal T01/T02/T03/T16/T17/T20/T22, full format/version coverage and G/N/P/S/R/M readiness | **NOT ACCEPTED**; retained open in [#8](https://github.com/july-rust/BoardSI/issues/8) / [#9](https://github.com/july-rust/BoardSI/issues/9) |
| Return path P009, production physical solver, cache validity, release/licensing/PH-08 | **OUT OF SCOPE; unchanged** |

**Classification:** #115 and #117 are source-superceded; #118 is **architecturally obsolete, not functionally equivalent**. Closing a historical PR is workflow cleanup, not formal feature acceptance.

## Follow-up and protection

- [Issue #8](https://github.com/july-rust/BoardSI/issues/8) remains **OPEN** for supported content/signature and version classification, safe unsupported schematic/unknown GUI reporting, native failure-path evidence and G/N/P/S/R/M readiness. [Explicit continuation comment](https://github.com/july-rust/BoardSI/issues/8#issuecomment-6541805371).
- [Issue #9](https://github.com/july-rust/BoardSI/issues/9) remains **OPEN** for real PADS exporter fidelity and full PADS importer acceptance; the current V9.4 MILS subset and synthetic regressions are not general importer certification.
- [M1 #12](https://github.com/july-rust/BoardSI/issues/12) and the C-series limited acceptance remain unchanged. In particular `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false` and `cache_reuse_eligible=false` in the affected limited return-path scope.
- The **documentation-only** C_025_03 PR must pass protected mandatory checks and exact-HEAD independent Codex review before a normal Verified Squash-only merge. Verify post-main signature and CI separately. No bypass; if unavailable, leave PR OPEN and the remaining integration gate NOT VERIFIED.
- `docs/development-checklist.md` was read before writing and updated only for this **historical PR disposition**; no Uxx/Txx completion checkboxes or historical first-check dates changed.
