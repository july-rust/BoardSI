# A_033 / P2-X — Source-to-artwork independent parity audit

Status: **SOURCE-GRID AUDIT IMPLEMENTED AND VERIFIED — bounded Synthetic PADS subset; independent hardware/vendor qualification NOT_VERIFIED**

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

## Exact-head acceptance evidence (PR #199)

- [Implementation and tests PR #199](https://github.com/july-rust/BoardSI/pull/199), final reviewed/tested HEAD `24a2eb529ffc0ed65c629ea7f64d0b239ab4412f`.
- **17/17 active protected-main required status contexts: SUCCESS** at exact final PR HEAD, including Rust `numerical` (fmt, clippy, workspace tests), native Windows x64/ARM64, macOS ARM64, and all four Qt native/offscreen contexts. Main ruleset #25046595 requires signed squash-only merge.
- **Codex independent GitHub review: COMPLETED** on exactly `24a2eb5`, `has_any_findings=false`, `has_unresolved_findings=false`. A prior environment-setup request applied to the *earlier* draft; it must not be treated as the final review disposition.
- **No open GitHub review threads** on PR #199 at merge.
- Merged by squash to protected main `770a51bc0e0315dfc5a37ad8d03776e57be6877e` and GitHub commit signature `verified=true; reason=valid`.
- [Post-merge main CI runs](https://github.com/july-rust/BoardSI/actions?query=branch%3Amain) must be examined for the merge SHA itself; a successful PR-only check is not proof of a successful post-merge push. Issue #198's final disposition records post-merge result.

### Existing native Qt evidence (not a hardware claim)

At [the PR exact-head Qt workflow](https://github.com/july-rust/BoardSI/actions/runs/40272281908), the Windows x64, Windows ARM64, and macOS ARM64 Qt native jobs succeeded. Existing *named* native test steps for V9–V14 filled/cutout/curved imported copper, CPU/RHI parity, source picking, import explorer, software RHI CPU fallback and source-bound Artwork Viewer succeeded. Linux x64 offscreen provisional job succeeded. These native results exercise the **existing** RHI/CPU code; A_033 adds source-grid oracle tests and does **not** change RHI/CPU engine internals.

The test names are success evidence for the respective bounded fixtures and CI environments, **not** whole-design fabrication fidelity, a physical GPU memory certification, and not a guarantee that a particular real vendor PCB will import.

## Input and regression matrix

- **PASS (Synthetic)**: exact `T|` route x/y, widths, source Net label/layer and object count; `P|` five-device placements, 10 RF pads, top/bottom mirror and orthogonal rotation; `O|` polygon BOARD/CUTOUT edges; `G|P` polygon copper and two explicitly linked `H|` holes.
- **PASS (Synthetic)**: opt-in A_032 source SHA-256, unchanged default wire prefix, one exact same-Net local endpoint and explicit `SI_NOT_RUN`. SHA oracle permits **only** authored 221-byte LF or Git Windows 236-byte CRLF versions, each with independently fixed digest; no importer digest used as its own reference.
- **PASS (Synthetic)**: source grid mutation changes emitted transport; unsupported revision, out-of-range coordinate and layer mutation fail closed, no partial successful stdout.
- **PARTIAL**: existing V1–V14 Qt coverage through native CI includes source shapes and CPU/RHI fallback; no separate A_033 physically calibrated external renderer oracle was produced.
- **NOT_VERIFIED**: vendor-produced, redistribution-cleared actual PADS V9.4 exports. Only permitted public synthetic fixtures were available in the test evidence; do not imply real CAD certification.
- **NOT_RUN / separately scoped**: measured physical SI/RLGC/S-parameters and full-board electrical Open/Short, physical GPU reset and VRAM, proprietary customer boards.
- **NOT_IMPLEMENTED in this audit slice**: general-purpose end-user JSON/CLI Golden comparison tool; exact-source CI process-level tests are the verification artifact. If needed, track this as a separate UI/diagnostic feature, not as existing importer behavior.

No parser grammar was widened, no unrelated source CAD data uploaded, and the legacy V1–V14 Qt transport remains unchanged.

## Explicit nonclaims

- All other source forms beyond the current parser support matrix: **UNSUPPORTED / NOT_EVALUATED**
- Vendor real exporter comparison: **NOT_VERIFIED**
- Real GPU device-loss, VRAM, 1M objects at 60 FPS: **NOT_RUN**
- Complete-board electrical Open/Short and Pad-to-Pin/Copper/thermal contacts: **NOT_VERIFIED**
- Solver / RLGC / S-parameters / measured SI: **NOT_RUN**
- Binary distribution license approval: **NOT_VERIFIED**

The full M1/P2 umbrella issues #9, #11, #165 remain open.

## Codex review chronology

An early draft review attempt responded with an environment setup instruction. The subsequent **ready-for-review** event triggered an independent Codex review at exact final PR HEAD `24a2eb5`. The bot's final structured status reported completed with no findings; this supersedes the stale draft status. No P1/P2 findings were unresolved at merge.

Source: [PR #199 review and merge history](https://github.com/july-rust/BoardSI/pull/199).
