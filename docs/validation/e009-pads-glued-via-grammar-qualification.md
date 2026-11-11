# E_009 — PADS V9.4 ROUTE glued-via (0x0800) SOURCE grammar qualification

**Date / first check:** 2026-11-12 KST. **Parent:** [M1 PADS importer Issue #9](https://github.com/july-rust/BoardSI/issues/9). **Predecessor:** [E_008 readiness](e008-pads-readiness-qualification.md).

## Selected single grammar gap and evidence

- **Documented grammar:** Mentor Graphics, *PADS Layout ASCII Format Specification*, PADS 9.4, Route Definition Format, p. 52, “Router conditions flag”: `0x00800` identifies a **glued via**. The ROUTE vertex base record is `x y layer segwidth FLAG [ARCDIR/vianame][THERMAL]`. Public reference: [PADS 9.4 ASCII PDF](https://www.freecalypso.org/pub/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf). The original document is copyrighted; **do not commit or redistribute the PDF or any proprietary vendor board**.
- **Actual gap:** `route_preview::parse_with_via_mode` allowed straight metadata bits `0x0080/0x0200/0x0400` on via vertices but rejected the documented `0x0800` glue bit, skipping an otherwise recognized, source-declared named via.
- **Bounded correction:** Permit `0x0800` **only** on a six-field named VIA placement whose definition is already validated under the existing simple or generalized PADS via rules. Preserve `ViaInstance.glued=true`, annotate Canonical Via source provenance and the import source conditions. Keep all physical route coordinates, layers, segment widths, drill/span, and normal connectivity policy unchanged. A glue declaration is **not** proof of manufacturing, actual placement lock, electrical barrel contact, or solver readiness.
- **Fail-closed:** A bare glue bit without a via name, an unrecognized via name, or malformed extension cannot silently become straight copper or a fabricated via; it produces `E009_GLUED_FLAG_REQUIRES_SUPPORTED_NAMED_VIA`. Combination with an unrelated arc-centre grammar remains unqualified, not accepted as a glued via. No change to other unsupported flag bits.
- **Compatibility:** Unflagged named vias retain the historical geometry/provenance. The canonical source identity gains an explicit glue condition **only** when a glued record occurs; the imported geometry and partial capability classification are unaffected.

## Tests and exact acceptance boundary

| Source case | Required result |
| :--- | :--- |
| Named 0x0800 through via, 2/4/6/8 layers | One Via + both adjacent route segments; correct Source flag and Canonical provenance |
| Same file without 0x0800 | Equivalent Via dimensions/traces, no glue declaration |
| `parse` and `parse_generalized` | Both preserve declared glue bit without inventing contact |
| Allowed metadata bits 0x0080/0x0200 combined with 0x0800 | Accepted only with a declared named via |
| 0x0800 without name, missing definition, or malformed extra data | Explicit fail-closed error |
| Unsupported 0x1000 arc-centre + 0x0800 | Not promoted to a valid copper chord |
| E_008 readiness on accepted file | Existing `PARTIAL` route capability, `solver_ready=false`, `physical_si=NOT_RUN` |
| Existing A–D/E_008 regression | Applicable tests must pass on exact PR head |

**Execution status:** IMPLEMENTED, **NOT_VERIFIED until PR-head CI and independent review**. Do not report the table as PASS without actual test/CI evidence; record links and observed outcomes in the Issue #9 E_009 closeout comment.

## Unverified scope / parent issue

This is only one source-grammar capability in the V9.4 MILS subset. Full PADS exporter support, revisions and units beyond the exact supported headers, other ROUTE flag semantics, global connectivity, built PCB plating, GUI picking, actual lock state, stackup, SI physics and formal U06/U07/U31 or T01/T02/T03/T16 acceptance remain **NOT_VERIFIED / NOT_RUN**. Keep original Issue #9 OPEN.

## Final closure gate

E_009 is **COMPLETE only after**: exact final PR-head required checks SUCCESS, independent review findings resolved, zero unresolved threads, protected GitHub Verified Squash, and independently confirmed exact merged-main Push workflow results. A skipped/cancelled/queued native job is not PASS. All unfinished E_009-scope steps are BLOCKED / INCOMPLETE, not COMPLETE.
