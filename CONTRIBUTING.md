# Contributing · BoardSI 개발·검증·문서 기여 지침

> 시작점: [전체 문서](docs/README.md) · [개발 환경과 실행](docs/getting-started.md) · [문서/증거 표준](docs/documentation-standards.md) · [사양서 v1.0](docs/specification/BoardSI_Development_Spec_v1.0.md) · [보안](SECURITY.md).
>
> BoardSI는 **pre-release**입니다. 공개 상태가 소스 재사용/재배포/배포판 허가를 뜻하지 않습니다. 공식 라이선스는 아직 선정되지 않았으므로, 외부 코드·데이터를 제출할 때는 권리와 기여 조건을 먼저 확인하십시오.

BoardSI is pre-release with M0 tested prototypes and restricted M2 FEM research coupons; no qualified product PCB Solver. Use short-lived branches and PRs, trace changes to requirement IDs (Uxx) and acceptance test IDs (Txx), and record reproducible validation. Mark a model or importer Validated only after independent evidence. Avoid proprietary PCB data, confidential results, private paths and unreviewed sample licenses in commits, logs, Actions and public releases. Technical units follow SI conventions. Require actual three-platform native build/runtime checks before any public binary release.


Before further original PH-08 physical validation, consult the [2026-10-18 project audit](docs/audits/2026-10-18-pre-ph08-project-audit.md). Historical HMM Gate Q FAIL remains FAIL regardless of synthetic CI success; maintain protected GitHub Verified Squash-only with 17 Strict required checks. The existing option B SVG is **DESIGN_APPROVED** by the owner (2026-10-18), although its historical reconstruction origin remains disclosed; original earlier PNG byte identity and platform release packaging are NOT VERIFIED.

## Documentation-only pull requests

Keep all A/B/C/D progress, first-check dates, dated evidence, and U/T acceptance status in the single canonical [development checklist](docs/development-checklist.md). Do not maintain a second independently edited checklist.

The protected `docs` check validates planning and traceability documents on **one Linux runner**. For pull requests changing only `docs/**/*.md`, `docs/**/*.mdx`, `docs/**/*.rst`, `README.md`, `CONTRIBUTING.md`, or `SECURITY.md`, required Rust/Qt/FEM status-check **jobs remain declared but are skipped before any Windows/macOS/native runner allocation**. A lightweight Ubuntu classification job runs in each required workflow; code, workflow, fixture, or metadata changes still run full platform validation. This is not a product release or an OS-specific documentation build. Every other change, including code, workflow files, test fixtures, and JSON metadata, keeps full required validation; comparison or fetch errors fail closed to full validation.

Do not add workflow-level `paths-ignore` to a required check: GitHub may leave it pending and block a protected merge. The [shared documentation-only classifier](.github/actions/classify-docs-only/action.yml) instead uses small Ubuntu classification jobs followed by **whole-job conditional skips before native runner allocation**. The current seven separate required workflows therefore still incur light classifier overhead; consolidation into one validation workflow is a future optimization, not grounds to relax protected checks. A genuine platform binary release still requires native runtime, packaging, licensing, and acceptance evidence on its target platforms.

Code features should include dated checklist updates in the **same code PR**. Avoid separate documentation PRs for the same existing code evidence. Group genuine Markdown-only corrections into one PR; do not rerun platform qualification solely to rewrite evidence.


## 코드 기여: 기능의 크기보다 불변성과 근거

1. **문제와 종료 조건을 먼저 고정합니다.** Issue/PR에 해당 사양 Uxx/Txx, 영향을 받는 공개 Fixture, 예상/실제 결과, 보존할 기존 API를 기록합니다.
2. **재사용 가능한 일반 알고리즘을 우선합니다.** Trace 각도(0°·45°·임의), Via 중심/비중심, SMD Pad 변/모서리별 별도 프로덕션 브랜치를 무한 증식시키지 않습니다. 지원되는 전체 형상에 대한 공통 수학적 접촉 판정과 경계/변환 불변성 시험을 사용합니다. [공통 알고리즘](docs/architecture/trace-copper-contact.md).
3. **파서의 지원 범위를 명시합니다.** PADS V9.4 MILS의 제한 서브셋은 모든 PADS 원본을 대체하지 않습니다. Source 해시·Net/Layer·드릴·Padstack 미지원 여부를 보존하고, 추론만으로 실제 전기적 도통을 확정하지 않습니다.
4. **독립 시험의 범주를 구분합니다.** Rust unit/CI, Qt 네이티브 GUI 회귀시험, 실제 PCB/물리 모델 독립 검증은 서로 다른 Gate입니다. PR의 \`PASS\` 제목에는 시험한 객체를 명시하십시오.
5. **기존 보호 정책을 유지합니다.** 짧은 브랜치와 보호된 main에 대해 Verified·Squash-only·필수 검사·미해결 리뷰 항목 해소 정책을 따릅니다. CI 실패/FAIL을 이력에서 지우지 않습니다.
6. **실물 데이터는 공개하지 않습니다.** 기업 소유 CAD/ODB++/PADS·Net/Stackup·Touchstone 또는 실측 수치를 Issue·PR·CI에 복사하지 않습니다.

## Pull request 설명에 포함할 내용

| 필수 요소 | 적는 방법 |
| :--- | :--- |
| **Problem / Why** | 어떤 실제 개발 문제를 고치는가? 동일 위치/각도별 임시 케이스가 필요한지 아닌지 |
| **Scope / Not in scope** | 구현한 SOURCE 형상·OS·품질 Gate와 미지원 항목 |
| **Source of truth** | 요구사항 U/T, 코드 경로, 관련 Issue, 공식 체크리스트 |
| **Evidence** | PR-head SHA, CI/workflow URL, 공개 입력 Fixture와 확인된 테스트 결과 |
| **Fail-closed and privacy** | 무효 형상·다른 Net/Layer·접선·오버플로와 기밀 입력 처리 |
| **Docs** | [문서 홈](docs/README.md)의 경로 업데이트, [한정 현황 체크리스트](docs/development-checklist.md)·검증 원장 링크 |

마크다운 문서만 바꾸는 경우, 위 본문에서 설명한 **Docs-only classifier**가 네이티브 Windows/macOS 작업을 실행 전에 skip하는지 확인하십시오. 문서 변경으로 컴파일 검사를 재실행해 거짓 인증 증거를 만드는 것은 허용하지 않습니다.

## 내부 링크와 기록을 검토할 때

- 상대경로 대상 파일의 존재와 현재 문서의 제목/링크를 검사합니다.
- 오래된 보고서는 지우지 않고 **historical checkpoint**로 표시해 현재 정본과 연결합니다.
- PR #35 등 과거 상태가 현재와 다르면 **현재 날짜·GitHub 확인 근거를 덧붙여** 정정합니다.
- 새 Markdown 보고서는 [전체 Validation 색인](docs/validation/INDEX.md)에 연결합니다.
- 정식 체크리스트에 한정 [x]가 생겨도 **제품 U/T 완료율로 환산하지 않습니다.**

