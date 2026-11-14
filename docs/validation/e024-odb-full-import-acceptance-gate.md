# E_024 — ODB++ Full Import Acceptance Gate Reassessment & Fail-Closed Qualification

> **Audit / evidence date:** 2026-11-14 KST. **Baseline:** protected `main@4c5de0ac4668ff4d793f440a8688b84b1ec519f1`, the [E_023 documentation-merge commit](https://github.com/july-rust/BoardSI/commit/4c5de0ac4668ff4d793f440a8688b84b1ec519f1). **Disposition:** E_024 *evidence-based gate review* completed; **whole ODB++ product gate NOT_ACCEPTED / BLOCKED**. The E_024 *PR-delivery closeout* is conditional on its own protected merge and observable CI/review receipts. This review does not newly execute Rust, Qt, physical GPU, vendor fixtures, formal U/T or SI.
>
> **No scope promotion:** `SOURCE subset PASS` below means a dated, evidence-linked prior **limited** implementation/test; it never implies full ODB++ syntax, manufactured electrical continuity, physical SI, full Issue #10 acceptance, or a new E_024 native test run.

## 1. Decision and exact issue boundary

- **Issue [#10](https://github.com/july-rust/BoardSI/issues/10): OPEN**, full ODB++ Design importer. All Definition-of-Done items remain independently assessable; its DoD is **not met** in full.
- **Issue [#578](https://github.com/july-rust/BoardSI/issues/578): CLOSED on 2026-11-12 (administrative/historical)**. That closure is **not** a whole ODB++ technical pass; do not reopen it just to balance status.
- **Issue [#11](https://github.com/july-rust/BoardSI/issues/11): OPEN**, full Qt viewer; **[#520](https://github.com/july-rust/BoardSI/issues/520): OPEN**, source-grounded multi-format via/topology.
- **E_023: `BLOCKED / DEPENDENCY_UNVERIFIED`**, only a candidate-discovery audit finished under [PR #626](https://github.com/july-rust/BoardSI/pull/626). No qualified independent vendor export, same-revision cross-CAD original, rights, raw SHA-256 or actual reference result. See [E_023 evidence](e023-odb-vendor-cross-format-oracle-qualification.md) and [EB-01](../e-stage-blocked-register.md).
- **Whole product verdict:** `NOT_ACCEPTED / BLOCKED` from mandatory missing feature coverage, vendor oracle and formal product acceptance. This is **not** equivalent to a failed newly executed E_024 test. Unsupported/proven-unqualified scope is explicitly `NOT_ACCEPTED`, absent external prerequisites are `BLOCKED`, absent independent receipts are `NOT_VERIFIED`, and formal tests not performed are `NOT_RUN`.
- **E_024 administrative outcome:** bounded audit/document/evidence-disposition job can finish without lying about the product; delivery is complete only after this report/checklist/plan/register PR has passed its own protected rules. Exact post-merge receipts, if observable, belong in the PR / [#10](https://github.com/july-rust/BoardSI/issues/10) closure comment, not fabricated in advance.

## 2. E_013–E_023 prior evidence crosswalk

| Unit | Evidence at dated source/PR HEAD | Accepted boundary / excluded |
| :-- | :-- | :-- |
| [E_013](e013-odb-zip-deflate-qualification.md) | [#601](https://github.com/july-rust/BoardSI/pull/601): exact HEAD 38/38 checks, bounded ZIP STORED/DEFLATE positive/negative regressions, Verified Squash; exact code-main Push 8/8 reported | ZIP method 0/8 **SOURCE transport only**; ZIP64, descriptors, arbitrary CAD and vendor remain unaccepted |
| [E_014](e014-odb-gzip-tar-qualification.md) | [#603](https://github.com/july-rust/BoardSI/pull/603): exact HEAD 30/30, GZIP 7/7 focused regression, Verified Squash, exact code-main Push 8/8 reported | single-member GZIP + bounded USTAR FileMap only; not general multi-member/PAX or electrical proof |
| [E_015](e015-odb-features-z-qualification.md) | [#605](https://github.com/july-rust/BoardSI/pull/605): exact HEAD 30/30, UNIX compress LZW subset and corruption rejection, reported code-main Push 8/8 | accepted restricted `features.Z` SOURCE grammar; no commercial syntax guarantee |
| [E_016](e016-odb-positive-multi-island-qualification.md) | [#608](https://github.com/july-rust/BoardSI/pull/608): exact HEAD 30/30, independent review 0 findings, code-main Push 8/8 | positive multi-island rectilinear SOURCE surface; negative Boolean, arc, intricate holes not accepted |
| [E_017](e017-odb-step-repeat-qualification.md) | [#610](https://github.com/july-rust/BoardSI/pull/610) 30/30 + [#611](https://github.com/july-rust/BoardSI/pull/611) 29/29; code-main Push 8/8 each reported | single-level exact translation, limited transforms and archive parity; no arbitrary nested/rotated step |
| [E_018](e018-odb-eda-package-pin-qualification.md) | [#613](https://github.com/july-rust/BoardSI/pull/613) 30/30, bounded EDA PKG/PIN SOURCE validation, 333/333 Rust tests on reported HEAD | PKG/PIN SOURCE identities only, **no Canonical Component/Pin electrical connection** |
| [E_019](e019-odb-drill-via-candidate-qualification.md) | [#615](https://github.com/july-rust/BoardSI/pull/615) 38/38, SourceDecl/Plated candidate regression, exact code-main Push 8/8 | PLATED STANDARD round drill **candidate only**, no barrel/plating/manufactured via connectivity |
| [E_020](e020-odb-stackup-material-z-qualification.md) | [#617](https://github.com/july-rust/BoardSI/pull/617) 9/9 HEAD workflows, 25/25 jobs; code-main Push 8/8 after retries | optional matrix/stackup.xml Dk/Df/thickness/z declared SOURCE; no physical material validation |
| [E_021](e021-odb-main-qt-import-qualification.md) | [#620](https://github.com/july-rust/BoardSI/pull/620) and [#622](https://github.com/july-rust/BoardSI/pull/622) merged, final #622 HEAD 10/10 PR workflows and Qt 88/88 Linux CTest | one directory STEP, straight Trace and round/rect Pad actual Qt/IPC/Artwork/Picking/reImport; exact merged-main Push / cryptographic verification / final-head independent review **NOT_VERIFIED** |
| [E_022](e022-odb-local-electrical-graph-qualification.md) | [#624](https://github.com/july-rust/BoardSI/pull/624) merged, 9/9 final-head PR workflows, Rust 343/343, 10/10 dedicated tests, Native Qt; historical macOS D006 attempt fail retained | opt-in same-layer straight Trace ↔ round Pad *positive-area source contact* only; final-head Codex review quota blocked; merged-main Push/signature **NOT_VERIFIED** |
| [E_023](e023-odb-vendor-cross-format-oracle-qualification.md) | [#626](https://github.com/july-rust/BoardSI/pull/626) merged candidate-discovery documentation; no file bytes downloaded / no oracle run | **technical BLOCKED / DEPENDENCY_UNVERIFIED**; no vendor qualification |

The stated counts above are **pre-existing report/PR evidence** collected at their original SHAs. E_024 did not freshly rerun these jobs. A prior report's conditional *docs-only PR* closeout should not be silently promoted to unconditional completion without checking its own delivery receipt.

## 3. G01–G15 whole-gate acceptance matrix

**Notation:** `SOURCE PASS (bounded)` is a partial test-supported implementation **not a whole-gate PASS**. `NOT_ACCEPTED` in the last column means mandatory full scope remains unmet; `BLOCKED` identifies an actual prerequisite. References above carry exact known PR/CI identifiers; no new E_024 test SHA is implied.

| Gate | What is actually supported or evidenced | Missing mandatory acceptance | Whole gate |
| :-- | :-- | :-- | :-- |
| **G01 Transport** | E_013–015: directory/USTAR, ZIP 0/8, single-member GZIP/tgz and restricted `features.Z`; bounded CRC/size/path corruption tests **SOURCE PASS (bounded)** | Unlimited ZIP variants/multi-member/PAX and independent supplier sample interoperability not proven | **NOT_ACCEPTED** (unrestricted product) |
| **G02 Copper** | E_016 positive isolated islands, selected precise SOURCE contours **SOURCE PASS (bounded)** | Negative Boolean subtract, curved contours, full nested hole/cutout + complex polarity interactions | **NOT_ACCEPTED** |
| **G03 Step** | E_017 single-level exact translation/archive parity **SOURCE PASS (bounded)** | nested repeats, arbitrary rotations/mirrors, full production step selection semantics | **NOT_ACCEPTED** |
| **G04 EDA** | E_018 bounded EDA package/pin/net source audit **SOURCE PASS (bounded)** | complete CMP/TOP placement, Canonical component/pad-to-pin and actual electrical association | **NOT_ACCEPTED** |
| **G05 Via** | E_019 source-qualified plated round drill candidates and exact spans **SOURCE PASS (bounded)** | actual plating/barrel, complex tools, all via types and canonical electrical edges | **NOT_ACCEPTED** |
| **G06 Stackup** | E_020 optional XML material/thickness/relative z declared SOURCE **SOURCE PASS (bounded)** | independent real PCB stack/material validation, complete reference conductor truth | **NOT_ACCEPTED** |
| **G07 Qt** | E_021 source-only one-STEP straight trace/round-rect pad preview, IPC, Picking and failed-candidate retention **PR-head PASS (bounded)** | full Rust geometry surfaced in Qt, all source types + final-head/remerged-main proof | **NOT_ACCEPTED / NOT_VERIFIED** |
| **G08 Graph** | E_022 10 source-local synthetic tests for same-layer positive contact, net conflict, stale SHA **PR-head PASS (bounded)** | full-board open/short, manufactured via/conductors/reference path, physical graph | **NOT_ACCEPTED** |
| **G09 Oracle** | E_023 names public candidates but neither imports nor raw-hashes them | independent vendor export, matching revision, reuse/CI rights, independent reader, actual compare | **BLOCKED — EB-01** |
| **G10 Revision** | E_020/E_022 cite restricted ODB++Design 8.1-U4 grammar subsets; other reports specify local grammar | authoritative complete revision/object coverage + actual vendor interoperability matrix | **NOT_ACCEPTED** |
| **G11 Fail-closed** | Strong documented negative transport/source/Qt/local-graph synthetic cases | no global proof for every unsupported commercial construct or adversarial concurrent mutations | **NOT_VERIFIED** (whole product) |
| **G12 Native/UI** | E_021/E_022 exact PR-head native Linux/Windows x64/Windows ARM64/macOS ARM64 CI reported | fresh E_024 native run NOT_RUN; clean-host/customer device/performance/product UI qualification absent; exact code-main receipts for E_021/022 unverified | **NOT_VERIFIED** |
| **G13 Formal U/T** | #10 specifies U07/U31/U32/U48 and T03/T16/T21/T51 | independent formal signed acceptance on representative same-revision real boards and complete coverage | **NOT_RUN / NOT_ACCEPTED** |
| **G14 Cross-stage** | Individual source→Canonical and selected Qt or opt-in local-graph synth tests reported | one independent vendor design spanning *all* physical objects through UI/Graph/SI input with oracle | **NOT_VERIFIED** |
| **G15 Repository process** | Prior E_013–020 reports contain bounded PR CI/main Push receipts; E_021/022 PR CI passed | E_021/#620/#622 and E_022/#624 cryptographic Verified / exact independent merged-main Push not evidenced by connected prior logs; final-head review receipt also absent | **NOT_VERIFIED** (those receipts); E_024 PR checks assessed separately |

No invented `G01–G15 = n/15 PASS` score, no conversion of `NOT_RUN` into `FAIL`, and no implication that native CI tests physical SI.

## 4. Issue #10 Definition-of-Done item-by-item

| Original DoD | Decision | Exact reason |
| :-- | :-- | :-- |
| Supported revisions/object coverage documented | **PARTIAL / NOT_ACCEPTED** | restricted grammar sources are documented, not comprehensive complete supported-revision/object coverage |
| Complex geometry, cutouts, step selection, nets and via spans validated | **PARTIAL / NOT_ACCEPTED** | E_016–020 subcases only; unsupported polarity/arc/nesting/manufactured via |
| Independent cross-format fixture comparison | **BLOCKED / NOT_RUN** | E_023 discovery did not produce a qualified vendor same-revision source pair or executed oracle |
| Preserve physical source geometry / provenance and report unsupported info | **PARTIAL / NOT_VERIFIED** | several Source SHA/Identity and reject paths verified in synthetic slices, not full vendor output/cross-stage |
| No confidential design data | **LIMITED EVIDENCE / NOT_VERIFIED (global)** | E_023 states no vendor/private binaries imported; repo-wide confidentiality and complete fixture licensing not independently audited here |

**DoD verdict: NOT_MET; keep #10 OPEN.** The generic planned-work wording in #10 is historical and does not negate the actual bounded E_013–E_022 implementations.

## 5. Formal requirements and acceptance tests (no false PASS)

| Trace | E_024 assessment |
| :-- | :-- |
| **U07** | limited ODB++ transport/Source feature support demonstrated; full specified product requirement **NOT_ACCEPTED** |
| **U31** | bounded Canonical provenance and main Qt source preview demonstrated; full product requirement **NOT_ACCEPTED** |
| **U32** | restricted multi-format/source comparison coverage; independent real vendor parity **BLOCKED / NOT_VERIFIED** |
| **U48** | some fail-closed/unsupported Source handling demonstrated, not full product/format coverage; **NOT_ACCEPTED** |
| **T03** | formal product acceptance **NOT_RUN** by E_024 |
| **T16** | formal product acceptance **NOT_RUN** by E_024 |
| **T21** | formal product acceptance **NOT_RUN** by E_024 |
| **T51** | formal product acceptance **NOT_RUN** by E_024 |

**Source of mappings:** [Issue #10](https://github.com/july-rust/BoardSI/issues/10), [BoardSI v1.0 specification](../specification/BoardSI_Development_Spec_v1.0.md), and [sole product acceptance ledger](../development-checklist.md). Existing A/B/C/D/SEC/U/T aggregate (166 verified / 128 unverified as historically recorded) is not altered by this E audit.

## 6. Fail-closed/security, UI, and physical truth boundaries

- **Transport:** documented bounds, CRC, unsafe paths, duplicate roots/entries, malformed headers and `features.Z` failures are covered **only for supported readers / fixtures**; no claim of all archive variants, arbitrary external contents or adversarial races.
- **Geometry:** rejected unsupported Arc/Area/step or invalid transforms must not silently turn into Copper. A partial successful parser is not license to drop unsupported *physical* geometry on the product path.
- **Net and graph:** a Source Net declaration is not a geometric edge; positive same-layer Copper intersection is not a manufactured barrel/via; `UNKNOWN` must remain `UNKNOWN`. E_022 rejects tangent/separate/cross-layer and conflicting nets in its bounded contract.
- **Via:** `SourceDeclaredPlated` and drill `TYPE=PLATED` do not prove fabrication, Copper-to-barrel contact, electrical layer bridges or RLGC/S.
- **Qt/reImport:** E_021 #622 corrected the earlier bug that cleared/relabelled a previously accepted PCB after a failed candidate. A rejected/cancelled/timed-out candidate must not overwrite accepted artwork, Source SHA, filename or Properties; stale responses must be ignored.
- **Ownership/security:** no external vendor fixture copied or executed as E_024 evidence; rights must be independently cleared before any subsequent download into CI, commit or redistribution.
- **Historical CI:** E_019 macOS reruns and E_020 code-main cancellations + retries are historical, *not* failures erased. E_021/022 independent merged-main proof gaps remain `NOT_VERIFIED`.

**Fresh E_024 runtime test matrix:** Rust `NOT_RUN`; Qt Native `NOT_RUN`; Windows x64 `NOT_RUN`; Windows ARM64 `NOT_RUN`; macOS ARM64 `NOT_RUN`; Vendor cross-format `BLOCKED/NOT_RUN`; formal U/T `NOT_RUN`. This is an *evidence-only audit*. The separate E_024 docs PR will obtain its own required documentation CI; docs-only Native SKIPPED does not replace historical Native PASS.

## 7. Disposition, ownership, and unblocking

| Blocker / deficit | Track in | Reopen only after |
| :-- | :-- | :-- |
| Qualified real exporter + matching CAD revision + rights + raw SHA256 + independent cross-CAD oracle | **[EB-01](../e-stage-blocked-register.md), [E_023](e023-odb-vendor-cross-format-oracle-qualification.md), [#10](https://github.com/july-rust/BoardSI/issues/10)** | independently sourced/authorized bytes and real reproducible comparison, negative tests |
| Negative/cutout/arc + generalized Step / EDA placement / manufactured via / full Qt parity | **[#10](https://github.com/july-rust/BoardSI/issues/10), [#520](https://github.com/july-rust/BoardSI/issues/520), [#11](https://github.com/july-rust/BoardSI/issues/11)** | explicit source grammar + geometrical and fail-closed fixtures, Native coverage per bounded unit |
| Full electrical Open/Short, SI/reference/physical acceptance | **[#520](https://github.com/july-rust/BoardSI/issues/520), relevant product U/T** | manufacturing/stackup provenance, independent electrical and physical validation |
| E_021/022 post-merge exact Push / verification / final-head independent reviews | **E_021/E_022 evidence reports and #10**, not an E_024 invented PASS | raw exact signed commit and `event=push` receipts, reviewer record |
| Formal T03/T16/T21/T51 | **[#10](https://github.com/july-rust/BoardSI/issues/10) and sole checklist** | original formal fixture/results/SHA/job and independent acceptance |

New engineering work must be separately sized for one complete PR/acceptance contract. **Do not open a placeholder implementation PR without prerequisites**. Leave appropriate product tracking issues OPEN; the existence of a BLOCKED product prerequisite must not leave E_024's completed audit PR indefinitely open. Avoid converting #578 historical CLOSED into product PASS.

## 8. Operational delivery ledger (complete after PR, no preclaims)

| Evidence | At report authoring |
| :-- | :-- |
| Immutable baseline | `4c5de0ac4668ff4d793f440a8688b84b1ec519f1` |
| E_024 source code changes | **NONE** (docs-only audit) |
| New E_024 native tests/independent vendor comparison | **NOT_RUN / BLOCKED**, never pretend otherwise |
| E_024 docs report + checklist + E plan/register | **prepared in one audit PR** |
| Exact PR head, required CI, reviews, zero unresolved threads | **PR-specific checks pending; must be evidenced separately** |
| Protected squash merge / cryptographic signature | **pending observable GitHub action; never preclaim** |
| Exact merged-`main` `event=push` | **NOT_VERIFIED until independently observable** |

**Final two different verdicts:**
1. **E_024 audit:** scope-complete **only when the docs PR is properly protected-merged and evidence disposition recorded**; this is a document/traceability result, not software functional acceptance.
2. **ODB++ product gate:** **NOT_ACCEPTED / BLOCKED**, with #10 OPEN, #578 historically CLOSED, E_023 / EB-01 blocked.

[E_025](../e-stage-closure-plan.md) IPC-2581 Source subset can proceed **independently**; it must not inherit an ODB++ whole-product acceptance it does not have.
