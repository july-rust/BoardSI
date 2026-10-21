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
