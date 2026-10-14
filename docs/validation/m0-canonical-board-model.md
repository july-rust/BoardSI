# M0 Canonical Board Model 검증 보고서

작업 일자: 2026-10-14 (KST). 관련 Issue #4는 **부분 진행**, 닫지 않는다.
기준 main: `098c3b069b2ef7845dddbdd5ae3d0ebcce917109` (사용자 승인으로 PR #27 squash merge).
사양서 v1.0 및 `boardsi-numerics` 소스/API/fixture/시험은 변경하지 않았다.

## 실행 결과

로컬 Linux x86_64, Rust 1.75.0 (82e1608df), 실제 실행 기준:

| 검사 | 결과 |
| --- | --- |
| cargo fmt --all -- --check | PASS |
| cargo clippy --workspace --all-targets --locked --offline -- -D warnings | PASS |
| cargo test --workspace --locked --offline -- --nocapture | PASS: model 28 / numerics 15, FAIL 0 |
| cargo test --workspace --release --locked --offline -- --nocapture | PASS: model 28 / numerics 15, FAIL 0 |
| model Native executable identity (x86_64/linux) | PASS |
| 기존 수치 기준점 180개 | PASS; 최대 복소 S 절대오차 4.0953603625310199e-10 (유지) |
| git diff --check | PASS |
| spec/numerics/toolchain 변경 없음 비교 | PASS |

model 28 = Rational unit test 1 + 합성 통합시험 27. 자동시험 코드가 공개 fixture이며
실제 회사 PCB, CAD 파일, 비밀 Net/소재/Touchstone을 사용하지 않는다.
초기 로컬 compile/lint에서 enum 크기·Arc 이름 충돌·example output 이름 중복을
발견해 SourceIdentity 공유 및 test alias/고유 example 이름으로 수정했다.
전체 재검토에서 서로 다른 Frame의 sub-tick 원점 차이가 같은 binary64에 합쳐지는
경우를 발견해 집합 단위 변환 검사와 회귀시험을 추가했다. 해결 전 결과는 PASS 근거로 쓰지 않는다.

## 필수 사례와 추적

| 요구 사례 | 주요 자동시험 | 결과/오류 검출 | 인수 항목 범위 |
| --- | --- | --- | --- |
| 단위·원점 | different_units_and_origins_exact_same_si_position | 정확 유리수/SI 좌표 동일, exact regrid 성공 | T21/T51 부분 |
| 미세 간격 | nanometre_gap_preserved_without_regridding | 1 nm 간격 유지; 거친 regrid는 PrecisionLoss | T51 부분 |
| 큰 원점·다른 Frame | origin_precision_loss_is_explicit, cross_frame_sub_tick_gap_cannot_collapse | 개인 tick 한계 및 집합 변환 충돌 모두 명시적 차단 | T21/T51 부분 |
| Layer 교차 | cross_layer_same_xy_never_connects_implicitly | Edge 없으면 분리; cross-layer 접촉 선언은 InvalidConnection | T03 부분 |
| Via Span | via_joins_only_explicit_span_layers, omitted_via_layer_not_connected | 실제 지정층만 연결, Span 밖/생략층 접촉 차단 | T03/T37 부분 |
| Blind/Buried/Microvia | blind_buried_microvia_contracts | 합법 유형 PASS, 잘못된 유형/층 범위 거부 | T37 부분 |
| Copper Island | same_name_islands_and_separate_traces_remain_disconnected | 동일 Net ID라도 암묵 접속 없음; 다른 위치 edge 거부 | T03/T04 부분 |
| Anti-pad/Cutout/도금 | antipad_and_cutout_are_not_conductors, unplated_drill_and_antipad_via_contacts_rejected | 절연 entry와 Cutout owner, nonplated 접속 거부 | T37 부분 |
| GND/Power/Floating/unknown | ground_power_floating_not_shorted | 서로 다른 Net 및 unknown-unknown edge 거부 | T04 부분 |
| 적층·물성 | missing_properties_reported_without_fr4_defaults, zero_negative_thickness_duplicate_order_bad_z_rejected, invalid_material_scalars_and_band_rejected | Missing 분리; 0/음수 두께, z 불일치, 비정상 Dk/Df/도전율/대역 검출 | T03/T04 부분 |
| Assumed/대역 밖 | assumed_values_and_out_of_band_are_explicit | Assumed 유지; 대역 밖 OutsideMaterialBand | T04 부분 |
| ID 재현성 | stable_ids_and_format_step_condition_separation | 재생성 ID 동일, 형식/Step/Importer/Hash/part 차이 분리 | T21 부분 |
| 손상 참조·ID·좌표 | duplicate_id_wrong_kind_unknown_source_rejected, invalid_layer_frame_padstack_node_references_rejected, coordinate_overflow_rejected | DuplicateId/InvalidIdentity/InvalidReference/CoordinateOverflow | T21/T51 부분 |
| PCB/Substrate | pcb_and_substrate_share_identical_schema | kind만 다르고 Layer/Via/Graph 계약 동일 | T21/T51 부분 |
| 추가 구조 | analytic_arcs_and_closed_boundaries, component_pin_and_pad_relations, provenance_quality_and_sorted_source_contract | 원호 radius/경계 closure/Pin Net/변환오차/원본 정렬 검사 | 제한된 데이터 계약 |

## Native CI

기존 `.github/workflows/numerical-core.yml`에 model 경로와 identity example만 추가했다.
기존 fmt/clippy 및 Linux/Windows x64/Windows ARM64/macOS ARM64 workspace debug/release
시험이 두 crate를 함께 검사한다. 별도 중복 Workflow/수동 dispatch/산출물 업로드는 없다.

새 코드의 실제 Native 결과는 PR 생성 후 같은 source commit의 run/job log를 확인한다.
아래는 아직 새 코드의 CI가 실행되기 전 상태이며, PR #27의 성공 이력을 재사용하지 않는다.

| 목표 플랫폼 | 현재 실제 실행 결과 |
| --- | --- |
| Windows x64 (windows-2022) | Not Run — PR CI 결과 확인 전 |
| Windows ARM64 (windows-11-arm) | Not Run — PR CI 결과 확인 전 |
| macOS Apple Silicon ARM64 (macos-14) | Not Run — PR CI 결과 확인 전 |

## Not Run / 미구현

- 실제 ASC/ODB++/IPC-2581/KiCad/Gerber 파서 및 파일 Hash 계산/bytes 대조.
- 실제 Polygon 교차·Boolean·containment·최소 간격·Anti-pad clearance/thermal 해석.
- 완전한 Net Connectivity Extraction 및 다른 CAD 형식 사이의 설계 동등성/대응표 검증.
- 실제 CAD 원본/EM/측정 검증, Port/Reference/Differential, RLGC 형상 추출/Solver 물리 확장.
- GUI/Renderer, 임의 회전과 Pad Custom의 board-space 인스턴스화, Snapshot/IPC/Cache/Sweep.
- 장기 저장 schema/version migration/대규모 성능/패키지/릴리즈.

T03/T04/T21/T37/T51 전체 PASS를 주장하지 않는다. 이것은 합성 데이터에 대한
**구조·좌표·명시적 연결 계약의 부분 PASS**다. AreaWitness/evidence는 선언이고
물리 판정 증거가 아니다. 재료 준비도나 구조 view는 SI Run 승인/물리 Validated가 아니다.

재현: 위 명령들을 저장소 루트에서 실행한다. 설계 근거/한계:
[Canonical 계약](../architecture/canonical-board-model.md),
[기준 사양](../specification/BoardSI_Development_Spec_v1.0.md),
[합성 시험](../../crates/boardsi-model/tests/integrity.rs).
