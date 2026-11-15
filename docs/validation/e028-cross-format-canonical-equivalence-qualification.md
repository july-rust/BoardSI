# E_028 — PADS/KiCad bounded cross-format via geometry qualification

Evidence date: **2026-11-15 KST**. Baseline: protected main 557a9c685f468768ae27a1da81a58beda1a9351c (E_027 #640 merged). Parent: #532 / #520.

**Status: IMPLEMENTED / FINAL PR CI, INDEPENDENT REVIEW, MERGE, MAIN PUSH PENDING. Not a PASS receipt.**

## Independent numeric oracle (synthetic original, not vendor)

Two separately parsed raw sources represent one **four-copper-layer through via**, with the following manually specified, integer-nm reference values. Neither importer derives the reference data.

| Parameter | Independent expected value |
|---|---|
| PADS position | (300 mil, 200 mil) |
| Physical centre | (7,620,000 nm, 5,080,000 nm) |
| PADS via diameter | 35 mil = 889,000 nm |
| PADS drill diameter | 15 mil = 381,000 nm |
| Copper order | L1 Top, L2 Inner1, L3 Inner2, L4 Bottom |
| Via Span and type | L1 to L4, Through |
| KiCad source | version 20241229, F.Cu/In1.Cu/In2.Cu/B.Cu |
| PADS-only pad proof | Circle pad on L1 and L4, no inferred inner copper pads |
| **NOT PROVED** | KiCad per-layer conductive pads; full trace/pad/Net parity; manufactured barrel; electrical contact; material, z, SI |

1 mil is exactly 25,400 nm. No floating-point tolerance is allowed. Both source files are independently parsed and each checked against this static witness. Source identities stay distinct.

Original fixture files:
- crates/boardsi-import/tests/fixtures/e028-via-pads-v94.asc
- crates/boardsi-import/tests/fixtures/e028-via-kicad9.kicad_pcb

Pinned original byte SHA-256 (not Git blob SHA-1):
- PADS: dda44f2b647c8c715c2fe2a3300fed8f0fd34ce7fa683e3ace0e9d4382cc6fba
- KiCad: 3ea780fea60bb02c91be5dcf91ecc1609cae6c15758b31e6201d8c609bf902f9

## Implementation

The opt-in e028_cross_format_oracle module calls the existing PADS generalized parser and **actual Canonical BoardModel conversion**, and the KiCad 9 Source Inventory reader. It checks exact PADS frame, physical copper layer count/order, canonical Via/Drill, source-declared end Padstack geometry and source SHA-256; KiCad inventory layer order, Via kind/envelope, physical centre and dimensions are checked independently. Reject partial/unsupported scope, missing or contradictory evidence, and invalid source. No existing Source Adapter is widened.

The KiCad inventory does not expose standalone traces, pads or canonical BoardModel, so this is intentionally **one single via common-subset qualification**. It cannot demonstrate full Canonical BoardModel geometry equivalence. KiCad envelope is **not a conductive pad/contact set**. Return fields for KiCad layer-pad contacts, electrical connectivity and solver readiness remain false.

## Negative tests

Run: cargo test -p boardsi-import --test e028_cross_format_via_oracle

- Fixed SHA and exact same geometry: bounded via equivalence, but no electrical proof.
- Changed raw PADS/KiCad Source SHA: reject without deliberate repin.
- Re-pinned PADS changed via coordinates, pad diameter, drill or added inner pad: reject.
- Re-pinned KiCad changed via centre, size or drill: reject.
- Re-pinned partial Through span, unsupported Layer/atom, unsupported remove-unused flag: reject.
- Truncation with new valid SHA: reject.
- Unchanged physical geometry but altered PADS net label with deliberate SHA repin: geometric match only, not Net equivalence.
- Unsupported oracle (e.g. Blind): reject.

## Evidence and acceptance boundary

Before concluding SOURCE-LIMITED QUALIFIED, require real final HEAD Debug/Release test execution, all required PR CI checks, independent reviewer outcome with 0 unresolved threads, protected Verified Squash merge and exact merged-main Push CI.

E_023 / EB-01 actual independent Vendor Source and same-revision CAD Oracle remain BLOCKED. Parent #532/#520 and full product U/T remain OPEN / NOT_ACCEPTED. No invented PAD contacts, electrical Graph or SI PASS.

Initial evidence ledger (update only from actual observed results):
- PR: PENDING
- Final HEAD: PENDING
- Local Rust toolchain: NOT_AVAILABLE in connected container
- CI: PENDING
- Independent review: PENDING
- Verified Squash: PENDING
- Exact main Push: PENDING
- Conclusion: NOT_VERIFIED
