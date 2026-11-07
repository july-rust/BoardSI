# BoardSI PADS Importer — 서명 검사와 제한 Artwork 파서

> **현재 기준: 2026-11-07 KST.** 이 디렉터리에는 **서로 다른 두 단계**가 있습니다. \`boardsi-import-probe\`는 여전히 첫 줄 형식 식별만 합니다. 반면 \`boardsi-import-artwork\`는 **PADS PowerPCB V9.4 MILS의 제한된 SOURCE 기하 서브셋**을 읽고 일부 Trace/Arc/Via/SMD Pad/구리 영역을 보존합니다. 어느 쪽도 완전한 PADS Import 또는 정식 PCB SI 해석기는 아닙니다.
>
> [프로젝트 문서 홈](../../docs/README.md) · [개발 가이드](../../docs/getting-started.md) · [정식 구현 상태](../../docs/development-checklist.md) · [Trace–Via–Pad 기하 계약](../../docs/architecture/trace-copper-contact.md).

## 도구별 실제 역할

| 명령 | 실제 지원 범위 | 결과 의미 |
| :--- | :--- | :--- |
| \`boardsi-import-probe\` | 제한된 ASCII 첫 줄 서명 검사만 | \`SIGNATURE_ONLY\`; **형상 파싱 없음** |
| \`boardsi-import-artwork\` | 제한 PADS V9.4 MILS ROUTE 및 일부 Copper/Component/Pad/Via/Arc | Source SHA·기하·부분 Capability; \`SI_NOT_RUN\` |
| \`boardsi-import-artwork --general-trace-contacts-v1\` | 선분/원호 Trace 사이 선폭을 고려한 SOURCE 국소 접촉 후보 | **D_054** Geometry-only |
| \`boardsi-import-artwork --trace-pad-via-contacts-v1\` | 임의 방향 Trace와 **비중심 Via 환형 Pad / SMD Pad 전체 면적**의 접촉 후보 | **D_055** Geometry-only; 전역 전기 연결/Short 미판정 |

**중요한 정정:** 이 아래 오래된 \`P2-A\`/A_005 기록의 ‘straight-segment-only’, ‘Via/Arc not rendered’는 **초기 단계 당시의 제한 사항**입니다. 현재 임포터 전체의 동작을 뜻하지 않습니다. 해당 절은 회귀시험 역사를 위해 보존합니다. 지원이 늘어난 형상은 제한된 SOURCE 자료 및 공개 합성 회귀시험으로만 입증됐습니다.

## Historical signature-probe behavior (still valid)

This directory originally implemented a bounded **read-only first-line signature check** for PADS PowerPCB and PADS Logic ASCII exports. **The signature-probe binary**, not the separate Artwork importer, does not parse board geometry, units, nets, layers, pads, vias or stackup. Its detections are **SIGNATURE_ONLY**, regardless of reported revision.

## Usage

```sh
cargo run --locked -p boardsi-import --bin boardsi-import-probe -- /path/to/synthetic.asc
cargo test --locked -p boardsi-import
```

The binary prints one of `PADS_PCB_ASCII_SIGNATURE_ONLY`, `PADS_SCHEMATIC_ASCII`, or `UNKNOWN_OR_UNSUPPORTED`. Schematic/unknown inputs return nonzero. No source bytes, filenames, private net names or geometry are logged. Only the first 514 bytes are read (512-byte first-line limit plus CRLF lookahead); headers longer than 512 bytes and truncated or bare-CR terminators are rejected.

These are public synthetic examples only; do **not** upload company PCB sources to GitHub Actions. The detector tests both `!` and `*` anchored PowerPCB forms and excludes schematic `PADS-POWERLOGIC`/`PADS-LOGIC` and non-PADS `.asc` files.

References:
- [KiCad PADS importer documentation](https://dev-docs.kicad.org/en/import-formats/pads/index.html)
- [KiCad header discriminator](https://docs.kicad.org/doxygen/pads__common_8cpp_source.html)

Roadmap trace: [Issue #8](https://github.com/july-rust/BoardSI/issues/8) (format detection) and [Issue #9](https://github.com/july-rust/BoardSI/issues/9) (future real PADS importer). Product acceptance tests T01/T02/T03/T16/T17/T20/T22 remain NOT_RUN: the new unit tests are limited parser-preflight evidence, **not** the M1 import acceptance gate. No SI accuracy, provenance transformation or redistribution claim.


## Historical P2-A first artwork subset (restricted, superseded)
**Historical checkpoint:** the original P2-A developer Preview used an **offline straight-segment-only** parser for PADS POWERPCB V9.4-MILS ROUTE entries. Its individual exclusions below document that early milestone; **they are not an up-to-date feature matrix**. Current limited Arc/Via/Pad/Copper subsets and D_054/D_055 geometry diagnostics are listed at the top of this README. Full PADS support and real-board physical SI still remain unverified.
Reference: PADS Layout ASCII Format Specification 9.4, pp. 51-55;
KiCad developer documentation on PADS units/ROUTE.

Build: cargo build --locked -p boardsi-import --bin boardsi-import-artwork
Usage: boardsi-import-artwork PRIVATE_LOCAL_FILE.asc
Output contains an explicit BOARDSI_P2A_PARTIAL_V1 magic and SI_NOT_RUN.
It is for local Qt IPC only; **do not upload your PCB**. Only segments
between two consecutive supported coordinate records (identical layer and
width, no arc/via/teardrop extension) are represented. Components, padstacks,
vias, pours, net connectivity, reference planes, board outline and stackup
are MISSING. Layer names are generic until verified layer-definition
import exists. The Canonical Model records Source SHA-256 and exact 0.01-mil integer
coordinate ticks (254 nm per tick) and produces a PARTIAL/MISSING capability report; it is
not SI-ready. Input cap: 8 MiB; line cap 16 KiB; segment cap 10,000.
Only publicly safe synthetic fixtures are exercised in automated tests.
Real PADS V9.4 ASCII exports and external reference comparison remain
NOT_VERIFIED until an independent real-export fixture is checked.

### P2-A parser correction gate (A_005)
- Accept standard annotated `*PCB*` controls and `*REMARK*` without losing the surrounding section.
- Decimal coordinate and width tokens are accepted to a maximum of two fractional digits,
  converted **exactly** to integer 0.01-mil ticks, then to nm for the Qt wire format.
  Excess precision and overflow cause a failed import rather than rounded geometry.
- A SIGNAL must contain a recognized pin pair before any route-coordinate line. Unexpected
  route records cause an error, not silent geometry loss. A fresh pin pair or SIGNAL
  breaks any carried path.
- This version **does not render** arcs, miters, vias, teardrops, jumpers or any
  extended coordinate record. Unsupported records reset path continuity; they
  never cause a substitute straight chord to be drawn. Counts are reported.
- The parser has synthetic independent expected-coordinate and CLI-transport tests,
  but **actual exporter compatibility and visual cross-reference remain NOT_VERIFIED**.
  Even when import succeeds, the result is PARTIAL and SI_NOT_RUN.

- General-parameter `MAXIMUMLAYER` is accepted over the PADS V9.4 source-domain
  range 1–64, including routes on copper layer 64; layer 65 remains a pin
  termination marker, not a copper layer. The Qt decoder and viewer accept
  the same 1–64 copper-layer range.

- The V9.4 MILS header may omit an encoding suffix (default CP1252) or explicitly
  declare `-CP1252`; optional `-250L` is also recognized. The restricted
  route importer does **not** decode all PADS code pages or accept every V9.4
  object. CP1252 high bytes in unsupported/ignored sections and comments do not
  invalidate supported ASCII straight ROUTE geometry. **Only ASCII route pin,
  net, and coordinate records are accepted**; non-ASCII active ROUTE data is
  rejected with `NON_ASCII_ROUTE_UNSUPPORTED`. Headers declaring CP932 or other
  code pages are rejected. This is not general CP1252 artwork/net support.

### A_007 spec-alignment correction (PADS V9.4, ROUTE p. 52)

The original Preview synthetic route fixture mistakenly used a five-token
Connection-style pin pair (`REF.PIN REF.PIN R 0 0`). In the official PADS 9.4
Layout ASCII **ROUTE** grammar the pin pair is exactly **two** tokens:
`REF.PIN REF.PIN`. The Preview parser and public fixture now enforce that
distinction. Five-token reuse/connection records in the ROUTE section fail
closed with `INVALID_PIN_PAIR_OR_ROUTE_RECORD`; they are never converted to
copper traces. Optional SIGNAL metadata remains accepted.

This is a spec-based synthetic regression, **not** an independent vendor-
exported board comparison. Proprietary PCB data remain excluded from GitHub.
The local UI continues to report PARTIAL and SI_NOT_RUN. Native desktop GUI
smoke does not qualify a self-contained, redistributable Qt package.

Reference: [PADS Layout ASCII Format Specification 9.4, ROUTE Definition pp. 51–52]
(https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf).

### A_007 Windows x64 local Developer Preview launcher

After reproducing the native `qt-runtime.yml` Rust build, Qt CMake build
and synthetic model-fixture generation, set `QT_PREFIX` to the pinned
Qt 6.8.3 MSVC x64 SDK and start the **local** GUI:

```powershell
$env:QT_PREFIX = "C:\\path\\to\\Qt\\6.8.3\\msvc2022_64"
./scripts/run-boardsi-preview-windows-x64.ps1
```

Add `-SelfTest` to run the same GUI window, Rust Importer and Worker against
the public synthetic fixture with a bounded automated smoke. The new
Windows x64 GitHub Actions step executes that launcher using the native
desktop Qt backend and uploads the same JSON result as the other Preview
tests. This verifies the *development launch path*; it does not produce or
authorize a portable redistribution package. Qt/LGPL and other bundled
third-party licensing checks remain a separate BLOCKED/NOT_VERIFIED gate.

### A_010 P2-B experimental via artwork (NOT FULL PADSTACK SUPPORT)
This experimental branch additionally accepts **explicit named round through-vias**
from the PADS 9.4 `*VIA*` section and the matching sixth `*ROUTE*` vertex token.
The intentionally strict definition is `name drill 2` plus **top -2 and
bottom 0**, each with equal `R` diameters exceeding the drill, or
`name drill 3` with the same `R` diameter at -2, -1 and 0. Drill
and diameter are exact 0.01-mil integer values (254 nm per tick).

These are only **explicit circular pad/drill marks** and known full-span
through-hole declarations; other padstack shapes, partial-layer via spans,
slots, unusual per-layer pads, implicit/default vias, inferred routing,
stitches, teardrops, thermal parameters, 3D plating and copper connectivity
remain unsupported. Unknown/unaccepted named-route extensions reset route
continuity, are counted as omitted, and **must not** be drawn as invented
straight chords. Canonical `connected_layers` is deliberately empty;
net names are labels, not independent evidence of electrical contact.
Geometric import is always `PARTIAL` and physical SI is `NOT_RUN`.

Public-only fixture: `tests/fixtures/pads_via_round_through_v94.asc`.
It is exercised by Rust unit tests and native Qt `--via-selftest`,
recorded in `qt-preview-p2b-via-results.json`. Actual manufacturer-exported
PADS board fidelity remains `NOT_VERIFIED`. No company PCB is uploaded.
PADS format source: https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf
(pp. 39-40, 51-52). Reference cross-check: https://dev-docs.kicad.org/en/import-formats/pads/index.html


### A_012 P2-D restricted rectangular-finger SMD preview

The importer accepts a deliberately narrow PADS V9.4 `RF` top-layer
rectangular-finger padstack: `-2 width RF orientation length 0 0`.
Only `orientation=0/90`, exact zero finger offset, exact zero corner radius,
no drill, and three top/inner/bottom stack rows with nonconductive
inner/bottom are supported. Component quarter-turns 0/90/180/270 rotate
final **board-axis** width and height. Other `RF` syntax fails closed.

When a restricted rectangle is present, the ASCII IPC reports
`BOARDSI_P2D_PARTIAL_V3` and uses a distinct `RF` pad shape with explicit
nm widths and heights. Existing no-pad V1 and round/square-only V2 records
remain unchanged. Pad Net stays `NOT_VERIFIED`; source
geometry and real exported PADS V9.4 compatibility remain `NOT_VERIFIED`.
A pad-only file is valid partial artwork even without traces or vias.

Public synthetic fixture: `tests/fixtures/pads_rf_rectangle_v94.asc`.
The CLI test independently checks all 8 pads for four component rotations
and two rectangular-finger orientations, while Qt native
`--rect-selftest` exercises pad-only Properties, 8 rendered rectangles,
source millimetre geometry, selection, and layer visibility. The Linux CTest
and native Windows x64/ARM64 and macOS ARM64 workflow are mandatory.
This does **not** implement bottom/mirrored components, custom pads,
connections or impedance/S-parameters; SI remains `NOT_RUN`.

Specification: [PADS Layout ASCII Format Specification, PADS 9.4,
Part Decal Definition, pages 36-37](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf).

### A_013 P2-E restricted Bottom SMD / mirror preview

PADS V9.4 `*PART*` explicit `M` (bottom) and `N` (top)
placements are supported only for the already restricted drill-free
round/square/RF rectangular decal cases, with primary decal (alternate 0),
zero piece/text/label/cluster, and orthogonal 0/90/180/270 degree turns.
No mirror interpretation is inferred for unsupported records.

In the PADS top-view Cartesian frame, local terminal geometry is first rotated
by the component turn, then mirrored about the component-origin vertical axis
for bottom placement. Existing Source 0.01-MILS integer coordinates are
preserved and mapped to the final declared copper layer. Canonical
`Component.placement.bottom_side` and `mirror_x` record this restricted
transform; `Pin.net` and `Pad.net` remain `None`.

When any bottom pad is accepted, IPC
`BOARDSI_P2E_PARTIAL_V4` emits `P|ref|pin|layer|shape|x_nm|y_nm|width_nm|height_nm|NET_UNVERIFIED|T_or_B|rotation_degrees`.
Legacy V1/V2/V3 output is unchanged for corresponding top-only boards.
Qt Artwork Viewer uses the fixed top-view projection on both sides, with
separate Top/Bottom copper-layer visibility and selection details.

Public synthetic asymmetric mixed fixture:
`tests/fixtures/pads_bottom_mirror_v94.asc`. Rust CLI checks all ten exact
positions, layers and dimensions, while Qt native `--bottom-selftest` checks
ten rendered pads, selected Properties and independent Bottom-layer hiding.
Linux CTest and native desktop Windows x64/ARM64/macOS ARM64 tests are part of
the existing Qt workflow.

**Limits:** This is experimental synthetic PADS V9.4 geometry preview only,
not general production PADS compatibility. Real vendor-exporter cross-reference
and independent CAD roundtrip remain **NOT_VERIFIED**; electrical connectivity
is **NET_UNVERIFIED** and physical SI is **SI_NOT_RUN**.

Source: PADS Layout ASCII Format Specification 9.4 Part List pp. 45-46;
KiCad PADS importer footprint bottom-layer `Flip(... LEFT_RIGHT)`
implementation. The KiCad reference does not constitute a formal vendor
compatibility certification. Issues #9/#11 remain open.

---

## Current generic Trace–Via–SMD Pad contact diagnostic (D_054/D_055; 2026-11-07 KST)

The common SOURCE-grid predicates do **not** enumerate separate cases for each trace angle, Via-centric endpoint, SMD pad edge or corner. Finite-width straight and circular-arc routes are checked against actual supported circular Via annuli (outer copper diameter minus drill aperture) and circular/square/rectangular SMD pad footprints. A Via that spans a layer without an explicitly supported conductive pad is not assumed to contact copper there. Source Net ownership is preserved; cross-net positive-area intersection is a **conflict candidate**, not a validated electrical Short.

From the repository root, build and run only publicly safe fixtures or approved offline local files:

\`\`\`sh
cargo build --locked -p boardsi-import --bin boardsi-import-artwork

# Source-local Trace vs Via/SMD Pad copper candidate report
./target/debug/boardsi-import-artwork \
  crates/boardsi-import/tests/fixtures/d040_multi_component_v94.asc \
  --trace-pad-via-contacts-v1

# Source-local straight/circular Trace vs Trace candidate report
./target/debug/boardsi-import-artwork \
  crates/boardsi-import/tests/fixtures/d040_multi_component_v94.asc \
  --general-trace-contacts-v1
\`\`\`

The output is **opt-in**, versioned and SHA-256-bound. The normal Qt Artwork V1–V14 wire format remains unchanged. A positive contact is a **local geometry observation**, not a full Canonical electrical graph edge, actual solder/plating evidence or physical RLGC/S accuracy guarantee. Fail-closed near-tangency, unsupported source records, invalid geometry and pair limits remain explicit.

Code: [straight/arc kernel](src/trace_contact.rs) · [Via/SMD footprint kernel](src/trace_attachment.rs). Verification: [D_054/D_055 scoped results](../../docs/validation/d054-d055-general-trace-contact-qualification.md). Product acceptance: [single checklist](../../docs/development-checklist.md).
