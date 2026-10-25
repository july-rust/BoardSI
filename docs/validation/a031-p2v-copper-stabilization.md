# A_031 / P2-V — PADS Copper import and Qt RHI stabilization audit

**Status: IMPLEMENTATION CANDIDATE / CI AND INDEPENDENT REVIEW PENDING**

- Tracking: [Issue #193](https://github.com/july-rust/BoardSI/issues/193).
- Baseline: protected `main@bcdfa269233730efbbfa35f34f5ee43d18153b0b`, A_030 [PR #192](https://github.com/july-rust/BoardSI/pull/192) merged. This is an evidence baseline, not a claim of full P2 completion.
- Task: fix confirmed bugs in **already supported** source-owned Copper subset; do not introduce new grammar or electrical inference.
- Verification claims in this document apply to prior accepted A_022–A_030 only where their own PR/Issue evidence supports them. All **A_031** native CI and exact-source independent review outcomes remain PENDING until separately recorded.

## Source-owned Copper support matrix

| Increment | Source form / semantic boundary | Preview handling | Unsupported or unverifiable meaning |
| --- | --- | --- | --- |
| A_022 P2-M | Single explicit source COPCLS straight polygon or COPCIR circle, no holes | Exact source rings/analytic circle, bounded Qt fill | PADS vendor-general export; flooded copper |
| A_023 P2-N | Same-COPPER rectangular COPCUT holes | Source-bound hollow fill, V11 indexed H | Loose/unowned COPCUT not associated by containment |
| A_024 P2-O | Same-COPPER straight orthogonal, including concave, polygons and polygon holes | Exact source-grid mesh, CPU/RHI | Unsupported piece kinds |
| A_025 P2-P | Bounded nonorthogonal straight polygon and polygon hole rings | Exact bounded source-grid topology and shared mesh | Arc-shaped holes |
| A_026 P2-Q | Explicit same-COPPER round COPCCO void(s) in bounded straight polygon | Analytic source, conservative outward hole proxy, V12 J | Unowned round void; radius outside bounds |
| A_027 P2-R | COPCIR outer with strictly enclosed COPCCO circular voids | Analytic source, conservative inner outer/outer hole proxies | Boundary tangency and overlap |
| A_028 P2-S | COPCIR outer with explicitly owned straight COPCUT, optionally COPCCO | Reuse V11 H and V12 J where appropriate | Contact/overlap; automatic owner inference |
| A_029 P2-T | Outward, cardinal quarter-arc convex source-owned COPCLS, without holes | Analytical source Arc/Line distinct from inward display proxy; V13 G/R | Arbitrary-angle/inward arcs |
| A_030 P2-U | Same curved COPCLS with **one strictly enclosed convex straight COPCUT** | V14 G/R/H, certified conservative Qt CPU/RHI mesh | Multiple, concave, round or curved holes in curved outer |

Restrictions include PADS V9.4 MILS explicit source ownership, maximum eight cutouts in supported non-curved shapes, bounded coordinates/vertices/proxies and strict fail-closed topology. The matrix does **not** promise every Cartesian product of rows is accepted; test the exact bounded combination. A V14 curved owner does not silently authorize mixing independent V11/V12 hollow owners in the same imported frame.

The **Canonical Model** preserves source analytical geometry and provenance, but source net labels remain distinct from verified electrical `NetId` connections. Geometry mesh used by CPU GraphicsView and Qt Quick/RHI is display-only and may be deliberately conservative. Do not identify its triangles as physically flooded/fabricated copper.

## A_031 source audit and repaired regression

### R1: orphan standalone COPCCO was silently skipped (source ownership fail-open)
The `copper_preview::parse_impl` top-level `*LINES*` item filter explicitly rejected standalone `COPCUT` but did **not** reject a standalone item whose second token is `COPCCO`. Without a following recognized five-token piece header, the parser could skip that item and still accept a separate source-owned COPPER primary, displaying copper without representing the unowned round void. Unowned geometry may **never** be silently interpreted as harmless.

**Correction:** reject top-level `COPCUT` **or** `COPCCO` with `UNSUPPORTED_COPPER_VOID`. This rejects ambiguous source association rather than guessing an owner; do not treat this as a general PADS/CAD syntax endorsement. A public synthetic mutation test places both orphan types before and after a valid COPPER record; known explicit owned COPCCO fixture remains accepted.

**Regression gate:** `cargo test -p boardsi-import`, plus existing A_022–A_030 native Qt import/IPC/CPU/RHI smoke. CI status **PENDING**, not PASS until the exact final source HEAD reports success.

## Native acceptance to complete

1. Rust source parser, geometry, Canonical and CLI tests; `cargo fmt --check`, clippy and existing numerical/Worker safety checks.
2. Native Qt: Linux x64 offscreen, Windows x64, Windows ARM64 and macOS ARM64; A_022–A_030 selftests and round Cutout rejection acceptance.
3. V1–V14 backward compatibility, imported-current RHI frame proof, valid Copper pick vs hole/exterior miss, layering/net labels, reimport and clear.
4. Latest exact PR HEAD independent Codex review, zero P1/P2 findings and zero unresolved review threads.
5. All protected required check contexts green before **GitHub-Verified Squash-only** merge; separately verify protected main post-merge checks and signature.

## Explicit limits
- Real vendor PADS export compatibility: **NOT VERIFIED**
- Actual flood/plane thermal relief, electrical continuity, trace/via/pad contact: **NOT VERIFIED**
- Physical 2D RLGC/S-parameters and real-board SI: **NOT RUN**
- Hardware GPU VRAM, device loss/driver reset and 1 million objects at 60 FPS: **NOT RUN**
- Newly requested curved-circle, multiple curved holes or arbitrary Arc variants: **UNSUPPORTED** in A_031

Do not close broad Issues #9, #11 or #165 by virtue of this incremental preview audit. Do not promote this A_031 document to VERIFIED until **all** final evidence exists.
