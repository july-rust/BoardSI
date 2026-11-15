# F_003 — Native Rust SQLite Writer/Reader qualification

> 2026-11-15 KST · [F_001 contract](../architecture/f001-bsi-format-contract.md) · [F_002 schema](../architecture/f002-bsi-sqlite-schema.md) · [BSIG layout](../architecture/f002-bsi-geometry-binary-layout.md)

## 확정 결과와 범위

- Native `.bsi` v1 Writer/Reader·BSIG 9종 구현 및 CI 실행 검증: **CORE COMPLETE**.
- 구현 [PR #651](https://github.com/july-rust/BoardSI/pull/651) 마지막 HEAD `5766601ba632ab12ce3013ee1040cdf57f23d727`: **47/47 SUCCESS**, review threads **0 unresolved**.
- 보호 Squash [main `cdbf57bd4c596e2643c78b59eee122accfe123d3`](https://github.com/july-rust/BoardSI/commit/cdbf57bd4c596e2643c78b59eee122accfe123d3): **Verified**, exact main push Checks **41/41 SUCCESS**.
- [Native four-OS PR evidence](https://github.com/july-rust/BoardSI/actions/runs/43702554917) · [exact-main native evidence](https://github.com/july-rust/BoardSI/actions/runs/43704091593).
- Product U/T, F_004 historical Golden migrations, F_005 vendor CAD equivalence, Qt native menu integration, EB-01~07 and PH-08 statuses are **not** upgraded by F_003.

## F003-T01–T12 — tested evidence versus untested scenarios

| Gate | Evidence status | Test boundary |
| :--- | :--- | :--- |
| T01/T02 | **PASS** | Rust SQLite save/open, BSIG nine kinds and Debug/Release round trips |
| T03/T04 | **PASS (fixture)** | Exact i64 values, i128 rational, Canonical semantic equality on exercised models |
| T05/T06 | **PASS (synthetic fixture)** | SourceIdentity, material/layer evidence and connectivity references; independent vendor-source proof is out of scope |
| T07/T08 | **PASS (tested negatives)** | Deterministic payload, corrupt digest/kind/unversioned SQL schema fail-closed |
| T09 | **PASS** | Required feature reject, unknown optional/read-only preservation, SHA evidence and future minor Save As guard |
| T10 | **PASS (automated failure boundaries); PARTIAL (physical power-fail)** | Staged SQLite verified before replacement, subprocess forced termination on 4 OS, simulated I/O failure, OS file-size quota, SQLite exclusive lock, and actual Linux 16 MiB tmpfs **ENOSPC** all preserve prior valid file. Actual machine power removal is **NOT_RUN**. |
| T11 | **PASS** | Ubuntu Writer fixture read across Windows x64, Windows ARM64, macOS ARM64 and Linux |
| T12 | **PASS (10k measured fixture/limit gate); PARTIAL (worst case + Qt integration)** | 10,000 Trace Fixture 3,215,360-byte .bsi Save 3,945 ms / Load 1,744 ms, isolated native RSS 35,564 KiB; >512 MiB sparse input rejected, count bound checked, worker subprocess forced termination. Actual full 512 MiB read/write, Qt Save/Open non-blocking UI integration and interactive cancellation are **NOT_RUN**. |

Do not turn a PARTIAL gate into PASS merely because broad CI was green. The implementation and tested native-core scope are complete; **full original F_003 acceptance of every physical failure and resource condition is not yet evidenced**.

## Closing evidence

- PR final HEAD: `5766601ba632ab12ce3013ee1040cdf57f23d727`; verified protected Squash: `cdbf57bd4c596e2643c78b59eee122accfe123d3`.
- 2026-11-15 KST main push: 41/41 checks SUCCESS. All historical PR failed attempts remain visible, with final run passing.
- Exact technical scope and exceptions are reflected in the single authoritative [development checklist](../development-checklist.md).

## 2026-11-16 KST — T10/T12 강화 독립 실행 근거

- 보완 작업 [PR #653](https://github.com/july-rust/BoardSI/pull/653) · [Linux/Windows/macOS 복원력 6개 CI 시험](https://github.com/july-rust/BoardSI/actions/runs/43764651939).
- T10 **실행 PASS**: Windows x64·ARM64, macOS ARM64, Linux에서 검증 완료된 Staging DB의 교체 직전 실제 자식 프로세스 종료; 기존 .bsi 원본 바이트 불변·재로드 성공. 별도의 I/O 실패 주입 후 임시 파일 청소·SQL EXCLUSIVE 잠금 확인.
- Linux **실제 ENOSPC PASS**: 루트 권한으로 독립 16 MiB tmpfs 마운트, 물리적으로 가득 채운 뒤 SQLite 저장 실패와 기존 파일 불변 검증. [실행 로그](https://github.com/july-rust/BoardSI/actions/runs/43764651939/job/132966989385).
- T12 **실행 PASS**: 10,000개 합성 Trace 원본 보존; 실제 Writer 3.945 s, Reader 1.744 s, 파일 3.21536 MB, 컴파일을 제외한 바이너리 최고 RSS 35.564 MB. [단독 프로세스 실행 로그](https://github.com/july-rust/BoardSI/actions/runs/43764651939/job/132966989309).
- **미입증을 성공으로 승격하지 않음**: 실제 PC 전원 차단 및 저장 장치 배리어·캐시 영속성, 512 MiB 최대 크기 실측, 타사 실보드, Qt 메뉴 UI 연결 및 Event Loop 응답성/취소 타임아웃의 F003 Native 입출력 통합.
- 강제 종료 뒤에는 완성된 Staging 임시 파일이 남을 수 있으며 시험 하네스가 이를 삭제했다. **제품 자동 고아 Staging 청소 기능은 이 테스트에서 검증하지 않았다.** 정상 파일의 무결성과 자원 청소는 서로 다른 합격 기준이다.
- 이 진척은 **F_003 Native Core + 자동화 가능한 T10/T12 경계에 한정**한다. 제품 전체 또는 전원 차단 수준의 100% 인증 근거가 아니다.
