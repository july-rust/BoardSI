# E_012 — Actual native hardware GPU RHI qualification (bounded, evidence audit)

> 최초 근거 확인 및 체크일: **2026-11-12 KST**. 상위 [Issue #165](https://github.com/july-rust/BoardSI/issues/165)는 전체 GPU 제품 Acceptance를 완료하지 않았으므로 **OPEN** 유지. 이 문서는 **새 하드웨어 계측을 수행했다는 주장이 아닌**, 같은 날짜 GitHub Actions에서 이미 실행된 정확한 code-main Native 원본 로그·Artifact·코드를 독립 재감사한 결과다. E_012는 최종 문서 PR의 필수 CI·독립 리뷰·보호 Verified Squash 및 정확 main Push가 성공한 후에만 한정 **COMPLETE**.

## 1. 한정 수락 범위와 재현 가능한 원본

- **실제 측정 소스:** [code-main `da880d29a4ada85f7f1da77b744a3700b64c1e48`](https://github.com/july-rust/BoardSI/commit/da880d29a4ada85f7f1da77b744a3700b64c1e48) (2026-11-12 KST).
- **Native CI:** [Qt runtime run #43150028282](https://github.com/july-rust/BoardSI/actions/runs/43150028282), `push`, completed SUCCESS. Native Job macOS ARM64 `130956801224`, Windows x64 `130956801228`, Windows ARM64 `130956801231` 모두 SUCCESS. 각 Job의 `A_019 native Metal and Direct3D 10k-1M RHI frame benchmark`, `A_019 native 10k-1M spatial picking, interaction and failover` 단계 SUCCESS.
- **Artifact:** [macOS ARM64 native receipt](https://github.com/july-rust/BoardSI/actions/runs/43150028282/artifacts/13913562203), [Windows x64 receipt](https://github.com/july-rust/BoardSI/actions/runs/43150028282/artifacts/13913991638), [Windows ARM64 receipt](https://github.com/july-rust/BoardSI/actions/runs/43150028282/artifacts/13914202279). 업로드 파일 중 `qt-preview-a019-native-benchmark.json` 및 `qt-preview-a019-interaction-results.json`. GitHub Actions 원본 Job 로그에도 JSON 전문이 남아 있으므로 Artifact 만료 후에도 원본 Run/Job을 우선 참조한다.
- **소스 동일성:** 감사 시 main `1f3cf7040f5653c60a6727c06640dabcfb2d3447`은 위 code-main 이후 **문서 4개만** 변경(체크리스트, E 계획, 검증 색인, E_011 보고서). GPU Qt/RHI 실행 소스·Workflow는 동일. 문서-only main의 Native `SKIPPED`를 새로운 GPU 성공으로 계산하지 않는다.
- **측정 계약:** `apps/boardsi-qt-prototype/a019_gpu_benchmark.h`, `rhi_artwork.h`, `rhi_metal_probe.mm`, `.github/workflows/qt-runtime.yml`에서 이미 구현된 한정 계약. 별도 GPU 코드 변경·새 측정·신규 60 FPS 인증 없음.

## 2. GPU 장치 확인 — 백엔드 명칭만으로 PASS 금지

- **macOS ARM64:** `RHI_FRAME_METAL_HARDWARE_CONFIRMED`. Qt scene graph **render thread**에서 `QSGRendererInterface::DeviceResource`의 실제 `MTLDevice`를 받아 `conformsToProtocol(MTLDevice)` 및 비어 있지 않은 device name을 확인한다. 장치 모델명·드라이버 버전 자체는 JSON에 기록되지 않았으므로 **NOT_RECORDED**.
- **Windows x64/ARM64:** `RHI_FRAME_DIRECT3D11_HARDWARE_CONFIRMED` Native 로그. 실제 `ID3D11Device` → `IDXGIDevice` → `IDXGIAdapter1` → `DXGI_ADAPTER_DESC1`으로 진입하여 `DXGI_ADAPTER_FLAG_SOFTWARE` 여부를 검사한다. Adapter Vendor/Device ID 및 Driver Version의 기록은 **NOT_RECORDED**.
- **공통:** `QSGRendererInterface` 선택, 단순 `frameSwapped` 이벤트, Headless `offscreen`, `QT_QUICK_BACKEND=software`만으로 Hardware PASS로 승격하지 않는다. 3 Native Job은 장치 확인 true, 각 Scene의 최소 8개 유효 연속 frame-swap sample과 `pass=true`를 보고했다. 이는 **GitHub-hosted Native 실행 환경의 장치 확인**이며 사용자 PC 및 물리 모니터의 독립 검증이 아니다.

## 3. macOS ARM64 Metal 실제 Native 계측

| Synthetic primitives | 최초 Frame receipt (ms) | Frame-swap interval p50 / p95 (ms) | Peak process RSS (bytes) | GPU confirmed / interval samples |
| ---: | ---: | ---: | ---: | :--- |
| 10,000 | 141 | 17 / 72 | 104,824,832 | true / 8 |
| 100,000 | 70 | 43 / 51 | 216,481,792 | true / 8 |
| 1,000,000 | 346 | 210 / 231 | 994,902,016 | true / 8 |

각 경우 `backend_status=RHI_FRAME_METAL_HARDWARE_CONFIRMED`, `pass=true`, `failure=""`, `native_platform=cocoa`, `host_arch=arm64`, `source_sha=da880d29...` 일치. Source load/setup은 순서대로 33/28/85 ms. Test Scene은 95% Track, 4% Pad, 1% Via인 공개 합성 Fixture이며 실제 Vendor PCB가 아니다.

**계측 해석:** p50/p95는 `QQuickWindow::frameSwapped`의 **연속 이벤트 간격**이고 GPU shader duration/가속률을 의미하지 않는다. 1M의 p50=210 ms는 60 FPS의 16.7 ms 프레임 간격을 충족하지 못한다. 최초 Frame receipt는 Source setup 이후 포괄 경과시간이므로 직접 GPU rendering-only latency가 아니다. RSS는 전체 프로세스 메모리이며 GPU VRAM이 아니다.

## 4. 보조 Native 근거와 Fail-Closed

| 플랫폼 | 10k/100k/1M A_019 native benchmark | 실제 RHI Hardware 판별 | Synthetic picking/filter/repeated-tab/injected fallback |
| --- | --- | --- | --- |
| macOS ARM64 | SUCCESS / SUCCESS / SUCCESS | Metal true | 3 cases PASS |
| Windows x64 | SUCCESS / SUCCESS / SUCCESS | D3D11 true | 3 cases PASS |
| Windows ARM64 | SUCCESS / SUCCESS / SUCCESS | D3D11 true | 3 cases PASS |

macOS `qt-preview-a019-interaction-results.json`에서 각 3 cases `picking_exact=true`, `layer_filter_restored=true`, `repeated_tab_survived=true`, `injected_error_cpu_fallback=true` 확인. 이는 **synthetic error injection**이고 진짜 GPU driver reset은 `actual_gpu_driver_reset=NOT_RUN`.

한정 Source geometry/CPU parity의 기존 회귀를 보존한다. 이 확인은 임의 PCB, 모든 GPU 드라이버, 실제 GPU Fault, 실제 제조 PCB Source 또는 30 GHz SI Solver를 검증한 것이 아니다.

## 5. 정확히 미측정·미인수인 부분

- GPU VRAM `NOT_MEASURED`, GPU shader timings `NOT_MEASURED`, 실제 GPU 이용률 `NOT_MEASURED`.
- Device 모델/Driver 버전/Windows DXGI Vendor ID `NOT_RECORDED`; Hardware 확인 경로는 소스·로그로 설명 가능하나 구체 장치명은 알 수 없음.
- 1M @ 60 FPS **NOT_VERIFIED**, macOS 1M p50은 210 ms.
- 실제 드라이버 리셋·device loss 재현 **NOT_RUN**; synthetic injected error와 구분.
- 고객 PC, 실제 Monitor/사람 입력 지연, VRAM 및 전체 #165 Windows/macOS 성능/배포/버전드 geometry adapter 인수 **NOT_VERIFIED**.
- 실제 제품 U/T(T11 등), 물리 SI, Solver **NOT_RUN**.

## 6. 종료 조건과 상위 이슈 처리

**E_012의 소형 수락 계약**은 '실제 Native hardware RHI backend 확인 1개 플랫폼 + 10k/100k/1M 원본 정량 프레임 근거 + Fail-Closed 경계 증거 감사'다. 실제 원본 macOS 및 Windows 로그로 기술 근거를 충족했다. 문서 PR의 정확 final HEAD required checks, 독립 review/unresolved 0, 보호 Verified Squash, 정확 docs-main Push 성공까지 확인된 후에만 **E_012 bounded COMPLETE**로 간주한다.

상위 [#165](https://github.com/july-rust/BoardSI/issues/165)는 전체 DoD 불충족으로 **OPEN**, [EB-03](../e-stage-blocked-register.md)은 고객 GPU·VRAM·60 FPS·driver reset 등의 잔여 의존성을 계속 추적한다. E_013 ODB++ ZIP DEFLATE는 독립 진행 가능하다.
