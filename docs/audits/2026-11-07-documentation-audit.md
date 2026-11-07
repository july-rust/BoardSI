# BoardSI documentation audit & reconciliation — 2026-11-07

**Type:** whole-project *documentation-system* audit, with prioritized in-depth review of current entrypoints, architecture/verification maps and developer instructions. **Not:** formal line-by-line recertification of every date-pinned archived test report, source code or original private PCB fixture.  
**Requested improvement:** a first-time user should find **what BoardSI is, what it can actually do today, how to build it, what passed, what failed and which document governs each claim** without guessing.  
**Tracking:** [Issue #511](https://github.com/july-rust/BoardSI/issues/511) · [Documentation hub](../README.md) · [Single status ledger](../development-checklist.md).  
**Audit source baseline:** protected `main@0549393b8912fbe9bfcde265de4f58560c62e3ee` after D_055 code and record updates; **evidence date 2026-11-07 KST**. CI results for this documentation cleanup itself must be checked at its actual PR-head and are **not** inferred from previous code-PR tests.

## Executive assessment

**Problem found:** the project has excellent detailed *date-specific evidence* but its front door was **chronology-first rather than task-first**. Implemented status, future goals, historic P0/P1 behavior, current PADS/Qt capabilities and M2 physical-research conditions were scattered across different READMEs. That architecture increases onboarding errors and raises the chance of mistaking a passing synthetic test for product readiness.

**Correction:** provide **one navigational layer** above existing records; separate three levels of authority; explain the actual current supported input and geometry subset; provide runnable and safe commands; make negative status and unresolved gates prominent; preserve archived evidence.

### Three levels of documentation authority

| Authority | Source | Editing rule |
| :--- | :--- | :--- |
| **Normative product contract** | [v1.0 specification](../specification/BoardSI_Development_Spec_v1.0.md), U01–U48/T01–T54 | Frozen unless a separate owner-approved versioned amendment is made |
| **Current engineering ledger** | [Single checklist](../development-checklist.md) | Source- and date-backed checked items. It is **not a completion percentage** |
| **Supporting material** | [Architecture](../architecture/README.md), [Validation](../validation/README.md), dated [earlier whole-project audit](2026-10-18-pre-ph08-project-audit.md) and this audit | Historical facts stay date-pinned; later corrections link to the original rather than rewrite it silently |

## Corrected findings and where to verify

| Severity | Finding confirmed in baseline documentation | Correction in this docs-only change | Guard against regression |
| :--- | :--- | :--- | :--- |
| High | **`docs/README.md` absent.** No coherent first-time navigation across requirement/spec, source/architecture, test and implementation records | Added [documentation hub](../README.md), decision table, terms, source-vs-target architecture diagram and status rules | Root README links to docs hub and Quickstart |
| High | Root README led with an old PH-07 merge/reference while later D_054/D_055 and current Preview scope were hard to discover | Rebuilt [repository README](../../README.md) around product purpose, **current support matrix**, limits, exact commands and doc entrypoints | Every high-level "works" entry points to scoped evidence, not release status |
| High | `crates/boardsi-import/README.md` started “this directory only performs signature check”; historical P2-A straight-only narrative contradicted later arc/Pad/Via code | Explicitly separated **signature probe vs geometry importer vs two SOURCE-local candidate diagnostic modes**, labelled P2-A as a historical baseline and fixed a split Markdown hyperlink | Cross-checked flags against actual `boardsi-import-artwork` CLI source and public fixture |
| High | Qt README had dated P0 “Open PCB only selects a filename” and P1 mock-only behavior without a current summary; GPU-first section described an older future target | Added **current P2 / historic P0/P1** scope table and optional Quick/RHI capability note while retaining original measured historical records | A Qt regression **is not** a new-candidate Qt global graph integration claim |
| Medium | Validation README was **M2/PH-08-centric**, missing an obvious entry for the already large M0/A/B/C/D evidence set | Added [validation family matrix](../validation/README.md), exact risk/status legend and links to D_054/D_055 | PASS/PARTIAL/FAIL/NOT_VERIFIED/BLOCKED/NOT_RUN/PROPOSED kept distinct |
| Medium | Architecture index was snapshot-heavy and did not explain new general Copper geometry or which transitions remain unimplemented | Replaced [architecture front page](../architecture/README.md) with Mermaid boundary map, crate responsibilities and product gate table | Dashed unimplemented paths distinguish geometry candidates from physical RLGC solver |
| Medium | No single practical developer setup connecting pinned Rust, actual Qt CMake/Worker, public PADS CLI and normal/diagnostic modes | Added [Quickstart](../developer-quickstart.md) with platform requirements, public synthetic examples, build/test commands and troubleshooting | Cross-check `Cargo.toml`, `rust-toolchain.toml`, `CMakeLists.txt`, launcher and CLI source |
| Medium | Contribution guide referenced a fixed historical count of required workflows and gave little guidance on future documentation quality | Expanded [CONTRIBUTING](../../CONTRIBUTING.md) with verified evidence shape, source-of-truth precedence, docs-only runner optimization and link/date practices | Avoid a second checklist and avoid changing YAML/native checks to edit Markdown |
| Medium | Project management guide described temporary capabilities of a connected AI GitHub tool as a permanent Projects v2 limitation | Replaced with tool-neutral [project workflow](../project-management.md) and explicit optional board vs authoritative Issues/ledger | Project board movement is **not** U/T PASS |
| Medium | Security guide was brief for a proprietary CAD-data threat surface | Expanded [SECURITY](../../SECURITY.md) to cover private reporting, confidentiality, trust boundaries and release limitations | No invented vulnerability channel or claim of passed security audit |

## Source, integration and negative evidence that were cross-checked

- **Repository build identities:** `Cargo.toml` Rust workspace members, `rust-toolchain.toml` Rust 1.75.0, `apps/boardsi-qt-prototype/CMakeLists.txt` Qt 6.5 minimum/C++17/CMake 3.21 minimum and optional Quick/OpenGL backends, `scripts/run-boardsi-preview-windows-x64.ps1` Windows x64-only source/package path.
- **Importer truth:** `crates/boardsi-import/src/bin/boardsi-import-artwork.rs` declares the normal partial protocol plus `--general-trace-contacts-v1` and `--trace-pad-via-contacts-v1`. The former is SOURCE-local trace/trace; the latter SOURCE-local trace/Via conductive annulus/SMD footprint. Neither is a board-wide electrical topology or actual solver output. [D_054–D_055 source/CI evidence](../validation/d054-d055-general-trace-contact-qualification.md).
- **Architecture baseline:** `crates/boardsi-model`, `crates/boardsi-ipc`, `crates/boardsi-numerics`, independent Qt/Worker; no truthful route from current restricted PADS geometry to full validated 30 GHz physical R/L/G/C extraction.
- **Progress:** [A_036 single checklist](../development-checklist.md) and [M0 completion gate](../validation/m0-completion-gate.md). A/B/C/D *scoped* checks and U/T product acceptance deliberately **not conflated**.
- **Physics:** [M2 numerical/physical validation index](../validation/README.md), [independent verification plan](../validation/m2-rlgc-independent-verification-plan.md), [PH-08 original replay](../validation/m2-g2-ph08-stage5b-original-replay-evidence.md). Historical original Gate Q **FAIL** must remain FAIL; synthetic PH-03/05/06/07 PASS does not certify a physical board.

## Reviewed file families / preservation decisions

| Family | Audit action | Decision |
| :--- | :--- | :--- |
| Root user-facing docs | README, CONTRIBUTING, SECURITY | Rewritten to remove misleading status and establish entrypoints |
| Project-wide docs | project management, Preview delivery policy, development history, development checklist | Cross-referenced; updated project-management tool assumptions, **did not re-open frozen requirements/acceptance counts** |
| Implementation docs | Rust Import README, Qt Preview README, `docs/architecture/README.md`, model/numerical/IPC/physical-model contracts | Corrected high-impact active indexes, retained versioned contract semantics |
| Verification docs | `docs/validation/README.md`, M0 gate, A_035, B_009, D_054/D_055 and M2 PH-03–PH-08 category reports | Re-indexed with precise scope; **old raw test evidence kept historically intact** |
| Roadmap metadata | `docs/project-plan.json` | Read as checked planning metadata; **not edited** because JSON would trigger full code/native CI and is not an editorial issue |
| GitHub CI/policy | `.github/workflows/docs-check.yml`, docs-only classifier, protected PR convention | Read for actual docs-only behavior; **no workflow file modified** |
| Brand/source art | approved SVG and icon decision | **No edits**; original ownership/design evidence preserved |

**Coverage qualification:** this is a cross-cutting review of architecture, navigation, current main developer entrypoints, representative high-risk evidence and their status contracts. The date-pinned A/B/C/D and M2 original reports are numerous; this task **does not claim independent line-by-line recreation of every numerical result or every hyperlink within every archived report**. Do not invent a "100% documentation correctness certificate" based on a docs-only PR.

## Executed documentation-link check

For the **11 documents edited or created in this reconciliation**, GitHub connector reads supplied **61 unique local relative-link target files** resolved against the source document directory. **Result: 61/61 target files exist in the documentation PR branch; 0 missing files.** The scan included the top-level README, CONTRIBUTING, SECURITY, new docs index/Quickstart/audit, architecture/validation indexes, project management, and both importer/Qt READMEs.

**Method limit:** this check verifies *file existence* only. It does **not** certify remote HTTP links, GitHub permissions for every reader, heading-anchor accuracy in all unedited records, GitHub Mermaid rendering on all devices, or the scientific correctness of the linked archived data. No source code, workflow, test fixture, JSON metadata or frozen acceptance contract was changed merely to perform the docs-only link check.

## CI-efficiency regression exposed by this audit — NOT YET FIXED

Actual [docs-only PR #512](https://github.com/july-rust/BoardSI/pull/512) changed **11 Markdown files only**, but the current `.github/actions/classify-docs-only/action.yml` returned `docs_only=false` with **`Full check required for: apps/boardsi-qt-prototype/README.md`**. Thus Rust/Qt native Windows/macOS jobs were allocated despite zero application code changes. The allowlist covers root `README.md`/`CONTRIBUTING.md`/`SECURITY.md` and `docs/*.md`-style paths, **not** `apps/*/README.md` or `crates/*/README.md`. In addition, M0 F6/F7 workflow jobs do not consistently share that classifier. This is **verified in the PR classifier job logs**, not a hypothetical limitation.

**Disposition:** [CI corrective Issue #513](https://github.com/july-rust/BoardSI/issues/513) created with exact failure evidence and acceptance criteria; **this documentation cleanup does not quietly change YAML, required check names or branch-protection rules**. Documentation statements that claimed *all* Markdown-only changes skip native builds have been corrected. A dedicated workflow modification must go through full applicable CI before a later all-Markdown PR can prove that optimization.

## Publication and editorial quality gates

1. [Documentation hub](../README.md) is the **only navigation front door**, not a new progress ledger.
2. [Single checklist](../development-checklist.md) remains the **only canonical current check-state accounting**; frozen v1.0 U/T definitions remain untouched.
3. The Quickstart uses **existing source options and public fixture** and warns that diagnostics are not S-parameters.
4. Historical PH-08 original Gate Q FAIL, M0 BLOCKED, unknown licensing and no final installer are **explicit**.
5. Markdown local links and section references should be verified in the PR; if a target does not exist, replace or remove that link rather than creating a fictitious citation.
6. The **required documentation GitHub check** must PASS at the actual docs PR-head. Docs-only native/Qt/FEM skipping is the **intended efficiency goal, presently incomplete** for component READMEs and some F6/F7 jobs ([Issue #513](https://github.com/july-rust/BoardSI/issues/513)). Until fixed, run all required jobs and report the cost honestly. Never bypass branch protection.
7. **Post-main Push CI** is a separate completion gate; do not equate it with the PR-head check if it has not been observed.

## Remaining known limitations (not hidden)

- Full private/vendor PADS export, ODB++ and all shape/Layer/Net configurations are **not** accepted by the importer; that is a **software capability gap**, not solved by documentation.
- D_054/D_055 candidates are not yet promoted to the full Canonical/Qt net connectivity graph and do not resolve Return Path, Via geometry model fitting or product S-parameter physics.
- Complete independent real-board, U01–U48/T01–T54, installation/redistribution and PH-08 original physical acceptance remain separate engineering gates.
- Old date-pinned validation records can legitimately contain feature-status statements that were correct **at their own dates**. The new [status hub](../README.md) prevents treating them as a present-day product declaration.
- Additional file-by-file historical-document review and comprehensive automated relative-link validation may be valuable, but **are not claimed completed** by this scoped documentation PR. A new CI linter would itself change workflow/code files and require the full existing code validation policy.

**Quality standard:** a reader should be able to reach *purpose → execution → design → original proof → limits → next implementation boundary* in a few clearly named links without needing to inspect hundreds of archived PRs.
