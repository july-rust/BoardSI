# C_020 — Legacy B_007 Codex Review and B_003–B_008 Connectivity Evidence Audit

**Audit date / evidence read:** 2026-10-30 KST. **Tracking:** [C_020 Issue #296](https://github.com/july-rust/BoardSI/issues/296). **Parent:** [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**. **Protected baseline:** `main@6728577345280950b9e6aff08047d86d6a2295a2`, after [C_019 documentation PR #295](https://github.com/july-rust/BoardSI/pull/295) merged. This is an **evidence/documentation audit only**, NOT a new Rust/Qt implementation or independently performed product-SI validation.

## 1. Exact original B_007 Codex evidence (direct GitHub reread)

- [B_007 code PR #231](https://github.com/july-rust/BoardSI/pull/231) is **CLOSED/MERGED**. Original final **PR HEAD**: `6a75eb5c5616d86407a04a946bc994958fb55ea6`; original code-merge SHA: [`main@8aae7504417f6aa8d212a0906de658ed173c4ef9`](https://github.com/july-rust/BoardSI/commit/8aae7504417f6aa8d212a0906de658ed173c4ef9).
- The original [latest-head Codex review summary](https://github.com/july-rust/BoardSI/pull/231#issuecomment-6455692908) still says `code_review.commit_sha=6a75eb5c5616d86407a04a946bc994958fb55ea6`, `status=running`, `has_any_findings=null`, `has_unresolved_findings=null`. **Historical original final-HEAD Codex closure = NOT_VERIFIED.** Do not replace that exact-head proof with a completed review of an earlier SHA or a later branch.
- GitHub review submissions returned one Codex `COMMENTED` review dated **2026-10-27T23:08:01Z** whose body says **Reviewed commit `40faa94332`**, not the original final PR HEAD. Three original inline review threads are now marked **resolved**; two are `outdated`. A resolved thread does not constitute a completed Codex review at the later final HEAD.
- The [historical B_007 validation ledger](b007-copper-cutout-connectivity.md) records **28/28 exact-code-HEAD PR checks SUCCESS**, Rust synthetic B_007 **Debug 11/11 + Release 11/11 PASS**, three previous P2 threads resolved, Signed Verified Squash on `8aae7504`, post-main **21/21 Checks / 9/9 Workflows SUCCESS**. During this C_020 reread the connected GitHub workflow API independently returned **12 completed/successful PR-associated workflow runs** for the original final SHA, including Numerical core and Qt runtime. Individual GitHub Check Runs were **not** directly re-enumerated here; retain the historical ledger as the source for the 28/28 and 21/21 figures rather than claiming a new independent check-run count.
- [B_007 follow-up documentation PR #232](https://github.com/july-rust/BoardSI/pull/232) is **CLOSED/MERGED**, its PR HEAD `695e43f62fa911af4f6b278481bbb7fdcc043690`, merge SHA [`43c5d54d5af58d110e98a3260f0e93221c2e4896`](https://github.com/july-rust/BoardSI/commit/43c5d54d5af58d110e98a3260f0e93221c2e4896). The older ledger's unchecked *future documentation PR* line was a historical pre-merge statement, **not** evidence that #232 is still unmerged. The original final-HEAD Codex review uncertainty **is separate** and remains.

**Conclusive distinction:** B_007 restricted geometry code is historically merged and CI-evidenced. Its *specific original final source HEAD independent Codex receipt* was **not recovered as completed**. This audit closes the evidence question as `NOT_VERIFIED`; it does **not** falsify a passing independent review. The new C_020 documentation PR requires its **own** exact-final-HEAD completed Codex review, which cannot retroactively qualify #231's original final HEAD.

## 2. Legacy source-only connectivity evidence cross-check

All B_003–B_008 PRs below were directly rechecked as **merged** in GitHub PR metadata on 2026-10-30. Their detailed regression and CI counts are **pre-existing ledger evidence**, not newly executed C_020 Rust tests.

| Stage | Original PR source HEAD / protected merged main | Restricted verified behavior and source |
| --- | --- | --- |
| B_003 | [#224](https://github.com/july-rust/BoardSI/pull/224) `e211b2e626afd89ac7a880eaeace40e91c8abc50` → `1c11d08be6e7c83d3f6b697f515f61a8c1a43ba8` | Source exact straight Trace, Pad-centre and plated Via contacts; no trace-width/pad-body general connection. [Ledger](b003-b001-reference-topology.md) |
| B_004 | [#226](https://github.com/july-rust/BoardSI/pull/226) `88cd3e034b26ebe83d394cc25fa3d237807989a9` → `cc673ab285af6c8f11314e1b4677d543b5c67415` | One hole-free straight COPCLS source area, same Net/Layer restricted local contacts; no source voids. [Ledger](b004-reference-copper-connectivity.md) |
| B_005 | [#227](https://github.com/july-rust/BoardSI/pull/227) `284f5f04fe3e1d5a07826c0c158893a6c685ea87` → `7a1ed48bf905df8357691a1f51b73e130a74537c` | Whole straight centreline source-order Reference geometry coverage **only**, not current return. [Ledger](b005-reference-geometry.md) |
| B_006 | [#229](https://github.com/july-rust/BoardSI/pull/229) `68cd29313d4b3ccfd340321196e286af0ef4ff80` → `d7a22f8230086cdf8aeb326882708dfee48d9ee1` | Declared plated Through Via centre ↔ source Copper area same-Net/same-Layer contact, strict Padstack/Drill. [Ledger](b006-via-copper-contact.md) |
| B_007 | [#231](https://github.com/july-rust/BoardSI/pull/231) `6a75eb5c5616d86407a04a946bc994958fb55ea6` → `8aae7504417f6aa8d212a0906de658ed173c4ef9` | One strict source-owned straight polygon COPCUT; rejects void interior/boundary Trace/Pad/Via-centre witnesses. **Original exact-HEAD Codex closure NOT_VERIFIED**. [Ledger](b007-copper-cutout-connectivity.md) |
| B_008 | [#233](https://github.com/july-rust/BoardSI/pull/233) `3dc3a681a2259369d3b1396a95f3d988fbad438c` → `9cba34f1eb1135da8e187c8ecb945ad90c78d073` | Entire straight trace centreline rejects intermediate cutout crossings and tangencies, not just endpoint contact. [Ledger](b008-reference-cutout-geometry.md) |

**Cross-check with subsequent C stage:** [C_001](c001-multi-cutout-island-connectivity.md) enlarges the expressly supported straight-COPCUT/island cases; [C_019](c019-mixed-cutout-via-pad-integration.md) rechecks the V9–V12 source receipt ancestry and existing Qt preview. Later successful **synthetic** regressions neither rewrite the original B_007 Codex receipt nor prove manufactured copper fill or physical SI. A suspected new code defect requires an exact reproducible source fixture and a **separate narrow code correction**; no such new code defect was established by this documentary audit.

## 3. Evidence grade and explicit non-claims

**Evidence grade:** historical, merged restricted PADS V9.4 MILS source-geometry behavior, with a separately recorded unresolved historical *review-completion evidence* gap for the final B_007 code HEAD. C_020 does **not** assert it re-ran Rust/Qt tests locally or qualified current Windows x64, Windows ARM64, macOS ARM64 or Linux Qt behavior. Such contemporary CI claims require this C_020 PR's actual HEAD-associated workflow outcomes.

The tested geometry subset requires original Source bytes/SHA, Canonical identity, exact integer ticks, declared Net/Layer/Area identities and Padstack/Drill/Plated Via provenance. Coincident geometry or identical Net labels alone do not prove connectivity. Unsupported curved/overlapping areas, manufactured thermal/antipad/fill, arbitrary pad bodies, arbitrary trace widths and actual continuous AC return path remain outside scope.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

Formal U09/U10/U39, T02/T04/T24/T52 remain **unchecked**. Parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**; no physical RLGC/impedance/S, Return Path/P009, full Scenario/Cache or product-readiness promotion.

## 4. C_020 new PR acceptance gates (not historical B_007 gates)

- [x] Read original and current checklist, B_007/B_009/C_019 ledgers, GitHub B_003–B_008 merged PR metadata, B_007 review submissions/threads and latest-head review summary; **audit date 2026-10-30 KST**.
- [x] Distinguish latest-original-HEAD `NOT_VERIFIED` from three resolved legacy threads and already-merged/CI-accepted source-only behavior.
- [ ] C_020 documentation PR exact final HEAD CI/workflows, including actual Native matrix outcomes, confirmed.
- [ ] C_020 PR completed independent Codex same-final-HEAD findings=0, unresolved=0 and review threads=0.
- [ ] Protected one-parent Signed Verified Squash Merge and independently verified post-C_020-main CI/workflows.
- [ ] Final completion evidence entered into the [living checklist](../development-checklist.md); first checked date not rewritten.

If final PR acceptance needs a separate documentation closeout PR, keep these gates unchecked until objectively evidenced. Never claim that this audit creates the missing historic Codex receipt.


## 5. C_020 verified PR and protected integration (2026-10-30 KST)

- [x] [Audit PR #297](https://github.com/july-rust/BoardSI/pull/297) exact final HEAD [`8c777a3ffc24c63a78ba83fae813feafa20bd76f`](https://github.com/july-rust/BoardSI/commit/8c777a3ffc24c63a78ba83fae813feafa20bd76f/checks): **22/22 GitHub Check Runs SUCCESS**, **8/8 PR-associated Workflows SUCCESS**, including Rustfmt/Clippy/Numerical and actual Qt Windows x64/ARM64, macOS ARM64, Linux offscreen Native CI. C_020 itself changed **only 3 Markdown documents**; inherited Rust/Qt tests passed in CI, not locally run new C_020 code.
- [x] Independent same-final-HEAD PR #297 Codex `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`, zero unresolved Review Threads. This review covers **C_020 HEAD**, never retroactively B_007 PR #231 original final HEAD.
- [x] GitHub protected Squash Merge [`main@fee8ae63e912ced0f8fb823374246949bc0f7edc`](https://github.com/july-rust/BoardSI/commit/fee8ae63e912ced0f8fb823374246949bc0f7edc), direct GitHub REST `verified=true`, `reason=valid`, one parent `6728577345280950b9e6aff08047d86d6a2295a2`; no branch protection bypass.
- [x] **Independent post-C_020-audit-main:** direct GitHub REST on exact [`main@fee8ae63e912ced0f8fb823374246949bc0f7edc`](https://github.com/july-rust/BoardSI/commit/fee8ae63e912ced0f8fb823374246949bc0f7edc/checks) returned **21/21 Check Runs SUCCESS** and **9/9 workflow runs SUCCESS**, including [Push on main #41020421444](https://github.com/july-rust/BoardSI/actions/runs/41020421444), Numerical core, Qt Windows x64/ARM64, macOS ARM64, Linux Offscreen, FEM Native and docs. **2026-10-30 KST.** Distinct independently completed post-main evidence; not an inferred PR-head green.
- [ ] **This final-acceptance documentation PR**: separate exact HEAD required CI, independent Codex review, protected Signed Verified Squash and post-document-main checks must each be independently recorded before Issue #296 is closed.

### Additional direct historical verification

In this C_020 session GitHub REST was later used to enumerate the **original** B_007 exact source HEAD `6a75eb5c5616d86407a04a946bc994958fb55ea6` **28/28 Check Runs SUCCESS**, `main@8aae7504417f6aa8d212a0906de658ed173c4ef9` **21/21 post-main Check Runs SUCCESS**, `verified=true/reason=valid` with one parent, and subsequent documentation `main@43c5d54d5af58d110e98a3260f0e93221c2e4896` also Verified/one parent. These later direct checks supersede section 1's narrower *initial inspection* qualification about counts; they **still do not supply the missing original exact-final-HEAD B_007 completed Codex review receipt**.

All physical SI and full M1 acceptance exclusions from section 3 remain unchanged.
