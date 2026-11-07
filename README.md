# BoardSI

**PCB & BGA Substrate Signal Integrity — Desktop analysis project**  
**Rust numerical core · Qt desktop UI · Canonical Board Model · Offline-first**

> **현재 단계 / Project status — Developer Preview (제한 검증).** 공개 합성 PADS Layout ASCII V9.4 MILS의 일부 형상과 네이티브 Qt Artwork/Worker 데모를 실행할 수 있습니다. **실제 PCB 전체 전기 연결성, 실물 기반 RLGC/S-parameter 물리 해석, 정식 U/T 인수, 사용자용 배포판은 아직 검증되지 않았습니다.**
>
> **State of development:** restricted source-geometry import and Qt/Rust developer prototypes; **not** a production-qualified PCB SI solver. Historical synthetic 2D FEM numerical passes do not certify measured-board accuracy.

## 무엇을 만드는가? | Product goal

BoardSI는 다양한 PCB CAD 입력을 하나의 **Canonical Board Model**로 통합하고, Stackup·Reference·Port를 확인한 뒤 검증된 **2D transmission-line** 모델로 Impedance/RLGC/S-parameter를 분석하는 데스크톱 프로그램을 목표로 합니다.

**목표 워크플로:** PCB Import → 2D Artwork → Stackup/Material → Reference Net → Signal & Port → Frequency/Resources → Run → Results/Touchstone

- **Importer:** PADS Layout ASCII 우선, ODB++ 등은 별도 어댑터의 후속 목표. ODB++가 내부 표준 데이터 모델인 것은 아닙니다.
- **연결성:** Layer/Net/Source Provenance를 바탕으로 실제 구리 형상과 Via·Pad 접촉을 처리하는 것이 목표.
- **Solver:** 주파수별 RLGC, 단일/차동 Port, S-parameter, 직접/Progressive sweep, 실시간 그래프·Touchstone export는 **개발 목표**이며 이 표기가 제품 검증을 뜻하지 않습니다.
- **범위 경계:** 완전한 3D full-wave EM Solver와 검증되지 않은 불연속 모델은 초기 대상에서 제외합니다. 세부 범위는 [사양서 v1.0](docs/specification/BoardSI_Development_Spec_v1.0.md) 기준입니다.

## 현재까지의 구현과 남은 일

| 분야 | 구현·회귀시험에서 확인된 범위 | 아직 완료되지 않은 제품 범위 |
| :--- | :--- | :--- |
| **Desktop/IPC** | Qt Widgets + 별도 Rust Worker, 제한 Artwork CPU/RHI, 플랫폼별 개발자 실행 시험 | 최종 설치·라이선스·배포 패키지, 전체 SI 사용자 워크플로 |
| **PADS Import** | **제한 PADS V9.4 MILS** 서브셋: 직선 Trace, 일부 원호·Via·Pad 및 Source-owned Copper 형상 | 실제 CAD 원본의 전체 사양 호환, ODB++ 및 광범위 변형 입력 |
| **Copper 접촉** | [D_054–D_055](docs/validation/d054-d055-general-trace-contact-qualification.md): 임의 방향 Trace·원호, Via의 **비중심 환형 구리**와 SMD 원형·사각형·직사각형 Pad 전체 면적 접촉 후보 | 모든 복잡한 Padstack·Pour·Thermal, **전체 전기 Connectivity Graph와 Qt 통합**, 실제 Open/Short |
| **Numerics/FEM** | 입력 기반 scalar RLGC→2-port S 코어, 별도 합성 PEC/lossless FEM 연구 쿠폰 | 실제 PCB 기하/재료→물리 모델·DC–30 GHz 제품 정확도 및 전체 다도체 |
| **공식 인증** | 합성 데이터 및 제한 플랫폼 CI에서의 테스트 기록 | **M0 완료 게이트 BLOCKED**, M1/M2 정식 인수, 사양서 U01–U48/T01–T54, 출시 승인 |

> **검증 표시 규칙:** \`코드 존재\` ≠ \`합성 회귀시험 PASS\` ≠ \`실측 PCB 검증\` ≠ \`정식 제품 인수 PASS\`. 최종 구현/인수 숫자와 증거 SHA는 [단일 개발 체크리스트](docs/development-checklist.md)에서 확인합니다.

## 문서는 어디부터 읽을까?

| 목적 | 문서 |
| :--- | :--- |
| **전체 문서의 시작점** | **[BoardSI 문서 홈 — 역할별 안내와 상태](docs/README.md)** |
| 실제로 코드를 빌드하고 시험하기 | [개발자 시작 가이드](docs/getting-started.md) |
| 요구사항과 공식 인수 기준 | [개발 사양서 v1.0](docs/specification/BoardSI_Development_Spec_v1.0.md) |
| 전체 구현 진행 현황과 미완료 Gate | [공식 개발 체크리스트](docs/development-checklist.md) |
| 모듈 구조와 기하 알고리즘 | [아키텍처 지도](docs/architecture/README.md) · [Trace–Via–Pad 구리 접촉 계약](docs/architecture/trace-copper-contact.md) |
| 시험·재현 증거 | [Validation 방법](docs/validation/README.md) · [**전체 158개 검증 보고서 색인**](docs/validation/INDEX.md) |
| 실행 가능한 제한 GUI와 PADS Import 세부 | [Qt 개발자 README](apps/boardsi-qt-prototype/README.md) · [Importer README](crates/boardsi-import/README.md) |
| 프로젝트 운영·문서 기여 | [Project management](docs/project-management.md) · [Contributing](CONTRIBUTING.md) · [문서 표준](docs/documentation-standards.md) |

## 개발자용 빠른 시작

저장소 [rust-toolchain.toml](rust-toolchain.toml)은 Rust **1.75.0**을 지정합니다.

\`\`\`sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace

# 제한 PADS Import 바이너리와 테스트
cargo build --locked -p boardsi-import --bin boardsi-import-artwork
cargo test --locked -p boardsi-import
\`\`\`

실제 Qt SDK 설정, Rust Worker, 합성 Snapshot 생성 및 CTest 실행은 **[Getting started](docs/getting-started.md)**를 사용하세요. 위 명령의 PASS는 개발자 환경의 기능 시험에 한정됩니다.

**목표 OS:** Windows x64, Windows ARM64, macOS Apple Silicon ARM64. Linux CI는 주로 개발/시험 기준입니다. 네이티브 CI 실행과 배포 가능한 사용자용 설치 패키지는 다릅니다.

## 검증과 위험의 중요한 구분

- **원본 PADS 제한:** 사용자에게 제공된 모든 실 PCB를 지원한다고 주장하지 않습니다. 특히 지원되지 않는 객체는 *구리 접촉을 추측해서 통과*시키지 않습니다.
- **PH-08:** 역사적 Original HMM **Gate Q FAIL**은 재현됐고, 연구 Issue의 행정 종료가 물리적 오류 해결이나 제품 Gate 통과를 뜻하지 않습니다. [PH-08 원본 증거](docs/validation/m2-g2-ph08-stage5b-original-replay-evidence.md).
- **개발·릴리즈:** [M0 완료 게이트](docs/validation/m0-completion-gate.md)와 [제품 전달 계획](docs/preview-delivery-playbook.md)을 별개로 추적합니다.
- **2026-10-18 과거 감사:** [당시 전체 프로젝트 감사](docs/audits/2026-10-18-pre-ph08-project-audit.md)는 역사적 기준점이며, **현재 구현 현황은 체크리스트**가 우선합니다.
- **브랜딩:** 기존 Option B SVG는 소유자가 선택한 디자인 마스터입니다. 원본 PNG 바이트 일치 및 플랫폼별 최종 패키징은 확인되지 않았습니다. [디자인 근거](docs/branding/boardsi-icon-decision.md).

## 데이터 보호 · License · Contributing

BoardSI 개발·실행은 **offline-first**를 원칙으로 합니다. 기밀 PCB Layout/Net/Stackup, 실측 Touchstone, 비공개 파일 경로, 인증정보를 공개 GitHub Issue·PR·Actions·Artifacts에 올리지 마세요. 공개 합성 예제 또는 권리가 검증된 자료만 CI에서 사용합니다.

**아직 공식 소프트웨어 라이선스가 선택되지 않았습니다.** 저장소가 공개되어 있어도 재배포/상업적 이용 권한이 자동 부여되는 것은 아닙니다. [보안 안내](SECURITY.md), [기여 지침](CONTRIBUTING.md).

작업은 [Issues](https://github.com/july-rust/BoardSI/issues)와 [Pull requests](https://github.com/july-rust/BoardSI/pulls)에서 추적합니다. 문서 체계와 원본 검증 기록의 경계는 [2026-11-07 문서 감사](docs/audits/2026-11-07-documentation-review.md)에서 설명합니다.
