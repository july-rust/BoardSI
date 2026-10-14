# BoardSI 개발 사양서

**문서 버전 v1.0 · 최종 확정용**

**BoardSI — Lightweight PCB & BGA Substrate Signal Integrity Program**  
**개정일:** 2026-10-14 (KST) · **문서 상태:** 전면 검토 및 수정 완료 · 사용자 최종 확정용 · **프로그램 릴리즈와 무관**

> **중요:** BoardSI는 ODB++ 기반 프로그램이 아니다. PADS ASC, ODB++, IPC-2581, KiCad, Gerber 및 다른 PCB 입력 형식 가운데 한 가지 또는 연관된 보조 파일 묶음을 Import한 뒤, **형식에 독립적인 BoardSI 내부 PCB 모델**로 정규화하고 그 모델로 2D Artwork·전기적 연결·Port·RLGC·S-parameter 해석을 수행한다. GUI 화면용 도형과 Solver 기하가 별개로 어긋나는 구조를 금지한다.

## 1 제품 목표 및 문서 적용 원칙

BoardSI는 사용자가 선택한 PCB 신호/결합 신호 그룹을 저사양 환경에서도 검토할 수 있는 **로컬·오프라인 경량 SI 분석 프로그램**이다. Stackup 및 배선 폭·두께 변경을 가상 시나리오로 비교하고, 경량 RLGC/전송선로 모델과 **검증된** 불연속 모델로 임피던스·S-parameter를 계산한다. PCB 및 BGA/FC-BGA Substrate의 Pad-to-Pad 범위를 대상으로 하되 전체 3D Full-wave EM 엔진은 초기 제품 범위에서 제외한다.

| 분류 | v1.0 기준 |
| --- | --- |
| 제품명 | BoardSI (확정) |
| 엔진 / UI | Rust 계산 코어 + Qt GUI (확정 방향) |
| 입력 | 다양한 PCB/제조 데이터 Import; PADS ASC와 ODB++ 우선, 점진적 확대 |
| 기준 데이터 | 원본 중립화 → **BoardSI Canonical Board Model** → Viewer·Port·Solver 공통 활용 |
| 대상 | PCB와 BGA/FC-BGA Substrate, 선택 Net/Pair/Analysis Group |
| 해석 핵심 | 임피던스, 단위 길이 RLGC, Single/Mixed-Mode S-parameter, Touchstone |
| Sweep | 0~30 GHz 등을 사용자가 지정; Progressive/Hierarchical 선형 스윕과 고정 Direct 기준선 |
| 실행 | Run 및 Import 동안 GUI 비차단; 그래프 실시간 갱신; 취소·일시정지·재개; RAM/SSD 캐시 |
| 플랫폼 | Windows x64 / Windows ARM64 / macOS Apple Silicon arm64 |
| 보안 | 회사 PCB/Net/Stackup/결과 외부 전송 없이 오프라인 처리 |
| 3D 범위 | Full-board 3D EM 제외; Via/Pad 정량 해석은 검증된 모델이 있는 경우에 한정 |

**적용 원칙:** 본 문서는 제품 요구사항의 최종 확정용 기준이다. 사용자 확정 후 v1.0을 요구사항 기준선으로 삼는다. 구현 완료나 실제 PCB 정확도 검증을 의미하지 않는다. 기존의 **확정**은 사용자 결정, **희망**은 후속 확장, **설계 기준**은 이번 검토에서 정한 구현·인수 규칙이다. 설계 기준의 수치는 제품에서 달성해야 할 목표이며 실측 성능이 아니다. 구현 선택과 검증 대역은 15장의 결정 게이트에서 별도 동결한다.

**적용 강도:** ‘해야 한다’, ‘금지’, ‘차단’, ‘필수’는 해당 단계의 의무 요구다. ‘예시’는 변경 가능한 설명값이며 보증값이 아니다. 이후 변경은 변경 사유·영향 요구 ID·회귀시험·승인 이력을 남긴다. 문서 버전과 실행 프로그램 버전은 별도로 관리한다.

**버전 연계:** v0.4를 직접 수정하고 v0.2와 대조하여 스윕·구조 위험 추정·복구 상세 조건을 복원했다. U01~U40 및 T01~T40은 v0.4의 의미를 유지한다. 신규 요구는 U41부터, 신규 시험은 T41부터 부여하며 구버전 번호 대응은 18장에 기록한다. 설계항목 P01~P12, Import 우선순위 I1~I4, 개발 단계 M0~M5, Port 오류 P001~P012를 구분한다.


## 2 요구사항 추적표

| ID | 요구사항 | 상태 | 근거 |
| --- | --- | --- | --- |
| U01 | Full-wave 3D EM을 제외한 경량 RLGC·임피던스·S-parameter | 확정 | D1 |
| U02 | Rust 계산 코어 + Qt UI | 확정 | D1 |
| U03 | Windows x64·ARM64, macOS Apple Silicon | 확정 | D1 |
| U04 | 저사양 PC에서도 가벼운 동작 | 확정 | D1 |
| U05 | 공개 GitHub에서 개발, 사내 데이터 비공개 | 확정 | D1 |
| U06 | PADS Layout ASCII (*.ASC) 지원 | 확정 | D1 |
| U07 | 다양한 PCB 형식 확장(ODB++, IPC-2581, KiCad, Gerber 등) | 확정(방향) | D1,D4 |
| U08 | 자체 2D Artwork, Layer/Net 확인 | 확정 | D1 |
| U09 | Stackup 두께, Copper, Dk/Df, Material 편집 | 확정 | D1 |
| U10 | GND·Power·Reference 지정 | 확정 | D1 |
| U11 | Single/Differential 선택, 선폭·두께 가상 변경 | 확정 | D1 |
| U12 | DC~30 GHz 등 사용자 주파수 범위 | 확정 | D1 |
| U13 | CPU/GPU 등 계산 자원 선택 UI | 확정 | D1 |
| U14 | 영어 기본, 한국어 옵션 | 확정 | D1 |
| U15 | 공개 검증 샘플, 기밀 PCB 오프라인 검증 | 확정 | D1 |
| U16 | 제품명 BoardSI 사용 | 확정 | D2 |
| U17 | Progressive/Hierarchical Sweep: 초깃값 1 GHz 간격 예시 → 절반 구간 세분화 | 확정(방법) | D2 |
| U18 | 계산 완료 주파수 샘플 캐시 재사용, 동일 조건 재계산 방지 | 확정 | D2 |
| U19 | 실행 중 새로운 계산 결과가 나오면 SI 그래프 실시간 갱신 | 확정 | D2 |
| U20 | Group × Frequency Region별 독립적인 수렴·상태 판정 | 확정 | D2 |
| U21 | 일부 그룹/대역 수렴만으로 전체 실행 조기 종료 금지 | 확정 | D2 |
| U22 | 긴 작업 중 GUI 응답·진행률·취소/일시정지 유지 | 확정 | D2 |
| U23 | 기능별 소형 모듈, UI와 Solver 분리(필요시 별도 프로세스) | 확정(목표) | D2 |
| U24 | RAM/SSD 하이브리드 캐시, 체크포인트, 장애 후 복구 | 확정 | D2 |
| U25 | 분석 전 배선·리턴경로·모델 한계 위험 표시 | 확정(목표) | D2 |
| U26 | 일반 PCB 외 BGA/FC-BGA Substrate SI 대상 | 확정 | D3 |
| U27 | Substrate Trace, Pad/Land, Via 및 Anti-pad, Plane 포함 | 확정 | D3 |
| U28 | Bump/Cu Pillar/Solder Ball/Wirebond 해석 영역에서 제외 | 확정 | D3 |
| U29 | Substrate 분석 Port 기준은 양 끝 금속 Pad-to-Pad | 확정(경계) | D3 |
| U30 | 검증된 모델 없는 3D 불연속은 정확한 해석인 것처럼 계산 금지 | 확정(원칙) | D3 |
| U31 | ODB++를 기준 내부 형식으로 사용하지 않고 자체 Canonical Model 사용 | 확정 | D4 |
| U32 | Format-specific Importer를 분리하고 동일한 Viewer/Port/Solver 사용 | 확정 | D4 |
| U33 | Altium, Allegro, EAGLE, EasyEDA, CADSTAR 등 Import 확대 검토 | 희망 | D4 |
| U34 | Auto Port 후보 생성 + Manual 수정 + 공통 자동 검증 | 확정(방향) | D5 |
| U35 | Single 기본 2-Port, Differential 기본 물리적 4-Port | 설계 기준 | D5 |
| U36 | Reference 도체와 기준 임피던스 분리, Pair Mapping 명시 | 확정(원칙) | D5 |
| U37 | Error Port는 Solver 차단, Warning은 표시 및 확인 | 설계 기준 | D5 |
| U38 | Format/Version/객체별 지원 수준과 누락 정보 명시 | 설계 기준 | D6 |
| U39 | Import 원본 불변, 정규화 및 Provenance 보존 | 설계 기준 | D4,D6 |
| U40 | Import할 때 보조 데이터(Netlist/Stackup/Drill) 연계 | 설계 기준 | D4,D6 |
| U41 | 고정 Linear 직접 Sweep과 수렴 검사의 독립 기준선 | 설계 기준 | D2,D6 |
| U42 | 전체 행렬 및 NEXT/FEXT의 방향·종단·결합 그룹 고정 | 설계 기준 | D6 |
| U43 | 실행·수렴·모델·Limit·입력 신뢰도 상태 분리 | 설계 기준 | D6 |
| U44 | 불변 Run Snapshot, IPC 버전과 늦은 응답 격리 | 설계 기준 | D6 |
| U45 | 프로젝트 스키마·이전 버전 이관·재개·저장 무결성 | 설계 기준 | D6 |
| U46 | Sweep만 변경 시 동일 물리 조건 직접 계산점 재사용 | 설계 기준 | D2,D6 |
| U47 | 수치 정확도·GUI 응답 목표 및 단계별 측정 게이트 | 설계 기준 | D6 |
| U48 | Import 좌표 정밀도·ODB Step·보조 자료 충돌 통제 | 설계 기준 | D6 |

**추적 원칙:** U16~U40은 v0.4 번호를 유지한다. U41~U48은 이번 검토에서 추가한 설계 기준으로, 과거 사용자 발언으로 소급하지 않는다. 각 요구의 본문·시험·개발 단계 연결은 18.2절을 따른다.


## 3 사용 흐름과 화면 동작

U08–U13에 따라 PowerSI와 유사한 순차 설정 흐름을 제공한다. 아래 화면 명칭과 세부 검증 동작은 설계항목 P01이다. 설정 화면을 이동해도 입력값을 보존하고, 선행 조건이 부족하면 누락 항목과 수정 위치를 표시한다.

| 단계 | 기본 화면 명칭 | 사용자 작업과 다음 단계 조건 |
| --- | --- | --- |
| 1 | Import PCB | 파일 선택, 형식·버전 판별, 가져오기 결과와 누락 항목 확인 |
| 2 | PCB Artwork | 보드·레이어·Net 표시, 연결과 선택 신호 위치 확인 |
| 3 | Stackup | 적층 순서, 유전체·동박 두께, 재질 및 Dk/Df 검증 |
| 4 | Reference Nets | GND·전원 역할과 해석 기준 도체 지정 |
| 5 | Signals and Ports | Single/Differential, P/N, 시작·끝 포트, 가상 치수 설정 |
| 6 | Frequency | DC 포함 여부, 시작·종료 주파수, Progressive Sweep·Region·예산 설정 |
| 7 | Compute Resources | CPU 스레드·메모리 한도·사용 가능한 계산 장치 설정 |
| 8 | Run and Results | 입력 사전 검사, 실행·Pause/Resume/취소, 실시간 그래프·수렴 상태·저장 |

### 화면 구성

왼쪽에는 단계 목록과 완료 상태, 가운데에는 Artwork 또는 해당 단계 편집 화면, 오른쪽에는 선택 객체의 속성을 배치한다. 아래에는 오류·경고·실행 진행률을 표시한다. 기본 버튼 명칭은 Open PCB, Open Project, Save Project, Run, Cancel이다. 그래프와 Artwork 영역은 크기 조절이 가능해야 한다.

### 변경에 따른 결과 무효화

Stackup, Reference, 신호, 치수 또는 부하가 변경되면 관련 물리 결과를 Out of Date로 표시한다. 원본과 이전 결과는 보존한다. 주파수 범위·간격·수렴 기준만 변경하면 이전 실행의 완료 판정은 무효화하되 동일 물리 조건의 직접 계산점은 재사용한다. 색상·줌·UI 언어 변경은 재계산 사유가 아니다. 계산 자원 변경은 기록하며, 엔진·정밀도·수치 Backend가 달라지면 호환성 검증 없이 캐시를 공유하지 않는다.

예: 0.10 mm 선폭을 0.12 mm로 바꾸면 해당 Scenario의 기하 모델과 해석 결과가 무효화된다. 화면에서 선폭만 바뀌고 계산에는 이전 값이 남는 동작을 허용하지 않는다. [P01, P05, T08]


## 4 다형식 Import와 Canonical Board Model

### 4.1 핵심 아키텍처와 파일 무관성

**Input File(s) → Signature/Version Detection → Format Importer → Geometry/Connectivity Normalizer → BoardSI Canonical Board Model → [Artwork, Stackup/Reference, Ports, Analysis Group, Solver, Export]**

- 한 번에 선택한 주 파일 하나(또는 ODB++ 폴더/제조 데이터 묶음)와 필요한 보조 파일을 Import한다. 여러 주 파일을 동시 병합하는 기능은 추후에 별도 정의한다.
- ODB++는 입력 플러그인 중 하나이며 내부 데이터 계약·해석 기준이 **아니다**. KiCad, PADS, Altium 등 모든 지원 입력은 똑같은 검증·Port 생성·Solver 경로를 따른다.
- Canonical 모델은 **좌표/단위·레이어 z·Copper Surface 및 Cutout·Padstack·Via Span·Net·Pin·Component Transform·연결 그래프·재료·Provenance·Validation 상태**를 보존한다.
- Viewer는 이 모델을 벡터 렌더링하고 Solver는 같은 모델에서 추출한 물리적 형상과 재료를 사용한다. 화면용 단순화만 Solver에 전달하면 안 된다.
- 원본 보드와 별도로 Scenario Overlay를 관리해 가상 선폭/동박 두께 변경이 실제 Solver Geometry에도 반영되게 한다.

### 4.2 파일 형식 확대 로드맵 지원 완료가 아닌 개발 목표 

| 우선 단계 | 형식/제품 | 대표 파일/패키지 | 지원 전략 및 한계 |
| --- | --- | --- | --- |
| I1 최우선 | Siemens/Mentor PADS Layout ASCII | `.asc` | 내부 레코드/버전 식별; Net, Trace, Pin/Pad, Copper, Via/Pads 정규화 |
| I1 최우선 | ODB++Design | `.zip`, `.tgz`, `.tar.gz`, 디렉터리 | Step/Layer/Net/Feature/Padstack 접근; 보드 및 Substrate 공통 |
| I2 | IPC-2581 / IPC-DPMX | XML 및 패키지 | 지원 XML Revision 명시; Layer/Net/Pad/제조 정보에 대한 필드 지원표 |
| I2 | KiCad | `.kicad_pcb` | S-expression 기반; Track/Via/Zone/Net/Footprint/Stackup 처리 |
| I2 | Gerber Layer (RS-274X / X2 / X3) | `.gbr`, `.gtl`, `.gbl` 등 | Layer/Artwork 중심; 존재하는 속성만 사용, Net 자동 추정은 별도 신뢰 표시 |
| I2 | Drill / Gerber Job | `.drl`, `.xnc`, `.gbrjob` 등 | Hole/Plating/Layer/Stackup 보완; 묶음 레이어 매핑 UI 필수 |
| I3 | Autodesk EAGLE / Fusion의 EAGLE Export | XML `.brd` | XML 확인 후 직접 파서 검토; `.brd` 확장자만으로 판정 금지 |
| I3 | EasyEDA Standard/Pro | JSON, ZIP/JSONL 계열 | 해당 포맷 버전과 데이터 스키마별 Parser |
| I3 | Altium Designer / Circuit 계열 | `.PcbDoc`, `.CMPcbDoc`, `.CSPcbDoc` | 바이너리 컨테이너/Revision 분석, 외부 기술문서·라이선스 점검 필요 |
| I3 | Cadence Allegro PCB Editor | `.brd` | 바이너리 레이아웃; 버전별 지원 난도 높음, 직접 또는 오프라인 변환 경로 평가 |
| I4 | Zuken CADSTAR | PCB Archive `.cpa` 등 | 공개 형식/샘플 확보 후 별도 Parser 평가 |
| I4 | Fabmaster, P-CAD, gEDA/Lepton EDA | 포맷별 개별 | 수요/샘플/라이선스 조건에 따라 순차 확대 |
| 보조 | IPC-D-356B Netlist | 테스트 넷리스트 파일 | Copper/층 구조를 대체하지 않고 Net·Test Point의 검증·보완에만 사용 |
| 보조 | Stackup 표, 소재 표 | CSV 등 | 두께/Dk/Df/재료의 별도 수입; 물성 출처 추적 |
| 보조/확장 | DXF Outline, Touchstone S-parameter | `.dxf`, `.sNp` | DXF는 윤곽 보조, Touchstone은 외부 네트워크 모델; PCB Artwork 자체를 대체하지 않음 |

**서로 다른 상태를 명확히 구분:** Direct Native Import(직접 해독) / Offline Converted Import(사용자 지정 로컬 변환 경로) / Geometry-only Preview(미완전 자료) / Unsupported(안전한 미지원). 직접 Import가 구현되지 않은 형식을 '지원 완료'로 광고하지 않는다. 추가 설치가 필요한 도구는 자동으로 설치하거나 데이터 송신하지 않는다.

### 4.3 파일 형식별 신뢰 해석 준비도 구분

| 내부 진단 | 의미 | 허용 범위 |
| --- | --- | --- |
| G: Geometry | 실제 도체/레이어/드릴 기하 확보 | Artwork 표시 |
| N: Net | Net 이름과 물리적 Copper Connectivity 일치 | Net 기반 선택 후보 |
| P: Pin/Pad | 부품·패드·핀·방향/극성 매핑 확인 | Port 자동 추천 후보 |
| S: Stackup | 층 순서, 도체/유전체 두께, Dk/Df가 유효 | 해당 모델의 전기적 계산 준비 |
| R: Reference | GND/Power 구조와 반송 전류 기준 검증 | Port Reference 후보 |
| M: Model | 선택한 구조가 검증된 Solver 모델 범위에 해당 | SI Solver Run 허용 |

Import Summary에서 각 항목은 Complete / Partial / Missing / Conflicting 중 하나다. Geometry-only 파일을 불러왔다고 SI 실행이 가능한 것으로 처리하지 않는다. 필요한 자료를 사용자가 추가하여 S, R, M 등이 충족된 경우에만 제한된 구간의 Run을 허용한다.

### 4.4 필수 안전 및 무손실 불완전성 정책

- 헤더·Content Signature·Version·인코딩으로 구분하며 `.asc` / `.brd`처럼 겹치는 확장자에 의존하지 않는다.
- 기하의 좌표 원점, 회전, 미러, 부품 Bottom Side, 좌표 단위, 폐곡선, Arc, Pour/Fill, Isolated Island, Thermal, Clearance, Anti-pad, Blind/Buried Via 등은 원본 정밀도와 정해진 변환 허용오차 내에서 보존하고, 손실·근사 변환은 항목별로 보고한다.
- Netlist의 이름만 같더라도 물리적으로 단절된 Copper Island를 연결 처리하지 않으며, Via Span 밖의 층을 접속시키지 않는다.
- 누락된 Stackup/Dk/Df를 임의 FR-4로 정상 표시하지 않는다. 사용자 가정값이 있으면 결과에 Scenario + Assumed 표시를 남긴다.
- 손상 파일/ZIP Slip/압축 폭탄/재귀 참조 및 비정상 XML을 제한한다. 최대 파일 크기·객체 수·파싱 시간 예산은 시제품 후 확정한다.
- 입력 원본은 프로그램에서 Read-only로 취급하고 Canonical Object ID와 SourceFile/SourceObjectID/SourceVersion/Hash를 매핑한다. 재불러오기 후 Port를 Object ID+Geometry+Net 재검증으로 다시 연결한다.
- 공개 문서화된 형식이나 법적으로 허용된 인터페이스를 활용한다. 외부 프로젝트 코드를 재사용할 때 해당 라이선스·링킹·재배포 의무를 독립적으로 확인한다.

### 4.5 변환 경로 및 플랫폼 간 재현성

직접 읽기 어려운 바이너리 형식은 **오프라인 변환 → 지원되는 교환 형식 Import**를 보조 경로로 허용한다. 변환 로그·원본/변환 파일 해시·손실 항목을 기록하고 직접 지원과 분리해 표시한다. 외부 CAD 라이선스가 필요한 경우 해당 도구 설치와 이용 권한을 사용자가 관리한다.

같은 설계의 ASC/ODB++/IPC-2581/KiCad 간 공통 객체(Nets, Pad/Via, Trace, Area, Layer, Anti-pad)를 교차 비교한다. 비교는 원본 CAD에서 추출된 정보에 한하며 Export에 없던 데이터까지 완전하다고 가정하지 않는다.

**검증 근거:** KiCad Importer 문서[R9]는 Altium/Allegro/CADSTAR/EAGLE/EasyEDA/Fabmaster/gEDA/P-CAD/PADS 형식을 실제로 다루는 사례를 보여준다. 단, 그 구현/정확도/라이선스가 BoardSI에 자동 이전되는 것은 아니다. Ucamco Gerber[R3,R11], ODB++Design[R10], IPC-2581[R12], KiCad Board[R2]를 해당 파서의 원전으로 참조한다.


### 4.6 좌표 정밀도와 입력 충돌 처리

좌표 단위·해상도·원점·회전·미러·레이어 방향을 Source별로 기록한다. Canonical 좌표는 정수 격자와 SI 환산 계수를 함께 보존하고 Solver 전달 시 SI Float64로 변환한다. 원본 해상도보다 거칠게 반올림하지 않으며, 표현 범위 초과나 최소 절연 간격 소실 시 차단한다. Arc의 화면용 분할과 전기적 경계 근사 오차를 분리한다. 접촉 허용오차가 절연 간격을 메울 수 있는 모호한 경우 자동 병합하지 않는다.

ODB++의 복수 Step·Panel 반복 중 해석할 Board/Substrate Step과 인스턴스를 선택한다. Panel을 중복 합성하거나 다른 Step의 동명 Net을 연결하지 않는다. 제조 Copper의 Positive/Negative 표현과 Cutout을 정규화한 실제 도체로 비교한다. ODB++ 내부 객체를 BoardSI 공통 형식인 것처럼 가정하지 않는다. [R10]

보조 파일은 보드 식별·단위·좌표 기준·층 대응을 확인한 뒤 연결한다. 서로 다른 Stackup·Net·도금 정보는 충돌 목록에 두며 사용자 선택과 근거를 남긴다. 원본 파일의 변경 시각만으로 동일 설계를 판정하지 않는다. Reimport의 ID 안정성은 동일 입력·동일 Importer에서 보장하고, 형식이 달라지면 기하·Net·Pin의 명시적 대응표를 사용한다. 우연히 같은 ID를 동등한 객체로 보지 않는다.

압축 해제는 상대경로만 허용하고 심볼릭 링크·외부 경로 접근을 차단한다. XML 외부 엔티티와 외부 리소스 요청을 비활성화한다. 파일 크기·압축 해제량·객체 수·중첩 깊이·시간의 유한한 상한을 Import Profile에 저장하며 M1 이전에 악성 입력 시험으로 동결한다. 제한에 걸린 입력은 원본을 보존하고 원인을 표시한다.

## 5 Artwork와 전기적 연결 모델

U08의 Artwork는 시각 확인과 해석 대상 선택의 기준이다. 설계항목 P03은 화면에 보이는 기하와 계산에 사용되는 기하가 동일한 보드 모델을 사용하도록 한다. 화면용 단순화가 원래 전기적 연결을 바꾸면 안 된다.

| 객체와 기능 | 필수 확인 항목 |
| --- | --- |
| Layer와 Net | 표시·숨김, 색상·투명도, Net 검색·강조, 선택 객체 속성 |
| Trace와 Arc | 좌표, 층, 폭, 중심선 길이, 연결 Pin·Via, 원본 ID |
| Pad와 Via | Net, 도금 여부, 층 범위, 드릴·패드, 실제 접속 층 |
| Copper와 Cutout | 도체 영역과 빈 영역, 분할 Plane, Thermal 접속, 고립 Copper |
| 측정과 탐색 | mm·µm 거리 측정, Fit Board, 줌·팬, 선택 신호로 이동 |
| 연결 검사 | Open·Short 의심, 중복 도형, 불명 Net, 분기, 누락 객체 |

### Topology 추출

Pin·패드·비아·분기점을 노드로, 배선과 접속을 엣지로 관리한다. 같은 화면 좌표에서 교차하더라도 서로 다른 층이면 자동 접속하지 않는다. Via와 도금 패드가 해당 층을 실제 연결할 때만 수직 접속을 만든다. Copper 이름과 Net 이름이 같다는 이유만으로 물리적으로 분리된 영역을 연결하지 않는다.

기하 허용오차는 입력 좌표 해상도와 최소 간격을 고려해 별도 정책으로 정한다. 지나치게 큰 병합 허용오차로 절연 간격을 없애지 않는다. 불일치가 있으면 원본 Netlist와 기하 접속을 각각 보여주고 사용자가 원인을 판단할 수 있어야 한다.

### 전체 보드와 선택 경로

전체 보드는 탐색에 사용하고, 계산은 선택 신호와 필요한 기준 도체·인접 도체로 제한한다. 관심 영역 경계에서 제외된 도체의 영향은 모델 범위에 기록한다. 면적을 줄였다는 이유만으로 기준 Plane을 잘라 무한 GND로 바꾸지 않는다.

2D 표시는 3D EM 해석을 의미하지 않는다. 보드상 길이·비아 수·P/N 길이차는 기하 결과이며, 전기적 지연차나 손실은 적용한 모델을 통해 별도로 계산한다. [P03, T03, T07]


## 6 Stackup과 Reference 설정

U09·U10에 따라 적층과 기준 도체를 별도 단계에서 편집한다. 설계항목 P04는 재료 출처와 누락값을 추적하고, 화면의 재질 명칭만으로 전기적 물성을 확정하지 않는 것이다.

| 분류 | 저장하고 검증할 값 |
| --- | --- |
| 공통 | 층 ID·순서·종류, z 위치, 전체 두께, 값의 출처와 사용자 수정 여부 |
| 유전체 | Core/Prepreg 구분, 재료명, 두께, Dk, Df, 기준 주파수 |
| 주파수 의존성 | 상수·표 데이터·분산 모델 중 선택, 유효 대역과 보간·외삽 정책 |
| 도체 | 층별 동박 두께, 도전율, 손실 모델, 표면 거칠기 적용 여부와 데이터 |
| 외층 환경 | Solder mask와 주변 매질의 포함 여부, 두께·Dk/Df 또는 생략 가정 |
| Reference | 기준 Net, 실제 도체 영역, 접속 경로, Signal/Power/Ground/Floating 역할 |

### 물성 적용 규칙

필수 물성이 없거나 두께가 0 이하이면 해당 계산을 차단한다. 예시값을 사용한 실행은 별도 Scenario로 표시하고 검증된 재료값과 혼합하지 않는다. 상수 Dk/Df로 넓은 대역을 계산할 때는 상수 물성 가정을 결과에 남긴다. 주파수 의존 모델은 수동성·인과성 검증 후 도입한다.

층 전체 동박 두께 변경과 특정 신호만의 두께 변경은 적용 범위를 분리한다. 특정 Trace의 두께만 바꾸는 가상 변경은 실제 제조 공정을 그대로 나타내지 않을 수 있으므로 가상 치수임을 표시한다. 내부층 두께 변화가 층 위치와 보드 총 두께를 바꾸는 정책도 명시한다.

### 전원 Net과 해석 기준

전원 Net을 지정하는 기능과 그 Net을 이상적인 AC 기준으로 놓는 기능은 구분한다. 서로 다른 GND·전원 Net을 자동 단락하지 않는다. 실제 연결 또는 명시된 회로 모델이 없는 Reference 전환은 경고하거나 지원 범위 밖으로 처리한다.

신호 아래 Copper의 유무, Plane 분할·Cutout, 기준층 변경을 검사한다. 단순 경량 모델에서 처리할 수 없는 리턴 경로는 Unsupported로 표시한다. GND Stitching Via의 존재 여부 검사는 가능하더라도 그 정량 개선 효과를 검증 없이 계산값으로 제시하지 않는다. [P04, R4, T04]


### 6.1 물성과 치수 변경의 결정 규칙

본 절은 BoardSI 설계 기준이다. Dk는 유한한 양수, Df는 유한한 0 이상, 도전율은 유한한 양수로 검사한다. 무손실 기준 쿠폰은 명시적 Ideal Conductor/Lossless 모델로 구분한다. 필수 층 두께는 양수이며 생략 가능한 Mask는 ‘미포함’으로 표현한다. 0 두께를 누락값 대용으로 사용하지 않는다.

층 전체 동박 두께 변경의 기본 정책은 유전체의 표면 간 두께를 유지하면서 Layer z와 전체 두께를 재계산하는 것이다. 총 두께 고정이 필요하면 사용자가 조정할 유전체 층을 지정한다. 특정 Trace만의 두께 수정은 선택 층의 아래쪽 Copper 면을 기준으로 위쪽으로 변경하고, 새로운 간격·충돌·Reference 거리를 다시 계산하는 가상 시나리오다. 채택한 z 기준과 변경 전후 단면을 표시하고 저장한다.

재료값에는 자료명·Revision·측정 방법·기준 주파수·온도(제공된 경우)를 보존한다. 단일 주파수 Dk/Df만으로 광대역 분산 모델이 검증되었다고 표시하지 않는다. 자료에 명시된 유효 대역 밖은 기본 차단하며, 사용자가 명시적으로 허용한 가정 외삽은 Assumed/Approximate로 남긴다. DC에서는 유전 손실과 누설 도전율을 구분하고 RF Df로 DC 누설을 임의 생성하지 않는다.

## 7 Port 자동 생성 수동 수정 Validation 및 가상 변경

### 7.1 공통 정의 Port는 원본 파일이 아닌 Canonical Model에 연결

Port = {Signal Contact Geometry ID, Signal Net, Layer/Position, Reference Conductor Geometry/Net, Port Type, Zref, Pair Mapping, Direction/Endpoint Role, Termination, Source Provenance, Validation Status}. Signal Reference Net(물리적 Return 도체)과 기준 임피던스 Zref(예: 50 Ω)는 **다른 데이터**다. Net 이름만으로 Reference를 임의 생성하거나 다른 GND/Power Net을 무단 Short하지 않는다.

Port는 실제 도체 접속 위치와 경계 조건을 표현한다. 내부 경량 전송선로 Solver의 네트워크 단자와 Full-wave Gap/Edge/Lumped Port는 동등하지 않으므로 **Solver가 검증한 Port 타입만 '해석 가능'**으로 표시한다. Edge/Lumped/Gap/Internal Excitation 구현 가능 여부는 Solver 모델 검증 후 결정한다.

### 7.2 자동 Port 후보 위치 우선순위

| 순위 | 후보 | 기본 규칙 |
| --- | --- | --- |
| 1 | Component Pin의 Signal Pad | Pin-Net-Copper 연결 일치할 때 최우선 |
| 2 | 전기적 끝점 Pad/Land | 선택 Net의 실제 끝단으로 확인될 때 |
| 3 | Connector Pad | Connector Pin과 연결 맵을 검증할 때 |
| 4 | Board Edge의 Trace Boundary | 형상·단면·Solver Port 타입이 지원될 때 |
| 5 | Test Point/Via | 보조 후보, 자동 확정하지 않음 |
| 6 | 임의 Trace 중간 | 수동 또는 명시적 해석 경계 설정에서만 가능 |

동일 Net에 다수의 Pin/Branch가 있으면 2-Port를 임의 확정하지 않는다. 불명확한 Source/Load, Branch, Series Capacitor/Resistor 등은 검증·종단·외부 회로 모델이 필요하다. Tx/Rx 방향은 이름/구조로 확정 불가한 경우 사용자 확인으로 지정한다.

### 7.3 Single Differential 기본 구성

- Single-Ended 2-Port: 시작점(Signal+Reference), 끝점(Signal+Reference). 포트 번호·단자 방향·정규화 기준을 기록한다.
- Differential: 기본 4개의 물리적 Single-Ended Port. 기본 순서는 Near P=1, Near N=2, Far P=3, Far N=4다. `P/N` 구성·물리적 순서·기준 도체를 명시적으로 저장한다.
- P/N Pair 탐색은 **명시적 Pair 속성 → 엄격한 이름 패턴 → 실제 대응 Pin/기하 확인** 순이다. Single/Differential 모드와 모든 Pair 후보는 사용자가 확인하며 모호한 복수 대응은 자동 확정하지 않는다. 예: `DATA0_P/DATA0_N`, `DATAP0/DATAN0`, `CLKP1/CLKN1`, `DPP0/DPN0`, `PDATA0/NDATA0`, `PCLK1/NCLK1`, `PDP0/NDP0` 등. 이름에 P/N 한 글자가 포함된다는 이유로 Pair 처리하지 않는다.
- Pair 안의 두 Port가 같은 기준 도체 및 동일한 50 Ω Single-ended reference를 사용하는 정의에서는 Differential 100 Ω / Common 25 Ω가 되지만 Reference Geometry와 특성 임피던스는 별도로 관리한다. 복잡한 비대칭 기준은 지원 검증 후 허용한다.
- Mixed-mode 변환은 사용자가 지정한 순서/극성으로 수행한다. 예시 (1=Near P,2=Near N,3=Far P,4=Far N): `Sdd21 = (S31-S32-S41+S42)/2` (동일 Zref의 일반적 정의).
- 중간 분기는 실제 Multiport 또는 검증된 Load 모델을 필요로 한다. 사용하지 않는 단자를 무조건 Open/Match 처리하지 않는다.

### 7.4 Reference 후보 자동 탐색

우선순위: (a) 사용자 확정 Reference → (b) 인접한 연속 GND Plane → (c) 유효한 GND Copper/Return Via 경로 → (d) 특별한 Power/기타 Return 조건은 모델 기반 사용자 확인. Plane Cutout·Split·Anti-pad 또는 신호 Layer 변경 구간에는 Return Path Warning을 발생시킨다.

### 7.5 Manual Port 및 검증 엔진

GUI에서 Signal Pad/Trace 면 선택 → 자동 Net/Layer/Component Pin 추출 → Reference 도체 선택 → Port Type/Zref/Direction/P/N Mapping/종단 확인 → 즉시 Validation 실행. 위치 변경·파일 재불러오기·Stackup/Net 변경은 Port 재검증을 트리거한다.

| 코드 | 검증 항목 | 심각도 기본값 |
| --- | --- | --- |
| P001 | Signal Terminal이 실제 Copper에 접속 | Error |
| P002 | Signal Net-Geometry 일치 | Error |
| P003 | Reference 도체 존재·Signal과 구분 | Error |
| P004 | Signal과 Reference 각 단자의 접속 및 상호 절연 | Error |
| P005 | Solver가 지원하는 Port 형상/모델 | Error |
| P006 | 시작·끝의 경로와 사용자 Endpoint 확인 | 미확정 시 Error |
| P007 | Differential P/N 대응·극성·순서 | Error |
| P008 | Port ID 중복/경계 충돌 | Error |
| P009 | Return 불연속과 모델 범위 | 모델 부재 Error, 유효한 근사 Warning |
| P010 | Branch와 부하·종단 정의 | 미정의 시 Error |
| P011 | Port 근처의 불연속·전기적 영향 범위 | Warning |
| P012 | Reimport 후 Object/Net/Geometry 매핑 | 재검증 필요 |

상태: Valid(기본 검증 통과), Warning(신뢰 제한 표기/확인), Error(해석 차단). 이 정적 Validation은 Full-wave 정확도 보증이 아니며 후속 수동성/상호성/수치 수렴 검증을 별도로 수행한다.

### 7.6 Scenario 및 선폭 두께 수정

원본은 불변, Overlay에 선택 Segment/Net/Pair/Layer 범위별 기하 변경을 저장한다. Trace 중심선 유지가 기본이며 폭 증가시 간격과 충돌을 재검사한다. Via·Pad를 선폭 변경에 맞춰 임의 변형하지 않는다. Port/Reference/Geometry 변경은 관련 캐시를 무효화하고 결과를 Out-of-date로 표시한다.


### 7.7 Port 경계와 종단 계약

기본 Port 타입은 검증된 전송선로 네트워크의 Terminal Pair다. GUI에서는 Signal Contact와 Reference Contact를 서로 다른 선택 색상으로 표시한다. P004는 두 단자를 서로 연결하라는 뜻이 아니다. 각 단자가 대응 도체에 접속하고 Signal과 Reference가 단락되지 않았는지 확인한다. Warning 확인으로 P001~P008의 오류나 미지원 모델을 우회할 수 없다.

S 행렬의 외부 Port는 모두 보존하며 여기하지 않는 Port의 입사파가 0인 기준 조건을 사용한다. 실제 부하를 적용해 포트를 제거할 때는 Load Network 결합 후 축소한 별도 결과를 생성한다. 행과 열 삭제는 일반적인 Open/Short/임의 부하 처리를 대신하지 않는다. Matched Load의 예외도 기준 Zref를 확인하고 시험한다.

Series R/C와 다른 Net을 잇는 부품은 서로 다른 Copper Net을 유지한다. Board-only 해석은 부품 Pad에서 끝내고, 부품을 포함하는 Channel은 검증된 부품 모델을 명시적으로 연결한다. 모델 대역·방향·핀 순서·Zref·참조면·포함 Pad를 확인하여 Pad/Via 기생성분을 중복 계산하지 않는다. 임의 Trace 중간의 수동 Port는 경계를 잘라 보존할 쪽·분기·부하를 정한 새 Analysis Group으로만 만든다.

### 7.8 NEXT와 FEXT의 방향 정의

두 Single 선로 예시는 A Near=1, A Far=2, B Near=3, B Far=4다. A Near를 입력으로 할 때 NEXT는 S31, FEXT는 S41이다. A Far를 입력으로 바꾸면 NEXT는 S42, FEXT는 S32다. 화면은 항상 Aggressor 입력과 Victim 출력 및 Near/Far의 실제 끝점을 병기한다. 물리 배치에서 방향이 반대인 Net을 이름만으로 같은 Near/Far로 취급하지 않는다.

차동 두 Pair는 8개의 Single-ended Port를 같은 결합 그룹으로 풀고 Pair별 Mixed-mode로 변환한다. Sdc는 출력 d·입력 c, Scd는 출력 c·입력 d로 정의한다. 독립 해석한 Pair 결과만으로 Pair 사이 NEXT/FEXT를 만들 수 없으며 미계산 결합항을 0으로 채우지 않는다. 결합 그룹 분리는 제외한 도체·거리·결합 근거를 기록하고 적용 범위 확대 시험으로 검증한다. [T42]

## 8 해석 엔진과 적용 한계

U01·U02를 구현하기 위한 설계항목 P06은 단면의 준정적 RLGC 추출과 분포 전송선로 네트워크 결합이다. 초기에는 검증하기 쉬운 Microstrip·Stripline·결합 차동 선로를 우선한다. 2D 단면 모델은 준 TEM과 균일 단면을 가정하며 비아·분기 등 3차원 불연속을 자체적으로 해결하지 않는다. [R4]

| 대상 | 초기 처리 기준 | 결과 상태 |
| --- | --- | --- |
| 균일 Single 선로 | 해석식 또는 검증된 2D 단면 추출 후 전송선로 계산 | 검증 조건 안에서 Validated |
| 균일 Differential | 결합 RLGC 행렬과 모드 변환 사용 | 검증 조건 안에서 Validated |
| 폭·적층 변화 | 구간별 모델 결합, 경계 기생성분 포함 여부 기록 | 기본적으로 Approximate |
| Via·패드·분기·급격한 굴곡 | 검증된 등가회로 또는 외부 모델이 있을 때만 포함 | 모델 없으면 Unsupported |
| Plane 분할·리턴 전환 | 가정이 맞는 기준 도체 모델만 사용 | 정량 모델 없으면 Unsupported |
| 커넥터·완성 패키지·능동 소자 | Substrate 자체를 제외한 외부 구조·소자는 기본 범위 밖 | 미포함 범위를 명시 |

### 계산 데이터와 수치 처리

RLGC는 단위 길이 기준으로 R Ω/m, L H/m, G S/m, C F/m를 저장한다. 다도체는 행렬의 도체 순서와 Reference를 저장한다. 구간 길이를 적용한 총량과 단위 길이 값을 혼동하지 않는다. 표피·근접 효과, 유전체 손실·분산, 도체 거칠기의 적용 여부와 모델 파라미터를 결과에 기록한다.

수치부는 Float64 및 복소수 Float64를 기본으로 한다. 긴 손실선로와 다포트 연결은 수치적으로 안정적인 네트워크 결합법을 사용하며, 조건수 악화·특이행렬·NaN을 탐지해 실패 원인을 반환한다. 단순한 구간 세분화만으로 3D 불연속의 물리 모델이 생겼다고 간주하지 않는다.

### 해석 수준 표시

Validated는 모델·구조·치수·재료·대역이 검증 범위 안이라는 뜻이다. Approximate는 계산되었지만 모델 가정 또는 검증 범위에 제한이 있다는 뜻이다. Unsupported는 필요한 모델이 없어 전체 경로의 정량 결과를 낼 수 없다는 뜻이다. 미지원 Via를 이상적인 선로로 숨겨 결과를 만드는 동작은 금지한다.


### 8.1 Substrate Pad to Pad 해석 범위

Cadence Package/Substrate 설계는 사용자가 로컬 CAD에서 출력한 ODB++를 우선 입력한다. `.MCM`, `.SIP` 및 Package용 `.BRD` 원본의 직접 해독은 이번 기준 범위에 포함하지 않는다. 4장의 Allegro PCB Native Import 확대는 별도의 후속 과제다. PCB와 Substrate는 동일 Canonical Model과 검증 체계를 사용한다. [D3,D4]

포트 경계는 Die-side Signal Pad와 대응 Reference Pad/Conductor에서 시작하여 Ball-side Signal Land와 대응 Reference Pad/Conductor까지다. Substrate 내 Trace·Pad/Land·도금 Via·Microvia·Blind/Buried Via·Plane·Anti-pad·재료를 포함한다. Bump, Cu Pillar, Solder Ball, Wirebond, Silicon Die, PCB 실장 Pad 및 보드 측 배선은 제외한다. Underfill/Mold/Solder Mask 등 주변 유전체는 데이터와 모델이 있을 때만 포함하며 생략 여부를 기록한다.

Pad는 기하에 포함되더라도 그 기생효과가 자동으로 계산되는 것은 아니다. Pad의 끝단 참조면·접촉 영역과 기생성분 모델의 포함 범위를 함께 저장한다. Pad 또는 Via의 유효 모델이 없으면 전체 Pad-to-Pad 정량 결과를 Unsupported로 둔다. 지원 가능한 균일 Trace 부분만 별도 경계로 해석할 수 있으나 이를 완전한 Substrate 결과로 표시하지 않는다. 미지원 경계를 재분할해 분석한 경우 별도 Group으로 이름과 Port Map을 남긴다.

### 8.2 수치 모델의 최소 계약

시간 조화 convention은 exp(+jωt), 전달파는 exp(−γl)로 고정한다. RLGC는 단위 길이 행렬이며 C/G가 Maxwell 행렬인지 회로 소자값인지 명시한다. 전송선로 결합에 쓰는 행렬은 동일한 도체·Reference 순서를 따른다. 행렬의 각 항에 개별 제곱근을 취하는 방식으로 다도체 전파상수를 만들지 않는다. [R21,R18]

동일한 양의 실수 Port Reference 및 power-wave 정규화에서 수동성은 S의 최대 특이값, 상호성은 S와 전치 행렬의 차이를 점검한다. 각 원소의 크기가 1 이하라는 조건만으로 다포트 수동성을 판정하지 않는다. 검사 잔차·대역·허용오차를 기록하고 자동 보정으로 원본 결과를 덮어쓰지 않는다. 유한 대역 샘플만으로 전 대역 인과성을 증명했다고 표시하지 않는다. [R22,T07,T43]

지원 모델 레코드는 Model ID/버전, 기하 파라미터 범위, 재료, 주파수 범위, Reference/Port 조건, 비교 대상, 오차 측정 결과를 가진다. Via Stub 유무·도금 두께·패드·Anti-pad·Return Via 조건이 다르면 검증 범위 적합성을 다시 판정한다. 30 GHz 입력 가능 여부와 해당 구조의 30 GHz 정확도는 별개다.

## 9 Progressive Sweep 수렴 실시간 실행과 계산 자원

### 9.1 목표 주파수와 초기 계층 스윕

사용자는 DC~30 GHz 같은 전체 범위를 직접 지정한다. 일반 예시: `0, 1, 2, …, 30 GHz`의 **31개 점**을 먼저 계산한 후 기존 샘플은 재사용하고 그 사이 midpoint를 채워 `0.5 GHz` → `0.25 GHz` → `0.125 GHz` → … 단계적으로 정밀도를 높인다. 고정 1 GHz 초깃값은 **예시값**이며 보드/대역/공진 위험/예산에 따라 조정 가능해야 한다.

- 처음 단계의 저해상도 전체 응답을 빠르게 출력하고, 이후 새 결과가 도착하면 S11/S21/Sdd 및 유효한 RLGC 그래프를 실시간 갱신한다.
- 계산한 주파수 점은 복소 S 등 **원래 계산값 그대로 캐시**하여 재활용한다. 보간값은 화면 보조표시와 오차 추정에만 사용하고 계산점과 구분한다.
- DC(0 Hz)는 별도 저항/누설 네트워크 또는 검증된 극한으로 계산한다. RF 근사식을 0으로 나눠 계산하지 않는다. Log Sweep은 양의 시작 주파수와 선택적 DC를 별도 저장한다. DC 모델 부재는 N/A로 기록하고 DC 포함 요청의 전체 완료·PASS를 차단한다. 사용자가 DC를 제외하면 새 실행 범위로 기록한다. DC의 Zc가 정의되지 않아도 유효한 DC S 행렬은 별도로 계산 가능하다.

### 9.2 Analysis Group Frequency Region 독립 수렴

사용자가 지정한 신호 하나 또는 실제 결합되는 여러 신호를 `Analysis Group`으로 관리한다. 각 Group을 주파수 영역(Region)으로 분할하여 개별 진행 상태와 수렴 기준을 유지한다. **어느 한 Pair/Region이 수렴했다는 이유로 다른 Group/Region을 중단하지 않는다.**

| 영역 상태 | 의미 | 다음 행동 |
| --- | --- | --- |
| Pending | 아직 계산하지 않음 | 스케줄러에 진입 |
| Running | 계산 중 | 새 점/진행률/시간 표시 |
| Converged | 최소 검증 조건과 오차 기준 충족 | 해당 Group/Region만 완료 |
| Budget-limited | 계산점·시간·RAM 상한에 도달 | 미수렴 상태로 결과와 경고 제공 |
| Unsupported | 해당 대역의 모델 한계 | 유효 모델 부재·미수렴으로 명시 |
| Failed / Cancelled / Paused | 실패/취소/일시정지 | 상태·이력·체크포인트 유지 |

수렴 평가는 선형 진폭·복소 S·위상/공진 형태·Slope/Curvature·Midpoint 잔차 및 노치 위험을 함께 검토한다. 깊은 노치 부근 dB 오차만으로 수렴 처리하지 않는다. 초깃값이 공진을 놓칠 위험을 줄이기 위해 **최소 분해능 또는 추가 검증점**을 적용한다. 기본 검증 프로파일은 9.6절, 수치 목표와 변경 게이트는 13.1절 및 15장을 따른다. 계산 완료를 `Converged`와 혼동하지 않는다.

### 9.3 작업 스케줄러의 우선순위와 종료

우선: (1) 아직 관측되지 않은 전체 대역/그룹 커버리지 (2) 불연속·Via/Pad/Plane Cutout 등 모델 위험 지점 (3) 큰 오차 또는 급격한 변화 (4) 사용자 관심 주파수/Limit 인접. 하나의 Group만 독점하지 않도록 실행 예산을 분배한다. 모든 Group/Region이 Converged 또는 Failed/Unsupported/Budget-limited/Cancelled 등 명시적 종료 상태에 도달하면 실행을 종료할 수 있다. 일부 실패 시 나머지 독립 작업은 자원이 안전한 범위에서 계속한다. 전역 Converged는 모든 필수 Group/Region이 Converged일 때만 허용한다. Paused는 종료 상태가 아니며, 고정 Sweep의 모든 점 계산 완료는 Completed Discrete로 표시한다. 종료 사유와 각 영역 상태를 보고한다.

### 9.4 GUI 작업 프로세스 요구

- Qt UI Event Loop에서 무거운 Solver/Parser를 동기 호출하지 않는다. 계산 워커는 독립 Worker, 기본 구조는 별도 **Solver 프로세스**와 비동기 IPC로 정하며 장애 격리를 구현한다. 무거운 Import·기하 정규화도 GUI 밖에서 실행한다.
- Start / Pause / Resume / Cancel, 작업 큐, 진행 단계/Group/Region, 계산점 수, 완료/미완료, 메모리/SSD 사용량을 실시간 표시한다.
- 네트워크 없이도 모든 기능 동작; RAM 압박 시 점진적으로 SSD에 Spill하고 복구 체크포인트를 주기적으로 저장한다. Checkpoint는 임시파일 작성 후 원자적 교체하며 원본/이전 정상 결과를 덮어쓰지 않는다.
- 계산 중 Pan/Zoom, Net 하이라이트, 다른 탭과 분석 진행 표시가 계속 반응해야 한다. 응답성은 13.1절의 목표로 측정하고 기준 PC와 테스트 데이터는 M0에서 고정한다.
- CPU 스레드·RAM 예산은 사용자 선택을 반영. GPU는 초기 필수 계산 기능이 아니며 지원 가능한 연산/정밀도에 대해 별도 검증 후 활성화한다.

### 9.5 캐시 무효화 계약

캐시 키 = Board Model Hash + Substrate/PCB Scenario Geometry + Stackup/Material/DkDf version + Net/Reference/Port/Load + Group/Coupling + Frequency + Solver/Model/Mesh/Precision + Engine Version. 화면 Zoom/언어/색상은 결과 캐시 무효화 사유가 아니다. 기존 점을 재사용하더라도 모델 조건이 달라지면 재계산한다.


### 9.6 스윕 프로파일과 직접 검증 규칙

고정 Linear/Discrete 직접 Sweep을 기준 기능으로 제공한다. Progressive는 전체 대역 격자를 균등하게 세분화하는 모드와 미수렴 Region만 세분화하는 모드를 제공하고 실제 주파수 목록을 저장한다. Log 직접 Sweep은 별도 모드로 두며 Linear midpoint 규칙과 혼용하지 않는다. 두 그룹의 주파수 목록이 다르면 결과를 각각 보존한다.

0~30 GHz 전체 균등 세분화의 산술 예시는 다음과 같다. DC가 계산 가능한 경우이며, Adaptive 선택 세분화에는 같은 점 수를 강제하지 않는다.

| 간격 | 신규 직접 계산점 | 누적 직접 계산점 |
| --- | --- | --- |
| 1 GHz | 31 | 31 |
| 500 MHz | 30 | 61 |
| 250 MHz | 60 | 121 |
| 125 MHz | 120 | 241 |
| 62.5 MHz | 240 | 481 |
| 31.25 MHz | 480 | 961 |

수렴 단위는 Group × Region × Observable Set × Model Revision이다. 기본 Observable Set은 전체 S 행렬이며 요청한 Mixed-mode와 NEXT/FEXT를 추가한다. 각 새 직접 계산점에서 주변의 기존 직접 계산값으로 예측한 복소 S와 비교한다. 계산값을 예측 입력에도 넣어 잔차를 인위적으로 0으로 만드는 검사는 금지한다.

초기 설계 기준은 복소 S 최대 절대 잔차 0.001 이하를 2회 연속 세분화에서 만족하는 것이다. midpoint 외에 각 종료 후보 구간의 1/3 지점 등 독립 검증점을 최소 1개 직접 계산한다. 관련 단면/네트워크 수치 오차는 이보다 충분히 작도록 13.1절의 기준을 적용한다. dB·위상·기울기·Limit 근접도는 보조 검사이며 복소 잔차 실패를 무시하는 데 사용하지 않는다. |S|가 0.0001 미만인 항은 위상 오차 평가에서 제외하되 복소 S 검사는 계속한다.

초기·최종 샘플 간격은 선로 지연 기반 위상 변화 30° 이하, 즉 단방향 최대 유효 지연 τ에 대해 Δf ≤ 1/(12τ)를 안전 검토용 상한으로 사용한다. 왕복 반사나 결합 모드가 중요하면 관련 지연도 포함한다. τ를 신뢰할 수 없으면 지연 기준 충족으로 판정하지 말고 사용자가 정한 최대 간격과 검증 프로파일을 요구한다. 이 값은 BoardSI의 보수적 설계 규칙이며 좁은 공진 부재를 보증하지 않는다.

Minimum Step은 더 이상 세분화하지 않는 하한, Maximum Accepted Gap은 종료 때 허용하는 간격 상한으로 구별한다. 간격 하한·계산점·시간 예산을 먼저 소진하면 Budget-limited다. 계산점·시간 예산은 실행 전 유한한 값을 지정하고 예상 RAM/SSD와 함께 표시한다. 두 수렴 반복을 하지 않았거나 별도 검증점이 없으면 Converged로 처리하지 않는다. 프로파일 이름·버전·임계값·검증점·최대 잔차를 저장한다.

어떤 유한 샘플링도 임의의 매우 좁은 공진을 모두 검출한다고 보증할 수 없다. 여기서 Converged는 지정된 샘플링 기준 충족이라는 뜻이다. 고Q 공진·긴 지연·노치 위치를 바꾼 독립 시험에서 더 촘촘한 직접 Sweep과 비교하고, 조기 종료를 발견하면 프로파일을 수정한다. Progressive가 항상 더 빠르다고 보장하지 않는다.

### 9.7 구조 위험 추정과 그룹 구성

길이·단면 유효 유전율로 추정한 지연, Via Stub, Reference 전환, Plane Cutout, 선폭 변화와 인접 결합 구간을 Artwork에 표시한다. 이는 계산 우선순위와 추가 검증점 배치를 위한 추정이다. 감쇠량·공진 깊이·정확한 공진 주파수를 기하만으로 확정하지 않는다. εeff=3, 길이 100 mm의 이상적 균일 선로는 지연 약 578 ps, 1 GHz 간격의 전파 위상 변화 약 208°라는 계산 예가 되어 1 GHz 초기 격자가 항상 충분하지 않음을 보여준다.

선택 경로 밖 인접 도체를 제거하거나 Reference 영역을 축소할 때 결합/영역 확대 시험으로 영향을 확인한다. 미계산 그룹 사이 S 항목은 Unknown이며 0이 아니다. 모델이 없는 Via나 리턴 불연속은 샘플링을 늘려 해결되는 것으로 처리하지 않는다.

### 9.8 일시정지 취소와 복구

Pause는 신규 작업 배정을 멈추고 진행 중 작업을 안전 지점까지 수행한 뒤 Paused로 바꾼다. Pause 요청 수신과 실제 정지 완료를 별도 표시한다. Cancel은 신규 작업을 중단하고 완료된 직접 계산점과 체크포인트를 보존한다. 응답 없는 Solver는 사용자에게 강제 종료를 제공하되 GUI는 유지하고 마지막으로 확정 저장된 청크만 복원한다.

Run 시작 시 BoardModel·Scenario·Ports·Materials·Solver 설정을 불변 Snapshot으로 만든다. 실행 중 편집은 다음 Run의 Draft에 반영한다. 메시지는 Protocol Version, Run ID, Snapshot Hash, Group/Region ID, Sequence Number를 포함하며 이전 Run의 늦은 응답·중복 응답은 격리한다. 무거운 결과 수신·압축·파일 I/O를 GUI 스레드에 올리지 않는다. 이벤트 큐는 유한 크기로 제한하고 그래프 갱신은 병합하되 직접 계산값은 버리지 않는다.

진행률은 현재 Stage의 확정 작업 수에 대해서만 계산한다. Adaptive 전체 예상 작업 수와 ETA는 추정으로 표시하고 신규 세분화 시 갱신한다. 진행률 100%를 수렴이나 전기적 PASS로 해석하지 않는다. Heartbeat는 계산점이 늦어도 별도로 전송한다.

### 9.9 RAM과 SSD 캐시 정책

해석 전 BoardModel·행렬·작업당 임시 버퍼·캐시의 예상 사용량을 산정한다. 전체 Worker와 내부 선형대수 스레드를 합친 자원 한도를 적용한다. 모든 자료구조가 SSD로 이동 가능한 것은 아니므로 활성 Solver 작업 하나가 RAM에 들어가지 않으면 동시 작업 축소 또는 안전 중단을 선택한다. SSD Spill로 OOM을 무조건 막는다고 보장하지 않는다.

완료한 결과는 청크별 Hash/Checksum, Frequency ID, 포트 차원, Model Key와 함께 기록한다. 임시 파일 검증 후 동일 파일시스템에서 원자적으로 교체하고 Manifest를 마지막에 확정한다. 디스크 용량 부족·권한 오류·손상 청크를 탐지하고 이전 정상 체크포인트를 유지한다. 원자적 이름 변경만으로 전원 차단 내구성이 보장된다고 가정하지 않고 데이터 동기화와 복구 시험을 시행한다. [R19]

RAM/SSD 한도·저장 위치·체크포인트 주기·보존 기간은 Settings와 프로젝트 메타데이터에 표시한다. 살아 있는 Run과 사용자가 저장한 결과는 자동 축출하지 않는다. 임시 캐시는 만료/LRU 정리 대상으로 구분하고 사용자 삭제 기능을 제공한다. 기본 체크포인트 목표는 완료점이 있을 때 30초 이내이며 I/O 지연과 확정 저장 시각을 표시한다.

물리 계산점 Cache와 수렴 판정 Cache를 분리한다. Sweep 범위/허용오차 변경은 수렴 판정만 무효화할 수 있다. 동일 Frequency는 정규화된 주파수 키로 중복 제거하고 Region 경계점·동시 요청은 하나의 직접 계산 결과를 공유한다. 재개 시 Hash·Schema·Solver 버전 호환성을 검사하고 불일치 데이터는 재사용하지 않는다.

## 10 결과 표시와 파일 저장

U01의 계산 결과를 확인하고 다른 분석 도구와 연결하기 위한 설계항목 P08이다. 초기 결과는 임피던스·RLGC·S-parameter를 중심으로 하고, TDR·Eye·규격 적합성 인증은 기본 완료 조건에 포함하지 않는다.

| 결과 | 표시와 저장 기준 |
| --- | --- |
| Impedance | 특성 임피던스와 기준 임피던스를 분리. Single·Differential·모드 정의 표기 |
| RLGC | 주파수·구간·행렬 도체 순서·단위 길이 단위를 함께 표시 |
| S-parameter | 복소수 원본 보존. dB·위상·Real/Imag 표시와 추적 가능한 포트명 |
| Mixed mode | Sdd·Scc·Sdc·Scd를 구분. 미구현·미검증 항목은 N/A |
| 차이 비교 | Baseline과 Scenario를 동일 포트·Reference·주파수 조건으로 비교 |
| Export | Touchstone, CSV와 실행 메타데이터. 그래프 PNG 저장 |

### RL과 IL의 표시 관례

Sij는 입력 j에 대한 출력 i의 응답으로 정의한다. S11/Sdd11 및 S21/Sdd21 dB 그래프를 기본으로 한다. 양수 손실량으로 표현하는 Return Loss 또는 Insertion Loss와 부호를 혼동하지 않도록 축·범례에 관례를 명시한다. Limit은 사용자가 설정하며 특정 제품 목표를 모든 보드에 고정하지 않는다.

예: Sdd11 dB 상한 −10 dB, Sdd21 dB 하한 −3 dB를 입력하면 요청 대역의 유효 직접 계산점과 정해진 샘플링 조건에 대해 각각 판정한다. 이는 UI 동작 예시이며 본 프로그램의 정확도 기준 또는 모든 인터페이스의 규격이 아니다. 모델 신뢰도와 전기적 Limit 판정도 서로 다른 필드로 표시한다.

### Touchstone과 프로젝트

기본 Export는 호환성을 위한 single-ended .sNp, RI 데이터, Hz 단위, 단일 실수 Zref를 기본으로 한다. 혼합 모드 또는 포트별 Reference는 Touchstone 2.x의 관련 규칙에 맞춰 별도로 제공한다. 저장 후 독립 파서로 다시 읽어 행렬·주파수·포트 순서가 보존되는지 시험한다. [R5]

프로젝트에는 원본 해시, 정규화 보드, Stackup, Reference, Port Map, Scenario, Frequency, 엔진 버전, 모델 가정, 결과 상태를 저장한다. 파일 경로가 바뀌어도 재연결할 수 있게 하며, 원본 파일을 자동 덮어쓰지 않는다. SParamView 연계는 Export 파일 열기 검증부터 시작하고 소스 통합은 별도 결정한다. [T05, T12]


### 10.1 상태와 Limit 판정

| 상태 축 | 값 예시 | 판단 대상 |
| --- | --- | --- |
| 실행 | Running, Paused, Completed, Failed, Cancelled | 작업 종료 여부 |
| 샘플링 | Not assessed, Converged, Budget-limited | 직접 샘플과 검증점의 충족 |
| 모델 | Validated, Approximate, Unsupported | 구조·재료·Port·대역 검증 범위 |
| 입력 근거 | Source, User-entered, Assumed, Conflicting | 입력값의 출처와 불확실성 |
| 전기적 Limit | PASS, FAIL, Inconclusive, Not configured | 지정 기준과 비교 |

FAIL은 유효 계산점에서 Limit 위반이 확인된 상태다. PASS는 요청 범위의 필수 항목·모델·샘플링 검사를 통과하고 위반이 없는 경우에 한정한다. 누락 대역·Unsupported·Budget-limited·실패 또는 계산 오차보다 작은 Limit 여유는 Inconclusive다. Approximate/Assumed 모델의 기준 충족은 ‘가정 조건에서 충족’으로 표시하고 무조건적인 PASS로 승격하지 않는다. 전체 대역의 무한한 모든 주파수를 검증했다는 의미로 표현하지 않는다.

그래프는 Sij[dB]=20 log10|Sij|를 사용하고, 양수 손실 표시를 선택하면 RL/IL은 해당 값의 부호 반전임을 범례에 명시한다. 원본 복소 S=0은 그대로 보존하고 파생 dB는 −∞, 위상은 미정으로 처리하며 화면 바닥값은 표시용으로만 사용한다. 다른 포트 순서·Reference·Load·주파수 격자의 Scenario는 자동으로 같은 조건이라 간주하지 않는다. 차이 그래프는 공통 직접 계산점 또는 명시적 보간 결과로 구별한다.

### 10.2 Touchstone 데이터 보존 규칙

기본 Export는 단일 양의 실수 Zref, Single-ended RI·Hz다. 2-Port의 순서와 N-Port 행렬 순서를 형식 규칙대로 처리한다. Mixed-mode Export는 [Mixed-Mode Order]와 지원되는 실수 Reference 규칙을 지키며 P/N·물리 Port 대응을 함께 저장한다. 다른 Zref로 바꾸는 것은 Header 변경이 아닌 검증된 재정규화 연산이다. [R5]

한 파일의 모든 주파수는 동일한 전체 Port 차원의 완전한 행렬을 가져야 한다. 주파수는 오름차순이며 중복을 제거한다. 그룹별 다른 격자나 미계산 결합항을 0/NaN으로 채워 합치지 않는다. 필요한 점을 직접 계산하거나 그룹별 파일로 내보낸다. Partial Export는 사용자가 명시적으로 선택한 유효 부분만 저장하고 요청/제외 대역·상태를 메타데이터에 남긴다. 보간 결과는 원본 직접 결과와 별도 이름으로 저장한다.

Touchstone 본문·주석은 ASCII 규칙을 따른다. 한글 Net·Pin 이름은 안정적 ASCII 별칭과 UTF-8 JSON Sidecar의 대응표로 보존한다. Sidecar에는 원본 Hash, Port Map, 단위, 모델 범위, 직접/보간 구분, 실행 및 수렴 상태를 넣는다. 충분한 유효 자릿수로 저장하고 독립 Parser의 Round-trip 오차를 검사한다. [R5,T12,T44]

### 10.3 프로젝트 저장과 재현성

프로젝트 Manifest는 Schema Version, 문서 기준 버전, BoardModel Hash, Source Hash 목록, Importer/엔진/모델 버전, Scenario, Port Map, Frequency Profile, Result Index를 포함한다. 원본 파일은 외부 링크 방식과 사용자 선택의 포함 방식으로 구분하고 캐시는 프로젝트 필수 데이터와 분리한다. 외부 경로가 사라져도 저장된 Snapshot으로 조회 가능한 범위를 표시하며 재Import 시 원본을 다시 요구한다.

신규 Schema를 구버전 프로그램이 만나면 읽기 전용 또는 명시적 오류로 처리하고 덮어쓰지 않는다. Migration은 새 파일에 수행한 뒤 의미·Hash·Port Map을 검증한다. 동일 프로젝트의 동시 쓰기는 잠금 또는 낙관적 버전 검사로 방지한다. 파일 저장 후 성공으로 표시하기 전에 오류·Checksum·Manifest 완결성을 확인한다. 오류 진단 파일은 사용자가 검토한 범위만 수동 내보내며 Net·좌표·개인 경로를 기본 제외한다.

## 11 BoardSI 모듈 구조와 내부 데이터 계약

### 11.1 모듈 구조

Format Detector와 Import Adapter는 입력을 Canonical Board Model로 정규화한다. Viewer·Topology·Port Validator는 같은 모델을 읽고, Run Snapshot을 Scheduler에 전달한다. Scheduler가 Solver 작업을 요청하고 결과는 Cache와 Results/Export로 전달된다. Progressive Sweep은 Solver를 사용하는 스케줄러이며 Solver 다음의 일회성 후처리가 아니다.

| 모듈 | 책임 | 소유 데이터와 경계 |
| --- | --- | --- |
| Import/Normalize | 형식 검출·단위·기하·연결 정규화 | Source와 Canonical Model |
| Viewer/Editor | Artwork·선택·Stackup·Port·Scenario 편집 | Draft, 화면 상태 |
| Validation | 입력·연결·모델·Port 사전 검사 | 오류와 근거 객체 |
| Scheduler/Region | 주파수·그룹 큐, 수렴, 취소 | Run Snapshot, 작업 상태 |
| Rust Solver | RLGC·네트워크·S 직접 계산 | 불변 입력, 주파수별 결과 |
| Resource/Cache | RAM/SSD 예산·청크·Checkpoint | 확정 결과와 임시 데이터 |
| Results/Export | Live Plot·비교·Limit·파일 | 결과 상태와 Port Map |


입력 형식별 Parser 외의 모든 기능은 Canonical Model API를 사용한다. Plot/Qt UI가 Solver 상태·메모리를 직접 수정하지 않고 버전이 있는 Snapshot/Message 형태로 통신한다. Source Importer가 하나 늘어나도 Solver, Port, Viewer의 로직을 재작성하지 않는다.

### 11.2 필수 데이터 타입 개념 설계 

| 객체 | 계약 |
| --- | --- |
| `ImportSource` | 파일 목록, Detect Result, 원본 SHA-256, 작성 프로그램/버전, 경로, 권한/출처 |
| `BoardModel` | Layers/Materials, Components/Pins, Nets, Copper Geometry/Area, Pads, Vias, Outline, Geometry IDs |
| `ConnectivityGraph` | 구리 영역별 실제 접속, Open/Short 의심, 물리 근거, Source Netlist 비교 |
| `GeometryProvenance` | Source File + Object ID/Type + 변환·오차·손실 로그 |
| `ImportCapability` | Format+Version+Object coverage+G/N/P/S/R/M 충족 상태 |
| `PortDefinition` | 접속면, Signal/Reference, Zref, 방향, Pair Mapping, Load, Validation |
| `AnalysisGroup` | 하나/복수 결합 Net, Port Set, 선택 영역, 모델 적용 범위 |
| `AnalysisConfig` | Material, Stackup, Reference, Group, Port, Frequency Regions, Resources, Solver |
| `Scenario` | 원본 보드에 대한 변경 Overlay, Undo/Redo, 유효성·결과 무효화 |
| `SweepJob` | Group×Region×Frequency별 상태, 우선순위, 시간/자원 예산 |
| `ResultCache` | 계산된 복소 S/RLGC/임피던스, 캐시 Key, Partial/Validated Flag, Checkpoint |
| `RunResult` | S matrix, Z/RLGC, validation/warnings, convergence audit, port order, model provenance |

### 11.3 성능 검증 이식성

Rust Core는 Float64/Complex Float64 기본, GUI 비차단, 인덱싱된 Geometry/Spatial Query, Chunked/On-disk 캐시를 목표로 한다. Qt 바인딩은 C ABI/CXX-Qt 등 후보를 시제품 비교 후 정한다. 다중 플랫폼 결과 비교에는 동일 BoardModel Snapshot·같은 포트/수치 설정/Precision·같은 결정적 Test Vector를 사용한다. 특정 아키텍처에서 데이터 길이/정렬이나 대수 계산 순서가 달라 결과가 변하면 재현성 기록과 허용오차 검증을 수행한다.


## 12 플랫폼 품질과 데이터 보안

U03·U04·U14·U15를 구현하는 설계항목 P10이다. 세 플랫폼 모두 동일 핵심 기능과 결과 규칙을 제공한다. 정확한 최소 OS·CPU·RAM·Qt/Rust 버전은 시제품 측정 후 지원표로 고정한다.

| 항목 | 개발 기준 |
| --- | --- |
| 플랫폼 | Windows x64·ARM64와 macOS arm64 각각 네이티브 빌드 및 실제 실행 검증 |
| 언어 | English 기본. Settings에서 한국어 전환, Net 이름과 원본 데이터는 번역하지 않음 |
| 화면 | 고해상도 배율·폰트 확대, 키보드 이동, 그래프·Artwork 영역 크기 조절 |
| 숫자와 단위 | 내부 SI. mm·µm·Ω·ps·GHz 등 표기. OS 소수점·경로 인코딩 영향 검사 |
| 반응성 | 계산 중 UI 유지, 취소, 메모리 한도 초과 전 사전 안내 |
| 복구 | 임시 저장 후 원자적 교체, 실패한 실행 격리, 손상 프로젝트 오류 안내 |

### 오프라인 처리

회사 PCB·Net 이름·좌표·Stackup·분석 결과를 외부 API, 클라우드 해석, 자동 오류 보고, 원격 로그에 전송하지 않는다. 네트워크를 차단한 환경에서 Import부터 Export까지 완료되어야 한다. 업데이트 확인이 있다면 사용자 실행에 한정하고 보드 데이터와 분리한다.

공개 저장소의 테스트 데이터는 공개 출처와 재배포 조건을 확인한 샘플 또는 직접 만든 합성 샘플로 한정한다. 회사 데이터는 Git 추적 경로·CI 작업·자동 첨부 대상에서 제외한다. .gitignore만으로 보호 완료라 간주하지 않고 커밋·배포 산출물에도 포함되지 않았는지 검사한다.

### 자원 목표의 확정 방식

M0에서 RAM 8 GiB급 CPU-only 환경을 저사양 검증 후보로 삼고 실제 기준 PC·OS·CPU·가용 RAM·SSD와 소형·중형 공개 보드 2종 이상을 고정한다. 이는 최소 지원 사양 확정이 아니다. Import 시간·최대 RAM·Artwork 조작·1001점 직접 해석 시간을 측정하고 13.1절의 GUI 목표 및 15장의 성능 게이트를 적용한다. 초기부터 근거 없는 수 초 완료나 GPU 배속을 보장하지 않는다.

Qt의 플랫폼 지원과 별개로 Rust 연결부·차트·배포 의존성의 지원을 검증해야 한다. CXX-Qt의 일반적인 Rust/Qt 연결 가능성만으로 BoardSI의 ARM64 빌드·배포를 보장하지 않으며 실제 검증 과제로 둔다. [R6, R7, T10, T13, T14]


## 13 인수시험 및 독립 교차 검증

T01~T40의 의미는 v0.4를 유지하고 묶여 있던 T01~T15를 개별화했다. T41~T54는 이번 검토에서 추가했다. 아래는 구현 후 실행할 인수시험 명세이며 이번 문서 검토에서 소프트웨어 시험을 실행했다는 뜻이 아니다.

| Test ID | 주요 검증 | 통과 조건 |
| --- | --- | --- |
| T01 | ASC 형식·버전·단위 | 회로도 ASC 오인 없음, 지원표와 일치 |
| T02 | 필수 물성·기하 누락 | 누락을 임의 대체하지 않고 관련 Run 차단 |
| T03 | Layer·Via·Padstack·Copper | 기준 보드의 연결·절연·층 범위 일치 |
| T04 | Reference 분할·전원 분류 | 서로 다른 Net 자동 단락 없음 |
| T05 | P/N·Mixed-mode | 지정 순서·극성·Reference로 독립 행렬과 일치 |
| T06 | 균일 선로 수치 정확도 | 13.1절 수학 검증 오차 충족 |
| T07 | 손실·수동성·상호성 | 정규화 및 오차 조건에 맞는 잔차 검사 |
| T08 | Scenario·다중 파일 | 원본 불변, 다른 Run 결과 혼입 없음 |
| T09 | DC·Log·주파수 | DC 별도 처리, 비정상 격자 거부 |
| T10 | 자원·취소·복구 | 13.1절 응답성과 안전 정지 충족 |
| T11 | GPU 선택 기능 | 미지원 비활성, 활성 Backend CPU 비교 |
| T12 | 프로젝트·Export | 독립 Parser 왕복 및 Port Map 일치 |
| T13 | 3 플랫폼과 언어 | Native 실행·Unicode 경로·소수점 정상 |
| T14 | 오프라인·기밀 보호 | 차단망에서 기본 흐름 완료, 외부 송신 없음 |
| T15 | 패키지 일관성 | 버전·Hash·의존성·설명서·샘플 일치 |
| T16 | ASC·ODB++ 등 동일 보드 Cross-Format | Net/Layer/Trace/Pad/Via/Anti-pad별 차이 Report |
| T17 | 동일 확장자 다른 형식 | `.brd`, `.asc` 헤더/구조 인식, 오탐 금지 |
| T18 | Gerber Only Import | Artwork 가능하되 미완전 Net/Stackup으로 Run 차단 |
| T19 | Gerber+Drill+IPC-D-356B+Stackup | 파일 병합/단위/좌표 매핑 검증, 잔여 불명 항목 표시 |
| T20 | 미지원/유실 객체 강제 입력 | 해석에 영향을 주면 Error, display only면 Warning |
| T21 | Canonical Model 안정성 | 공통 API 유지, 동일 입력 ID 안정·형식 간 대응표 검증 |
| T22 | Import 압축/비정상 파일 | ZIP Slip·압축 폭탄·큰 객체 수 안전 처리 |
| T23 | 수동 Port Copper 외부 클릭 | P001 Error 및 Run 차단 |
| T24 | GND Plane Cutout/분리 | P009 검출, Unsupported 경로면 Run 차단 |
| T25 | Pair P/N 역전/Port 번호 변화 | Matrix 변환·Sdd/Scd 값 독립 계산과 비교 |
| T26 | Branch/Series R·C 포함 신호 | 미정의 부하·Net 단절 검출, 임의 Short 금지 |
| T27 | Reimport 후 Port 이동/Net 변경 | P012 재검증, 이전 매핑 자동 신뢰 금지 |
| T28 | Near/Far 4-Port 행렬 | 포트 순서·정규화·기본 50/100 Ω 근거 유지 |
| T29 | 단순 Microstrip·Stripline 1차 Sweep | 처음 1 GHz 간격 후 0.5/0.25 GHz 중간점만 추가 |
| T30 | Group×Region 독립 수렴 | 한 영역 완료해도 다른 영역 실행 지속 |
| T31 | 노치·급변 응답 | Midpoint 검증/최소 해상도로 과도한 조기 수렴 방지 |
| T32 | Abort/Pause/Resume/Crash Restart | Partial과 완료 데이터 분리, 체크포인트 일관성 |
| T33 | Stackup/Port/Load 변경 후 캐시 | 물리 변경 무효화, Sweep 변경 점 재사용, 혼입 금지 |
| T34 | GUI 스트레스(무거운 Import/해석) | Viewer Zoom/Pan/Cancel 등 응답 유지 |
| T35 | RAM 부족 및 SSD Spill | 예산 선점·안전 중단, 디스크 오류·복구 무결성 |
| T36 | Substrate Pad-to-Pad 모델 경계 | Die/Ball Pad 포함, Bump/Ball/Wirebond 제외 확인 |
| T37 | Microvia/IVH/Plane Anti-pad | Geometry와 연결 상태 유효, 미검증 효과는 Unsupported |
| T38 | 3D 구조의 모델 미지원 | Solver가 결과를 근거 없이 채우지 않고 Unsupported 표시 |
| T39 | 세 플랫폼 결과·Import 재현성 | 공통 테스트 보드·Point/Port/Complex S 비교 |
| T40 | 정식 공개 릴리즈 산출물 | 사양서/실행/제약/샘플·라이선스·경고 이력 일치 |
| T41 | 고정 Direct와 Progressive 교차 비교 | 전 행렬·별도 밀집 검증점에서 프로파일 오차 충족 |
| T42 | NEXT/FEXT와 결합 그룹 | 7.8절 정·역방향 매핑, 미계산 항 Unknown |
| T43 | 비수동·비상호·특이행렬 입력 | 단순 원소 검사로 오판하지 않고 원인 표시 |
| T44 | 불균등 격자·부분 Export·한글 Net | 완전 행렬만 저장, Sidecar·ASCII·왕복 검증 |
| T45 | 완료·수렴·모델·Limit 상태 조합 | 미수렴/미지원/누락 요청은 전역 PASS 금지 |
| T46 | 늦은 IPC·중복 결과·편집 중 Run | Snapshot 일관, Run ID 혼입·중복 저장 없음 |
| T47 | 프로젝트 이관·동시 저장·경로 변경 | 원본 보존·충돌 차단·Schema 호환성 |
| T48 | 세분화·Region 경계·재개 | 동일 키 중복 직접 계산 없음, 필요점만 재계산 |
| T49 | 진단 출력·캐시·공개 패키지 | 기밀 원본·좌표·Net·개인 경로 자동 유출 없음 |
| T50 | UI·Heartbeat·Cancel 시간 계측 | 13.1절 목표, 포화/느린 Solver에서도 충족 |
| T51 | 미세 Substrate·ODB Step·극성 | 최소 간격 보존·Step 선택·Net/도체 정합 |
| T52 | 치수·물성 범위와 외삽 | z/간격 재계산, 모델 범위 밖 자동 Validated 금지 |
| T53 | DC 미지원·깊은 노치·긴 지연 | N/A 전파·위상 제외·오프그리드 검증 정상 |
| T54 | SSD 부족·쓰기 중 강제 종료 | 마지막 확정 청크 복원, Partial 완료 위장 없음 |

**검증 원칙:** 파서의 구조 정확도와 SI Solver의 물리 정확도를 **별개로 검증**한다. 단순 무손실선로·해석식, 독립 구현, 필요 시 상용 해석 또는 측정 데이터로 교차 확인한다. 결과 수렴과 물리모델 Validated 상태도 독립적인 필드로 기록한다.


### 13.1 정량 인수 목표와 측정 조건

다음 값은 이번 검토의 **설계 목표**다. 제품의 현재 측정값이나 실제 PCB 정확도 보증이 아니다. 표의 기준을 바꾸려면 실패 사례·비교 결과·영향 범위와 변경 이력을 기록한다.

| 항목 | 기준 | 측정 범위 |
| --- | --- | --- |
| 동일 수학 모델 S | 최대 복소 절대오차 0.000001 이하 | 해석식·독립 구현과 동일 조건 비교 |
| 단순 무손실 기준선 | 임피던스·지연 상대오차 0.1% 이하 | 정확한 기준해가 있는 지원 모델 |
| 파일 왕복 S | 최대 복소 절대오차 0.000000000001 이하 | 정규화된 유한값 기준, 충분한 자릿수 |
| Progressive 잔차 | 복소 절대오차 0.001 이하 | 9.6절 직접 검증 및 2회 연속 충족 |
| 수동성·상호성 | 수학 기준 쿠폰 잔차 0.000001 이하 | 정규화 조건 일치; 모델 검증과 별개 |
| UI 조작 반영 | p95 200 ms 이하, 1 s 초과 정지 없음 | 기준 PC의 Pan/Zoom/탭/취소 100회 이상 |
| Pause/Cancel 수신 표시 | 1 s 이내 | Solver 종료 완료 시간과 구별 |
| Heartbeat | 1 s 주기, 5 s 무수신 시 상태 경고 | 무응답 탐지이지 강제 실패 확정 아님 |
| 강제 종료 제공 | Cancel 후 10 s 내 정상 종료 불가 시 선택 제공 | GUI 생존, 확정 청크 보호 |
| Live 결과 반영 | 새 결과 수신 후 1 s 이내 | 대량 갱신은 병합, 원본 샘플 보존 |

수동성 잔차는 max(0, 최대 특이값−1), 상호성 잔차는 최대 |Sij−Sji|로 정의한다. 수치적으로 민감한 행렬은 조건수와 오차를 별도로 평가한다. 단면 수치해의 Mesh/경계 영역 수렴, 전송선 구간화, 주파수 샘플링을 각각 시험한다. 한 항목의 통과가 다른 항목의 오차를 대신하지 않는다.

실제 PCB/Substrate 정량 정확도는 구조·치수·주파수별 독립 EM 또는 측정 기준과 비교해 정한다. 비교할 때 재료·Reference·Pad 포함 범위·Port 기준면·종단·재정규화를 맞추고 해당 기준 데이터의 불확실성도 기록한다. 증거가 없는 구조는 Validated로 공개하지 않는다. 성능 보고에는 CPU/OS/RAM/SSD, 보드 객체 수, 도체/Port 수, 주파수 수, Worker 수, 최대 RAM, I/O와 경과시간을 함께 남긴다.

## 14 검증용 데이터와 단계별 개발 로드맵

### 14.1 공개 샘플 확보

- 각 형식별 공식 공개 Test Data와 상호 변환 가능한 동일 설계 샘플을 확보한다. 샘플 출처·파일 해시·작성 툴 버전·재배포 권한을 기록한다.
- 제조데이터 Gerber/Drill/IPC-D-356B는 실제 완성 데이터 번들로 테스트한다. Layer Map 및 Drill Plating 정보도 검증한다.
- 자체 합성 Microstrip·Stripline·차동 및 Substrate Pad-to-Pad 단순 쿠폰, Blind/Buried Via, Reference Cutout, 손상 입력도 포함한다.
- 사내 PCB 파일을 GitHub·CI·온라인 변환 도구에 업로드하지 않는다. Local Test Results와 공개 테스트 결과를 분리한다.

### 14.2 단계와 제품 완료 범위

| 단계 | 목표 | 종료 게이트 |
| --- | --- | --- |
| M0 | Architecture/Canonical Model/기술 검증 | 인터페이스·별도 Solver IPC 시제품, 기준 PC, 3 플랫폼 최소 빌드 |
| M1 | PADS ASC 및 ODB++ Import, Viewer | 객체/접속 검증 및 비동기 Import·취소 통과 |
| M1a | PCB+BGA Substrate 공통 Layer/Pad/Via/Anti-pad | Pad-to-Pad 경계·제외 구조 확인 |
| M1b | Port 후보 자동화·수동 UI·Validation | P001~P012, Single/Diff·Reference 확인 |
| M2 | 경량 RLGC/전송선로/S Export MVP | 고정 Direct·DC·CPU·독립 수치·Touchstone 검증 |
| M2a | Progressive Sweep + Group×Region 상태 | 캐시 재사용·실시간 곡선·조기 종료 방지 |
| M2b | Worker/Solver 격리 + RAM/SSD Cache | 기초 비차단 구조 강화·Spill·저장·장시간 장애 시험 |
| M2c | 검증 가능한 Substrate Via/Pad 모델 | 범위/정확도/모델 한계 투명 표시 |
| M3 | IPC-2581, KiCad, Gerber/Drill, Netlist 보조 Import | G/N/P/S/R/M Import Summary와 Cross-Format 검증 |
| M4 | EAGLE, EasyEDA, Altium, Allegro 등 확대 | 형식/버전별 직접/변환/미지원 명확화 |
| M5 | 기타 CAD, 모델 범위·성능 최적화 | 수요/라이선스/고정 테스트로 판단 |

**주의:** 위 단계는 의존관계와 완료 범위의 기준이며 일정·인력 투입량·지원 버전 확정이 아니다. M4 형식 확대 전이라도 미지원 형식의 안전한 오류 처리는 M1에서 제공한다.


**초기 정식 범위:** M0~M2b 및 M1a/M1b를 통과한 PADS ASC·ODB++와 검증된 균일 선로 해석을 필수로 한다. Substrate 기하와 Port 정의를 지원하더라도 Via/Pad 모델이 없는 전체 경로는 Unsupported다. M2c를 통과하기 전 ‘완전한 Substrate Pad-to-Pad SI 지원’으로 표현하지 않는다. M3~M5, GPU 가속, 완성 패키지·커넥터 외부 모델, TDR/Eye/규격 인증은 후속 범위다.

GUI 비차단·오프라인·기밀 보호는 M0/M1부터 적용하는 공통 요구이며 M2b까지 미루지 않는다. M2b는 장애·장시간·자원 압박 검증의 완료 게이트다. 구현 단계가 앞섰다고 필수 품질 게이트를 생략하지 않는다.

## 15 구현 결정과 릴리즈 게이트

제품 요구사항을 확정하는 것과 실제 구현 선택을 동결하는 것은 구별한다. 아래 항목은 방치된 요구사항이 아니라 증거가 필요한 기술 결정이다. 담당 개발자는 결정 레코드를 작성하고 검증 담당이 시험 근거를 확인한다. 제품 범위·필수 기준 변경은 사용자 승인 대상이다. 게이트를 넘길 때 관련 값이 미정이면 그 기능의 정식 릴리즈를 차단한다.

| 결정 항목 | 결정 시점 | 필수 증거 |
| --- | --- | --- |
| Qt Widgets/Quick·연결부·IPC Schema | M0 종료 | 3 플랫폼 실행·응답·배포 시제품 |
| Rust/Qt/컴파일러·의존성 버전 | M0 종료 | 고정 버전·Lockfile·Native 빌드 |
| 저사양 기준 PC·시험 보드·성능 예산 | M0 종료 | 2종 보드, 재현 환경, 목표값 등록 |
| PADS·ODB++ 지원 Revision/객체 | M1 종료 | 출처·Hash·기대 결과·손실 목록 |
| 좌표 오차·Import 자원 상한 | M1 종료 | 미세 간격·대형·악성 입력 시험 |
| Port Terminal 모델·단면 알고리즘 | M2 종료 | 수치 기준해·Reference/경계 검증 |
| Progressive 프로파일의 실증 검증 | M2a 종료 | 9.6절 기준과 노치·지연·공진 비교 |
| RAM/SSD 한도·주기·수명 | M2b 종료 | 기준 PC·용량 부족·Crash 복구 시험 |
| Via/Pad 불연속 모델·유효 대역 | M2c 종료 | 독립 EM/측정 비교와 오차표 |
| 추가 형식과 Gerber 병합 범위 | 해당 M3~M5 종료 | 형식·객체별 지원표와 교차시험 |
| GPU Backend와 정밀도 | 기능 활성화 전 | CPU 오차·실측 시간·Fallback |
| 최소 OS/CPU/RAM와 설치 패키지 | 정식 릴리즈 전 | 3 Native 환경·깨끗한 설치 시험 |
| 코드·Qt 모듈·Parser·샘플 라이선스 | 의존성 도입 및 공개 전 | 구성별 고지·의무·재배포 검토 |
| 저장소명·배포 서명·업데이트 정책 | 공개/패키징 전 | 공개 범위·기밀 제외·배포 안내 |
| 실제 구조 Accuracy와 출시 일정 | 기능별 릴리즈 전 | 모델 검증 보고서 및 완료 게이트 |

**릴리즈 산출물:** Windows x64, Windows ARM64, macOS Apple Silicon arm64의 Native 패키지, 같은 소스 태그·Commit·SHA-256, 사용설명서, 변경 이력, 지원 형식/버전/객체 표, Import 손실, 모델·Port·Reference·대역/오차표, Schema, 수렴 및 복구 정책, 공개 샘플 출처, 재현 가능한 시험 보고서와 의존성/라이선스 고지를 제공한다. 빌드 성공만으로 실제 실행을 대체하지 않는다.

자체 코드의 컴파일·정적 검사 경고는 해결한다. 외부 의존성 경고가 남으면 원인·영향·대응과 수용 근거를 기록하고 경고를 숨겨 무경고로 표시하지 않는다. 서명·Notarization 상태를 실제 상태대로 안내하며 임시 서명을 정식 인증으로 표현하지 않는다. 릴리즈 문서와 패키지의 기능 범위는 일치해야 한다.

**릴리즈 차단:** 원본 손상, 임의 Net/Copper 단락, Port/P-N 오류, 누락 Material 무단 채움, 미지원 구조의 무경고 결과, 미수렴을 수렴으로 표시, 잘못된 Cache 재사용, 민감 데이터 송신, 무응답 GUI, 저장·복구 손상, 필수 시험 미실행 또는 실패, 해당 기능 결정 게이트의 미정 상태다. 단순 문서 완성은 이 게이트의 통과가 아니다.

## 16 참고 자료와 근거 구분

### 16.1 기존 사용자 대화 근거

- **D1 (2026-10-13)**: BoardSI 경량 Rust+Qt SI, 3개 플랫폼, PADS ASC와 입력 확대, Artwork/Stackup/Reference/Signals/Resources 흐름, 로컬 데이터 보호.
- **D2 (2026-10-14)**: BoardSI 명칭, 1 GHz→0.5→0.25 GHz Progressive Sweep, 계산점 Cache, Group×Region 독립 수렴, GUI 비차단·Solver 분리·RAM/SSD 체크포인트.
- **D3 (2026-10-14)**: PCB 및 BGA/FC-BGA Substrate, Pad-to-Pad, Trace/Via/Pad/Plane/Anti-pad, Bump/Cu Pillar/Solder Ball/Wirebond 제외.
- **D4 (2026-10-14)**: ODB++는 입력의 하나에 불과하며 자체 PCB 모델로 그래픽화·해석. 가능한 많은 PCB 파일 형식 지원 요청.
- **D5 (2026-10-14)**: Port 자동 후보/수동 수정/검증, Single 및 Differential Port/Reference/Mapping, 잘못된 설정 차단.
- **D6 (2026-10-14)**: 최신 개발 사양서 전체 검토·문제 수정·필요 항목 추가 요청. U41~U48 및 새 정량 기준은 이 검토에서 작성한 설계 기준이며 과거 승인 사항과 구분한다.

### 16.2 기술 근거 공식 문서 중심 

- **R1** KiCad PADS Importer (PADS ASCII 구조/헤더): https://dev-docs.kicad.org/en/import-formats/pads/index.html
- **R2** KiCad Board File Format (`.kicad_pcb`): https://dev-docs.kicad.org/en/file-formats/sexpr-pcb/
- **R3** Ucamco Gerber Format (X2/X3, Layer/속성, 생산 데이터): https://www.ucamco.com/en/gerber
- **R4** Ansys 2D Extractor Technical Notes (준 TEM·균일 단면·3D 불연속 한계): https://ansyshelp.ansys.com/public/Views/Secured/Electronics/v252/en/Subsystems/Q3DExtractor/Content/Q3D/2DExtractorTechnicalNotes.htm?TocPath=Q3D+Extractor+Help%7C2D+Extractor+Technical+Notes%7C_____0
- **R5** IBIS Touchstone v2.1: https://ibis.org/touchstone_ver2.1/touchstone_ver2_1.pdf
- **R6** CXX-Qt Developer Documentation: https://kdab.github.io/cxx-qt/book/
- **R7** Qt Supported Platforms: https://doc.qt.io/qt-6/supported-platforms.html
- **R8** Qt Licensing: https://doc.qt.io/qt-6/licensing.html
- **R9** KiCad Import Format Overview (Allegro/Altium/CADSTAR/EAGLE/EasyEDA/Fabmaster/gEDA/P-CAD/PADS): https://dev-docs.kicad.org/en/import-formats/index.html
- **R10** ODB++Design official resources and specifications: https://odbplusplus.com/design/our-resources/
- **R11** Ucamco official Gerber Specifications/Test Files: https://www.ucamco.com/en/guest/downloads/gerber-format
- **R12** IPC-2581 Consortium / IPC-DPMX: https://www.ipc2581.com/
- **R13** KiCad Altium Binary File Import Developer Notes: https://dev-docs.kicad.org/en/import-formats/altium/index.html
- **R14** Autodesk EAGLE Board XML/Object Types: https://help.autodesk.com/cloudhelp/ENU/Fusion-ECAD/files/ECD-ULP-OBJECT-TYPES.htm
- **R15** IPC-D-356B Bare Substrate Electrical Test Data Format (공개 목차, 전체 규격이 아님): https://www.ipc.org/TOC/IPC-D-356B.pdf
- **R16** KiCad PCB Editor Import User Guide: https://docs.kicad.org/master/ko/pcbnew/pcbnew.html

- **R17** Qt Threads and QObjects (GUI 주 스레드와 Worker의 역할): https://doc.qt.io/qt-6/threads-qobject.html
- **R18** Ansys Lossy Transmission Lines (주파수 의존 R/L/G/C 네트워크): https://ansyshelp.ansys.com/public/Views/Secured/Electronics/v261/en/Subsystems/Q3DExtractor/Content/Q3D/LossyLines.htm
- **R19** Qt QSaveFile (임시 파일·Commit·직접 덮어쓰기의 원자성 차이): https://doc.qt.io/qt-6/qsavefile.html
- **R20** Qt QProcess (별도 프로세스와 비동기 실행): https://doc.qt.io/qt-6/qprocess.html

- **R21** Ansys Matrix Generation for 2D Extractor (Maxwell 행렬과 소자값 구분): https://ansyshelp.ansys.com/public/Views/Secured/Electronics/v252/en/Subsystems/Q3DExtractor/Content/Q3D/MatrixGenerationfor2DExtractor.htm
- **R22** Ansys Passivity (다포트 S 행렬의 최대 특이값 조건): https://ansyshelp.ansys.com/public/Views/Secured/Electronics/v242/en/Subsystems/Circuit/Content/Circuit/Passivity.htm

**기술 근거 확인일:** 2026-10-14 KST. R1~R22 중 R3는 R11의 공식 자료 목록으로 함께 확인했다. R6/R7 등의 웹 문서는 현재 문서를 가리키므로 실제 의존성 채택 시 버전별 문서를 고정한다. R15의 공개 목차만으로 Parser의 레코드 상세를 구현·검증하지 않는다. 원문 규격·합법적 자료·샘플 확보가 필요하다.

**근거 한계:** 외부 문서가 형식을 정의하거나 타 프로그램의 Import 가능성을 보여주더라도 BoardSI의 구현 완료·바이너리 호환성·물리 해석 정확도를 입증하지 않는다. 본 문서의 우선순위·상태 규칙·정량 목표는 BoardSI의 **설계 기준**이며 외부 표준의 강제 요구나 달성된 성능으로 인용하지 않는다. 문서 개정일 기준으로 재검토한다.

## 17 변경 이력

| 버전 | 일자 (KST) | 변경 요지 |
| --- | --- | --- |
| v0.1 | 2026-10-13 | 경량 Rust+Qt SI 및 PADS ASC 중심의 최초 사양서 |
| v0.2 논의 | 2026-10-14 | BoardSI 이름, Progressive Sweep, 독립 수렴, GUI 비차단, 하이브리드 Cache |
| v0.3 논의 | 2026-10-14 | PCB+BGA Substrate 범위, Pad-to-Pad 경계, 제외 구조 |
| v0.4 | 2026-10-14 | 다형식 Import·중립 내부 모델, Port 자동/수동/Validation 통합 |
| **v1.0** | **2026-10-14** | **최종 확정용 검토본. Pair 오타·캐시·종료 조건 수정, Substrate/스윕/복구 보강, T01~T54와 결정 게이트 정리** |

**문서 작성/수정은 개발 완료·테스트 성공·릴리즈 수행을 의미하지 않는다.** 지원 형식, 알고리즘, Port 타입, 수치 성능은 코드를 구현한 뒤 지정된 인수시험으로 별도 검증해야 한다.


## 18 검토 결과와 추적 연결

### 18.1 v0.4 문제와 반영 내역

| 검토 항목 | 확인한 문제 | v1.0 반영 |
| --- | --- | --- |
| 차동 이름 예시 | PDP0/DPN0는 요청된 대응과 불일치 | PDP0/NDP0로 수정, 사용자 Pair 확인 |
| 캐시 재사용 | 주파수 변경 시 결과 재사용 금지와 점 재사용 요구 충돌 | 물리 점 Cache와 수렴 Cache 분리 |
| 스윕 종료 | 실패/미지원만 남으면 종료 조건 불명확 | Terminal 상태와 전역 Converged 분리 |
| 정확도 표현 | 대역 전체 PASS와 샘플 검증의 차이 불명확 | 상태 5축 및 Inconclusive 기준 |
| 스윕 상세 | v0.2의 직접 기준선·오프그리드·지연 규칙 축약 | 9.6~9.7절 복원 및 기준 정량화 |
| Substrate 범위 | U26~U30 대비 본문 경계 상세 부족 | Pad-to-Pad·제외 구조·Cadence 입력 경로 |
| Port 오류 | Signal/Reference 접속 문구의 오해 가능성 | 접속과 절연 분리, 미정의 부하 Error |
| 누화 | 방향·그룹·미계산 항 정의 부족 | NEXT/FEXT 예시와 Unknown 규칙 |
| 데이터 손실 | 그룹별 격자·한글·Partial Export 정책 부족 | 완전 행렬·Sidecar·직접/보간 구분 |
| 장시간 실행 | Snapshot·Heartbeat·늦은 응답·복구 규칙 부족 | Run/IPC 계약과 자원·청크 원자성 |
| 구현 단계 | GUI 비차단이 후기 작업처럼 보임 | M0/M1 공통 요구로 전진 배치 |
| 추적 번호 | P 우선순위와 단계 중복, T 묶음/재정의 | I/M/P 번호 분리, T 개별화·구버전 대응 |
| 완료 판정 | 성능과 상세 결정이 모두 미정 | 설계 목표와 기능별 결정 시점·증거 지정 |
| 근거 | 개괄 문서로 상세 구현 보장 가능성 | 실제 확인한 자료·한계·표준/설계 구분 |

### 18.2 요구사항과 시험 연결

| 요구 ID | 본문 | 주요 시험 | 완료 단계 |
| --- | --- | --- | --- |
| U01,U02 | 8,11 | T06,T07,T43 | M2 |
| U03,U14 | 12 | T13,T39 | M0 및 릴리즈 |
| U04,U13,U22~U24 | 9,12,13.1 | T10,T11,T32,T34,T35,T46,T50,T54 | M0~M2b |
| U05,U15 | 12,14,15 | T14,T15,T40,T49 | 전체 단계 |
| U06~U08,U31~U33,U38~U40,U48 | 4,5 | T01~T03,T16~T22,T27,T51 | I1은 M1, 확대 M3~M5 |
| U09,U10 | 6 | T02,T04,T24,T52 | M1 |
| U11,U34~U37 | 7 | T05,T08,T23~T28 | M1b |
| U12,U17~U21,U25,U41,U46 | 9 | T09,T29~T31,T33,T41,T48,T53 | M2~M2a |
| U16 | 1,15 | T15,T40 | 전체 단계 |
| U26~U30 | 8.1 | T36~T38,T51 | 기하 M1a, 정량 M2c |
| U42 | 7.8 | T42 | 결합 모델 제공 시 M2 |
| U43 | 10.1 | T45,T53 | M2a |
| U44 | 9.8,11 | T32,T46,T50 | M0~M2b |
| U45 | 9.9,10.3 | T12,T47,T54 | M2b |
| U47 | 13.1,15 | T06,T10,T12,T41,T50 | 단계별 게이트 |

P01=화면 흐름(3장), P02=Import(4장), P03=Artwork/Topology(5장), P04=Stackup/Reference(6장), P05=Port/Scenario(7장), P06=Solver(8장), P07=Sweep/자원(9장), P08=결과/저장(10장), P09=모듈/IPC(11장), P10=플랫폼/보안(12장), P11=검증(13장), P12=개발/릴리즈(14~15장)이다.

### 18.3 구버전 ID 이관

v0.2 U01~U16은 같은 의미를 유지한다. U17의 직접 계산 원칙은 U41, U18은 U17/U18, U19~U21은 같은 번호, U22는 U25 및 9.7절, U23은 U22, U24는 U23, U25는 U24 및 9.9절에 대응한다. v0.2의 설계 P13~P22 상세는 9.6~9.9절과 11장으로 연결한다. 구문서 ID를 새 의미로 소급하지 않는다.

| v0.2 시험 | 현재 대응 | 의미 |
| --- | --- | --- |
| T01~T15 | T01~T15 | 기존 기본 검증 |
| T16~T19 | T29,T30,T45,T48 | 세분화·독립 진행·전체 종료 |
| T20~T22 | T31,T41,T42,T50,T53 | 공진·전체 행렬·Live 표시 |
| T23~T26 | T32,T34,T35,T50,T54 | UI·Crash·RAM·SSD |
| T27 | T33,T48 | 캐시와 재개 |
| T28 | T01~T54 중 관련 모듈 시험 | Parser/Solver/Sweep/Cache/UI 분리 회귀 |
| T29 | T14,T32,T39,T49,T54 | 플랫폼·복구·보안 |

### 18.4 최종 문서 검토 범위

v0.4 Word와 Markdown 및 v0.2 핵심 요구를 대조하고 용어·ID·수식·포트 방향·스윕 점 수·상태 전이·단계 게이트를 다시 점검했다. 문서의 모든 ‘검증 목표’와 ‘통과 조건’은 향후 구현에 대한 요구다. 실제 Parser/Solver 정확도, 3개 플랫폼 성능, 실제 PCB 대조 시험은 구현과 시험 결과가 있어야 판정할 수 있다.