# F_002 — BSIG v1 Geometry Binary Byte Layout

> 상위 불변 계약 [F_001](f001-bsi-format-contract.md) · [SQLite 물리 스키마](f002-bsi-sqlite-schema.md) · 테스트 [test_f002_schema.py](../../tools/validation/test_f002_schema.py) · 2026-11-15 KST.
>
> 범위: Rust Writer/Reader를 위한 **규범적 바이너리 설계**와 독립 Python 참조 Vector 일부. 전 Geometry Encoder의 제품 구현/실 CAD Lossless Round-trip은 F_003/F_005 전까지 NOT_RUN.

## 1. 고정 Envelope (offset 0~51)

| Offset | Size | Encoding | 의미 |
| ---: | ---: | --- | --- |
| 0 | 4 | ASCII `BSIG` | Magic |
| 4 | 2 | u16LE | `codec_version=1` |
| 6 | 2 | u16LE | `geometry_kind` (아래 표) |
| 8 | 4 | u32LE | `flags=0` (v1 필수) |
| 12 | 8 | u64LE | **비압축** payload 길이 |
| 20 | 32 | SHA-256 | 비압축 payload의 Digest |
| 52 | N | byte[] | Payload; v1은 **압축하지 않음** |

BLOB의 byte 배열은 `52+uncompressed_length`와 정확히 같아야 한다. 지나치게 큰 데이터를 먼저 할당하지 말고 선언 길이·파일 정책을 검사한다. Unknown codec/kind/flags, 길이 오차, SHA 불일치, 잔여 bytes ⇒ `BSI_INVALID` 또는 `BSI_UNSUPPORTED_REQUIRED`로 거부한다. SHA-256은 **변조 방지 서명이나 출처 진위 보증이 아니다**.

## 2. 공통 Payload v1 / TLV

모든 `geometry_kind`에 동일한 Record Prefix를 사용한다.

```text
u16LE record_version = 1
u16LE field_count = 1..64
repeat field_count:
    u16LE field_id      (strictly ascending; no duplicates)
    u16LE type_id
    u32LE byte_length
    byte[byte_length]  value
No padding, no trailing bytes.
```

**기본 타입(숫자는 v1에서 재사용·재해석 금지)**

| Type | 번호 | Value payload |
| --- | ---: | --- |
| Bool | 1 | u8, 정확히 0/1 |
| U8 | 2 | u8 |
| U16 | 3 | u16LE |
| U32 | 4 | u32LE |
| Rational128 | 5 | i128LE numerator (16) + i128LE denominator (16), 총 32 |
| Point | 6 | u32LE frame_ref_index + i64LE x_ticks + i64LE y_ticks, 총 20 |
| PointList | 7 | u32LE count + 정확히 count×Point (Trace는 count>=2) |
| Boundary | 8 | u32LE count + count×BoundarySegment |
| RefList | 9 | u32LE count + count×u32LE ref_index |
| Orientation | 10 | 아래 4-byte Prefix와 필요 시 8-byte f64LE radians |
| PadstackEntries | 11 | 아래 Padstack Layer Entry 반복 |
| U32Pair | 12 | u32LE first_ref_index + u32LE last_ref_index |

`i128LE`: 정확히 16 bytes, signed two's complement little-endian. Denominator는 양수, numerator는 `i128::MIN` 금지, `gcd(abs(num),den)=1`. NULL/Missing은 0값으로 치환하지 않고 관계형 `*_status`에서 표시한다. `Point` 좌표는 단위 변환/반올림하지 않은 정확한 원본 i64 tick이다.

**BoundarySegment (연속 packed bytes):**

- `tag=1` Line: tag(u8) + Point(start,20) + Point(end,20). 총 41 bytes.
- `tag=2` Arc: tag(u8) + Point(centre,20) + Point(start,20) + Point(end,20) + clockwise Bool(u8). 총 62 bytes.
- count는 segment 개수이며, 각 segment의 연속성/닫힘 및 frame 참조는 Reader가 검증한다. 곡선은 tessellation으로 대체하지 않는다.
- Copper Area의 Hole/Cutout은 별개 정식 Cutout 객체로 저장하고 **RefList**로 순서와 참조를 보존한다. 독립 원본 Contour가 표현 불가능하면 필수 확장·오류로 다룬다.

**Orientation:** `u8 mode`(0 Orthogonal, 1 PendingRotation), `u8 mirror_x`(0/1), `u8 quarter_turn`(Orthogonal=0..3, Pending=255), `u8 reserved=0`; Pending에서만 `f64LE radians` 8 bytes 추가. NaN/Infinity 거부. 임의 회전은 Pending 상태를 유지하며 직각으로 근사하지 않는다.

**RefIndex:** BLOB의 `frame_ref_index`/span/ref 필드는 `geometry_refs(geometry_id, ref_index, target_object_id, ref_role)`의 연속 `0..N-1` 값이다. Writer는 **전체 SourceIdentity + ObjectId 의미 튜플** 기준으로 참조 사전을 결정적 정렬한다. 참조 번호는 내부 로컬 index이지 ObjectId가 아니다. Reader는 예상 Kind, 존재 및 Source Provenance, 중복·누락 참조를 검증한다.

## 3. Geometry 종류별 필수 v1 필드

| Kind | ID | Required TLVs (field_id:type) | 관계형 참조 / 추가 규칙 |
| --- | ---: | --- | --- |
| Trace | 1 | `1:PointList, 2:Rational128(width_m)` | `geometry.layer_id`, 선택 Net; >=2 points |
| Arc | 2 | `1:Point(centre),2:Point(start),3:Point(end),4:Bool(clockwise),5:Rational128(width_m)` | Layer 및 optional Net, analytic arc |
| Area | 3 | `1:Boundary(outer),2:RefList(cutouts)` | Layer, optional Net; 현재 Canonical은 **positive copper**. negative-only 입력은 확장 지원 전 거부 |
| Cutout | 4 | `1:Boundary` | 외곽/면적 진위는 Geometry Validator |
| Padstack | 5 | `1:PadstackEntries` | Layer/role/shape별 데이터 보존 |
| Pad | 6 | `1:Point(position),2:Orientation` | `geometry.padstack_id`, optional Net |
| Via | 7 | `1:Point(position),2:U8(via_kind),3:U32Pair(span),4:RefList(connected_layers)` | Padstack, Drill (role Drill ref), Net, Copper span |
| Drill | 8 | `1:Rational128(diameter_m),2:Bool(plated),3:U32Pair(span)` | 제품 기본 Drill은 원형. Slot/Backdrill 등은 Source-only 별도 required Feature로 구분 |
| Outline | 9 | `1:Boundary(outer),2:RefList(cutouts)` | Cutout의 객체/근거를 유지 |

추가 필드는 규격에 명시되지 않은 v1 Reader가 조용히 무시할 수 없다. 미지원 필드가 필요한 데이터 의미를 담으면 `minimum_reader`/Required Feature를 상향하여 **명시적으로 거부**한다. v1 정식 파일의 요구/타입/필드 의미를 추후 변경하지 않는다.

### PadstackEntries (type 11)

```text
u32LE entry_count
for each entry (canonical layer order, stable source evidence):
    u32LE layer_ref_index
    u8 role: 0=Conductive, 1=AntiPad, 2=Clearance
    u8 shape: 0=Circle,1=Rectangle,2=Annulus,3=Thermal,4=Custom
    u16LE reserved=0
    u32LE shape_data_length
    byte[] shape_data
```

| Shape | shape_data (순서 고정) |
| --- | --- |
| Circle | `Rational128(diameter)` (32 B) |
| Rectangle | `Rational128(size_x)+Rational128(size_y)` (64 B) |
| Annulus | `Rational128(outer_diameter)+Rational128(inner_diameter)` (64 B) |
| Thermal | `Bool(round)+Rational128(centre_size)+Rational128(outer_size)+Rational128(spoke_width)+u16LE first_spoke_degrees+u32LE spoke_count` (103 B) |
| Custom | `Boundary` (pad-local Source Frame 참조), fixed size 아님 |

Role/Shape는 기존 Rust `PadRole`/`PadShape` 정의와 동형이다. Thermal의 Outer는 Relief envelope이지 전체 채워진 Copper가 아니다. AntiPad/Clearance를 접점으로 승격하지 않는다.

### Via 및 Drill

- ViaKind u8: Through=0, Blind=1, Buried=2, Microvia=3.
- Span은 `(first_layer_ref_index,last_layer_ref_index)`; 둘 다 Copper, first<last, 실제 Stackup 순서와 일치해야 한다.
- connected_layers는 실제 선언된 도체 접점 layer 목록이며, **Span만으로 접촉을 생성하지 않는다**.
- `plated`는 기존 `Drill.plated`의 Source 모델 bool이며 **실제 제조 barrel 품질/전기 접속 증명과 다르다**. 제조 근거 검증 없이는 VERIFIED 불가.
- 별도 `FabricationHole`의 Slot/Backdrill/MicroviaTransitions를 기본 Drill로 축소하지 않는다. Native v1이 이를 손실 없이 수용하지 못하는 경우 Required Extension을 통해 거부한다.

## 4. 경계값, 보안 및 정상 Vector

**Python 참조 테스트:** [test_f002_schema.py](../../tools/validation/test_f002_schema.py)는 Trace와 Arc의 실제 BSIG Envelope + TLV bytes를 생성·검증하며 SHA, length, flags, codec, field order, i64 한계, i128 감소분수, invalid type을 시험한다. 이는 **F_002 검증 Vector**이지 Qt/Rust Writer의 round-trip 증명이 아니다.

Test vector (분리 검사 가능한 52B prefix):

```text
magic                  42 53 49 47
codec_version          01 00
geometry_kind (Trace)  01 00
flags                  00 00 00 00
uncompressed_length    64-bit LE, actual payload bytes
payload_sha256         SHA-256(payload), exact 32 bytes
payload                01 00 [field_count: 2 in u16LE] ...
```

파일 크기 최대 512 MiB, Payload 64 MiB, TLV count 64, 객체 1M 등 [SQLite 상세 규격](f002-bsi-sqlite-schema.md)의 초기 정책 상한을 따른다. 필드 수/길이/contour count의 곱셈은 반드시 checked arithmetic을 사용한다. 미지원 required field/extension/codec은 fail-closed. Buffer Allocation 전에 먼저 header/length를 검증한다.

## 5. 합격과 미합격 경계

**F_002**: Byte Layout 계약, SQLite DDL, SQL/참조 Vector 테스트를 실행 가능한 근거로 마무리한다.

**F_003**: Rust Production Encoder/Decoder, 전체 타입별 정확한 round-trip, SQL↔BLOB 상호 검증, 이종 OS Save/Load, atomic 저장과 악성 입력 차단. **F_004**: 첫 정식 Golden/migration·미래 확장 회귀. **F_005**: 실제 CAD→Canonical→.bsi→Canonical 동등성.

[F_001](f001-bsi-format-contract.md) · [Rust Canonical Model](canonical-board-model.md) · [SQLite File Format](https://www.sqlite.org/fileformat2.html)
