# BoardSI M1 PADS ASCII signature probe (partial)

This directory implements a bounded, **read-only first-line signature check** for PADS PowerPCB and PADS Logic ASCII exports. It **does not** parse board geometry, units, nets, layers, pads, vias or stackup; it does not populate the Canonical Board Model or run an SI calculation. All detections are **SIGNATURE_ONLY**, regardless of reported source revision.

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


## P2-A (restricted, experimental)
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
rectangular-finger padstack: `-2 width RF 0 orientation length 0 0`.
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
