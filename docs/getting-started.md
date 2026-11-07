# BoardSI 개발 환경 및 실행 시작하기

> **대상:** 제한된 개발 프리뷰/공개 합성 입력을 직접 실행·검증하려는 개발자. **이것은 완제품 설치 안내서가 아닙니다.** 실제 회사 PCB 파일이나 기밀 측정 데이터는 GitHub·CI에 올리지 않습니다.
>
> 상위 문서: [문서 안내](README.md) · [프로젝트 소개](../README.md) · [Qt 앱 상세](../apps/boardsi-qt-prototype/README.md) · [Importer 상세](../crates/boardsi-import/README.md).

## 1. 먼저 구분할 실행 대상

| 실행 대상 | 목적 | 입력 | 출력/한계 |
| :--- | :--- | :--- | :--- |
| \`boardsi-import-probe\` | PADS ASCII 첫 줄 형식 서명 확인 | 로컬 \`.asc\` | \`SIGNATURE_ONLY\`. 지오메트리, Net 또는 SI 검증 없음 |
| \`boardsi-import-artwork\` | 지원하는 PADS V9.4 MILS 서브셋을 제한적으로 읽는 Artwork Import | 공개 합성/로컬 원본 PADS | 버전형 부분 Artwork; 미지원 항목은 부분/거부 처리, \`SI_NOT_RUN\` |
| \`boardsi-import-artwork ... --general-trace-contacts-v1\` | 임의 방향 직선·원호 Trace 간 SOURCE 국소 접촉 후보 | 제한 원본 | **국소 기하 후보**, 전체 연결 그래프 아님 |
| \`boardsi-import-artwork ... --trace-pad-via-contacts-v1\` | Trace↔Via 환형 구리/부품 Pad의 비중심 접촉 후보 | 제한 원본 | **국소 기하 후보**, 실제 전기적 Open/Short 아님 |
| \`boardsi-worker\` | 독립 Rust Worker/합성 계산·IPC | 합성 Snapshot/IPC | 실제 PCB 기반 Solver가 아닌 제한 데모 |
| \`boardsi-preview-p0\` | 네이티브 Qt Widgets 개발자 프리뷰 | Rust Worker + 공개 합성 프레임 | 별도 설치 패키지/제품 SI 아님 |

## 2. 개발 환경

- **Rust:** 저장소의 [rust-toolchain.toml](../rust-toolchain.toml)은 **1.75.0**과 \`rustfmt\`, \`clippy\`를 지정합니다. 이는 현재 **임시 구현 툴체인**으로, 영구적인 배포 런타임 승인과 다릅니다.
- **Qt/CMake:** 기존 Qt 앱 README의 실제 검증 예는 **Qt 6.8.3**, CMake 3.31.6, C++17입니다. 기존 CMake 최소 Qt 버전 **6.5**와 *시험한* SDK 버전을 혼동하지 마세요. 렌더링 대상에 따라 Qt Quick/RHI 모듈도 필요할 수 있습니다.
- **대상 OS:** Windows x64, Windows ARM64, macOS ARM64. Linux는 주로 CI/개발자 검증 경로입니다. 플랫폼별 바이너리·Qt SDK·기능이 자동으로 배포됐다는 뜻이 아닙니다.
- **입력:** 저장소에 포함된 공개 합성 PADS 픽스처만으로 먼저 재현하십시오. 라이선스와 실물 PCB 파서 호환성이 확인되지 않은 데이터는 저장소에 추가하지 마세요.

## 3. Rust 코어 검증

저장소 루트에서 실행합니다.

\`\`\`sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
\`\`\`

Importer만 빠르게 검증할 때:

\`\`\`sh
cargo build --locked -p boardsi-import --bin boardsi-import-artwork
cargo test --locked -p boardsi-import
\`\`\`

**판정 기준:** 명령이 종료 코드 0을 반환해도 특정 개발 환경의 테스트가 통과했다는 뜻일 뿐, **실제 PCB 전기적/물리적 SI 시험을 통과했다는 뜻은 아닙니다.** 정식 기준은 [사양서 U/T](specification/BoardSI_Development_Spec_v1.0.md)와 [상태 체크리스트](development-checklist.md)를 참조하십시오.

## 4. 오프센터 Trace–Via–Pad 접촉 진단

PADS 지원 서브셋의 공개 합성 입력에 한정하여 실행합니다. 아래 경로는 예시이므로 실제 저장소 파일을 지정합니다.

\`\`\`sh
# Rust 빌드 후 저장소 루트 기준
./target/debug/boardsi-import-artwork crates/boardsi-import/tests/fixtures/d040_multi_component_v94.asc --trace-pad-via-contacts-v1

# 직선 및 원호 Trace 사이의 유한 선폭 기하 접촉
./target/debug/boardsi-import-artwork crates/boardsi-import/tests/fixtures/d040_multi_component_v94.asc --general-trace-contacts-v1
\`\`\`

Windows에서는 실행 파일에 \`.exe\`를 붙이고 Windows 경로를 사용합니다. 정상 진단 결과는 입력 파일의 SHA-256과 \`SOURCE_LOCAL_ONLY\`, \`SI_NOT_RUN\` 등의 한계를 표시합니다. 서로 다른 선언 Net의 기하학적 중첩은 **충돌 후보**이지 자동 전기적 Short 판정이 아닙니다.

판정 범위: [범용 Trace–Via–Pad 기하 계산 계약](architecture/trace-copper-contact.md). 미지원 객체/근접 접선/잘못된 원본을 임의로 연결 성공 처리하지 않는 **Fail-Closed** 원칙을 따릅니다.

## 5. Qt GUI/Worker 개발자 빌드

검증된 Linux 개발자 흐름의 예입니다. SDK 경로와 Rust 실행 파일은 실제 환경에 맞게 **절대 경로**로 지정하십시오.

\`\`\`sh
cargo build --locked -p boardsi-worker
cmake -S apps/boardsi-qt-prototype -B build-qt \
  -DCMAKE_PREFIX_PATH=/absolute/qt/prefix \
  -DBOARDSI_WORKER=/absolute/path/to/target/debug/boardsi-worker
cmake --build build-qt --parallel 2

# 독립된 공개 합성 모델 A/B 테스트 프레임
cargo run --locked -p boardsi-worker --example m0_model_snapshot_fixture -- \
  build-qt/qt-model-a.frame build-qt/qt-model-b.frame

QT_QPA_PLATFORM=offscreen ctest --test-dir build-qt --output-on-failure

# 준비된 데스크톱에서 실제 개발자 Preview 실행
./build-qt/boardsi-preview-p0 /absolute/boardsi-worker \
  /absolute/qt-model-a.frame /absolute/qt-model-b.frame
\`\`\`

주의: (1) Windows CMake 다중 설정 생성기에서는 \`Debug\` 등의 출력 하위 폴더를 확인해야 합니다. (2) Windows PowerShell에서는 \`$env:QT_QPA_PLATFORM="offscreen"\`으로 지정합니다. (3) \`ctest\`보다 먼저 합성 프레임 A/B를 생성해야 합니다. (4) GUI가 실행돼도 실제 PCB 물리 해석이 구현된 것은 아닙니다.

상세 UI 역할·환경 변수·부속 RHI 빌드는 [Qt 개발자 README](../apps/boardsi-qt-prototype/README.md), 플랫폼별 실패/성공 증거는 [검증 원장](validation/README.md)에서 확인하십시오.

## 6. 작업 결과를 어떻게 증명할까?

1. **대상 SHA 고정:** 테스트 명령, 워크플로 실행, 소스 변경이 동일 Git 커밋을 기준으로 하는지 확인합니다.
2. **입력 데이터 표시:** 합성 픽스처/공개 레퍼런스/실물 비공개 입력을 구분합니다. 기밀 입력의 경로나 내용은 공개하지 않습니다.
3. **검증 종류 분리:** 형식 파싱 → 기하 접촉 → 전체 전기 연결 그래프 → RLGC/S 물리 모델 → 실제 PCB 대조를 다른 Gate로 관리합니다.
4. **거짓 완료 방지:** \`NOT_RUN\`, \`NOT_VERIFIED\`, \`BLOCKED\`, \`FAIL\`을 \`PASS\`로 바꾸지 않습니다.
5. **정식 반영:** 코드 PR과 CI 증거를 남기고, 날짜와 근거를 [단일 체크리스트](development-checklist.md)에 기록합니다. 순수 문서 수정은 [Docs-only CI 정책](../CONTRIBUTING.md)을 따릅니다.

더 긴 작업 흐름은 [Developer Preview 전달 계획](preview-delivery-playbook.md)에서 확인할 수 있습니다.
