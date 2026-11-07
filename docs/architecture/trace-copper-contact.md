# 설계 계약: 임의 Trace–Via–SMD Pad의 구리 접촉

> **구현 범위:** 2026-11-07 KST \`D_054\`/\`D_055\`의 **PADS V9.4 MILS 지원 SOURCE 형상에 한정된 2차원 기하 접촉 후보**. 이 문서는 전기적 Open/Short, 도금 불량, Reference Plane/Return Path, Port 추출, 실제 RLGC/S 해석이 검증됐다는 증거가 아닙니다.
>
> 코드: [범용 직선·원호](../../crates/boardsi-import/src/trace_contact.rs) · [Trace–Via/Pad](../../crates/boardsi-import/src/trace_attachment.rs) · [PADS 소스 파서](../../crates/boardsi-import/src/route_preview.rs) · [Part/Pad 파서](../../crates/boardsi-import/src/part_preview.rs).
> 시험 근거: [D_054–D_055 검증 원장](../validation/d054-d055-general-trace-contact-qualification.md).

## 1. 문제 정의

PCB Trace는 Via나 Pad의 중심을 지나지 않아도 실제 구리 영역과 접촉할 수 있습니다. 따라서 중심 좌표가 일치하는지, Trace가 0°/45°인지, Pad의 어느 꼭짓점에 끝나는지를 생산 코드에서 **나열하지 않습니다**. 지원되는 **실제 도체 형상** 사이의 양의 면적 겹침을 검사합니다.

| 객체 | 사용되는 SOURCE 기하 | 판정에서 고려하는 요소 |
| :--- | :--- | :--- |
| Trace | 임의 방향의 선분 또는 방향성 CW/CCW 원호 | 좌표·회전 방향·유한 선폭·Layer·Net·SOURCE 행 |
| Through Via | 중심점, 구리 외경, 드릴 직경, 명시적 도체 Layer/Span | **외부 도금 패드에서 드릴 구멍을 제외한 환형 도체** |
| 원형 SMD Pad | 중심, 직경, Layer, Component.Pin | 원형 **전체 도체 면적** |
| 정사각형 SMD Pad | 중심, 가로·세로 치수, Layer, Component.Pin | 네 변과 네 모서리를 포함한 **전체 도체 면적** |
| 직사각형 SMD Pad | 중심, 가로·세로 치수, Layer, Component.Pin | 종횡비와 변·모서리·내부, 제한된 원본 회전/미러 |

좌표는 이 Import 서브셋에서 **0.01 mil SOURCE Tick**, 즉 **1 Tick = 0.000254 mm**입니다. 원본 입력은 정수로 보존하고, 기하 거리 계산은 검증된 범위의 \`f64\`로 수행하며, 반 Tick 경계가 필요한 직사각형은 **2배 정수 좌표계**를 사용합니다. 이것은 무한 정밀도를 의미하지 않습니다.

## 2. 공통 수학적 판정

\`C\`를 Trace의 *중심선*(선분 또는 원호), \`r_t\`를 Trace 반폭이라 하면, 반원 끝단을 포함하는 유한 선폭 도체는 \`C\`를 반경 \`r_t\`만큼 확장한 영역입니다.

### 2.1 원형 SMD Pad

Pad 중심 \`P\`와 반경 \`r_p\`에 대해

\[
d_{\min}(C,P) < r_t+r_p
\]

이면 양의 면적 구리 중첩 후보입니다. **중심선이나 끝점이 Pad 중심을 지나야 할 필요는 없습니다.** 거리가 경계값과 구분되지 않는 경우에는 \`BoundaryUncertain\`으로 처리합니다.

### 2.2 사각형 SMD Pad

Pad의 채워진 직사각형 영역을 \`R\`이라 하면

\[
d_{\min}(C,R) < r_t
\]

이면 양의 면적 중첩 후보입니다. Trace 중심선이 사각형 내부에 있는 경우 거리는 0입니다. 경계에 접근하는 원호/직선의 거리도 사각형의 **모든 변을 하나의 공통 알고리즘**으로 계산합니다. 따라서 중심·변·꼭짓점별 연결 분기를 따로 작성할 필요가 없습니다.

### 2.3 Via의 도금 환형 Pad

Via의 외부 구리 반경 \`R_o\`, 드릴 반경 \`R_d\`를 사용합니다. Via 패드 영역은 바깥 원판에서 **드릴 구멍 영역을 제외한 도체**입니다.

\[
d_{\min}(C,P) < R_o+r_t
\quad\land\quad
d_{\max}(C,P)+r_t > R_d
\]

두 조건을 모두 충족해야 환형 Copper와 겹칠 **후보**로 인정합니다. 두 번째 조건은 중심의 드릴 구멍에만 위치하는 짧은 Trace를 잘못 연결시키지 않기 위한 것입니다. 또한 SOURCE에서 그 Layer의 실제 도체 패드가 확인되어야 하며, **Via가 그 레이어를 관통한다는 사실만으로 Copper 접촉을 추론하지 않습니다.**

### 2.4 수치 경계와 Fail-Closed

\`PositiveArea\`(엄격한 양의 면적 겹침), \`Separated\`(충분한 간격), \`BoundaryUncertain\`(접선 또는 경계 근처)의 세 상태를 구분합니다. D_054/D_055 구현은 동일한 \`BOUNDARY_GUARD_TICKS\` 보호 구간을 거리 계산과 빠른 Bounding Box 검사에 적용합니다.

- **명확한 양의 면적:** SOURCE-local **후보**로 기록.
- **확실한 간격 또는 다른 Layer:** 후보에 기록하지 않음.
- **접선 부근 수치 불확실성:** 전체 국소 후보 보고를 중단하고 오류 처리. 자동 연결 PASS 금지.
- **유효하지 않은 Padstack, 반지름, 드릴, 좌표 또는 미지원 SOURCE:** 지원됐다고 간주하거나 중심점 모델로 대체 금지.
- **대량 Pair 예산 초과:** 성공을 조작하지 않고 예산 초과 오류.

거리 알고리즘의 유효 범위 밖에 있는 도체 형상에는 이 공식을 무비판적으로 적용하지 않습니다.

## 3. Net·Layer·Provenance의 별도 검증

기하학적 구리 중첩은 *전기 연결의 필요 증거 중 하나*일 뿐, 그 자체로 충분하지 않습니다.

1. **Source Binding:** 원본 SHA-256, 객체 식별, SOURCE 행/Layer 정보 보존.
2. **Layer Qualification:** Via의 외측/내측 실제 도체 Pad와 Trace의 동일 층 접촉만 고려. SMD Pad도 실제 배치 층으로 제한.
3. **Declared Net:** 원본 ROUTE Pin 선언으로 연결된 Pad의 소유 Net과 Trace의 Net을 비교.
4. **Conflict:** 서로 다른 Net이 겹치면 정상 연결이 아닌 **충돌 후보**. Pad 선언 Net이 확인되지 않으면 \`PadNetUnverified\` 유지.
5. **Canonical Integration:** 물리·도체 범위·Net 소유권과 그래프 근거를 추가 검증한 뒤에만 전기적 연결 Edge로 승격할 수 있음. **이 단계는 아직 별도 미완성 작업입니다.**

## 4. 예시 시나리오와 기대 동작

| 시나리오 | 기대 결과 | 이유 |
| :--- | :--- | :--- |
| Trace가 Via 외주 쪽을 비스듬히 통과 | 후보 가능 | 도금 패드 면적과 양의 중첩 |
| Trace가 Via 중심을 지나지만 패드 폭까지 도달하지 않음 | 비접촉 | 드릴 구멍 내부만 겹침 |
| Trace가 사각 Pad 꼭짓점에 비중심으로 들어감 | 후보 가능 | 실제 유한 선폭과 직사각형 영역의 중첩 |
| Trace가 Pad 옆을 평행하게 스침 | 결과는 실제 간격에 따라 결정 | 각도·이름이 아닌 최소 거리 기준 |
| Trace가 접선으로만 닿음 | 불확실 / Fail-Closed | 양의 구리 면적이 확정되지 않음 |
| Pad와 Trace가 다른 Layer | 비접촉 | 투영된 화면의 겹침은 층간 도통을 의미하지 않음 |
| 다른 Net의 도체가 기하학적으로 겹침 | 충돌 후보 | 전기적 Short를 확인한 것은 아님 |

이 표의 사례는 **시험 범주 예시**이고, 모든 실제 PADS/제조 형상을 독립 인증했다는 선언이 아닙니다.

## 5. 아직 지원/인수되지 않은 범위

- 임의 모양의 Custom Pad/슬롯, 타원·Obround, 임의 각도 회전된 Part 형상, 모든 각도의 Local Padstack, Blend/Spline/Bezier 원본 등 **Importer가 지원하지 않는 형상**.
- Blind/Buried/Microvia, 복잡한 Anti-pad·Layer별 Drill/Copper·Thermal/Spoke, Plane/Pour 등 전체 금속 도통 형상.
- 전기적 Open/Short·전체 보드 다중 요소 Connectivity, 실제 Qt GUI 연결 그래프 반영, 임의 대규모 보드의 시간·메모리 성능 보장.
- 물리적 제조·접합 품질, Reference/Return Path, 실제 RLGC/임피던스/S-Parameter 정확도, DC–30 GHz 정식 수락 시험.

**다음 개발 단계:** 위치별 케이스 나열이 아니라, SOURCE 소유 도체 형상의 **일반화된 객체/층/Net 계약**을 Canonical 전기 연결 그래프에 연결하고, 공개 기준 데이터 및 실제 허용된 보드 대조로 검증 범위를 확장해야 합니다. 상세 미완료 범위는 [체크리스트](../development-checklist.md)와 [사양서 §5, §7, §8, §13](../specification/BoardSI_Development_Spec_v1.0.md)에서 관리합니다.
