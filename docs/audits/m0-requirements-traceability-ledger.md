# BoardSI M0 — Requirement / Test Traceability Ledger

**Date:** 2026-10-18 KST. **Baseline:** specification v1.0, section 2 requirements and section 13 tests, crosswalk section 18.2. **Scope:** metadata integrity / historical prototype evidence only, NOT new product acceptance or physical solver certification.

## Interpretation and source control

- **DEFINED** is a documented requirement, not implementation. **PARTIAL** means only restricted M0 prototype evidence, not a passed full T acceptance. **NOT_RUN** means no complete ID-specific acceptance evidence is established in this ledger. All T01–T54 require case-by-case evidence before a PASS.
- The section 18.2 grouped mapping is a **coarse, group-level association**, **not** proof that every test fully validates every requirement in that group. Precisely map individual T cases as implementation matures.
- T44 was present in the specification's test definitions but absent from section 18.2; it is now linked under the S-parameter/export functionality U01, without changing any test criterion.
- M1–M5 functionality and actual measured PCB validation remain NOT_RUN here. No complete release or original HMM Gate Q claim.

## U01–U48, full coverage (48/48)

| Requirement ID | Phase from v1.0 18.2 | Linked test IDs | Definition | Implementation | Full acceptance | Evidence scope |
| --- | --- | --- | --- | --- | --- | --- |
| U01 | M2 | T06,T07,T43,T44 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U02 | M0 | T13,T39 | DEFINED | PARTIAL | NOT_RUN | M0 restricted prototype; see #3/#4/#5/#6 |
| U03 | M0 및 릴리즈 | T13,T39 | DEFINED | PARTIAL | NOT_RUN | M0 restricted prototype; see #3/#4/#5/#6 |
| U04 | M0~M2b | T10,T11,T32,T34,T35,T46,T50,T54 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U05 | 전체 단계 | T14,T15,T40,T49 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U06 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U07 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U08 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U09 | M1 | T02,T04,T24,T52 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U10 | M1 | T02,T04,T24,T52 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U11 | M1b | T05,T08,T23,T24,T25,T26,T27,T28 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U12 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U13 | M0~M2b | T10,T11,T32,T34,T35,T46,T50,T54 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U14 | M0 및 릴리즈 | T13,T39 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U15 | 전체 단계 | T14,T15,T40,T49 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U16 | 전체 단계 | T15,T40 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U17 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U18 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U19 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U20 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U21 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U22 | M0~M2b | T10,T11,T32,T34,T35,T46,T50,T54 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U23 | M0~M2b | T10,T11,T32,T34,T35,T46,T50,T54 | DEFINED | PARTIAL | NOT_RUN | M0 restricted prototype; see #3/#4/#5/#6 |
| U24 | M0~M2b | T10,T11,T32,T34,T35,T46,T50,T54 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U25 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U26 | 기하 M1a, 정량 M2c | T36,T37,T38,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U27 | 기하 M1a, 정량 M2c | T36,T37,T38,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U28 | 기하 M1a, 정량 M2c | T36,T37,T38,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U29 | 기하 M1a, 정량 M2c | T36,T37,T38,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U30 | 기하 M1a, 정량 M2c | T36,T37,T38,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U31 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | PARTIAL | NOT_RUN | M0 restricted prototype; see #3/#4/#5/#6 |
| U32 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U33 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U34 | M1b | T05,T08,T23,T24,T25,T26,T27,T28 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U35 | M1b | T05,T08,T23,T24,T25,T26,T27,T28 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U36 | M1b | T05,T08,T23,T24,T25,T26,T27,T28 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U37 | M1b | T05,T08,T23,T24,T25,T26,T27,T28 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U38 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U39 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | PARTIAL | NOT_RUN | M0 restricted prototype; see #3/#4/#5/#6 |
| U40 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U41 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U42 | 결합 모델 제공 시 M2 | T42 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U43 | M2a | T45,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U44 | M0~M2b | T32,T46,T50 | DEFINED | PARTIAL | NOT_RUN | M0 restricted prototype; see #3/#4/#5/#6 |
| U45 | M2b | T12,T47,T54 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U46 | M2~M2a | T09,T29,T30,T31,T33,T41,T48,T53 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U47 | 단계별 게이트 | T06,T10,T12,T41,T50 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |
| U48 | I1은 M1, 확대 M3~M5 | T01,T02,T03,T16,T17,T18,T19,T20,T21,T22,T27,T51 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |

## T01–T54, full coverage (54/54)

| Test ID | Related stage(s) | Linked requirement IDs (coarse) | Complete acceptance result | Evidence |
| --- | --- | --- | --- | --- |
| T01 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T02 | I1은 M1, 확대 M3~M5; M1 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48,U09,U10 | NOT_RUN | No complete case-specific T acceptance claimed |
| T03 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T04 | M1 | U09,U10 | NOT_RUN | No complete case-specific T acceptance claimed |
| T05 | M1b | U11,U34,U35,U36,U37 | NOT_RUN | No complete case-specific T acceptance claimed |
| T06 | M2; 단계별 게이트 | U01,U47 | NOT_RUN | No complete case-specific T acceptance claimed |
| T07 | M2 | U01 | NOT_RUN | No complete case-specific T acceptance claimed |
| T08 | M1b | U11,U34,U35,U36,U37 | NOT_RUN | No complete case-specific T acceptance claimed |
| T09 | M2~M2a | U12,U17,U18,U19,U20,U21,U25,U41,U46 | NOT_RUN | No complete case-specific T acceptance claimed |
| T10 | M0~M2b; 단계별 게이트 | U04,U13,U22,U23,U24,U47 | NOT_RUN | No complete case-specific T acceptance claimed |
| T11 | M0~M2b | U04,U13,U22,U23,U24 | NOT_RUN | No complete case-specific T acceptance claimed |
| T12 | M2b; 단계별 게이트 | U45,U47 | NOT_RUN | No complete case-specific T acceptance claimed |
| T13 | M0; M0 및 릴리즈 | U02,U03,U14 | PARTIAL | M0 native Qt/Worker prototype only; see #3/#5/#6; full T unqualified |
| T14 | 전체 단계 | U05,U15 | NOT_RUN | No complete case-specific T acceptance claimed |
| T15 | 전체 단계 | U05,U15,U16 | NOT_RUN | No complete case-specific T acceptance claimed |
| T16 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T17 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T18 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T19 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T20 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T21 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T22 | I1은 M1, 확대 M3~M5 | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48 | NOT_RUN | No complete case-specific T acceptance claimed |
| T23 | M1b | U11,U34,U35,U36,U37 | NOT_RUN | No complete case-specific T acceptance claimed |
| T24 | M1; M1b | U09,U10,U11,U34,U35,U36,U37 | NOT_RUN | No complete case-specific T acceptance claimed |
| T25 | M1b | U11,U34,U35,U36,U37 | NOT_RUN | No complete case-specific T acceptance claimed |
| T26 | M1b | U11,U34,U35,U36,U37 | NOT_RUN | No complete case-specific T acceptance claimed |
| T27 | I1은 M1, 확대 M3~M5; M1b | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48,U11,U34,U35,U36,U37 | NOT_RUN | No complete case-specific T acceptance claimed |
| T28 | M1b | U11,U34,U35,U36,U37 | NOT_RUN | No complete case-specific T acceptance claimed |
| T29 | M2~M2a | U12,U17,U18,U19,U20,U21,U25,U41,U46 | NOT_RUN | No complete case-specific T acceptance claimed |
| T30 | M2~M2a | U12,U17,U18,U19,U20,U21,U25,U41,U46 | NOT_RUN | No complete case-specific T acceptance claimed |
| T31 | M2~M2a | U12,U17,U18,U19,U20,U21,U25,U41,U46 | NOT_RUN | No complete case-specific T acceptance claimed |
| T32 | M0~M2b | U04,U13,U22,U23,U24,U44 | NOT_RUN | No complete case-specific T acceptance claimed |
| T33 | M2~M2a | U12,U17,U18,U19,U20,U21,U25,U41,U46 | NOT_RUN | No complete case-specific T acceptance claimed |
| T34 | M0~M2b | U04,U13,U22,U23,U24 | NOT_RUN | No complete case-specific T acceptance claimed |
| T35 | M0~M2b | U04,U13,U22,U23,U24 | NOT_RUN | No complete case-specific T acceptance claimed |
| T36 | 기하 M1a, 정량 M2c | U26,U27,U28,U29,U30 | NOT_RUN | No complete case-specific T acceptance claimed |
| T37 | 기하 M1a, 정량 M2c | U26,U27,U28,U29,U30 | NOT_RUN | No complete case-specific T acceptance claimed |
| T38 | 기하 M1a, 정량 M2c | U26,U27,U28,U29,U30 | NOT_RUN | No complete case-specific T acceptance claimed |
| T39 | M0; M0 및 릴리즈 | U02,U03,U14 | PARTIAL | M0 native Qt/Worker prototype only; see #3/#5/#6; full T unqualified |
| T40 | 전체 단계 | U05,U15,U16 | NOT_RUN | No complete case-specific T acceptance claimed |
| T41 | M2~M2a; 단계별 게이트 | U12,U17,U18,U19,U20,U21,U25,U41,U46,U47 | NOT_RUN | No complete case-specific T acceptance claimed |
| T42 | 결합 모델 제공 시 M2 | U42 | NOT_RUN | No complete case-specific T acceptance claimed |
| T43 | M2 | U01 | NOT_RUN | No complete case-specific T acceptance claimed |
| T44 | M2 | U01 | NOT_RUN | No complete case-specific T acceptance claimed |
| T45 | M2a | U43 | NOT_RUN | No complete case-specific T acceptance claimed |
| T46 | M0~M2b | U04,U13,U22,U23,U24,U44 | PARTIAL | M0 native Qt/Worker prototype only; see #3/#5/#6; full T unqualified |
| T47 | M2b | U45 | NOT_RUN | No complete case-specific T acceptance claimed |
| T48 | M2~M2a | U12,U17,U18,U19,U20,U21,U25,U41,U46 | NOT_RUN | No complete case-specific T acceptance claimed |
| T49 | 전체 단계 | U05,U15 | NOT_RUN | No complete case-specific T acceptance claimed |
| T50 | M0~M2b; 단계별 게이트 | U04,U13,U22,U23,U24,U44,U47 | NOT_RUN | No complete case-specific T acceptance claimed |
| T51 | I1은 M1, 확대 M3~M5; 기하 M1a, 정량 M2c | U06,U07,U08,U31,U32,U33,U38,U39,U40,U48,U26,U27,U28,U29,U30 | NOT_RUN | No complete case-specific T acceptance claimed |
| T52 | M1 | U09,U10 | NOT_RUN | No complete case-specific T acceptance claimed |
| T53 | M2~M2a; M2a | U12,U17,U18,U19,U20,U21,U25,U41,U46,U43 | NOT_RUN | No complete case-specific T acceptance claimed |
| T54 | M0~M2b; M2b | U04,U13,U22,U23,U24,U45 | NOT_RUN | No complete case-specific T acceptance claimed |

## Scope and blocking actions

1. **M0 F1 is not PASS** merely because all U/T identifiers appear here. Reconcile each case-specific expected result, fixture, executed job, immutable SHA and reviewer before progressing from PARTIAL/NOT_RUN.
2. Link per-case implementation evidence for M0 #2–#7, preserving incomplete acceptance; keep M1–M5 unrun until their own milestone.
3. Resolve M0 baseline decisions (Qt/IPC release architecture, toolchain, low-RAM PC, fixtures/licenses), and test model→snapshot→Worker→Qt minimal integration. Qt #46 root cause remains OPEN.
4. Resolve audit PR #81 and protect Ruleset 25046595 without changing PH-03/05/06/07 frozen contracts or original PH-08 Gate Q FAIL.

**Sources:** [specification](../specification/BoardSI_Development_Spec_v1.0.md), [M0 architecture](../architecture/README.md), [model tests](../validation/m0-canonical-board-model.md), [native Qt tests](../validation/m0-qt-native-runtime.md), [M0 issues](https://github.com/july-rust/BoardSI/issues/2), [audit #78](https://github.com/july-rust/BoardSI/issues/78).
