# BoardSI PCB Import — source-format subsets and historical PADS milestones

**실행 경로 연결:** [Qt Developer Preview](../../apps/boardsi-qt-prototype/README.md) · [독립 Rust Worker](../../apps/boardsi-worker/README.md). Importer의 제한 SOURCE 국소 판정이 전체 전기적 연결성 또는 실제 물리 SI 결과로 자동 승격되지는 않습니다.

[← Documentation hub](../../docs/README.md) · [Developer Quickstart](../../docs/developer-quickstart.md) · [Canonical checklist](../../docs/development-checklist.md) · [Detailed D_054–D_055 evidence](../../docs/validation/d054-d055-general-trace-contact-qualification.md)

> **2026-11-10 current scope.** This crate contains (1) a signature-only PADS probe, (2) a **restricted** V9.4 MILS PADS geometry importer feeding the partial Canonical/Qt V1–V14 Artwork protocol, (3) a separate opt-in source-backed partially layered VIA geometry diagnostic (R/S/A conductive copper and RA/SA anti-pad), and (4) a **standalone limited ODB++Design unpacked-directory import diagnostic**, not yet connected to the main Qt Import path. The original P2-A straight-only introduction below is a **dated baseline**, not the total current capability. Neither tool is a complete vendor-export compatible CAD converter or a validated PCB SI solver.

## Current command matrix

| Executable / mode | What it does | What it does **not** prove |
| :--- | :--- | :--- |
| `boardsi-import-probe FILE.asc` | Read-only PADS PowerPCB/Logic first-line signature only | Not full file, object or board recognition |
| `boardsi-import-artwork FILE.asc` | Supported SOURCE straight/arc Trace, Via, SMD Pad and selected Copper/outline partial Geometry → versioned Qt Artwork transport | Not all PADS objects or electrical connectivity |
| `boardsi-import-artwork FILE.asc --general-trace-contacts-v1` | D_054 generic angle straight/arc **Trace-to-Trace** local finite-width overlap candidates | Not global nets, actual copper continuity, solver |
| `boardsi-import-artwork FILE.asc --trace-pad-via-contacts-v1` | D_055 generic non-centred Trace contact with **Via conductive annulus** and **round/square/rectangle SMD Pad** footprints | Not proof of electrical Open/Short, plating or all Padstacks |
| `boardsi-import-via-topology FILE.asc [--candidate-contacts]` | **D_056 opt-in** PADS V9.4 source-declared R round, S square, A annular copper and RA/SA plane anti-pad, plus explicit Through/Blind/Buried start/end spans; local Trace-Via candidates | Not Qt artwork V1–V14, RT/ST thermal spokes, full vendor PADS compatibility, verified plating, global Open/Short or SI |
| `boardsi-odb-import UNPACKED_PRODUCT_ROOT STEP [--eda-nets]` | **D_056_08 opt-in** ODB++Design MATRIX/Layer + positive L/P/A and bounded S/Cutout → Canonical; optional exact `eda/data` NET/SNT/FID design ownership and source-line diagnostics | Not compressed archive, full EDA/PIN/PKG/Via/Drill, Main Qt IPC/Artwork, fabricated connectivity or SI |

Example from the **repository root** (public synthetic fixture only):

```sh
cargo test --locked -p boardsi-import
cargo run --locked -p boardsi-import --bin boardsi-import-probe -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc
cargo run --locked -p boardsi-import --bin boardsi-import-artwork -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc \
  --trace-pad-via-contacts-v1
```

### D_056_08 — ODB++Design source-only diagnostic (opt-in, bounded)

The independent `boardsi-odb-import` CLI reads an **already-unpacked ODB++Design product root** (must contain `matrix/matrix` and `steps/<step>/layers/<layer>/features`) and a selected Step name. It checks the full selected Copper-layer record inventory against its **explicitly narrow accepted syntax** and produces a Canonical BoardModel subset. Exact-coordinate and source SHA evidence are retained, but no manufactured copper or electrical connectivity is inferred. The main Qt Import menu does **not** invoke this diagnostic.

```sh
cargo test --locked -p boardsi-import
cargo run --locked -p boardsi-import --bin boardsi-odb-import -- \
  /path/to/your-local-unpacked-odb-product pcb
cargo run --locked -p boardsi-import --bin boardsi-odb-import -- \
  /path/to/your-local-unpacked-odb-product pcb --eda-nets
cargo run --locked -p boardsi-import --bin boardsi-odb-drill-audit -- \
  /path/to/your-local-unpacked-odb-product pcb
cargo run --locked -p boardsi-import --bin boardsi-odb-drill-audit -- \
  /path/to/your-local-unpacked-odb-product pcb --hits
```

By default the diagnostic outputs `status=SOURCE_GEOMETRY_PARTIAL`, Layer/Trace/Arc/Pad/CopperArea/Cutout counts, and explicit `NOT_VERIFIED` for Via, manufactured connectivity and SI. With explicit `--eda-nets`, it additionally requires `steps/<step>/eda/data` with valid `HDR`, `UNITS`, `LYR`, `NET`, `SNT`, `FID C`, and prints **design-Net ownership** counts with `status=SOURCE_GEOMETRY_AND_DESIGN_NET_PARTIAL`. Unsupported EDA syntax fails closed; `SNT VIA` identifies a design subnet **not plated Via connectivity**. Multiple `$NONE$` groups are unassigned. Main Qt Import remains unconnected. Source-limited import may reject an otherwise **valid** ODB++ exporter file if it contains unsupported net/geometry features; that is deliberate Fail-Closed behavior, **not** evidence that the exported PCB is invalid. Tests use synthetic grammar fixtures and cross-platform CI; commercial-vendor export qualification and actual Qt main integration are outstanding.

The independent `boardsi-odb-drill-audit` entrypoint (merged [#568](https://github.com/july-rust/BoardSI/pull/568)) strictly validates source `matrix` drill layer start/end, `layer/tools` tool numbers, unit, type and process intent. Without `--hits` it reports **declared tooling only**. With explicit `--hits` ([merged code PR #569](https://github.com/july-rust/BoardSI/pull/569), 30/30 PR Checks, signed Squash `f07442a1`) it additionally verifies drill-layer P hit positions with exact nonzero tool dcode cross-references and SHA/file/line evidence; the CLI prints the count and first 20 records. Neither mode produces Canonical Drill/Via, an actual plated barrel or a verified Open/Short. The source-only drill audit is independent of Copper artwork parsing; real vendor exports, canonical Qt IPC and physical CAM verification are still outstanding.

Documentation: [D_056_08 Source Adapter & Surface contract](../../docs/architecture/d056-08-odb-source-adapter.md). Code: [PR #562](https://github.com/july-rust/BoardSI/pull/562) and [PR #564](https://github.com/july-rust/BoardSI/pull/564), both verified protected Squash with PR-head 30/30 checks. An additional opt-in EDA NET/FID import is [PR #566](https://github.com/july-rust/BoardSI/pull/566) (30/30 checks, verified squash `ffdad981`, independent main Push 8/8 SUCCESS), but this is only partial design ownership and cannot make an Open/Short or SI claim. **No confidential PCB files should be committed to public CI.**

### D_056 — source-declared Via layers (opt-in, limited)

The PADS 9.4 [Via Definition grammar, pp. 39–40](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf) permits optional `drill_start`/`drill_end`; these define the actual span and assign relative `-2/-1/0` pad selectors to its top/interior/bottom. The opt-in source parser uses the actual layer count and source-confirmed pad roles: **R circular copper, S square copper, A annular copper with separate inner/outer diameter, and RA/SA nonconductive anti-pads**. Unimplemented RT/ST thermal spokes and other unsupported geometry are explicitly rejected instead of silently rounded. The same-name/source-span and malformed shape validations remain fail-closed.

To test a **local, publicly safe** V9.4 synthetic fixture:

```sh
cargo test --locked -p boardsi-import
cargo run --locked -p boardsi-import --bin boardsi-import-via-topology -- \
  crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc --candidate-contacts
```

The new command is a separate versioned **SOURCE geometry-only** diagnostic: R-only records use V1; shaped pad records use V2, including the annular A inner diameter. Legacy `boardsi-import-artwork` remains compatible with the original Qt V1–V14 source subset; a real L2–L4 via is **not** rendered as a valid through-via by that old Qt contract. The new local contact candidates do not prove plating continuity or global board connectivity. The old endpoint-based barrel graph deliberately does not promote generalized source via objects.

Implementation: [source-layer-aware via design and outstanding gates](../../docs/architecture/input-normalization.md) · [D_056 Issue #520](https://github.com/july-rust/BoardSI/issues/520) · [code PR #523](https://github.com/july-rust/BoardSI/pull/523) (final HEAD `b113e8a`, 30/30 PR checks SUCCESS, 10/10 workflows SUCCESS; independent duplicate-VIA-name review fixed) → signed protected squash [main `63183cb`](https://github.com/july-rust/BoardSI/commit/63183cbcf94e30134c08aee795b51ae9057bdc08). Additional merged [code PR #526](https://github.com/july-rust/BoardSI/pull/526) final HEAD `2f65d08321d183f4a446163f7b05705d5b5fb7f9` **38/38 GitHub checks, 13/13 PR workflows SUCCESS**, automatic review no findings, 0 unresolved threads → signed protected one-parent squash [main `7e10d7b`](https://github.com/july-rust/BoardSI/commit/7e10d7ba07d6c9e0f959eb3e2e4be570af6dd88b). This is limited source geometry support, not full vendor format or product SI acceptance; code-main Push CI is separate.

For D_054/D_055 the result is bounded, SOURCE-SHA-bound *geometry-candidate diagnostic output*. **Different-Net overlaps are only flagged as conflict candidates; missing Pad-Net ownership stays unverified; near-boundary uncertainty and unsupported records are not silently certified.** Via drill-only overlap is not assumed conductive. The normal Qt artwork stdout protocol is unchanged by those two opt-in modes.

**Supported scope remains narrow:** only the current PADS V9.4 MILS subset (plus a separate opt-in R/S/A conductor, RA/SA anti-pad SOURCE Via span diagnostic); specific SOURCE-anchored part decals, SMD round/square/limited RF rectangle, declared Via copper layers, segmented straight/circular routes and selected explicit Copper/outline. Arbitrary custom/rotated-oval Padstack, complete blind/buried/microvia manufacturing semantics, thermal reconstruction, complete pours, full-board physical connectivity and real vendor-export cross-qualification are **NOT_VERIFIED**. A parse result, screenshot or source-local candidate must never be reported as measured PCB SI.

## Signature probe (standalone)

The `boardsi-import-probe` executable implements a bounded **read-only first-line signature check** for PADS PowerPCB and PADS Logic ASCII exports. **This executable alone** does not parse board geometry, nets, layers, pads, vias or stackup and returns `SIGNATURE_ONLY` regardless of revision.

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


## Historical P2-A (initial straight-only baseline; superseded by later additive features)
The developer Preview has an **offline straight-segment-only** parser for
PADS POWERPCB V9.4-MILS ROUTE entries. This is NOT full PADS support.
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

Reference: [PADS Layout ASCII Format Specification 9.4, ROUTE Definition pp. 51–52](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf).

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
