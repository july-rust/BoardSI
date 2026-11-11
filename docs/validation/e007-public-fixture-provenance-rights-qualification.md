# E_007 — Public fixture provenance, raw SHA-256 and distribution-rights qualification

**Audited:** 2026-11-11 KST. **Parent:** [Issue #7](https://github.com/july-rust/BoardSI/issues/7), M0 F6; U05/U15/U47 and T14/T15/T49. **Starting protected main:** [`81bcf8d90222e9d844dd498d5cb83a2d891796b4`](https://github.com/july-rust/BoardSI/commit/81bcf8d90222e9d844dd498d5cb83a2d891796b4) (GitHub Verified, read before this change). **Scope:** two immutable public KiCad candidate *source* fixtures and the pre-existing synthetic F6 integrity test. This document is **NOT** a legal opinion, redistribution sign-off, commercial release, independent target PC test or BoardSI CAD/SI import acceptance.

## 1. Source provenance and bounded technical evidence

Immutable upstream reference: [KiCad/kicad-source-mirror @ `181d683185b20890085b0e8f1e0e89fbda63a56b`](https://github.com/KiCad/kicad-source-mirror/tree/181d683185b20890085b0e8f1e0e89fbda63a56b). The original directory LICENSE/README texts and directory listings were re-read from that pinned ref on **2026-11-11 KST**. Board bytes were **not** newly downloaded or rehashed in this review; the SHA values and geometry below are the existing historical native-test evidence of [C_025_10](c02510-m0-f6-public-fixture-reconciliation.md), [F6/F7 inventory](m0-f6-f7-qualification-evidence.md) and [Issue #7 native-job history](https://github.com/july-rust/BoardSI/issues/7).

| Candidate | Pinned upstream board source | Original bytes | Raw-file SHA-256 | Git *blob* SHA-1 (not raw-file SHA-256) | Source license file |
| :-- | :-- | --: | :-- | :-- | :-- |
| RoyalBlue54L Feather | [`RoyalBlue54L-Feather.kicad_pcb`](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/royalblue54L_feather/RoyalBlue54L-Feather.kicad_pcb) | 3,549,654 | `535dbe212f43a8bab12b23277e176a94ac9694177258af2981c0e7b08e4b7252` | `5d47510c87e3d4c09bfd18d91dec003a718a6ca6` | [CERN-OHL v2 Permissive text](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/royalblue54L_feather/LICENSE) |
| TinyTapeout demo | [`tinytapeout-demo.kicad_pcb`](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/tiny_tapeout/tinytapeout-demo.kicad_pcb) | 4,541,774 | `0d26bf9b6d561e32cf610d428541d6433e60b45e1838f34f73467e25b9144814` | `a8465878d46656c36617870f511e62c979298854` | [Apache License 2.0 text](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/tiny_tapeout/LICENSE.txt) |

**Previously qualified limited native evidence (not newly executed by this audit):** Ubuntu 24.04 + KiCad 9.0.9 `pcbnew` one valid closed, non-self-intersecting outer contour, zero holes; RoyalBlue54L **58.42 mm × 22.86 mm**, 8 copper layers, 96 declared nets, 71 footprints, 428 pads and 1,126 track/via objects; TinyTapeout **104.50 mm × 81.00 mm**, 4 layers, 115 nets, 309 footprints, 757 pads and 2,548 track/via objects. Historic [hash job 118900157258](https://github.com/july-rust/BoardSI/actions/runs/39501592390/job/118900157258) and [native geometry job 118900157449](https://github.com/july-rust/BoardSI/actions/runs/39501592463/job/118900157449) were recorded successful in Issue #7; earlier audit noted current connector access to those old logs failed (404). **Historical reported SUCCESS is not claimed as a fresh exact-HEAD PASS.** Dimensions/typed object counts are KiCad-native, not cross-tool or BoardSI solver results. Relative complexity does not prove a formally approved small/medium benchmark class.

## 2. Copyright/license distinctions — fail closed

- **Confirmed license document presence and family:** the pinned RoyalBlue54L directory has a CERN-OHL-P v2 text and its README says hardware files use CERN-OHL-V2-P; the TinyTapeout directory has an Apache License 2.0 text. This proves **observed directory metadata**, not that every embedded asset has the necessary grant.
- **Rights not independently qualified:** each PCB's contributors and embedded footprints, symbols/artwork, third-party components and notices, exact BoardSI deliverable boundaries, any separate trademark/attribution obligations, and legal review of specific intended redistribution modes are **NOT_VERIFIED / LEGAL_REVIEW_REQUIRED**. Upstream directory listings include additional libraries/assets: inspect the exact proposed bundle and attributions before any distribution.
- **Allowed in this PR:** metadata links, deterministic synthetic regression data and CI-time pinned public-source fetch. No original external board bytes are copied into BoardSI or release artifacts, and no enterprise/private PCB, stackup, nets, Touchstone or credentials are uploaded.
- **No automatic legal approval:** an observed permissive license heading or successful source hash is insufficient to assert `redistribution_approved=true`. The existing `verify()` explicitly emits `redistribution_legal_clearance=NOT_VERIFIED` and `redistribution_approved=false` for otherwise valid source candidates.

## 3. Current code and adversarial contract

The pre-existing `.github/scripts/m0_f6_board_integrity.py` pins upstream commit, exact raw byte size, SHA-256 and Git blob identity and rejects mismatches; it verifies a license family heading but **cannot** grant redistribution permission. E_007 adds only local **fully synthetic, no-network** negative regression cases to `.github/scripts/test_m0_f6_board_integrity.py`:

1. A same-length tampered board must fail exact-byte hash checks.
2. A truncated board must fail exact size/hash checks.
3. A wrong Git blob ID must fail even if raw SHA-256 matches.
4. A missing license document must fail before any approval claim.

Prior tests also cover a wrong license family and the non-equivalence of a recognized family heading and legal approval. These do **not** enumerate rights holders or independently revalidate source contents. The existing [F6 integrity workflow](../../.github/workflows/m0-f6-board-integrity.yml) triggers on the changed test script, exercises the synthetic suite and checks pinned upstream bytes, retaining only JSON evidence. Inspect the **actual PR-head run and its artifacts/logs** rather than declaring a pass in advance. The KiCad native-outline script was not modified; reuse its existing historical evidence unless a new concrete defect emerges.

## 4. Formal acceptance and external blockers

| Item | Evidence-bounded state |
| :-- | :-- |
| Two original public source paths/commit/metadata | **SOURCE IDENTIFIED**; original hash and byte counts are historical tested evidence, fresh PR F6 run separately pending |
| Source mutation/length/blob/license absent negative guard | **IMPLEMENTED IN E_007 PR**; PASS requires actual exact-head CI |
| Directory license text/family | **OBSERVED** at pinned upstream commit, **not legal approval** |
| Third-party contributor rights, attribution, repackaging permissions | **LEGAL_REVIEW_REQUIRED / BLOCKED** |
| Formal small/medium board size/complexity classification | **NOT_ACCEPTED** (relative native metrics only) |
| Genuine representative ~8 GiB CPU-only reference PC, actual OS/CPU/free RAM/SSD and end-user GUI/disk workload | **NOT_RUN / BLOCKED**; Linux 8 GiB cgroup + 2 CPUs is synthetic |
| Issue #7 whole Definition of Done, M0 F6 and U05/U15/U47, formal T14/T15/T49 | **OPEN / BLOCKED / NOT_ACCEPTED**; no upgraded formal T PASS |
| BoardSI KiCad/PADS/ODB++ import, electrical graph, 30 GHz RLGC/S, PH-08 Gate Q | **NOT QUALIFIED BY E_007**; original PH-08 FAIL remains |

Existing [EB-05](../e-stage-blocked-register.md) records distinct F6/F7 licensing, reference PC and deployment prerequisites rather than inventing a redundant new blocker. Full Issue #7 must remain OPEN until independently supported original DoD; the **bounded E_007 engineering unit** can close after its own regression, code review, protected Squash and exact main Push CI.

## 5. Verification/closeout gate

The **reviewed PR source, latest exact HEAD check-runs**, independent review findings and unresolved threads, GitHub Verified Squash commit, and **exact merged-main Push workflows** are to be recorded from the actual workflow in [Issue #7 E_007 closeout comments](https://github.com/july-rust/BoardSI/issues/7). Do not pre-fill fabricated URLs, dates or PASS outcomes. A documentation-only post-merge native SKIPPED is not a new native PASS. If any gate is not achieved, retain E_007 as EVIDENCE_QUALIFIED rather than asserting COMPLETE. E_008 input/readiness is the next independent work item once this bounded work is closed.
