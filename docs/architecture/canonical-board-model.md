# M0 Canonical Board Model 계약 (최소 구현)

기준: `docs/specification/BoardSI_Development_Spec_v1.0.md` §4/4.6/5/6/8.1/11.2/13/14.2.
관련: Issue #4, T03/T04/T21/T37/T51 **부분 검증**. 전체 M0, Importer 또는 물리 해석 완성 아님.

## 결정 및 공통 소유권

`crates/boardsi-model`은 외부 의존성 없는 Rust 1.75 workspace member다.
기존 `boardsi-numerics` API/시험, 사양서 및 toolchain 파일은 수정하지 않는다.

입력 형식 → 향후 Importer/Normalizer → **한 BoardModel** → Viewer/Topology/Port/Solver.
ODB++는 `SourceFormat` 값 중 하나이며 내부 형식이 아니다. PCB/Substrate 구분은
`BoardKind` 메타데이터만 사용한다. Bump/Ball/Wirebond 해석은 포함하지 않는다.
`BoardModel`은 Clone을 제공하지 않는다. 소비자는 같은 소유 객체의 불변 borrow 또는
`Arc<BoardModel>`을 공유한다. `validated_view()`는 그 객체의 불변 참조와 진단을 가진다.
편집은 해당 참조가 끝난 뒤 builder/owner에서 수행하고 **다시 검증**한다. 향후 버전
Snapshot/Scenario/IPC는 별도 계약이며, 복제된 원본을 각 소비자가 독립 편집하면 안 된다.

| 계약 | 타입·관계 |
| --- | --- |
| 원본 | ImportSource → SourceIdentity → 형식/버전/파일 SHA-256/Importer 버전/정렬된 조건 |
| 객체 | ObjectMeta → ObjectId + GeometryProvenance(변환 이력/정확·근사·손실·미처리/오차) |
| 좌표 | Point → CoordinateFrame → 원본 i64 XY + 유리수 metre/tick + 원점/회전/미러/층 방향 |
| 적층 | Stackup → Layer(고유 ID/order/kind/상면 z/두께/Material ID) |
| 재료 | Material → Dk/Df/도전율/유효 대역 + Missing/Assumed/Confirmed 및 자료 정보 |
| 부품 | Component → Placement; Pin → Component/Pad/Net ID |
| 도체 | Trace/Arc → Layer/Net/중심선·폭; CopperArea → 외곽 Boundary + 별도 Cutout |
| 패드·비아 | Pad → Padstack; Via → Padstack/Drill/Span/실제 접속 층/Net |
| 경계 | Outline → 외곽 Boundary + Cutout; 선분/원호는 해석적 표현 보존 |
| 연결 | ConnectivityNode → 도체/Layer/접촉 지점; Edge → 두 Node + 접촉/도금 Via 근거 |
| Import 요약 | ImportCapability의 G/N/P/S/R/M 각각 Complete/Partial/Missing/Conflicting |

## 좌표와 정밀도 정책

- 원본 좌표는 부호 있는 `i64` tick, 단위/원점은 정규화된 `i128/i128` 유리수다.
  별도 Source Frame을 허용하므로 모든 입력을 한 거친 전역 격자로 강제 변환하지 않는다.
  예: 1 nm/tick일 때 저장 범위는 약 ±9.22×10⁹ m지만, Solver `f64` 사용 범위는
  정밀도 검사 때문에 훨씬 작을 수 있다. 이 범위는 제품 PCB 크기 지원 주장이 아니다.
- `Rational`의 분모는 양수, 분자는 i128::MIN 제외; 모든 정수 연산은 checked 연산이다.
  중간 곱의 범위 초과도 오류다. 약분 전 중간값이 넘으면 수학적 최종값이 표현 가능해도
  보수적으로 거부할 수 있다. Arbitrary precision 도입은 필요한 실제 사례 확인 후 결정한다.
- `exact_m()`은 정수·유리수로 원점 이동/90° 단위 회전/미러를 수행한다.
  임의 각도는 `PendingRotation`으로 보존하며 현재 `UnsupportedTransform`을 반환한다.
  Importer가 임의 회전을 조용히 정수 반올림하거나 화면 tessellation으로 대체하면 안 된다.
- `regrid()`는 목적 격자의 정수 tick으로 **정확히 표현되는 경우만** 성공한다.
  소수 tick, 범위 초과는 `PrecisionLoss`/`CoordinateOverflow`이며 round/cast로 숨기지 않는다.
- `solver_m()`은 Solver 경계에서 SI `f64`로 변환한다. 십진 유리수의 binary64 정확 표현을
  주장하지 않는다. 정수 cast 두 번과 division의 보수적 상대 오차 한계
  `4 × f64::EPSILON × |coordinate|`가 원본 반 tick 이상이면 변환을 차단한다.
  i128 유리수 범위의 nonzero 최소값은 binary64 normal 범위 안이므로 여기서 subnormal
  근사 규칙에 의존하지 않는다. 원본 데이터는 변하지 않는다.
- `solver_points_m()`는 완전한 좌표 집합을 같이 변환하고, 서로 다른 정확한 축 좌표가
  같은 binary64 값으로 합쳐지면 거부한다. `validate()`도 모든 board-space 위치/경계/
  AreaWitness를 함께 검사한다. Padstack Custom 형상은 **pad-local frame**이며 전역
  비교에서 제외한다. 각 경계의 국소 변환은 별도로 검사한다.
- 수학적으로 다른 Frame 좌표도 정확한 유리수 board 좌표가 같으면 같은 위치다.
  근접점 snap이나 접촉 tolerance 기반 자동 병합은 없다. 완전한 최소 절연 거리와
  polygon edge 사이 간격 판정은 후속 Geometry Engine 책임이다.

## ID와 Provenance

ObjectId는 `(전체 SourceIdentity, ObjectKind, source_object, part)` 튜플이며
잘린 해시나 process-random hash 값이 아니다. SourceIdentity를 `Arc`로 공유하여
파일 목록을 모든 객체 참조마다 복제하지 않으며 동등성은 내용 기준이다.
원본 파일 목록/Import 조건은 정렬·키 중복·상대경로를 검사한다.
동일 입력·동일 조건·동일 Importer 버전·동일 deterministic part이면 ID가 재현된다.
다른 형식/Step/Instance/파일 Hash/Importer 버전이면 같은 원본 객체 문자열도 별개다.
전 모델에서 ID 중복, 잘못된 kind, 미등록 Source를 검출한다.

파일 SHA-256 계산/원본 bytes 검증은 향후 Importer 책임이다. 이 모델은 공급된 32 byte
SHA-256 값을 보존할 뿐 검증 완료를 주장하지 않는다. 합성 시험의 `[7;32]`는 명시적
테스트 토큰이며 실제 CAD 파일의 검증된 digest가 아니다. Source-object 타입과
변환별 operation/오차/상태를 보존한다. Exact에 nonzero 오차를 주거나 근사/손실에
오차 한계를 누락하면 오류다. 비정확 변환은 진단에 남고 물리 Validated가 되지 않는다.

## 연결과 절연

- 그래프는 주어진 명시적 Edge만 사용한다. 같은 XY/Net 이름, 같은 Copper Layer,
  같은 Via Span만으로 Edge를 만들지 않는다. Trace 내부 노드 사이의 Edge도 후속
  Topology가 명시적으로 생성해야 하며 현재 extractor는 없다.
- Node는 실제 도체 ID와 Layer 및 접촉 지점(Trace vertex/Arc endpoint/Pad·Via centre/
  AreaWitness)을 참조한다. Cutout은 도체 타입이 아니므로 Node owner가 될 수 없다.
  AntiPad/Clearance-only entry는 conductive 접촉이 될 수 없다.
- SameLayerContact는 같은 Layer, 정확히 같은 board 위치, 동일한 **Net ID**, 비어 있지
  않은 근거를 요구한다. 다른 GND/Power/Floating Net 또는 두 unknown Net은 연결하지 않는다.
  동일 Floating Net의 명시적 물리 접속은 허용하지만 Reference로 자동 지정하지 않는다.
- ViaBarrel은 같은 plated Via의 두 ViaCentre Node만 연결한다. Span 양 끝은 Copper Layer,
  순서는 first < last, Drill span과 일치한다. connected_layers는 중복 없이 Span 안이며
  conductive pad entry가 있어야 한다. Padstack의 모든 entry도 Span 안이어야 한다.
  Through/Blind/Buried/Microvia의 층 위치 계약을 별도 검사한다.
- AreaWitness와 접촉 근거는 Importer/Topology의 **선언**이다. 이 모듈은 실제 Polygon
  containment/교차/Anti-pad clearance/thermal spoke를 증명하지 않는다. 잘못된 선언을
  문자열만으로 물리적 진실로 승격하지 않는다. 후속 Geometry Engine은 Edge별
  근거를 재검증하고 Netlist와 실제 Copper 연결을 비교해야 한다.

## 적층·물성·진단

z는 top-to-bottom 증가, 각 Layer의 **상면**이다. order는 고유하며 인접 Layer의
`z + thickness == next.z`를 정확히 검사한다. 누락된 z/두께/재료는 Missing 진단;
잘못된 양수 두께·중복 order·공백/겹침은 Error다. 빈 모델도 Missing으로 표시한다.
Layer 변경에 따른 z 자동 재계산은 이번 범위에 없다.

Dk: finite > 0, Df: finite ≥ 0, conductivity: finite > 0.
물성은 Missing/Assumed/Confirmed를 타입으로 구분하며 자동 FR-4 기본값은 없다.
자료명/Revision/측정법/기준 주파수/온도를 저장하고 유효 대역은 finite·순서 검사한다.
`validate_materials_at(f)`는 Layer 종류별 필수 물성과 유효 대역을 검사하며 대역 밖은
`OutsideMaterialBand`다. Assumed는 명시적으로 남는다. RF Df로 DC 누설을 생성하지 않는다.
이 검사는 **물성 준비도만**이며 Solver 실행 승인, 인과성 검증, Reference 검증이 아니다.

`validate()` → ValidationReport(Error/Missing/Assumption).
`validated_view()`는 Error가 없어야 생성된다. Missing/Assumed는 report에 계속 남으므로
이 view를 "SI Run 가능" 또는 "물리 Validated" 상태로 취급하면 안 된다.
ImportCapability의 Complete는 검증으로 자동 설정되지 않으며 시험에서도 모두 Missing이다.

## 후속 아키텍처 주의점 (3개)

1. **임의 회전/Pad Custom 인스턴스화**: 원본과 pad-local geometry를 보존했지만 임의 회전의
   전기적 근사·오차 정책과 pad-local→board 변환은 미구현이다. 정수 변환 실패 시 silent snap 금지.
2. **물리 접촉 근거**: 현재 통과는 구조 계약이다. Polygon Boolean/containment/thermal/
   최소 간격 판정과 Netlist 대조를 붙여야 실제 연결 신뢰성을 판정할 수 있다.
3. **장기 저장/규모**: ID 튜플은 인메모리 재현 가능하지만 디스크 serialization schema,
   version migration, 객체 인덱스/큰 보드 성능/변경 revision은 아직 확정하지 않았다.
   Importer 버전 변경은 ID가 바뀌므로 Port 재연결은 ID 외 Geometry/Net 재검증도 필요하다.

근거: 위 사양서(제품 정책), 본 모듈의 API/검증 코드(구현 사실), 합성 시험(제한된 증거).
외부 기술 문서에서 새로운 물리 모델을 도입하지 않았다.
