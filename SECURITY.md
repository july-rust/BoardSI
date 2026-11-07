# BoardSI 보안 정책 | Security (Pre-release)

> **현재 상태:** BoardSI에는 아직 승인된 사용자 배포판·정식 릴리즈 버전이 없고, 공식 소프트웨어 라이선스도 선정되지 않았습니다. 공개 GitHub 저장소에 코드가 존재해도 제조/회사 데이터 사용이나 재배포 권리가 자동으로 부여되지 않습니다.
>
> 문서: [전체 안내](docs/README.md) · [개발 환경](docs/getting-started.md) · [기여 규칙](CONTRIBUTING.md).

## 1. 데이터 보호 — 반드시 지킬 것

- **기밀 PCB Layout/CAD Export, 부품/핀 Net 이름, Stackup·재료, ODB++/PADS, Touchstone/IBIS/실측 신호 데이터**를 공개 저장소, 이슈, Pull request, Actions 로그, Release/Artifact에 올리지 않습니다.
- PCB 분석은 **offline-first/local-only**를 원칙으로 합니다. 자동 업로드, Telemetry 또는 외부 Cloud Solver를 제품 필수 동작으로 요구하지 않습니다.
- 개발·자동 검증에는 **권리 상태를 확인한 공개 예제** 또는 의도적으로 만든 합성 데이터를 사용합니다.
- 개인 경로, 계정정보, 토큰, 개인키, IP 주소, 사용자 원본 SHA와 결합된 기밀 객체 이름도 로그·크래시 리포트에 노출하지 않습니다.
- \`.gitignore\`, Secret scanning, CI 환경변수 필터는 방어의 일부일 뿐입니다. **실제 비밀 파일을 저장소에 입력하지 않는 것**이 기본 원칙입니다.

## 2. 보안 결함 신고

GitHub 저장소의 **Security → Report a vulnerability (Private vulnerability reporting)** 메뉴가 활성화되어 있다면 해당 **비공개 신고 기능**을 사용하십시오. 활성 여부는 저장소 설정에 따라 달라지므로 사용 가능하다고 단정하지 않습니다.

비공개 채널이 없는 상태라면 공개 Issue에 **재현 가능한 악성 입력, 인증정보, 실제 회사 PCB, 개인 파일 경로 또는 악용 절차**를 그대로 기재하지 마십시오. 운영자가 공개적으로 안내한 별도 연락 경로가 있는지 확인해야 합니다. 현재 문서에는 검증되지 않은 전용 보안 이메일이나 응답 SLA를 만들지 않습니다.

안전하게 공유 가능한 최소 보고 항목은 문제의 종류, 영향 범위, 취약 버전/커밋, 데이터가 비공개여야 하는 사유, 공개 가능한 합성 재현의 존재 여부입니다.

## 3. 주요 위협과 필요한 방어

| 위협 영역 | 주요 고려사항 | 현 제품 검증 주의 |
| :--- | :--- | :--- |
| **CAD/Archive Import** | 잘못된 길이, 자원 고갈, 파일 경로 탐색, 과도한 압축 해제, 중복/손상 객체, 비정상 코드페이지 | 제한된 PADS 서브셋의 fail-closed 테스트가 모든 형식·아카이브 보안 인수를 뜻하지 않음 |
| **Copper/Net 처리** | 위조된 Source/Net/Layer, Via 도금·Drill·Padstack 혼동, 경계 접선 오인식 | SOURCE 국소 Geometry와 실제 전기적 Open/Short 판정을 분리 |
| **Worker/IPC** | 잘못된 Frame/순서/길이/취소, 오래된 Snapshot/결과, 실행 파일 교체 | 테스트된 특정 시나리오와 전체 위협 모델을 혼동하지 않음 |
| **캐시/프로젝트** | 오래된 Solver Cache, 사용자 파일 경로 유출, 삭제 실패·레이스, 남은 임시 파일 | 실제 배포 전 완전한 검증 필요 |
| **공급망/배포** | 의존성/Qt SDK 라이선스, 타사 바이너리, 서명, Windows/macOS Bundle | CI PASS와 배포 권한/서명/패키징은 별개 |

기본 원칙은 **데이터를 유출하지 않고, 무효 또는 미지원 입력을 성공 결과로 가장하지 않으며, 위험한 작업의 영향을 제한하는 것**입니다. [정식 제품 체크리스트](docs/development-checklist.md)와 [검증 원장](docs/validation/README.md)에서 구현된 보호와 남은 검증을 별도로 확인하십시오.

## 4. 버전·지원 범위

2026-11-07 KST 기준 본 프로젝트는 **Developer Preview** 단계입니다. 기술 데모의 네이티브 회귀시험과 독립 수학 쿠폰 PASS는 보안 완전성, 상용 배포 승인 또는 실물 PCB 분석 적합성을 증명하지 않습니다. 공개 검증 실패를 문서에서 숨기지 않으며, PH-08 원본 Gate Q FAIL 상태도 그대로 유지합니다.
