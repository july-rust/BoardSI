# A_033 / P2-X — Source-to-artwork independent parity audit

Status: **IMPLEMENTATION CANDIDATE — EXACT-HEAD CI/REVIEW AND QT/VENDOR GATES PENDING**

Tracking: [Issue #198](https://github.com/july-rust/BoardSI/issues/198).
Start: protected `main@a875024969a0a5cc4096c6d26fc933269b1f8681`, A_032 #196 and #197 confirmed merged.

## Goal and source boundary

This audit verifies *only the explicitly supported source-authored* PADS V9.4 MILS partial-artwork subset. It is not a general vendor PADS importer and does not certify real-world board-to-board topology or physical SI. The existing pipeline is PADS source → Rust Importer → Canonical Board Model → legacy Qt V1–V14 IPC, plus the **opt-in** A_032 local contact suffix. Renderer GPU/RHI/CPU consistency requires separate native Qt execution evidence.

### Independent exact-source grid reference

Two new native Rust process-level suites (`tests/a033_source_parity.rs` and `tests/a033_artwork_shapes.rs`) check complete sets of source-owned `T|`, `P|`, `O|`, `G|P`, and `H|` rows against manually calculated, literal source-coordinate expectations. It is not an importer-output recording.

| Fixture | Exact authored evidence | Check | Limit |
| --- | --- | --- | --- |
| `pads_route_straight_v94.asc` | 3 original traces on layers 1 and 4, exact source-mil→nm endpoints and widths | full unordered `T|` row equality, no unexpected conductive primitives, strict legacy V1 terminal and layer count | supported straight subset, no trace width-based junction proof |
| `pads_bottom_mirror_v94.asc` | 10 authored RF pads at 5 placements, 0/90/180/270 rotations, mirrored layer 4 and top layer 1 | full unordered `P|` row equality, no phantom traces or copper, exact side/rotation/net-unverified labels | synthetic orthogonal RF only; no electrical Pad/Pin net proof |
| `pads_route_straight_v94.asc` opt-in | exactly one equal-Net endpoint contact at 300,200 mil; SHA-256 of original bytes | byte-identical legacy prefix; strict hash/count/trailer; one exact local `E|S` evidence, no fabricated conflict | **not complete Open/Short**, no source Net-to-pad assertion |
| Mutated public route fixture | source 100→101 mil, oversized coordinate, MAXLAYER 65, V9.5 header | successful modified source yields modified x-coordinate; unsupported/hostile inputs reject with empty stdout | synthetic mutation only, not exhaustive fuzzing |
| `pads_board_outline_cutout_v94.asc` | source-owned MAIN_BOARD + INNER_CUT, four exact edges each at authored 100/200 mil origin | entire `O|` row-set equality, duplicate/no-extra copper rejection, non-SI-only suffix | source-designed polygon bounds only; actual routing keepouts NOT_VERIFIED |
| `pads_copper_cutout_v94.asc` | explicit one `G|P` copper polygon and two `H|0` hole polygons relative to 40/40 mil source origin | original complete outer polygon points and hole owner/points match independent authored MIL references, with source-line provenance maintained | neither thermal/flood nor electrically proved GND plane |

Golden arithmetic: 1 mil = 25,400 nm; one supported canonical 0.01-mil tick = 254 nm. An IPC/display mesh has no authority to alter the original source-grid coordinates.

The tests check whole transport primitive sets rather than isolated substring presence, and reject duplicate rows. Previously existing Rust/Qt tests are preserved.

## Work still required before closing A_033

1. Run **final exact-head** Rust format, clippy, all-target workspace tests and existing native Qt full/filtered imported-artwork CTests; reconcile any failure.
2. Audit GPU and CPU RHI/GraphicsView parity on the same source-owned geometry and source-picking filters. An offscreen CPU success is **not** hardware GPU validation.
3. Recheck failure/clear, rejected wrong-layer/out-of-range input, renderer marker overlap, repeated import with no stale source objects and bounded performance.
4. Run independent Codex review of the exact PR HEAD, address P1/P2 findings, and verify strict protected required contexts. Only then merge signed squash and confirm main post-merge checks.
5. Seek *legally redistributable* real PADS V9.4 exports. If unavailable, keep real CAD fidelity `NOT_VERIFIED`. Never publish proprietary CAD files.
6. Assess remaining A_033 coverage before Issue #198 is closed; if nontrivial native/real-file gates remain, keep Issue #198 open and record an accurate partial closeout.

## Explicit nonclaims

- All other source forms beyond the current parser support matrix: **UNSUPPORTED / NOT_EVALUATED**
- Vendor real exporter comparison: **NOT_VERIFIED**
- Real GPU device-loss, VRAM, 1M objects at 60 FPS: **NOT_RUN**
- Complete-board electrical Open/Short and Pad-to-Pin/Copper/thermal contacts: **NOT_VERIFIED**
- Solver / RLGC / S-parameters / measured SI: **NOT_RUN**
- Binary distribution license approval: **NOT_VERIFIED**

The full M1/P2 umbrella issues #9, #11, #165 remain open.

## Independent Codex availability (2026-10-26)

The exact prior candidate `551662e` passed the strict 17/17 required CI contexts. An explicit `@codex review` returned **'To use Codex here, create an environment for this repo'**, rather than a completed review verdict. Do NOT report a Codex PASS or zero findings on this basis. The newer geometry tests must independently clear all new exact-HEAD CI contexts; third-party review remains **NOT_RUN / ENVIRONMENT_NOT_CONFIGURED** until a real review completes. This is not evidence that the source tests are free of unknown flaws.
