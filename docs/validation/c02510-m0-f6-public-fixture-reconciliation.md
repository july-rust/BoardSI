# C_025_10 — M0 F6 public reference PCB evidence and Issue #7 safe disposition

**Audit date:** 2026-10-31 KST. **Tracker:** [Issue #330](https://github.com/july-rust/BoardSI/issues/330). **Parent:** [Issue #7](https://github.com/july-rust/BoardSI/issues/7). **Inspected baseline:** protected main@60c52670feb9cbdbb0bab524b01d9635102aa903. This is **evidence reconciliation only**, not a new import test, legal opinion, native rerun, signed artifact or production release. Recheck final PR/main independently.

## 1. Independent scope and distinction

The original source SHA-256 integrity script [m0_f6_board_integrity.py](../../.github/scripts/m0_f6_board_integrity.py) checks immutable KiCad upstream commit, exact expected byte lengths, raw SHA-256, Git blob SHA-1 and the expected license-family heading. The separate [native KiCad 9 outline script](../../.github/scripts/m0_f6_native_kicad_outline.py) verifies official typed board contour/geometry. Its [workflow](../../.github/workflows/m0-f6-native-kicad-outline.yml) pins KiCad 9.0.9 on Ubuntu 24.04. **These are limited technical proofs; license-family text is not redistribution authorization.**

Current connector access to the historical workflow **job logs** 118900157258 and 118900157449 returned HTTP 404 during this C_025_10 audit. Accordingly, reported job measurements below are **existing historical issue-comment/merged-source records, not independently re-executed current native jobs**. No invented fresh PASS. Source and code contracts were fetched again at the audited baseline.

## 2. Pinned public PCB fixtures — accepted limited technical evidence

Upstream [KiCad source commit 181d683185b20890085b0e8f1e0e89fbda63a56b](https://github.com/KiCad/kicad-source-mirror/tree/181d683185b20890085b0e8f1e0e89fbda63a56b) is immutable. The following digest values are pinned in the existing runnable integrity script, **not newly calculated in this audit**.

| Item | RoyalBlue54L Feather | TinyTapeout Demo |
| --- | --- | --- |
| Original board path | demos/royalblue54L_feather/RoyalBlue54L-Feather.kicad_pcb | demos/tiny_tapeout/tinytapeout-demo.kicad_pcb |
| Byte length | 3,549,654 | 4,541,774 |
| Raw file SHA-256 | 535dbe212f43a8bab12b23277e176a94ac9694177258af2981c0e7b08e4b7252 | 0d26bf9b6d561e32cf610d428541d6433e60b45e1838f34f73467e25b9144814 |
| Git blob SHA-1 (different identity) | 5d47510c87e3d4c09bfd18d91dec003a718a6ca6 | a8465878d46656c36617870f511e62c979298854 |
| Candidate upstream license-family heading | CERN Open Hardware Licence Version 2 - Permissive | Apache License Version 2.0 |
| Native closed board outline | 58.42 mm × 22.86 mm, one contour, zero holes | 104.50 mm × 81.00 mm, one contour, zero holes |
| Native copper layers / declared nets | 8 / 96 | 4 / 115 |
| Native footprints / pads / tracks+vias | 71 / 428 / 1,126 | 309 / 757 / 2,548 |

The native typed geometry metrics are taken from the [dated Issue #7 evidence entry](https://github.com/july-rust/BoardSI/issues/7) referencing [KiCad job 118900157449](https://github.com/july-rust/BoardSI/actions/runs/39501592463/job/118900157449). Pinned byte/hash gate: [job 118900157258](https://github.com/july-rust/BoardSI/actions/runs/39501592390/job/118900157258). Original technical promotion and limitation: [PR #144](https://github.com/july-rust/BoardSI/pull/144), [PR #147](https://github.com/july-rust/BoardSI/pull/147). **The relative larger/more complex second board is not a certified normative small/medium class**, which still needs explicit acceptance criteria. Neither is BoardSI KiCad/PADS/ODB++ import, electrical connectivity, or physical SI qualification.

## 3. License and redistribution: known vs unknown

- **Observed technical license metadata:** the pinned integrity script downloads the board directory's license file and checks the named family heading. [RoyalBlue54L license](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/royalblue54L_feather/LICENSE), [TinyTapeout license](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/tiny_tapeout/LICENSE.txt). That check **does not enumerate every embedded design/footprint/artwork contributor** and cannot constitute a legal clearance.
- **NOT_VERIFIED / external review required:** exact release bundle scope, all applicable third-party notices and attribution, any downstream share-alike/source obligations, compatible commercial distribution of the specific files, and distribution approval. No raw board files are added to BoardSI by this audit. Public GitHub availability is not permission for every distribution mode.
- Technical CI fetching pinned public bytes to validate hashes is **not equivalent** to packing third-party board source into a redistributable BoardSI installer.
- Record legal approval only after an independently evidenced review; no fake approval and no changed license notices.

## 4. Reference PC and F7 are separate, still blocked

- The [Linux 8 GiB / 2 CPU synthetic container evidence](https://github.com/july-rust/BoardSI/issues/7) includes constrained RAM and an SDK-detached limited Worker/Qt bundle after [PR #147](https://github.com/july-rust/BoardSI/pull/147). It does **not** qualify an actual representative approximately 8 GiB CPU-only PC with documented OS, CPU, installed/available RAM, SSD, disk workload and representative end-user GUI test.
- F6 formal baseline **BLOCKED**: independent reference PC and license/distribution approvals remain absent. T14/T15/T49 full acceptance **NOT_VERIFIED**; do not convert partial fixture proofs to user acceptance.
- F7 formal **BLOCKED**: clean native Windows x64/ARM64 + macOS ARM64 SDK-free installation proof, full Qt/third-party license manifest, replace/relink obligation and distribution clearance not proven.
- Neither a paid cloud instance nor any confidential PCB/Stackup/Net/Touchstone data may be introduced without explicit authorization.

## 5. Final issue decision and integration gate

| Acceptance item | Status and scope |
| --- | --- |
| Historical pinned board exact bytes/SHA-256, KiCad typed contour/objects | **PASS — prior technical evidence only**; current original job-log re-read **UNAVAILABLE (404)** |
| Source license heading/family check | **PASS — limited string/metadata check**; not a legal judgment |
| Third-party provenance, notices and redistribution rights | **BLOCKED / LEGAL_REVIEW_REQUIRED** |
| Representative ~8 GiB baseline PC, native disk/GUI measurements | **BLOCKED / NOT_RUN** |
| Full Issue #7 Definition of Done, M0 F6, U05/U15/U47, T14/T15/T49 | **OPEN / NOT_ACCEPTED** |
| Product PCB import/30 GHz numerical SI, PH-08 Gate Q, GPU release, Issue #46/#41 | **No status promotion** |

**Merge conditions:** fresh exact PR HEAD required CI, independent current-source review, zero unresolved threads, strict protected Verified Squash, and independently verified post-main CI. Fill actual links after those events; do not pre-certify from this proposal. Parent [Issue #7](https://github.com/july-rust/BoardSI/issues/7) remains OPEN while original DoD is unmet. C_025_10 tracker [#330](https://github.com/july-rust/BoardSI/issues/330) can close after this bounded document/change integration is verified. Preserve [Draft PR #35](https://github.com/july-rust/BoardSI/pull/35) unmerged and historical PH-08 Gate Q FAIL.
